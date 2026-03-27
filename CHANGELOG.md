# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
