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
negative `GH_ERR_*` code instead.

## Attack Surface

| Area | Risk | Mitigation |
|------|------|------------|
| Sample-rate validation | Division by zero, NaN propagation | `ghurni_validate_sample_rate` rejects ≤ 0, NaN, ±inf |
| Duration validation | Unbounded allocation, process abort | `ghurni_validate_duration` rejects ≤ 0, NaN, ±inf |
| `duration × sample_rate` | `f64_to` overflow to `INT64_MIN`; vec growth past the stdlib cap aborts the process | `ghurni_sample_count` compares in f64 *before* the conversion and caps at `GHURNI_MAX_SAMPLES` (2^26) |
| Enum-id parameters | Out-of-range id falls through to a wrong default | Every constructor range-checks its id and returns `GH_ERR_INVALID_PARAMETER` |
| `shift_to` gear index | Negative index silently corrupts state | Both bounds checked |
| Mixer channel synth | A failed constructor's error code or null dereferenced as a struct (SIGSEGV) | `add_channel` rejects `ghurni_is_err(synth)` and `synth == 0`; the dispatch loops skip an absent synth |
| Mixer kind tag | Type confusion between synth structs | `add_channel` range-checks the `GH_KIND_*` tag |
| Caller-owned vecs | Caller mutating a stored vec desynchronises lengths → out-of-bounds abort | `set_firing_order` and `transmission_new` copy the vec (restoring Rust's move semantics); the engine's event loops are bounded by their own lengths |
| RPM / load parameters | Out-of-range values | Clamped to valid ranges |
| DC blocker coefficient | Oscillation at low sample rates | Pole radius clamped to [0.9, 0.9999] |
| Buffer lengths | Mismatched stereo buffers | `min()` of the two lengths |
| Serde deserialization | Crafted JSON injecting non-finite f64 bit patterns into filter state | `GhDcBlocker` / `GhSmoothedParam` store f64 bit patterns as i64; a hostile value yields non-finite audio, not memory unsafety. Callers deserializing untrusted state should validate with `ghurni_is_finite` |

### Known limitations

- **`alloc()` results are not null-checked.** The library assumes the bump
  allocator succeeds; there is no allocation-failure path anywhere in the
  Cyrius port, and none of the sibling libraries have one either. Under memory
  exhaustion a constructor would write through a null base pointer.
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
| [naad](https://github.com/MacCracken/naad) | 2.2.1 | DSP primitives (oscillators, filters, noise, additive) | Same ecosystem; the only backend |
| [hisab](https://github.com/MacCracken/hisab) | 2.11.2 | Math / geometry, referenced by naad's bundle | Same ecosystem; pure computation |
| [goonj](https://github.com/MacCracken/goonj) | 2.0.4 | Acoustics, referenced by naad's bundle | Same ecosystem; pure computation |
| [sakshi](https://github.com/MacCracken/sakshi) | 2.4.11 | Structured logging | Same ecosystem; writes to stderr |
| cyrius toolchain | 6.5.35 | Compiler, pinned in `[package].cyrius` | Build-time |

The pinned set is the one naad 2.2.1 itself pins, so the vendored bundles agree
with what each was built against. `cyrius deps --verify` re-checks the recorded
hashes.

> **Toolchain note.** `dist/ghurni.cyr` is compiled by the *consumer's*
> toolchain. cyrius ≤ 6.5.27 silently miscompiled decimal float literals past
> ~9 significant digits, which corrupted an audio constant in ghurni 2.0.0.
> ghurni now stores such constants as IEEE-754 bit patterns so an older
> consumer toolchain cannot reintroduce it, but consumers should still be on
> cyrius ≥ 6.5.28.
