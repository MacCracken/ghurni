# ghurni — Claude Code Instructions

## Project Identity

**ghurni** (Sanskrit: rotation / spinning) — Mechanical sound synthesis for AGNOS.

- **Type**: Port (Rust → Cyrius), complete. The original Rust crate (3,054 src lines) served as the parity oracle and was **retired in 2.0.4**; it is recoverable from git at tag `2.0.3` (as `rust-old/`) or `1.0.0` (as `src/*.rs`).
- **License**: GPL-3.0-only
- **Language**: Cyrius (toolchain pinned in `cyrius.cyml` `[package].cyrius`)
- **Version**: `VERSION` at the repo root is the source of truth — **2.0.0** marked full-parity Cyrius (1.x was the Rust crate). Bump with `scripts/version-bump.sh`.

## Consumers

kiran (game engine), joshua (game manager), dhvani (audio engine), and any AGNOS component needing mechanical/vehicle audio.

## Dependencies

- **naad** — audio synthesis primitives (oscillators, filters, envelopes, noise). Consumed from the `dist/naad.cyr` bundle via `cyrius.cyml [deps.naad]`; transitively pulls hisab / goonj / sakshi (all declared explicitly so the bundle's symbols resolve).

## Quick Start

```sh
cyrius deps                              # resolve deps into lib/
cyrius build src/main.cyr build/ghurni   # compile the smoke entry
cyrius test                              # run tests/*.tcyr
cyrius bench                             # run benches/*.bcyr
cyrius distlib                           # rebuild dist/ghurni.cyr from [lib].modules
```

## Architecture (the port)

- **L0 foundations**: `error` (integer codes + shared f64 constants/helpers), `logging` (sakshi wrappers), `dsp` (GhDcBlocker + validation + naad-error map), `smooth` (GhSmoothedParam), `event` (MechanicalEvent), `traits` (GH_KIND_* dispatch tags).
- **L1 synths** (each self-contained on naad + L0): engine, gear, motor, turbine, clock, transmission, differential, forced_induction, belt_drive, chain_drive.
- **L2 composites**: mixer (tag-dispatched multi-synth), presets.
- `src/main.cyr` is the smoke entry; `dist/ghurni.cyr` is the consumer bundle.

## Key Principles

- **The correctness bar is `tests/`, not the oracle.** Until 2.0.4 it was "matches
  what the Rust naad-backend path did", checkable by reading `rust-old/`. The
  oracle is now retired, so the bar is the frozen suite that replaced it:
  - `tests/goldens.tcyr` — rendered-audio checksums. **If a golden moves, it is
    not a patch release.** Work out *why* before touching the value; never
    "fix" a golden to make CI pass.
  - `tests/oracle_pins.tcyr` — every constant and contract the oracle used to
    prove, with the originating `rust-old/src/*.rs:line` cited beside each.
  - `tests/spectral.tcyr` — rendered audio sits where the RPM physics says.
  Deliberate audio changes still need an ADR, and belong in a MINOR release.
- RPM is the fundamental parameter — everything derives from rotational speed.
- Every public enum maps to `GH_<ENUM>_*` integer constants + `ghurni_<enum>_name`/`_from_name` (serde parity).
- `#must_use` on pure functions; `#inline` on hot-path sample processing.
- Zero panics in library code; fallible fns return a negative `GH_ERR_*` code (check `ghurni_is_err`).
- Never skip benchmarks before claiming performance improvements.

## Port Conventions (hard, learned rules)

- **f64 everywhere** (naad/hisab are f64-only). Use the `f64_*` builtins; decimal float literals (`0.08`) lex to f64 bit patterns.
- **naad is the ONLY backend** — the Rust `#[cfg(not(naad-backend))]` fallback (rng/math) is intentionally not ported (naad owns the randomness).
- **No inline `#` comments inside a `struct { }` body** — the parser breaks; document fields in a comment block above the struct.
- **No trait objects** (ADR-004): `Box<dyn Synthesizer>` is replaced by a `(GH_KIND_*, pointer)` pair + a tag-dispatch switch in `mixer.cyr`.
- All top-level symbols are `ghurni_` / `GHURNI_` / `Gh`-prefixed to coexist with the naad bundle in one flat namespace.
- Audio buffers are stdlib vecs of f64; `process_block` fills a caller vec in place, `synthesize` returns a fresh vec.

## DO NOT

- **Do not commit or push** — the user handles all git operations.
- **NEVER use `gh` CLI** — use `curl` to the GitHub API only.
- Do not resurrect `rust-old/` into the tree. Cite it (`rust-old/src/gear.rs:33`) when recording where a value came from — those paths still resolve in git history — but do not re-add the directory.
- Do not modify `lib/` (resolved dep bundles).
- Do not add unnecessary dependencies.
- Do not skip tests/benchmarks before claiming changes work.

## Documentation

- **The retired Rust oracle** — `git show 2.0.3:rust-old/src/engine.rs` (or tag `1.0.0`, where it lives at `src/`). Retirement record: [`docs/development/rust-old-retirement.md`](docs/development/rust-old-retirement.md).
- [`docs/development/state.md`](docs/development/state.md) — live port state.
- [`docs/architecture/`](docs/architecture/) — ADRs + overview (Rust-era ADRs 001-003 remain conceptually valid; ADR-004 records the Cyrius port).
