# ghurni — Claude Code Instructions

## Project Identity

**ghurni** (Sanskrit: rotation / spinning) — Mechanical sound synthesis for AGNOS

- **Type**: Flat library crate
- **License**: GPL-3.0
- **MSRV**: 1.89
- **Version**: SemVer 1.0.0

## Consumers

kiran (game engine), joshua (game manager), dhvani (audio engine), and any AGNOS component needing mechanical/vehicle audio.

## Dependencies

- **naad**: Audio synthesis primitives (oscillators, filters, envelopes, noise generators, effects)

## Key Principles

- Never skip benchmarks
- `#[non_exhaustive]` on ALL public enums
- `#[must_use]` on all pure functions
- `#[inline]` on hot-path sample processing functions
- Every type must be Serialize + Deserialize (serde)
- Zero unwrap/panic in library code
- All types must have serde roundtrip tests
- RPM is the fundamental parameter — everything derives from rotational speed

## DO NOT

- **Do not commit or push** — the user handles all git operations
- **NEVER use `gh` CLI** — use `curl` to GitHub API only
- Do not add unnecessary dependencies
- Do not skip benchmarks before claiming performance improvements
