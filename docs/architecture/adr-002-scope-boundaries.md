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

> Terminology: this ADR was written for the Rust crate and says "crate"; the
> shipping units are Cyrius libraries consumed as `dist/*.cyr` bundles.

## Consequences
- ghurni does NOT implement Doppler, reverb, or spatialization
- ghurni exposes each synth as a `(GH_KIND_*, pointer)` pair plus the tag-dispatch
  helpers `ghurni_synth_process_block` / `ghurni_synth_set_rpm`, which is what an
  external mixer integrates against. (The Rust 1.x `Synthesizer` trait has no
  Cyrius equivalent — Cyrius has no trait objects. See [ADR-004](adr-004-cyrius-port.md).)
- MechanicalMixer is a convenience; production mixing belongs in dhvani
