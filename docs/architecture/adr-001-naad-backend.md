# ADR-001: naad as Optional DSP Backend

## Status

**Superseded by [ADR-004](adr-004-cyrius-port.md)** (2026-07-04).

Accepted for the Rust crate (1.x). The decision below describes the Rust oracle
(retired in 2.0.4; recoverable at tag `2.0.3`) — **not** the shipping Cyrius
library. In ghurni 2.x, naad is the *only* backend: there is no feature flag, no
fallback path, and no dual implementation. Read this ADR as history.

## Context

ghurni needs oscillators, filters, noise generators, and envelope shaping for mechanical sound synthesis. Options: hand-roll everything, depend on naad, or use a third-party DSP crate.

## Decision (Rust 1.x)

Use naad as an optional dependency behind the `naad-backend` feature flag (default on). Maintain a fallback path using libm + internal PCG32 RNG for no_std environments without naad.

## Consequences (Rust 1.x)

- naad types stored as owned `#[cfg]`-gated struct fields
- Dual implementation in every synthesizer (naad + fallback)
- Code duplication between paths, but clear separation
- Consumers can disable naad for minimal dependency footprint

## Why it was superseded

AGNOS always ships naad, so the fallback was dead weight that doubled the
surface needing parity tests. ADR-004 ports the naad path only:

- `rust-old/src/rng.rs` (PCG32) and `rust-old/src/math.rs` (libm wrappers) are **not** ported. naad's `NoiseGenerator` owns all stochastic content.
- Every per-synth `#[cfg(not(feature = "naad-backend"))]` loop is dropped.
- There are no feature flags in the Cyrius library at all.
- A naad constructor failure is caught and mapped to `GH_ERR_SYNTHESIS_FAILED`, porting the Rust `.map_err(GhurniError::SynthesisFailed)`.

The one consequence that outlived the ADR: because naad is f64-only, the port is
f64 throughout where the oracle was f32. That is recorded in ADR-004 and in the
divergence list in [state.md](../development/state.md).
