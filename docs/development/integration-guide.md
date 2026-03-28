# Integration Guide

## One-Shot Synthesis

Allocates a buffer and returns it:

```rust
let mut engine = Engine::new(EngineType::Diesel, 6, 44100.0)?;
let samples = engine.synthesize(2000.0, 0.7, 1.0)?;
```

## Streaming (Real-Time)

Fill a caller-provided buffer repeatedly:

```rust
let mut engine = Engine::new(EngineType::Gasoline, 4, 44100.0)?;
engine.set_rpm(3000.0);
engine.set_load(0.6);

let mut buffer = vec![0.0f32; 512]; // ~11.6ms at 44100
loop {
    engine.process_block(&mut buffer);
    // Send buffer to audio output...

    // Update parameters between blocks
    engine.set_rpm(new_rpm);
    engine.set_load(new_load);
}
```

## Parameter Changes

Parameters set via `set_rpm()` / `set_load()` are smoothed internally via `SmoothedParam` to avoid clicks. Changes take effect gradually over the next block.

## Event Triggers

Discrete events (backfire, misfire, knock) are triggered and processed in the next `process_block`:

```rust
engine.trigger_event(MechanicalEvent::Backfire);
engine.trigger_event(MechanicalEvent::Misfire { cylinder: 0 });
```

## Multi-Component Mixing

Use `MechanicalMixer` to combine sources:

```rust
let mut mixer = MechanicalMixer::new();
mixer.add_channel("engine".into(), Box::new(engine));
mixer.add_channel("turbo".into(), Box::new(turbo));
mixer.set_rpm(3000.0); // Sets RPM on all channels
mixer.process_block_stereo(&mut left, &mut right);
```

## Presets

Factory presets create pre-configured synthesizers:

```rust
let mut engine = ghurni::presets::v8_muscle_car(44100.0)?;
let mut trans = ghurni::presets::manual_5speed(44100.0)?;
```
