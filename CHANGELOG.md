# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.1] - 2026-08-27

Toolchain and dependency refresh. **ghurni's public API is unchanged** — the
naad renames below are internal to ghurni's own call sites. Audio output *does*
change for gear / clock / engine, but only because the toolchain bump repairs a
silently miscompiled constant (see **Fixed**); the change moves output toward
`rust-old/` parity, not away from it.

### Changed

- **Toolchain**: cyrius `6.4.2` → `6.5.35`.
- **Dependencies** bumped to the coherent set naad 2.2.1 itself pins:

  | dep | from | to |
  |-----|------|----|
  | naad | 2.1.0 | 2.2.1 |
  | hisab | 2.6.7 | 2.11.2 |
  | goonj | 2.0.0 | 2.0.4 |
  | sakshi | 2.4.3 | 2.4.11 |

  naad 2.2.1 pins hisab 2.11.2 + goonj 2.0.4; goonj 2.0.4 pins hisab 2.11.2;
  hisab 2.11.2 pins sakshi 2.4.11 — so the vendored bundles are mutually
  consistent rather than merely individually latest.
- **Vendored `lib/` bundles refreshed.** This is the coordinated consumer
  refresh naad 2.2.0 called for; ghurni was one of the five named copies.
- **Migrated onto naad's namespace wave** (naad 2.1.3 + 2.2.0 renamed bare
  top-level symbols to escape Cyrius's flat distlib namespace). Values and
  behaviour are unchanged — these are pure renames:

  ```
  FILTER_BANDPASS  -> NAAD_FILTER_BANDPASS   (10 call sites)
  db_to_amplitude  -> naad_db_to_amplitude   ( 4 call sites)
  ```

  `naad_db_to_amplitude` is body-identical to the old `db_to_amplitude`; it is
  *not* the separate approximate `db_to_amplitude_lut`.
- `dist/ghurni.cyr` regenerated (2,904 lines, v2.0.1).

### Fixed — `GHURNI_DB_SCALE` was silently miscompiled (audible)

⚠ **This changes gear / clock / engine audio output**, in the direction of
`rust-old/` parity. It is a toolchain fix, not a naad one.

`src/error.cyr` declared `GHURNI_DB_SCALE = 46.051701859880914` (20 / LOG10_E).
Cyrius **≤ 6.5.27** packed a decimal float literal as a 32/32 rational,
`(denom << 32) | (numer & 0xFFFFFFFF)`; past ~9 fractional digits *both* fields
overflowed and the literal **compiled clean to a different number** — here
`0.598756873065743`, 76.9× too small. Fixed upstream in **cyrius 6.5.28**,
which this release's 6.4.2 → 6.5.35 bump crosses.

The constant is a live per-sample audio value, not a diagnostic. It feeds
`naad_db_to_amplitude(-x * GHURNI_DB_SCALE)` = `10^(-x·S/20)` at four sites —
`gear.cyr:196` (mesh ring), `clock.cyr:203` / `clock.cyr:207` (tick body ring),
`engine.cyr:312` (combustion pulse). At `x = 1` the envelope was **0.9334**
(essentially undamped) where it should be **0.00498**: a 187× difference, so the
ring/pulse decay shaping was effectively absent.

Scope — this is a property of the **compiling** toolchain, not of the shipped
source. `dist/ghurni.cyr` always carried the correct decimal *text*, so anyone
who compiled it with cyrius ≥ 6.5.28 already had the right value and has nothing
to regenerate. Affected artifacts are those built by cyrius ≤ 6.5.27, which
includes ghurni's own committed `build/*` binaries at 2.0.0. A consumer still on
an older toolchain pin should regenerate any stored reference renders.

`GHURNI_DB_SCALE` is now stored as the IEEE-754 bit pattern
`0x4047069E2AA2AA5B` — the idiom `GHURNI_EPSILON` / `GHURNI_POS_INF` /
`GHURNI_NEG_INF` in the same file already use — so the value cannot be re-broken
by a consumer compiling the bundle on an older toolchain. It was the only
decimal float literal past the threshold anywhere in the link unit (ghurni
sources, all four dep bundles, and the stdlib leaves were swept; every other hit
is a comment beside a hex pattern).

No test caught this in either direction: the suite asserts finiteness, non-zero
energy and one relative energy ordering, all of which hold for both values.

### Verified

- Old-vs-new top-level symbol diff across all four refreshed bundles,
  intersected against every identifier ghurni references: **zero** remaining
  references to a removed symbol.
- `cyrius build` · `cyrius test` (6 suites, 135 assertions + 6 from the smoke
  entry, 0 failures) · `cyrius bench` · all 5 `docs/examples/*.cyr` build and run.

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
