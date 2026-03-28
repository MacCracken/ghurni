# Architecture Overview

## Module Structure

```
ghurni/
├── src/
│   ├── lib.rs              # Public API, prelude, trait assertions
│   ├── traits.rs           # Synthesizer trait
│   ├── error.rs            # GhurniError enum
│   ├── event.rs            # MechanicalEvent enum
│   ├── smooth.rs           # SmoothedParam (click-free transitions)
│   ├── dsp.rs              # DcBlocker, validation helpers
│   ├── mixer.rs            # MechanicalMixer (multi-component mixing)
│   ├── presets.rs          # Factory preset configurations
│   │
│   ├── engine.rs           # Combustion engines (4 types, firing order)
│   ├── gear.rs             # Gear mesh (4 materials)
│   ├── motor.rs            # Electric motors (4 types)
│   ├── turbine.rs          # Turbines/fans/propellers
│   ├── clock.rs            # Clock mechanisms (4 types)
│   ├── forced_induction.rs # Turbocharger / supercharger
│   ├── transmission.rs     # Gearbox with shift transients
│   ├── differential.rs     # Differential whine
│   ├── chain_drive.rs      # Chain link engagement
│   ├── belt_drive.rs       # Belt squeal and flap
│   │
│   ├── math.rs             # no_std math (fallback, cfg-gated)
│   └── rng.rs              # PCG32 PRNG (fallback, cfg-gated)
```

## Synthesizer Pattern

Every synthesizer follows the same structure:

1. **Constructor**: `new(params, sample_rate) -> Result<Self>` — validates inputs, creates naad DSP objects
2. **One-shot**: `synthesize(params, duration) -> Result<Vec<f32>>` — allocates, calls process_block
3. **Streaming**: `process_block(output: &mut [f32])` — fills caller buffer, preserves state
4. **Real-time setters**: `set_rpm()`, `set_load()` — take effect on next process_block
5. **Trait impl**: `impl Synthesizer` — enables generic composition via MechanicalMixer

## Dual Code Paths

All synthesizers have two implementations behind `#[cfg(feature = "naad-backend")]`:

- **naad path** (default): Uses naad oscillators, filters, noise generators, additive synthesis
- **fallback path**: Uses internal `math.rs` (libm) and `rng.rs` (PCG32)

Both paths produce valid audio. The naad path provides higher quality (proper filtering, band-limited oscillators) while the fallback is dependency-free.

## Data Flow

```
set_rpm(rpm)  ──┐
set_load(load) ─┤
                v
         SmoothedParam (exponential approach)
                │
                v
         process_block(output)
                │
                ├── naad_path: Oscillator/AdditiveSynth/NoiseGenerator/BiquadFilter
                │
                └── fallback_path: sin/cos/exp (libm) + PCG32 noise
                │
                v
         DcBlocker (remove DC offset)
                │
                v
         sample_position += len
```

## AGNOS Ecosystem

| Crate | Domain |
|-------|--------|
| **ghurni** | Mechanical sound (engines, gears, motors) |
| **garjan** | Environmental sound (weather, impacts, fire) |
| **naad** | Audio synthesis primitives (oscillators, filters) |
| **goonj** | Sound propagation (distance, occlusion, Doppler) |
| **dhvani** | Audio engine (mixing, buses, spatialization) |
| **kiran** | Game engine integration |
