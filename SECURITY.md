# Security Policy

## Scope

ghurni is a pure computation library for mechanical sound synthesis. It performs
no I/O and no networking; its only side channel is structured logging through
sakshi. All synthesis is deterministic from seeded parameters.

Cyrius has no memory-safe subset to opt into — every pointer is an `i64` — so
the library's safety properties come from explicit validation at the public
boundary, not from the type system. That makes the boundary the thing worth
auditing.

## Threat Model

Assume every public `ghurni_*` entry point is reachable with attacker-chosen
arguments: a game engine feeding RPM and load from a save file, a network peer,
or a preset shipped as data. The library's contract is that no such input causes
memory unsafety, an unbounded allocation, or a process abort — it returns a
negative `GH_ERR_*` code instead. The exceptions are listed under
[Known limitations](#known-limitations): a handle a caller did not check, and a
wild pointer above the null page.

## Attack Surface

| Area | Risk | Mitigation |
|------|------|------------|
| Sample-rate validation | Division by zero, NaN propagation; an **integer** passed by mistake is a positive subnormal that renders NaN for life | `ghurni_validate_sample_rate` rejects below 1 Hz, NaN, ±inf |
| Duration validation | Unbounded allocation, process abort | `ghurni_validate_duration` rejects ≤ 0, NaN, ±inf |
| `duration × sample_rate` | `f64_to` past 2^63 (`INT64_MIN` before cyrius 6.6.8, `INT64_MAX` since); vec growth past the stdlib cap aborts the process | `ghurni_sample_count` compares in f64 *before* the conversion and caps at `GHURNI_MAX_SAMPLES` (2^26) |
| Enum-id parameters | Out-of-range id falls through to a wrong default | Every constructor range-checks its id and returns `GH_ERR_INVALID_PARAMETER` |
| `shift_to` gear index | Negative index silently corrupts state | Both bounds checked |
| Mixer channel synth | A failed constructor's error code, null, or a small integer dereferenced as a struct (SIGSEGV) | `add_channel` rejects `ghurni_is_err(synth)` and anything below `GHURNI_MIN_VALID_PTR` (the null page); the dispatch loops skip an absent synth. See the pointer-contract limit below |
| Mixer kind tag | Type confusion between synth structs | `add_channel` range-checks the `GH_KIND_*` tag |
| Caller-owned vecs | Caller mutating a stored vec desynchronises lengths → out-of-bounds abort | `set_firing_order` and `transmission_new` copy the vec (restoring Rust's move semantics); the engine's event loops are bounded by their own lengths |
| RPM / load parameters | Out-of-range values, NaN | `ghurni_finite_clamp` maps non-finite to the range's off end, and `ghurni_rpm_gain` sinks NaN to silence. Belt tension and pan check separately, because `lo` is the loud or hard-left end of those ranges |
| DC blocker coefficient | Oscillation at low sample rates | Pole radius clamped to [0.9, 0.9999] |
| Buffer lengths | Mismatched stereo buffers | `min()` of the two lengths |
| Serde deserialization | Crafted JSON overrunning the buffer, or planting a non-finite or unstable value in filter state | Hand-written strict readers: every read stops at the NUL, each key exactly once, integers only, nothing after the closing brace. A kind outside `GH_EVENT_*`, a non-finite state, an `r` outside the constructor's own clamp and a `coeff` outside [0, 1] are refused with `GH_ERR_INVALID_PARAMETER` |
| Name strings | Null or an error code handed to a `*_from_name` | Each checks `GHURNI_MIN_VALID_PTR` and returns its not-found value |
| Output buffers, event and vec arguments | A failed `synthesize` or parse returns a negative code, so passing it on is reachable | Every `process_block`, `trigger_event`, `set_firing_order` and `transmission_new` checks `GHURNI_MIN_VALID_PTR` |
| Firing offsets and gear ratios | A finite 1e60 offset, or a ratio of 0, reaching phase or a smoother | Offsets past ±1e6 degrees or non-finite leave the order unchanged; an unusable ratio leaves the shaft undriven |
| Feedback state | One non-finite sample latching in a smoother or the DC blocker for life | Both recover on the next sample |

### Known limitations

- **A `(GH_KIND_*, pointer)` pair must MATCH, and nothing can check that.** The
  kind tag declares which struct the pointer addresses; registering a `GhGear`
  under `GH_KIND_ENGINE` makes the dispatcher read `GhEngine` fields off a
  smaller allocation. `add_channel` range-checks the tag and rejects non-pointers
  below the null page, but it cannot verify that a given pointer is of the
  claimed type, and an arbitrary wild pointer above the floor is undefined
  behaviour. This is inherent to ADR-004: the oracle's `Box<dyn Synthesizer>`
  carried its own vtable, and a raw `i64` does not. Consumers must pair the tag
  with the synth that produced it.
- **Synth handles are the caller's to check.** `ghurni_engine_set_rpm(e, ...)` with
  `e` left holding a failed constructor's negative code faults, because guarding
  `self` on all of the hot accessors is what `ghurni_is_err` is for (ADR-004). The
  all-zero sweep puts those faults in the null page.
- **A refused allocation is fail-stop.** Decided in
  [ADR-011](docs/architecture/adr-011-allocation-failure.md), on measurement: the
  process ends, and it never returns silently wrong audio. A consumer cannot
  survive heap exhaustion inside ghurni. It could not in Rust either.
- **Two finite gains can overflow.** Channel and master gain have no upper bound,
  so 1e200 × 1e200 renders non-finite output for as long as they stay set. Nothing
  persists: it clears when the gain does.
- **Log messages must be NUL-terminated cstrings.** `ghurni_log_*` forwards
  `(ptr, strlen(ptr))` to sakshi; a null or unterminated pointer faults. This is
  the ecosystem-wide cstring convention, and it is not reachable through
  ghurni's own API — every message ghurni logs is a literal.
- **No `free`.** Nothing in the library releases memory; synth objects and
  one-shot buffers live for the life of the allocator. Long-running consumers
  should reuse synths and stream with `process_block` rather than calling
  `synthesize` in a loop.

## Reporting Vulnerabilities

Please report security issues via
[GitHub Security Advisories](https://github.com/MacCracken/ghurni/security/advisories/new).
Do not open public issues for security vulnerabilities.

## Dependencies

Resolved by `cyrius deps` from `cyrius.cyml` and vendored into `lib/`, each
pinned to a git tag and recorded with its commit hash in `cyrius.lock`.

| Dependency | Version | Purpose | Risk |
|------------|---------|---------|------|
| [naad](https://github.com/MacCracken/naad) | 2.3.3 | DSP primitives (oscillators, filters, noise, additive) | Same ecosystem; the only backend |
| [hisab](https://github.com/MacCracken/hisab) | 3.4.1 | Math / geometry, referenced by naad's bundle | Same ecosystem; pure computation |
| [goonj](https://github.com/MacCracken/goonj) | 2.3.1 | Acoustics, referenced by naad's bundle | Same ecosystem; pure computation |
| [sakshi](https://github.com/MacCracken/sakshi) | 2.5.10 | Structured logging | Same ecosystem; writes to stderr |
| cyrius toolchain | 6.7.18 | Compiler, pinned in `[package].cyrius` | Build-time |

The pinned set is the one naad 2.3.3 itself pins, so the vendored bundles agree
with what each was built against. `cyrius deps --verify` re-checks the recorded
hashes.

> **Toolchain note.** `dist/ghurni.cyr` is compiled by the *consumer's*
> toolchain. cyrius ≤ 6.5.27 silently miscompiled decimal float literals past
> ~9 significant digits, which corrupted an audio constant in ghurni 2.0.0.
> ghurni now stores such constants as IEEE-754 bit patterns so an older
> consumer toolchain cannot reintroduce it, but consumers should still be on
> cyrius ≥ 6.5.28. Building the pinned set above needs cyrius ≥ 6.7.2; older
> toolchains fail at compile time.
