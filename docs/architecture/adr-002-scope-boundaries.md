# ADR-002: Scope Boundaries

## Status
Accepted

## Context
The AGNOS ecosystem has multiple audio crates. Clear boundaries prevent overlap and keep each crate focused.

## Decision

| ghurni owns | Another crate owns |
|---|---|
| Mechanical sound synthesis | Environmental sound — **garjan** |
| Rotational/RPM-driven sources | Vocal/speech — **svara** |
| Raw parameter API (RPM, load) | RTPC mapping — **dhvani/kiran** |
| Individual synthesizers | Mixing/buses — **dhvani** |
| DC-blocked mono output | Spatialization/Doppler — **goonj** |

## Consequences
- ghurni does NOT implement Doppler, reverb, or spatialization
- ghurni provides the `Synthesizer` trait for integration with external mixers
- MechanicalMixer is a convenience; production mixing belongs in dhvani
