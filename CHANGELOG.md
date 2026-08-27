# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.2] - 2026-08-27

A P-1 audit / refactor / hardening / optimization / security sweep, plus the
repair of every Rust-era document left behind by the 2.0.0 port.

**Nothing here changes audio output.** Every functional fix is a rejection of
input that previously crashed, corrupted state, or lied about success; the two
refactors were proved bit-identical by checksumming rendered audio before and
after. Suite: **7 suites / 205 assertions** (was 6 / 135). `cyrius audit` exits
**0** for the first time.

### The two defect classes

Both come from Cyrius's single signed `i64`, and both are invisible in a
straight transliteration — the same two shapes naad 2.1.3 found:

1. **Rust `usize`/`u32`/enum guarantees erased.** Where the oracle typed a
   parameter `usize` or as an enum, a bad value is *unrepresentable*, so the
   Rust body validates nothing and a transliterated guard is only **half a
   bound**.
2. **`f64_to` truncates where Rust's `as` casts SATURATE.** Rust maps an
   over-range f64 to `usize::MAX`; `f64_to` overflows to `INT64_MIN`, which is
   neither `> MAX` nor `== 0`, so one-sided clamps let it straight through.

### Fixed — crashes

- **The mixer dereferenced a failed constructor's error code (SIGSEGV).**
  `ghurni_mixer_add_channel` stored whatever `i64` it was handed, and the tag
  dispatch then used it as a struct base pointer — confirmed under gdb faulting
  at `base + 0x90` with base `-1`. The oracle's field is
  `Option<Box<dyn Synthesizer>>`: a `Box` is always a live synth (`Engine::new`
  returns `Result`, so the caller must unwrap first) and all three use sites are
  guarded by `if let Some(synth)` (`rust-old/src/mixer.rs:101,123,156`).
  `add_channel` now rejects an error code, a null synth, and an out-of-range
  `GH_KIND_*` tag; the three dispatch loops skip an absent synth.
- **A caller-supplied `duration` could abort the process.** Every
  `*_synthesize` grew its output with `vec_push` past the stdlib's
  `VEC_CAP_MAX`, and `vec_push` calls `_vec_die()` → `exit(1)`. Above ~6087 s at
  44.1 kHz this killed the host process out of library code.
- **A desynchronised firing order aborted the process.** `set_firing_order`
  stored the *caller's* vec where Rust moved it, so a later push by the caller
  left `vec_len(firing_offsets)` larger than `misfire_flags`, and the per-sample
  loop indexed out of bounds into `_vec_die()`. The oracle is defensive at both
  spots (`.get(cyl).unwrap_or(false)`, `knock_remaining.iter_mut()`).
  `set_firing_order` and `ghurni_transmission_new` now **copy** their vec
  (restoring Rust's move semantics), and the engine's event loops are bounded by
  their own lengths.

### Fixed — silent corruption

- **`synthesize` returned an empty buffer and called it success** (CLASS 2). All
  ten one-shot functions computed `f64_to(sample_rate * duration)`; past 2^63
  that is `INT64_MIN`, the fill loop never ran, and the length-0 vec is a heap
  pointer so `ghurni_is_err` reported `GH_ERR_NONE`. New shared guard
  `ghurni_sample_count` (`src/dsp.cyr`) compares in f64 **before** the
  conversion, caps at `GHURNI_MAX_SAMPLES` (2^26 ≈ 25 min at 44.1 kHz), and
  rejects a count below 1. Wired into all ten, **before** the state-mutating
  setters — matching the oracle, which validates first.
- **Out-of-range enum ids fell through to a wrong default** (CLASS 1). All five
  enum constructors accepted any `i64`. `ghurni_forced_induction_new` was the
  sharpest: the `inertia = 0.01` default was only overridden inside the Turbo
  branch, so an unrecognised type **silently discarded the caller's
  `spool_inertia`**. Each constructor now range-checks its id.
- **`ghurni_transmission_shift_to` accepted a negative gear** (CLASS 1), setting
  `current_gear = -1` and firing the 200 ms shift transient. Rust typed the
  parameter `u32`, which made its one-sided `< ratios.len()` a complete bound.
- **Every Turbo shared one bit-identical noise stream.** The oracle seeds with
  `induction_type as u32 * 777 + drive_ratio.to_bits()`
  (`rust-old/src/forced_induction.rs:95`); the port dropped the ratio term
  entirely, collapsing the seed space to two values, so two layered turbos summed
  coherently (~+6 dB, phasey) instead of decorrelating. The seed now folds in
  `f64_to(ratio * 1000)` — the same behavioural-integer-seed convention
  `belt_drive` and `turbine` already document.

### Performance

Measured with interleaved A/B runs against the pre-fix bundle; run-to-run
variance on this machine is ±30%, so single measurements were not trusted.

- **The mixer leaked its scratch buffer on every block.** It built a fresh vec
  per channel per `process_block` and dropped the previous one; this library
  never frees. Measured: **278 MB of unreclaimable heap per 100 s of 4-channel
  audio → now 0**. The oracle reuses a grow-only buffer
  (`rust-old/src/mixer.rs:124-131`); the port now rebuilds only when the block
  size actually changes, since Cyrius vecs have no sub-slice and `process_block`
  must see a vec of exactly `len`. Also ~7% faster on the streaming path
  (1.239 ms → 1.155 ms, 4ch × 512).
- **The engine scanned its event vecs every sample even when idle.** Skipping
  the knock scan while nothing is armed is ~4% off engine synthesis
  (1.448 ms → 1.389 ms). Noise-stream consumption is unchanged — the old loop
  consumed no noise when nothing was armed either, so output is bit-identical.
- **belt_drive** hoisted `osc_set_frequency` and `belt_omega` out of the
  per-sample loop (both block-invariant; `osc_set_frequency` only validates and
  stores a field, so it cannot touch the phase). **Bit-identical**, but honestly:
  **no measurable time win**.

### Testing

- **New `tests/hardening.tcyr`** (70 assertions) pins every defect above.
  **Each assertion was mutation-checked**: the fix was reverted and the suite
  confirmed to fail. That pass caught one **vacuous test of my own** — the
  forced-induction seed test compared rendered audio, but `drive_ratio` also
  sets the compressor speed, so it passed with the defect reintroduced. It now
  pulls from the noise generator directly.
- **`GHURNI_DB_SCALE` is pinned** to its exact bit pattern and to the envelope
  value it produces. The 2.0.1 post-mortem showed the entire 135-assertion suite
  passed with that constant 76.9× wrong; that gap is now closed permanently.
- New benchmarks for the streaming 4-channel mixer path and belt_drive's block
  loop — the bench header had claimed mixer coverage it did not have.

### Documentation — the Rust-era backlog, cleared

Every one of these described the 1.x Rust crate and contradicted `CLAUDE.md`:

- **`README.md`** — sold a Rust crate (crates.io link, Rust quick start, a
  feature-flag table that does not exist, and a "~1,000× real-time" claim that
  benchmarks put at **~43×**). Rewritten against the real API.
- **`CONTRIBUTING.md`** — told contributors to run `cargo fmt/clippy/test/audit`
  and to write synthesizers "following the dual-impl pattern" with a
  `Synthesizer` trait. Rewritten around `cyrius audit`, plus the two defect
  classes above.
- **`SECURITY.md`** — the dependency table was `rust-old/Cargo.toml` verbatim
  (serde, thiserror, tracing, libm, naad "Optional"), none of which are in the
  graph. Now lists the real pinned set, and its attack-surface table reflects
  the guards this release actually added, with the known limitations stated.
- **`docs/architecture/overview.md`** — listed 20 `.rs` files under a
  `ghurni/src/` header, including the deliberately-unported `math.rs`/`rng.rs`,
  and documented a "Dual Code Paths" backend that does not exist.
- **`docs/architecture/adr-001-naad-backend.md`** — status `Accepted`, specifying
  naad as *optional* behind a feature flag with a libm/PCG32 fallback. Now marked
  **Superseded by ADR-004**, retained as history.
- **`docs/architecture/adr-002-scope-boundaries.md`** — promised consumers a
  `Synthesizer` trait; corrected to the `(GH_KIND_*, pointer)` tag dispatch.
- **`docs/development/integration-guide.md`** — all five code blocks were Rust.
  Rewritten in Cyrius, error handling first.
- **`docs/guides/testing.md`** — documented `cargo test` and a no-naad fallback
  path that cannot exist. Rewritten around the real gates, with the
  "a vacuous test is worse than no test" rules this sweep learned the hard way.
- **`docs/development/roadmap.md`** — planned `cargo-fuzz`, rayon and portable
  SIMD. Re-scoped to what the toolchain provides.

### Also

- Wrapped the 7 over-120-character lines that `cyrius lint` had always flagged,
  and documented the 7 public functions `cyrius doc --check` had always flagged
  (the logging wrappers, `main`, `ghurni_synth_set_rpm`). Both were pre-existing;
  with them fixed `cyrius audit` exits 0. The line wrapping was verified
  bit-identical by checksum.
- New helper `ghurni_vec_copy` (`src/error.cyr`) for restoring the oracle's
  move semantics at vec-taking boundaries.

### Known — still open

- **Allocation failure is unhandled.** No `alloc()` result is null-checked
  anywhere in the port, and no sibling library checks either; the contract wants
  deciding ecosystem-wide rather than patching here alone.
- **Presized output buffers.** `*_synthesize` still grows one `vec_push` at a
  time (~2× heap waste). The stdlib has no `vec_with_capacity`; this wants an
  upstream `vec` API, not a reach into vec internals.
- **Resonant-RPM combustion amplitude** is characterised but deliberately NOT
  changed — the fix would alter audio output and needs its own ADR. See
  `docs/development/state.md`.
- No fuzz harness (`cyrius fuzz` runs `fuzz/*.fcyr`; ghurni has none), and CI
  still does not run `audit`/`deny`.

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
