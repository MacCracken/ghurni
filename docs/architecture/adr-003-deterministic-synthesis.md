# ADR-003: Deterministic Synthesis

## Status
Accepted

## Context
Game engines may need reproducible audio for replays, testing, and debugging. Non-deterministic synthesis (system RNG, timing-dependent state) makes this impossible.

## Decision
All stochastic processes use seeded PRNGs derived from constructor parameters. The same inputs always produce the same output. Seeds are computed from enum discriminants, tooth counts, cylinder counts, etc.

## Consequences
- Bit-identical output given same parameters and call sequence
- No dependency on system entropy
- Golden-file tests are possible (hash comparison)
- naad's NoiseGenerator seeds must also be deterministic (they are — seeded from constructor params)
