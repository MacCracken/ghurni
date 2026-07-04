# ADR-004: Port ghurni from Rust to Cyrius

**Status**: Accepted
**Date**: 2026-07-04
**Context**: ghurni is being ported to the Cyrius language for AGNOS, alongside
its sibling libraries (naad, prani, svara, …). The Rust crate (1.0.0) is
preserved at `rust-old/` as the parity oracle.

## Decision

Port the **naad-backend path only**, to f64, with integer error codes and
tag-based synth dispatch, mirroring the conventions of the already-ported
siblings (naad, prani). Bump to **2.0.0** on full parity.

## Rationale & consequences

1. **naad is the only backend.** The Rust crate was feature-gated: a default
   `naad-backend` path and a fallback that used a local PCG32 (`rng.rs`) + libm
   wrappers (`math.rs`). Since AGNOS always ships naad, only the naad path is
   ported; `rng.rs` / `math.rs` and every per-synth fallback loop are dropped.
   naad's `NoiseGenerator` owns the stochastic content.

2. **f32 → f64 throughout.** naad and hisab are f64-only, so widening is forced
   and is a precision improvement. Decimal float literals lex to f64 bit
   patterns; the `f64_*` builtins do the arithmetic. The Rust tests asserted
   *behaviour* (finite output, non-zero energy, energy ordering, frequency
   math) rather than exact samples, so parity is preserved without bit-exactness.

3. **Integer error codes.** Cyrius has no `Result<T, String>`; the `GhurniError`
   enum becomes negative `GH_ERR_*` codes. Fallible functions return a heap
   pointer (success) or a negative code (failure), distinguished by
   `ghurni_is_err`. naad's own constructors follow the same convention, so a
   failed `filter_biquad_new` is caught and mapped to `GH_ERR_SYNTHESIS_FAILED`
   (ports the Rust `.map_err(GhurniError::SynthesisFailed)`).

4. **Tag dispatch, not trait objects.** Cyrius has no vtables or dynamic dispatch
   (Cyrius ADR-004: method dispatch is convention-based and compile-time). The
   Rust `Box<dyn Synthesizer>` (mixer channels + the heterogeneous dispatch test)
   is replaced by a `(GH_KIND_*, pointer)` pair and a hand-written tag-dispatch
   switch (`ghurni_synth_process_block` / `_set_rpm` in `mixer.cyr`). This is the
   explicit equivalent of the trait's vtable.

5. **Serde parity where it is meaningful.** Public enums map to `GH_<ENUM>_*`
   integers with `name`/`from_name` helpers that use serde's externally-tagged
   variant names — so the enum roundtrip tests port directly. POD structs
   (DcBlocker, SmoothedParam, MechanicalEvent) get `#derive(Serialize)`. The
   synth structs hold opaque naad backend pointers; the Rust already `#[serde(skip)]`ped
   the live backend + scratch fields, so deep serialization of a synth carried no
   meaningful state and is dropped.

## Notes for future edits

- **Never put an inline `#` comment inside a `struct { }` body** — the parser
  breaks. Document fields in a comment block above the struct.
- All ghurni symbols are `ghurni_` / `GHURNI_` / `Gh`-prefixed so the distlib
  bundle coexists with naad's flat namespace.
- Cross-check every change against `rust-old/`; diverge only with a new ADR.
