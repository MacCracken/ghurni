# ghurni — Live Port State

> Volatile state (version, module sizes, coverage). Durable rules live in
> [`CLAUDE.md`](../../CLAUDE.md). Refreshed at each milestone.

## Version

**2.6.0** — Cyrius port, hardened and pinned; oracle retired; RPM loudness law
(ADR-005), acoustic depth (ADR-006), load tilt + diesel clatter (ADR-007), a
control surface that no longer lies (ADR-008), stems deferred on measurement with
convolution closed (ADR-009), and a quarter-wave exhaust waveguide (ADR-010).
Dependency set:
cyrius 6.5.36 · naad 2.2.2 · hisab 2.11.2 · goonj 2.0.4 · sakshi 2.4.11.
(1.0.0 was the final Rust crate; it served as the parity oracle at `rust-old/`
and was retired in 2.0.4 — recoverable at tag `2.0.3`.)

## Port status: COMPLETE

18 of the 20 Rust source modules are ported. `math.rs` (libm wrappers) and
`rng.rs` (PCG32) are deliberately **not** ported — they existed only for the
non-naad fallback path, which ADR-004 drops. All 44 Rust integration tests are
reproduced as behavioural-parity `.tcyr` suites; smoke binary + benchmarks green.

## Modules (`src/*.cyr`, 19 modules, 4,112 lines)

| Layer | Modules |
|-------|---------|
| L0 foundations | `error` `logging` `dsp` `smooth` `event` `traits` |
| L1 synths | `engine` `gear` `motor` `turbine` `clock` `transmission` `differential` `forced_induction` `belt_drive` `chain_drive` |
| L2 composites | `mixer` `presets` |
| entry | `main` (smoke) |

Bundle: `dist/ghurni.cyr` (4,099 lines, `cyrius distlib`).

## Tests (`tests/*.tcyr`, 10 suites, 640 assertions, 0 failures)

> Count with `cyrius test 2>&1 | grep -E '^[0-9]+ passed, [0-9]+ failed \([0-9]+ total\)$'`
> and sum the leading numbers. Do NOT sum every `N passed` line: the run's final
> `10 passed, 0 failed` is the **suite** count, and including it inflated the
> figures reported for 2.3.0, 2.4.0 and 2.5.0 by exactly 10 each (corrected in
> the 2.5.1 CHANGELOG entry).

| Suite | Covers |
|-------|--------|
| `foundations` | error codes/helpers, validation, DcBlocker, SmoothedParam, MechanicalEvent + serde |
| `engine` | all engine integration tests + firing frequency + param sweep + EngineType serde |
| `synths` | gear/motor/turbine/clock/transmission/differential/forced_induction/belt/chain + gear sweep |
| `serde` | EngineType/GearMaterial/MotorType/ClockType/InductionType + GhurniError name mapping |
| `mixer` | mono/stereo/mute, trait dispatch, process-block continuity, presets |
| `smoke_all` | full-unit integration: every synth + mixer + preset produce finite audio |
| `hardening` | the 2.0.2 P-1 regression suite — every defect it fixed, each assertion mutation-checked |
| `oracle_pins` | 2.0.3 — every value that lived only in `rust-old/`, derived from the oracle's source; extended each release since with the behaviour that release made real |
| `goldens` | 2.0.3 — rendered-audio checksums; the gate that makes "a patch may not change audio" enforceable |
| `spectral` | 2.0.3 — FFT peak assertions that audio sits where the RPM physics says it should |

`cyrius bench` (`benches/ghurni.bcyr`): DcBlocker ~20 ns/sample, smoother
~15 ns/sample, engine/gear one-shot synthesis, the streaming 4-channel mixer
path, and belt_drive's block loop.

`cyrius audit` exits **0** (fmt · lint · docs · tests · bench).
`cyrius coverage`: 137/185 functions referenced (74%) — a floor, not a proof.
Fuzz: `fuzz/ghurni.fcyr`, 1631 adversarial checks (`cyrius fuzz`).

## Deliberate divergences from `rust-old/`

> The list below is the **port-time** set. Since 2.1.0 the audio has also diverged
> deliberately, each time with an ADR, and those are the larger divergences now:
> the RPM loudness law and integrated combustion pulse (ADR-005), acoustic depth
> (ADR-006), the load tilt and diesel clatter (ADR-007), five events that the
> oracle also ignored (ADR-008), and the quarter-wave exhaust waveguide
> (ADR-010), which replaces the oracle's bandpass-on-a-noise-bed outright. The
> oracle was retired in 2.0.4; the correctness bar is the frozen suite.

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
  naad pointers drop deep serialization.
  ⚠ The old justification here — "nothing meaningful survived the Rust
  `#[serde(skip)]`" — was **false for `MixerChannel`**: only `synth` and
  `scratch` were skipped, so `name`, `gain`, `pan` and `muted` all serialized
  and a Rust consumer could persist a whole mixer configuration as a reloadable
  template. Dropping it is a real capability loss, not a no-op. The synth
  structs proper (Transmission, Differential, ForcedInduction, …) did round-trip
  their live DSP state too. Treat this as a deliberate scope cut.
- **RNG seeds**: only the three synths whose oracle seed folded in a FLOAT are
  behavioural rather than bit-identical — `belt_drive` (pulley diameter +
  tension), `turbine` (blades + duct resonance) and `forced_induction`
  (induction type + drive ratio); each derives a behavioural integer seed
  instead of the f32 `to_bits()` pattern. **`engine`, `gear`, `motor`, `clock`,
  `transmission`, `differential` and `chain_drive` seed from integers only and
  reproduce the oracle's seed EXACTLY** — do not "harmonise" them onto the
  behavioural convention, it would silently change seven synths' audio.
  Note the behavioural form must still DEPEND on its float parameter: a bare
  `f64_to` of a sub-unity value truncates to a constant, which is the defect
  fixed in `forced_induction` and `belt_drive` (see the 2.0.2 CHANGELOG).

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
each ports the matching example from the retired oracle, includes the dep bundles +
`dist/ghurni.cyr`, and builds/runs standalone:

```sh
cyrius build docs/examples/simple_engine.cyr build/ex_simple_engine && ./build/ex_simple_engine
```

## CI

`.github/workflows/{ci,release}.yml` use the cyrius toolchain (install from the
`cyrius.cyml [package].cyrius` pin). As of 2.0.3 CI runs the full gate chain:
`deps` → `build` → `test` → `distlib --check` → `audit` → `deny` → `fuzz` →
`coverage --min` → build **and run** the examples. Release verifies VERSION/tag
consistency and ships a source tarball + SHA256SUMS.

⚠ **12 compiled ELF binaries (11 MB) are still tracked under `build/`** and ship
in every release tarball. `.gitignore` covers them as of 2.0.3, but untracking
needs `git rm -r --cached build/`.

## `rust-old/` — RETIRED in 2.0.4

The oracle is gone from the tree. Record: [`rust-old-retirement.md`](rust-old-retirement.md).
Everything it proved is pinned by `tests/oracle_pins.tcyr` (304 assertions),
`tests/goldens.tcyr` and `tests/spectral.tcyr`. Recover it with
`git show 2.0.3:rust-old/src/<mod>.rs`, or from tag `1.0.0` where the same crate
lives at `src/*.rs`.

**The correctness bar moved with it.** It is no longer "matches what the Rust
naad-backend path did" — it is the frozen suite above. A moved golden is not a
patch release.

## Known follow-ups

- None outstanding for parity.
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
- **16 KB per `GhEngine`** for the 2.6.0 exhaust delay line, allocated
  unconditionally — including for HYBRID, where the duct is silent. Sizing it
  from `exhaust_resonance` at construction is NOT safe: the parameter is
  settable afterwards and a smaller buffer would silently truncate.

## Fixed, kept as precedent

- **Resonant-RPM combustion amplitude** (fixed 2.1.0). The combustion pulse was
  point-sampled, so its peak depended on where the sample lattice fell: at RPMs
  where samples-per-cycle is an integer the thump lost up to two thirds of its
  amplitude in a notch ~50 rpm wide. Deliberately NOT repaired in 2.0.2, because
  the fix changes the pulse computation and so needed an ADR and a minor. It got
  both — [ADR-005](../architecture/adr-005-rpm-loudness-law.md) integrates the
  envelope across each sample, which is energy-conserving and therefore
  lattice-invariant. The port's longest-lived characterised defect, and the
  reason the golden suite exists.

## Float-literal hardening

`GHURNI_DB_SCALE` is stored as an IEEE-754 bit pattern, not a decimal literal —
cyrius <= 6.5.27 miscompiled decimal literals past ~9 fractional digits (see the
2.0.1 CHANGELOG entry). It was the only such literal in the link unit. Any new
high-precision float constant should follow the same hex idiom, because
`dist/ghurni.cyr` is compiled by the **consumer's** toolchain, not ghurni's.
`tests/hardening.tcyr` pins the constant's exact bit pattern and the envelope
value it produces, so the regression cannot recur silently.
