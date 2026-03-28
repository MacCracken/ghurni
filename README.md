# ghurni

**ghurni** (Sanskrit: घूर्णि — rotation / spinning) — Mechanical sound synthesis for Rust.

Procedural synthesis of engines, gears, motors, turbines, and clocks. All sounds driven by rotational physics — RPM determines firing frequency, tooth mesh rate, blade pass frequency, and escapement timing. Built on [naad](https://crates.io/crates/naad) for audio synthesis primitives.

## Features

- **Engine**: 4 types (Gasoline, Diesel, TwoStroke, Hybrid) with combustion impulses, exhaust resonance, cylinder count (1-16), RPM-driven firing frequency
- **Gear**: 4 materials (Steel, CastIron, Brass, Nylon) with tooth mesh frequency, resonant ringing, material-specific decay and brightness
- **Motor**: 4 types (DcBrushed, AcInduction, Brushless, Servo) with electromagnetic hum harmonics, commutator/bearing noise, pole-count-driven frequency
- **Turbine**: Configurable blade count (2-64), blade pass frequency, whoosh noise, optional duct resonance
- **Clock**: 4 types (Wristwatch, WallClock, GrandfatherClock, PocketWatch) with escapement tick, resonant decay, type-specific timing
- **Performance**: ~1,000x real-time, `no_std` compatible, all types `Send + Sync`

## Quick Start

```rust
use ghurni::prelude::*;

let mut engine = Engine::new(EngineType::Diesel, 6, 44100.0).unwrap();
let samples = engine.synthesize(2000.0, 0.7, 1.0).unwrap();

let mut clock = Clock::new(ClockType::GrandfatherClock, 44100.0).unwrap();
let samples = clock.synthesize(5.0).unwrap();
```

## Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `std` | Yes | Standard library. Disable for `no_std` + `alloc` |
| `naad-backend` | Yes | Use naad for DSP primitives (oscillators, filters, noise) |
| `logging` | No | Structured logging via tracing-subscriber |

## Consumers

- **kiran** — AGNOS game engine
- **joshua** — Game manager / simulation
- **dhvani** — AGNOS audio engine

## License

GPL-3.0-only
