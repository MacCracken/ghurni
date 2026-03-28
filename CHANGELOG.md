# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-03-28

### Changed

- **Breaking**: All constructors now take `sample_rate: f32` and return `Result<Self>`
- **Breaking**: `synthesize()` methods no longer take `sample_rate` (stored in struct)
- Replaced hand-rolled DSP with naad primitives (oscillators, filters, noise generators, additive synthesis)
- Engine exhaust uses `BiquadFilter` bandpass for resonance shaping
- Motor EM hum uses `AdditiveSynth` (3 partials) instead of manual sin loops
- Turbine blade pass uses `AdditiveSynth` (2 partials), duct uses `Oscillator`
- Gear resonance uses `BiquadFilter` for material coloring
- Clock tick uses `BiquadFilter` for body resonance

### Added

- **`Synthesizer` trait** — common interface (`process_block`, `set_rpm`, `rpm`, `sample_rate`) enabling generic composition, mixers, and wrappers. Implemented by all synthesizer types.
- **`MechanicalMixer`** — multi-component mixer with per-channel gain, pan, mute. Supports `process_block` (mono) and `process_block_stereo` (equal-power pan law).
- **`MechanicalEvent` enum** — discrete event triggers: Backfire, Misfire, Knock, Stall, RevLimiterHit, GearShift, Startup, Shutdown.
- **`SmoothedParam`** — one-pole exponential parameter smoother for click-free RPM/load transitions.
- **`DcBlocker`** — one-pole highpass DC blocker applied to all synthesis output.
- **Engine enhancements**:
  - Multi-cylinder firing order — per-cylinder crank-angle offsets (V8 burble vs inline-4 drone)
  - `set_firing_order()` for custom firing patterns (e.g., cross-plane V8)
  - Intake manifold Helmholtz resonance — separate BiquadFilter path
  - Deceleration crackle/pop — stochastic impulses on sharp load drop at high RPM
  - Load-dependent timbre — roughness, harmonic content scale with load
  - `trigger_event()` for backfire, misfire, knock events
  - Parameter smoothing on RPM and load
- **New synthesizer types**:
  - `ForcedInduction` — turbocharger (spool lag, wastegate) and supercharger (direct drive) with blow-off valve burst
  - `Transmission` — gear mesh at current ratio, synchronizer whine during shifts, `shift_to()` method
  - `Differential` — hypoid gear whine with housing resonance
  - `ChainDrive` — periodic link engagement rattle on sprocket teeth
  - `BeltDrive` — friction squeal and belt flap with tension control
- **Preset system** (`presets` module) — shipped factory presets: `v8_muscle_car`, `inline4_economy`, `diesel_truck`, `motorcycle_single`, `electric_vehicle`, `turbocharger`, `supercharger`, `manual_5speed`, `manual_6speed`, `steel_spur_gear`, `industrial_turbine`, `propeller`
- `naad-backend` feature flag (default on) — enables naad DSP primitives
- `process_block(&mut self, output: &mut [f32])` streaming API on all synthesizers
- Real-time parameter setters: `set_rpm()`, `set_load()`, `set_tension()`, `shift_to()`
- `sample_position` tracking for seamless streaming across `process_block()` calls
- `ComputationError` variant on `GhurniError`
- `dsp` module with `DcBlocker`, `validate_sample_rate`, `validate_duration`
- Fallback path when `naad-backend` is disabled (original math.rs + rng.rs)
- 44 integration tests: all types, events, presets, mixer, trait dispatch, parameter sweeps, continuity, serde roundtrips
- Benchmarks: block size sweep (64-4096), mixer, turbocharger, transmission

### Removed

- Direct `hisab` dependency (unused; naad depends on it transitively)

## [0.1.0] - 2026-03-27

### Added

- Initial scaffold of the ghurni crate
- **Engine**: 4 types (Gasoline, Diesel, TwoStroke, Hybrid) with combustion pulses, exhaust resonance, mechanical noise. RPM-driven firing frequency with cylinder count
- **Gear**: 4 materials (Steel, CastIron, Brass, Nylon) with tooth mesh frequency, resonant ringing, material-specific decay and brightness
- **Motor**: 4 types (DcBrushed, AcInduction, Brushless, Servo) with electromagnetic hum harmonics, commutator/bearing noise, pole-count-driven frequency
- **Turbine**: Blade pass frequency synthesis with harmonic content, whoosh noise, optional duct resonance
- **Clock**: 4 types (Wristwatch, WallClock, GrandfatherClock, PocketWatch) with escapement tick, resonant decay, type-specific frequency and amplitude
- `GhurniError` with serde roundtrip
- PCG32 PRNG for stochastic mechanical noise
- Integration tests: all engine/gear/motor/clock types, firing/mesh frequency verification, energy comparison, serde roundtrips
- Criterion benchmarks: V8 gasoline, diesel 6-cyl, steel gear, brushless motor, turbine, wristwatch
- `no_std` support via `libm` + `alloc`
- Strict `deny.toml` matching hisab production patterns
- Send/Sync compile-time assertions on all public types
