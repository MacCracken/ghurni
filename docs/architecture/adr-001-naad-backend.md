# ADR-001: naad as Optional DSP Backend

## Status
Accepted

## Context
ghurni needs oscillators, filters, noise generators, and envelope shaping for mechanical sound synthesis. Options: hand-roll everything, depend on naad, or use a third-party DSP crate.

## Decision
Use naad as an optional dependency behind the `naad-backend` feature flag (default on). Maintain a fallback path using libm + internal PCG32 RNG for no_std environments without naad.

## Consequences
- naad types stored as owned `#[cfg]`-gated struct fields
- Dual implementation in every synthesizer (naad + fallback)
- Code duplication between paths, but clear separation
- Consumers can disable naad for minimal dependency footprint
