# ghurni

**ghurni** (Sanskrit: घूर्णि — rotation / spinning) — Mechanical sound synthesis for AGNOS, written in [Cyrius](https://github.com/MacCracken/cyrius).

Procedural synthesis of engines, gears, motors, turbines, clocks, transmissions, differentials, forced induction, and belt/chain drives. Every sound is driven by rotational physics — RPM determines firing frequency, tooth mesh rate, blade pass frequency, and escapement timing. Built on [naad](https://github.com/MacCracken/naad) for audio synthesis primitives.

> ghurni 2.x is the Cyrius port of the original Rust crate (1.0.0). That crate
> served as the parity oracle and was retired in 2.0.4 — it is recoverable from
> git at tag `2.0.3`. See [ADR-004](docs/architecture/adr-004-cyrius-port.md).

## Features

- **Engine** — 4 types (Gasoline, Diesel, TwoStroke, Hybrid — Hybrid is a true electric drive, Diesel has its own injection clatter), combustion impulses, 1–16 cylinders, custom firing order, and the full event set: startup / shutdown / stall / rev-limiter / backfire / misfire / knock / decel-pop
- **A real exhaust** — the combustion pulse train propagates down a **quarter-wave waveguide** and reaches the listener as the radiated field from the open end, so the engine booms where the physics says it should: firing harmonics sweep through the pipe's fixed modes as RPM changes. `exhaust_resonance` sets the pipe's fundamental (`f1 = c/4L`), not a filter centre. Intake is still a resonant bandpass ([ADR-010](docs/architecture/adr-010-radiation-paths.md))
- **Gear** — 4 materials (Steel, CastIron, Brass, Nylon), tooth mesh frequency, resonant ringing, material-specific decay and brightness
- **Motor** — 4 types (DcBrushed, AcInduction, Brushless, Servo), electromagnetic hum harmonics, commutator/bearing noise, pole-count-driven frequency
- **Turbine** — configurable blade count, blade pass frequency, whoosh noise, optional duct resonance
- **Clock** — 4 types (Wristwatch, WallClock, GrandfatherClock, PocketWatch), escapement tick, resonant decay
- **Drivetrain** — Transmission (gear mesh + synchro whine on shift), Differential (hypoid whine + housing resonance, drive/coast flank, ring-revolution warble), ChainDrive (link engagement rattle + per-lap wobble), BeltDrive (friction squeal + flap). Gear, Transmission and Differential all take load.
- **ForcedInduction** — turbocharger (spool lag) and supercharger, with blow-off-valve burst
- **Mixer** — multi-component mixing with per-channel gain, pan (equal-power law), and mute; mono and stereo
- **Presets** — 12 shipped factory configurations
- **Physically-grounded controls** — RPM drives loudness by one shared law
  ([ADR-005](docs/architecture/adr-005-rpm-loudness-law.md)); load drives both
  level *and* brightness ([ADR-007](docs/architecture/adr-007-load-tilt-and-diesel.md)).
  Every parameter clamps, and non-finite input never reaches your audio.

## Quick Start

```sh
cyrius deps                                   # resolve dependencies into lib/
cyrius build src/main.cyr build/ghurni        # compile the smoke entry
./build/ghurni
```

Using ghurni from a consumer program — include the dependency bundles, then `dist/ghurni.cyr`:

```cyrius
include "lib/sakshi.cyr"
include "lib/hisab.cyr"
include "lib/goonj.cyr"
include "lib/naad.cyr"
include "dist/ghurni.cyr"

fn main() {
    alloc_init();

    # A V8 gasoline engine at 44.1 kHz.
    var engine = ghurni_engine_new(GH_ENGINE_GASOLINE, 8, f64_from(44100));
    if (ghurni_is_err(engine) == 1) { return 1; }

    # One-shot: 1 s at 3000 RPM, 60% load.
    var samples = ghurni_engine_synthesize(engine, f64_from(3000), 0.6, F64_ONE);
    if (ghurni_is_err(samples) == 1) { return 1; }

    print_num(vec_len(samples));   # 44100
    return 0;
}

var r = main();
syscall(60, r);
```

Runnable programs live in [`docs/examples/`](docs/examples/):

```sh
cyrius build docs/examples/simple_engine.cyr build/ex_simple_engine && ./build/ex_simple_engine
```

## Error handling

There are no panics and no exceptions. Fallible functions return either a heap
pointer (success) or a negative `GH_ERR_*` code, distinguished by
`ghurni_is_err`:

| Code | Meaning |
|------|---------|
| `GH_ERR_NONE` (0) | success |
| `GH_ERR_INVALID_PARAMETER` (−1) | a parameter is out of range |
| `GH_ERR_SYNTHESIS_FAILED` (−2) | a naad backend object failed to construct |
| `GH_ERR_COMPUTATION` (−3) | a computation produced an invalid result |

`ghurni_err_name(code)` gives the diagnostic string. Always check
`ghurni_is_err` on a constructor result before using it — passing a failed
constructor's result to another function is rejected, not dereferenced.

## Performance

Measured by `cyrius bench` on the reference machine (see [`benches/ghurni.bcyr`](benches/ghurni.bcyr)); re-measure rather than trusting these numbers:

| Path | Cost |
|------|------|
| `ghurni_dcblocker_process` | ~20 ns / sample |
| `ghurni_smooth_next_value` | ~15 ns / sample |
| `ghurni_engine_synthesize` (gasoline 4-cyl, 0.05 s @ 44.1 kHz) | ~1.17 ms (**~43× real-time**) |
| `ghurni_gear_synthesize` (steel, 0.05 s @ 44.1 kHz) | ~0.72 ms |
| `ghurni_mixer_process_block` (4 channels × 512 samples) | ~1.15 ms |

Synthesis is deterministic: identical parameters and call sequence give
bit-identical output ([ADR-003](docs/architecture/adr-003-deterministic-synthesis.md)).

## Dependencies

Resolved from `cyrius.cyml` by `cyrius deps` into `lib/`. All are pinned to the coherent set naad itself pins.

| Dependency | Version | Role |
|------------|---------|------|
| [naad](https://github.com/MacCracken/naad) | 2.2.2 | audio synthesis primitives (oscillators, filters, noise, additive) |
| [hisab](https://github.com/MacCracken/hisab) | 2.11.2 | math / geometry (referenced by naad's bundle) |
| [goonj](https://github.com/MacCracken/goonj) | 2.0.4 | acoustics engine (referenced by naad's bundle) |
| [sakshi](https://github.com/MacCracken/sakshi) | 2.4.11 | structured logging |
| cyrius toolchain | 6.5.36 | pinned in `[package].cyrius` |

## Consumers

- **kiran** — AGNOS game engine
- **joshua** — game manager / simulation
- **dhvani** — AGNOS audio engine

## Documentation

- [Architecture overview](docs/architecture/overview.md)
- [Integration guide](docs/development/integration-guide.md)
- [Testing guide](docs/guides/testing.md)
- [Live port state](docs/development/state.md)
- [ADRs](docs/architecture/)

## License

GPL-3.0-only
