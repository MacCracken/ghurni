# ghurni — Live Port State

> Volatile state (version, module sizes, coverage). Durable rules live in
> [`CLAUDE.md`](../../CLAUDE.md). Refreshed at each milestone.

## Version

**2.0.0** — full-parity Cyrius port. (1.0.0 was the final Rust crate, preserved at `rust-old/`.)

## Port status: COMPLETE

All 20 Rust source modules ported; all 44 Rust integration tests reproduced as
behavioural-parity `.tcyr` suites; smoke binary + benchmarks green.

## Modules (`src/*.cyr`, 2,967 lines)

| Layer | Modules |
|-------|---------|
| L0 foundations | `error` `logging` `dsp` `smooth` `event` `traits` |
| L1 synths | `engine` `gear` `motor` `turbine` `clock` `transmission` `differential` `forced_induction` `belt_drive` `chain_drive` |
| L2 composites | `mixer` `presets` |
| entry | `main` (smoke) |

Bundle: `dist/ghurni.cyr` (2,904 lines, `cyrius distlib`).

## Tests (`tests/*.tcyr`, 6 suites, 135 assertions, 0 failures)

| Suite | Covers |
|-------|--------|
| `foundations` | error codes/helpers, validation, DcBlocker, SmoothedParam, MechanicalEvent + serde |
| `engine` | all engine integration tests + firing frequency + param sweep + EngineType serde |
| `synths` | gear/motor/turbine/clock/transmission/differential/forced_induction/belt/chain + gear sweep |
| `serde` | EngineType/GearMaterial/MotorType/ClockType/InductionType + GhurniError name mapping |
| `mixer` | mono/stereo/mute, trait dispatch, process-block continuity, presets |
| `smoke_all` | full-unit integration: every synth + mixer + preset produce finite audio |

`cyrius bench` (`benches/ghurni.bcyr`): DcBlocker ~20 ns/sample, smoother ~15 ns/sample, plus full engine/gear synthesis paths.

## Deliberate divergences from `rust-old/`

- **naad-only backend.** The Rust `#[cfg(not(feature="naad-backend"))]` fallback
  (`rng.rs`, `math.rs`, per-synth fallback loops) is not ported — naad's
  NoiseGenerator owns the randomness. See ADR-004.
- **f32 → f64 throughout** (naad/hisab are f64-only). Test tolerances loosened
  where f32 bit-exactness is not meaningful; audio parity is behavioural
  (finite / has-energy / energy-ordering), matching what the Rust tests asserted.
- **Integer error codes** replace the `GhurniError` String-payload enum;
  diagnostic text via `ghurni_err_name`.
- **Tag dispatch** replaces `Box<dyn Synthesizer>` trait objects (ADR-004).
- **Serde**: enums roundtrip via name↔code helpers; POD structs (DcBlocker,
  SmoothedParam, Event) via `#derive(Serialize)`; synth structs holding opaque
  naad pointers drop deep serialization (nothing meaningful survived the Rust
  `#[serde(skip)]` on the backend fields either).
- **RNG seeds** derived from parameters are behavioural, not bit-identical to the
  Rust `to_bits()` seeds (tests don't depend on exact sample values).

## Examples (`docs/examples/*.cyr`, 5 runnable programs)

`simple_engine`, `vehicle_scene`, `mixer_demo`, `error_handling`, `logging` —
each ports the matching `rust-old/examples/*.rs`, includes the dep bundles +
`dist/ghurni.cyr`, and builds/runs standalone:

```sh
cyrius build docs/examples/simple_engine.cyr build/ex_simple_engine && ./build/ex_simple_engine
```

## CI

`.github/workflows/{ci,release}.yml` use the cyrius toolchain (install from the
`cyrius.cyml [package].cyrius` pin → `cyrius deps` → `cyrius build` → `cyrius
test` → build examples; release verifies VERSION/tag consistency + ships a
source tarball + SHA256SUMS). Mirrors the prani / naad sibling CI.

## Known follow-ups

- None outstanding for parity. (Optional: a `CHANGELOG.md` 2.0.0 entry for the
  port, if you want the release workflow's changelog extraction to populate.)
