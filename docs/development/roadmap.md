# ghurni Roadmap

## v1.1 — Advanced Sound Models

### Sound Quality
- [ ] Spectral morphing — FFT-based interpolation between synthesis states (cold start vs warm, new vs worn engine)
- [ ] RPM-range crossfading — different harmonic profiles blended across idle/mid/high RPM regions with `EngineProfile`

### New Mechanical Types
- [ ] Hydraulic pump — pulsation frequency model with pressure-dependent noise
- [ ] Bearings — ball/roller/plain bearing defect frequencies (BPFO, BPFI, BSF, FTF)
- [ ] Brake squeal — friction-induced oscillation at pad natural frequency
- [ ] Compressors — reciprocating chug, rotary/screw whine
- [ ] Pneumatic tools — rapid impulse train (impact wrench), air ratchet

### DSP
- [ ] Convolution/IR hooks — `AcousticEnvironment` trait for exhaust pipe and engine bay impulse responses

### Testing
- [ ] Spectral validation — FFT-based assertion that peak frequency matches expected firing/mesh/blade-pass frequency
- [ ] Golden-file regression — hash-based comparison of deterministic synthesis output
- [ ] Perceptual quality metrics — spectral centroid, flatness, crest factor, harmonic-to-noise ratio assertions per machine type
- [ ] Fuzzing — cargo-fuzz targets for constructors and process_block with arbitrary parameters

### Performance
- [ ] SIMD inner loops — feature-gated portable SIMD for hot-path sample processing
- [ ] Multi-threaded component mixing — rayon-based parallel processing behind `parallel` feature flag

## v1.2 — Automation and Spatial Audio

### API
- [ ] Parameter automation curves — keyframed RPM/load trajectories for scripted sequences / cutscenes
- [ ] Doppler effect wrapper — variable-rate resampling based on relative velocity, wraps any `Synthesizer`
- [ ] Multi-channel output — per-component spatial channels (front exhaust, rear exhaust, intake, mechanical body) for 3D audio positioning

## Architectural Notes

- `sample_position` uses `usize` — safe on 64-bit (~13B years at 44.1kHz), overflows after ~27 hours on 32-bit targets. Consider `u64` if 32-bit no_std targets are a priority.
- The fallback (non-naad) path duplicates synthesis logic. Consider extracting physical model (frequencies/amplitudes) from DSP implementation to reduce duplication.
- Deterministic output depends on naad's NoiseGenerator seed behavior — pin noise generation or use ghurni's own RNG for golden-file test stability.
