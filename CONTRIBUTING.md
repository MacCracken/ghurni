# Contributing to ghurni

## Development Workflow

1. Fork and clone the repository
2. Create a feature branch from `main`
3. Make your changes following the conventions below
4. Ensure `cyrius audit` exits 0
5. Open a pull request

## Prerequisites

- The cyrius toolchain, at the version pinned in `cyrius.cyml [package].cyrius`
  (currently **6.5.35**). CI installs exactly that pin — never hardcode a
  version anywhere else.
- Nothing else. ghurni has no Rust build; [`rust-old/`](rust-old/) is the
  preserved 1.0.0 crate kept only as the parity oracle, and is **never edited**.

```sh
cyrius deps                                # resolve dependencies into lib/
cyrius build src/main.cyr build/ghurni     # compile the smoke entry
```

## Cleanliness Check

Run the full gate locally before submitting:

```sh
cyrius audit
```

That runs fmt, lint, docs, tests and bench, and must exit 0. The individual
pieces, if you need to narrow something down:

```sh
cyrius fmt src/engine.cyr --check
cyrius lint src/engine.cyr
cyrius doc --check src/engine.cyr
cyrius test
cyrius bench
cyrius coverage
cyrius deny src/main.cyr
```

After changing anything under `src/`, regenerate the consumer bundle:

```sh
cyrius distlib      # rewrites dist/ghurni.cyr from [lib].modules
```

`src/main.cyr`, the examples and the benchmarks all compile against
`dist/ghurni.cyr`, so a stale bundle produces confusing errors about symbols
you just renamed.

## Code Conventions

- **f64 everywhere.** naad and hisab are f64-only. Use the `f64_*` builtins.
- **naad is the only backend.** There is no fallback path and no feature flag;
  `rust-old/src/rng.rs` and `math.rs` are deliberately not ported.
- **No trait objects.** Heterogeneous synth collections use a
  `(GH_KIND_*, pointer)` pair plus the tag dispatch in `mixer.cyr`.
- **Prefix every top-level symbol** `ghurni_` / `GHURNI_` / `Gh`. The distlib
  bundle shares one flat namespace with naad, hisab, goonj and sakshi, and a
  duplicate top-level definition is silently resolved with **no diagnostic**.
- **Zero panics.** Fallible functions return a negative `GH_ERR_*` code;
  callers check with `ghurni_is_err`. Nothing in library code may abort the
  process — note that the stdlib's `vec_get`/`vec_set` *do* abort on an
  out-of-range index, so bound every index yourself.
- `#must_use` on pure functions; `#inline` on hot-path per-sample processing.
- **Never put an inline `#` comment inside a `struct { }` body** — the parser
  breaks. Document fields in a comment block above the struct.
- Lines stay under 120 characters (`cyrius lint` enforces it).
- Every public function needs a doc comment (`cyrius doc --check` enforces it).
- Store high-precision float constants as **IEEE-754 bit patterns**, not decimal
  literals — `dist/ghurni.cyr` is compiled by the consumer's toolchain, and
  cyrius ≤ 6.5.27 miscompiled long decimal literals.
- `validate_sample_rate` in constructors; `ghurni_sample_count` for any
  buffer length derived from a duration.
- DC-block every synthesis output.

### The two defect classes this port keeps producing

Both come from Cyrius's single signed `i64` integer type, and both are invisible
in a straight transliteration:

1. **Rust's `usize` / `u32` / enum guarantees are erased.** Where the oracle
   typed a parameter `usize` or as an enum, a bad value was *unrepresentable*,
   so the Rust body validates nothing — and a transliterated guard is only half
   a bound. Add the check Rust got from its type system: reject negatives, and
   range-check every enum id.
2. **`f64_to` truncates where Rust's `as` casts saturate.** Rust maps an
   over-range f64 to `usize::MAX`; `f64_to` overflows to `INT64_MIN`, which is
   neither `> MAX` nor `== 0`, so one-sided clamps let it straight through.
   Never feed a raw `f64_to` result to a loop bound or an index — bound the
   value in f64 first.

## Adding a New Synthesizer

1. Create `src/your_synth.cyr` with no `include` lines — the entry file, test
   harness and distlib bundle order the modules.
2. Add a `GH_KIND_YOUR_SYNTH` tag in `src/traits.cyr` and wire both dispatch
   arms in `src/mixer.cyr`, then widen the range check in
   `ghurni_mixer_add_channel`.
3. Store `sample_rate`, `sample_position`, a `GhDcBlocker`, and a
   `GhSmoothedParam` for RPM.
4. Provide `_new` / `_synthesize` / `_process_block` / `_set_rpm` plus the
   trait-shaped accessors (`_rpm`, `_sample_rate`).
5. Validate the sample rate and every enum id in `_new`; route the sample count
   in `_synthesize` through `ghurni_sample_count`.
6. Register the module in `cyrius.cyml [lib].modules` in dependency order and
   run `cyrius distlib`.
7. Add tests and a benchmark. **Mutation-check every new assertion**: revert the
   behaviour it pins and confirm the test fails. See
   [docs/guides/testing.md](docs/guides/testing.md).

## Parity

The correctness bar is "matches what the Rust naad-backend path did". Read the
corresponding `rust-old/src/*.rs` before changing behaviour, and diverge only
with a new ADR in [`docs/architecture/`](docs/architecture/). Deliberate
divergences already recorded live in
[docs/development/state.md](docs/development/state.md).

If a change touches a synthesis path but is meant to preserve behaviour, prove
it: render the same fixture through the bundle before and after and compare a
checksum over every sample.

## License

By contributing, you agree that your contributions will be licensed under GPL-3.0-only.
