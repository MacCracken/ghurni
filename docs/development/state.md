# ghurni — Live Port State

> Volatile state (version, module sizes, coverage). Durable rules live in
> [`CLAUDE.md`](../../CLAUDE.md). Refreshed at each milestone.

## Version

**2.0.2** — full-parity Cyrius port, hardened. Dependency set:
cyrius 6.5.35 · naad 2.2.1 · hisab 2.11.2 · goonj 2.0.4 · sakshi 2.4.11.
(1.0.0 was the final Rust crate, preserved at `rust-old/`.)

## Port status: COMPLETE

18 of the 20 Rust source modules are ported. `math.rs` (libm wrappers) and
`rng.rs` (PCG32) are deliberately **not** ported — they existed only for the
non-naad fallback path, which ADR-004 drops. All 44 Rust integration tests are
reproduced as behavioural-parity `.tcyr` suites; smoke binary + benchmarks green.

## Modules (`src/*.cyr`, 19 modules, 3,137 lines)

| Layer | Modules |
|-------|---------|
| L0 foundations | `error` `logging` `dsp` `smooth` `event` `traits` |
| L1 synths | `engine` `gear` `motor` `turbine` `clock` `transmission` `differential` `forced_induction` `belt_drive` `chain_drive` |
| L2 composites | `mixer` `presets` |
| entry | `main` (smoke) |

Bundle: `dist/ghurni.cyr` (3,072 lines, `cyrius distlib`).

## Tests (`tests/*.tcyr`, 7 suites, 205 assertions, 0 failures)

| Suite | Covers |
|-------|--------|
| `foundations` | error codes/helpers, validation, DcBlocker, SmoothedParam, MechanicalEvent + serde |
| `engine` | all engine integration tests + firing frequency + param sweep + EngineType serde |
| `synths` | gear/motor/turbine/clock/transmission/differential/forced_induction/belt/chain + gear sweep |
| `serde` | EngineType/GearMaterial/MotorType/ClockType/InductionType + GhurniError name mapping |
| `mixer` | mono/stereo/mute, trait dispatch, process-block continuity, presets |
| `smoke_all` | full-unit integration: every synth + mixer + preset produce finite audio |
| `hardening` | the 2.0.2 P-1 regression suite — every defect it fixed, each assertion mutation-checked |

`cyrius bench` (`benches/ghurni.bcyr`): DcBlocker ~20 ns/sample, smoother
~15 ns/sample, engine/gear one-shot synthesis, the streaming 4-channel mixer
path, and belt_drive's block loop.

`cyrius audit` exits **0** (fmt · lint · docs · tests · bench).
`cyrius coverage`: 79/164 functions referenced (48%) — a floor, not a proof.

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

### Added by the 2.0.2 P-1 sweep

Each was introduced knowingly, to satisfy the zero-panic rule where a faithful
transliteration could not:

- **Bounded buffer length.** `ghurni_sample_count` caps `duration x sample_rate`
  at `GHURNI_MAX_SAMPLES` (2^26 samples, ~25 min at 44.1 kHz) and returns
  `GH_ERR_INVALID_PARAMETER` beyond it. The oracle is uncapped: Rust's `as usize`
  saturates and `vec![0.0; usize::MAX]` fails loudly, whereas Cyrius `f64_to`
  overflows to `INT64_MIN` (silently empty) and the stdlib `vec_push` aborts the
  whole process past `VEC_CAP_MAX`. Both outcomes violate the zero-panic rule,
  so the port reports an error where the oracle would have died or lied.
- **Enum ids are range-checked.** Rust's exhaustive `match` proved the id was a
  real variant; Cyrius's `i64` cannot, so each constructor now rejects an
  out-of-range id instead of falling through to a default variant.
- **`shift_to` rejects negative gears.** Rust typed the parameter `u32`, making
  its one-sided bound complete.
- **The mixer rejects a failed or absent synth, and an unknown kind tag.** The
  oracle's field is `Option<Box<dyn Synthesizer>>`, so neither an error code nor
  a null could reach dispatch; the port has to check explicitly.
- **`set_firing_order` and `transmission_new` copy their vec.** Rust took
  `Vec<f32>` by value, so the move made caller mutation impossible.
- **forced_induction's noise seed** folds in `drive_ratio` as
  `f64_to(ratio * 1000)` where the oracle used `drive_ratio.to_bits()` — the
  same behavioural-integer-seed convention `belt_drive` and `turbine` already
  document. 2.0.1 and earlier dropped the term entirely, which was a defect,
  not a divergence.

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

- None outstanding for parity.
- `CHANGELOG.md` still has **no `2.0.0` entry** for the port itself. 2.0.1 and
  2.0.2 have one, so `release.yml`'s changelog extraction populates for those
  tags, but a re-cut of the 2.0.0 tag would still produce an empty release body.
- **Allocation failure is unhandled.** No `alloc()` result is null-checked
  anywhere in the port, and no sibling library checks either. The contract wants
  deciding ecosystem-wide rather than patching here alone.
- **Presized output buffers.** Every `*_synthesize` grows its result one
  `vec_push` at a time, so the doubling allocator burns ~2x the delivered heap.
  The stdlib has no `vec_with_capacity`; this wants an upstream `vec` API rather
  than reaching into the vec's internals.
- **`cyrius vet` reports `dist/ghurni.cyr` as UNTRUST.** Structural: it is the
  project's own generated bundle, not a `[deps]`-resolved one, and there is no
  trust-list mechanism in the manifest. `cyrius deny` and `cyrius audit` both
  exit 0. CI does not run `vet`.
- **No fuzz harness.** `cyrius fuzz` runs `fuzz/*.fcyr`; ghurni has none yet.
- **CI does not run the quality gates** it now could: `cyrius audit` exits 0 as
  of 2.0.2, and `cyrius deny` passes.

## Characterised, NOT fixed: resonant-RPM combustion amplitude

The engine's combustion pulse peak depends on where the sample lattice falls
relative to crank angle 0. `src/engine.cyr` is an expression-for-expression
transliteration of `rust-old/src/engine.rs:265-266`, but the oracle computes it
in f32, whose ULP at ~45000 degrees is coarse enough to snap the phase lattice
back onto the cycle. In f64 that accidental snapping is gone.

At RPMs where samples-per-cycle is an integer (`5292000/rpm` is an integer at
44.1 kHz: 2250, 3000, 4200, 6000, 7000, ...) the port lands consistently just
past crank 0 every cycle and the thump loses up to two thirds of its amplitude
(measured: 2-stroke 1-cyl @6000 rpm, peak 0.176 vs the oracle's 0.517). At the
other ~92% of RPMs f64 matches or beats f32.

**Not repaired in 2.0.2**, because the fix is to change the pulse computation —
an audio change that diverges from the oracle by construction and therefore
needs its own ADR. Filed in [roadmap.md](roadmap.md) under Robustness.

## Float-literal hardening

`GHURNI_DB_SCALE` is stored as an IEEE-754 bit pattern, not a decimal literal —
cyrius <= 6.5.27 miscompiled decimal literals past ~9 fractional digits (see the
2.0.1 CHANGELOG entry). It was the only such literal in the link unit. Any new
high-precision float constant should follow the same hex idiom, because
`dist/ghurni.cyr` is compiled by the **consumer's** toolchain, not ghurni's.
`tests/hardening.tcyr` pins the constant's exact bit pattern and the envelope
value it produces, so the regression cannot recur silently.
