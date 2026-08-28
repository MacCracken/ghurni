# Testing Guide

## Running

```sh
cyrius test                  # every tests/*.tcyr suite + the [build] entry
cyrius test tests/engine.tcyr    # one suite
cyrius bench                 # benches/*.bcyr
cyrius coverage              # reference coverage of src/
cyrius audit                 # fmt + lint + docs + tests + bench (the CI gate)
```

`cyrius audit` must exit 0 before a release. It is the same gate naad's CI
enforces.

## Suite Layout

Each `tests/*.tcyr` file includes the `src/*.cyr` modules it needs directly
(not the `dist/` bundle), defines its own small audio helpers, and ends with
`assert_summary()`.

| Suite | Covers |
|-------|--------|
| `foundations` | error codes/helpers, validation, DcBlocker, SmoothedParam, MechanicalEvent + serde |
| `engine` | all engine types, firing frequency, custom firing order, events, decel pop, load ordering, param sweep, EngineType serde |
| `synths` | gear / motor / turbine / clock / transmission / differential / forced_induction / belt / chain |
| `serde` | EngineType / GearMaterial / MotorType / ClockType / InductionType roundtrips + GhurniError name mapping |
| `mixer` | mono / stereo / mute, tag dispatch, process-block continuity, presets |
| `smoke_all` | full-unit integration: every synth + mixer + preset produce finite audio |
| `hardening` | the P-1 regression suite — every defect fixed in 2.0.2 |
| `oracle_pins` | every constant and contract the retired Rust oracle used to prove, each citing its source line |
| `goldens` | rendered-audio checksums — the gate that makes "a patch may not change audio" enforceable |
| `spectral` | FFT peak assertions that audio lands where the RPM physics says it should |

## Writing Assertions That Are Worth Having

**A vacuous test is worse than no test, because it makes the gap invisible.**
ghurni has shipped proof of this: `GHURNI_DB_SCALE` was wrong by a factor of
76.9 for the whole of 2.0.0, flattening the gear / clock / engine decay
envelopes — and all 135 assertions passed, because every audio assertion was
`audio_all_finite` / `audio_has_energy` / an energy ordering, and all three hold
for both the right and the wrong value.

So:

- **Assert exact values wherever the arithmetic is exact.** Firing frequency is
  `cylinders * rpm / 120` for a 4-stroke — a 4-cylinder at 3000 RPM is exactly
  100 Hz, not "approximately". Pick fixtures that make values exact: sample
  rates and RPMs that give whole-number periods, dyadic coefficients, short
  tables.
- **Derive the expected value from first principles or from the oracle, never
  from what ghurni prints.** A value transcribed from current output freezes
  whatever bug is there today as "correct" — the worst possible outcome. The
  Rust oracle was retired in 2.0.4 but is still readable in git history
  (`git show 2.0.3:rust-old/src/gear.rs`), and `tests/oracle_pins.tcyr` already
  carries the values it proved, each citing its source line.
- **Prefer structural facts over audio predicates**: error codes, buffer
  lengths, gear indices, "this call no longer aborts the process", "these two
  noise streams differ". They fail loudly and for one reason.
- **Guard against `0 == 0`.** If a loop compares two buffers, first assert the
  reference actually contains signal. Several assertions in `hardening.tcyr`
  exist only to prove another assertion is not vacuous.
- **Isolate what you claim to test.** The first version of the
  forced-induction seed test compared rendered audio from two drive ratios — but
  drive ratio also sets the compressor speed, so the audio differed even with
  the seed defect reintroduced. It had to pull from the noise generator directly.

## Mutation-Check Every New Assertion

Before a fix ships, revert it and confirm the new test **fails**. A test that
passes both with and without the fix is pinning nothing. Every assertion in
`tests/hardening.tcyr` was verified this way — the mutation pass is what caught
the vacuous seed test described above.

```sh
# revert the guard in src/…, then:
cyrius test tests/hardening.tcyr    # must FAIL
# restore it, then:
cyrius test tests/hardening.tcyr    # must PASS
```

## Verifying a Refactor Changed No Audio

For any edit to a synthesis path that is supposed to be behaviour-preserving
(hoisting a block-invariant, extracting a subexpression, wrapping a long line),
prove it rather than reasoning about it: build the bundle before and after, run
the same fixture through both, and compare a checksum over every sample. The
2.0.2 sweep did this for the belt-drive hoist and for the engine/clock line
wrapping; both came back bit-identical.

## Coverage

```sh
cyrius coverage
```

Reports reference coverage of `src/` — a floor, not a correctness proof. A
function being referenced by a test says nothing about whether the assertion
around it is meaningful; read the section above before treating a percentage as
progress.

## Benchmarks

```sh
cyrius bench
```

`benches/ghurni.bcyr` has 14 entries: the per-sample primitives (DC blocker,
smoother), one-shot synthesis for all the major synths, the **streaming**
4-channel mixer path a real audio callback uses, belt-drive's block loop, and a
64 / 512 / 4096 block-size sweep.

Never claim a performance improvement without a measurement, and interleave the
A/B runs — run-to-run variance on the per-sample benchmarks is comfortably
±30%, enough to invent or hide a small win if you measure once each.
