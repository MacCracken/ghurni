# Architecture Overview

ghurni is a Cyrius library. It has no `include` lines between its own modules —
the entry file, the test harness, or the `dist/ghurni.cyr` bundle orders them.
The order below is the dependency order declared in `cyrius.cyml [lib].modules`.

## Module Structure

```
ghurni/
├── src/                       19 modules, ~3,140 lines
│   │  -- L0 foundations (no internal deps beyond each other, in this order)
│   ├── error.cyr              GH_ERR_* codes, f64 sentinels, ghurni_fmod, ghurni_vec_copy
│   ├── logging.cyr            sakshi-backed structured logging (replaces `tracing`)
│   ├── dsp.cyr                GhDcBlocker, sample-rate/duration validation,
│   │                          ghurni_sample_count, naad error map
│   ├── smooth.cyr             GhSmoothedParam (one-pole click-free ramps)
│   ├── event.cyr              MechanicalEvent kinds + payloads + serde names
│   ├── traits.cyr             GH_KIND_* dispatch tags (Synthesizer replacement)
│   │
│   │  -- L1: the ten RPM-driven synths, each self-contained on naad + L0
│   ├── engine.cyr             combustion engines (4 types, firing order, events)
│   ├── gear.cyr               gear mesh (4 materials)
│   ├── motor.cyr              electric motors (4 types)
│   ├── turbine.cyr            turbines / fans / propellers
│   ├── clock.cyr              clock mechanisms (4 types)
│   ├── transmission.cyr       gearbox with shift transients
│   ├── differential.cyr       hypoid whine + housing resonance
│   ├── forced_induction.cyr   turbocharger / supercharger
│   ├── belt_drive.cyr         belt squeal and flap
│   ├── chain_drive.cyr        chain link engagement
│   │
│   │  -- L2 composites
│   ├── mixer.cyr              GhMixer: tag-dispatched multi-synth mixer
│   ├── presets.cyr            12 shipped factory presets
│   └── main.cyr               smoke entry
│
├── dist/ghurni.cyr            the consumer bundle (cyrius distlib)
├── lib/                       resolved dependency bundles (cyrius deps)
└── rust-old/                  the Rust 1.0.0 crate — parity oracle, never edited
```

`rust-old/src/` holds the 20 original `.rs` files. Two of them — `math.rs` and
`rng.rs` — are deliberately **not** ported; see below.

## Synthesizer Pattern

All ten synths follow the same shape:

1. **Constructor** — `ghurni_<synth>_new(params…, sample_rate)` → heap pointer, or a negative `GH_ERR_*`. Validates the sample rate and every enum id, then builds the naad DSP objects.
2. **One-shot** — `ghurni_<synth>_synthesize(self, params…, duration)` → a fresh vec of f64, or a negative `GH_ERR_*`. The sample count goes through `ghurni_sample_count`, which bounds it before the conversion.
3. **Streaming** — `ghurni_<synth>_process_block(self, out)` fills a caller-supplied vec in place and preserves state across calls.
4. **Real-time setters** — `ghurni_<synth>_set_rpm(self, rpm)` (and `_set_load`, `_set_tension`, `shift_to`, …) take effect on the next block, smoothed through `GhSmoothedParam`.
5. **Dispatch tag** — each synth has a `GH_KIND_*` constant so the mixer can hold a heterogeneous set.

## Single Backend

**naad is the only backend.** The Rust crate was feature-gated: a default
`naad-backend` path, and a fallback using a local PCG32 (`rng.rs`) plus libm
wrappers (`math.rs`). AGNOS always ships naad, so only the naad path is ported —
`rng.rs`, `math.rs` and every per-synth fallback loop are dropped, and naad's
`NoiseGenerator` owns all stochastic content. There is no feature flag and no
second code path. See [ADR-004](adr-004-cyrius-port.md); [ADR-001](adr-001-naad-backend.md),
which described the dual-path design, is superseded.

## Dispatch Without Trait Objects

Cyrius has no vtables and no dynamic dispatch, so Rust's
`Box<dyn Synthesizer>` is replaced by a `(GH_KIND_*, pointer)` pair plus a
hand-written switch in `mixer.cyr` (`ghurni_synth_process_block` /
`ghurni_synth_set_rpm`). That switch is the explicit equivalent of the trait's
vtable. `ghurni_mixer_add_channel` range-checks both the tag and the pointer, so
a channel can never hold an unknown kind or a failed constructor's error code.

## Data Flow

```
set_rpm(rpm) ──┐
set_load(load) ┤
               v
        GhSmoothedParam (one-pole exponential approach, per sample)
               │
               v
        process_block(out)
               │
               ├── naad: Oscillator / AdditiveSynth / NoiseGenerator / BiquadFilter
               │
               v
        GhDcBlocker (remove DC offset)
               │
               v
        sample_position += vec_len(out)
```

## Numeric Conventions

- **f64 everywhere.** naad and hisab are f64-only, so the Rust f32 widens. This is a precision improvement, but it is a real behavioural difference in a few places — see the divergence list in [state.md](../development/state.md).
- **High-precision float constants are stored as IEEE-754 bit patterns**, not decimal literals (`GHURNI_EPSILON`, `GHURNI_DB_SCALE`). `dist/ghurni.cyr` is compiled by the *consumer's* toolchain, and cyrius ≤ 6.5.27 miscompiled long decimal literals.
- **Signed i64 is the only integer type.** Where the oracle typed a parameter `usize`, `u32` or an enum, a bad value was unrepresentable in Rust and the Rust body validated nothing — so the port must add the bound Rust got from its type system. Where Rust's `as` casts saturate, `f64_to` overflows to `INT64_MIN`, so no `f64_to` result is ever used directly as a loop bound.

## AGNOS Ecosystem

| Library | Domain |
|---------|--------|
| **ghurni** | Mechanical sound (engines, gears, motors) |
| **garjan** | Environmental sound (weather, impacts, fire) |
| **svara** | Vocal / speech |
| **naad** | Audio synthesis primitives (oscillators, filters) |
| **goonj** | Sound propagation (distance, occlusion, Doppler) |
| **dhvani** | Audio engine (mixing, buses, spatialization) |
| **kiran** | Game engine integration |
