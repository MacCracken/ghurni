# ghurni Roadmap

Post-2.0 (Cyrius). Items are written against what the Cyrius toolchain and the
naad stack actually provide — the pre-port roadmap planned `cargo-fuzz`, rayon
and portable SIMD, none of which exist here.

## Near term

### Testing
- [ ] **Raise reference coverage.** `cyrius coverage` reports ~48% of `src/`
      functions referenced. The largest gaps are the per-synth
      `process_block_naad` cores and the trait-shaped accessors on turbine,
      differential, chain_drive and belt_drive. Coverage is a floor, not a
      correctness proof — see [testing.md](../guides/testing.md) before treating
      a percentage as progress.
- [ ] **Spectral validation** — FFT-based assertion (via hisab's `num_fft`) that
      the peak frequency matches the expected firing / mesh / blade-pass
      frequency. This is the assertion class that would have caught the
      `GHURNI_DB_SCALE` miscompile on its own.
- [ ] **Golden-file regression** — checksum comparison of deterministic
      synthesis output, so any unintended audio change fails loudly. The 2.0.2
      sweep used this ad hoc to prove refactors were bit-identical; make it a
      standing suite.
- [ ] **A fuzz harness** — `cyrius fuzz` runs `fuzz/*.fcyr`. ghurni has none.
      Drive the public boundary with the values that break Cyrius ports:
      `INT64_MIN`, NaN, ±inf, subnormals, `f64::MAX`, and non-power-of-two
      buffer lengths. Note naad's lesson — sweep enum ids at valid parameters
      and values at a valid id, or the sweeps correlate and the harness proves
      nothing.
- [ ] **CI quality gates** — CI runs deps/build/test plus the examples. Add
      `cyrius audit` (now exiting 0), `cyrius deny`, and `cyrius fuzz` once a
      harness exists.

### Robustness
- [ ] **Allocation failure.** No `alloc()` result is null-checked anywhere in
      the port, and no sibling library checks either. Decide the contract
      ecosystem-wide before adding a one-off path here.
- [ ] **Resonant-RPM amplitude.** The combustion pulse peak depends on where the
      sample lattice falls relative to crank angle 0. At RPMs where
      samples-per-cycle is an integer (5292000/rpm ∈ ℤ at 44.1 kHz) the port
      lands consistently off-peak and the thump loses up to two thirds of its
      amplitude; the f32 oracle's coarser rounding accidentally snapped the
      lattice back onto the cycle. Fixing it means changing the pulse
      computation, which is an audio change and therefore needs an ADR. See the
      divergence list in [state.md](state.md).

## Sound Quality
- [ ] Spectral morphing — interpolation between synthesis states (cold vs warm, new vs worn)
- [ ] RPM-range crossfading — different harmonic profiles blended across idle / mid / high RPM regions
- [ ] Convolution / IR hooks — exhaust-pipe and engine-bay impulse responses, via goonj

## New Mechanical Types
- [ ] Hydraulic pump — pulsation frequency model with pressure-dependent noise
- [ ] Bearings — ball/roller/plain bearing defect frequencies (BPFO, BPFI, BSF, FTF)
- [ ] Brake squeal — friction-induced oscillation at pad natural frequency
- [ ] Compressors — reciprocating chug, rotary/screw whine
- [ ] Pneumatic tools — rapid impulse train (impact wrench), air ratchet

## Performance
- [ ] **Presized output buffers.** Every `*_synthesize` grows its result one
      `vec_push` at a time, so the doubling allocator burns roughly 2× the
      delivered heap and never reclaims it. The stdlib has no
      `vec_with_capacity`; this wants an upstream `vec` API rather than reaching
      into the vec's internals.
- [ ] Per-sample accessor hoisting in the remaining synth loops — belt_drive's
      block-invariants were hoisted in 2.0.2 (measured bit-identical, no
      measurable time win), so measure before assuming the others pay.

## API
- [ ] Parameter automation curves — keyframed RPM/load trajectories for scripted sequences
- [ ] Doppler wrapper — variable-rate resampling from relative velocity (or defer to goonj)
- [ ] Multi-channel output — per-component spatial channels (front/rear exhaust, intake, body) for 3D positioning

## Architectural Notes

- `sample_position` is an `i64` sample counter — no practical overflow (~6.6
  million years at 44.1 kHz).
- Deterministic output depends on naad's `NoiseGenerator` seed behaviour. Seeds
  are derived from constructor parameters; if you add a synth, make sure every
  seeded parameter actually reaches the seed, or two instances that should
  decorrelate will share one stream.
- The `dist/ghurni.cyr` bundle is compiled by the **consumer's** toolchain, so
  anything sensitive to compiler behaviour (long float literals, in particular)
  must be written in a form an older toolchain cannot get wrong.
