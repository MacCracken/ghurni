# ADR-007: Load Changes the Spectrum, and Diesel Gets Its Clatter

**Status**: Accepted
**Date**: 2026-08-28
**Release**: 2.3.0 ("Depth II")

## Context

[ADR-006](adr-006-acoustic-depth.md) split Arc 2 and deferred five items. This
resolves three of them; the other two are re-scoped at the end, one of them
because my earlier cost estimate was **wrong**.

### Load was a pure gain control — and on the engine it went the wrong way

Measured spectral centroid, 8192-sample block at 44.1 kHz:

| | load 0.1 | load 0.5 | load 0.9 | RMS change |
|---|---|---|---|---|
| motor | 2061.2 Hz | 2061.2 Hz | 2061.2 Hz | ×1.94 |
| engine | 8882.5 Hz | 8800.1 Hz | **8689.5 Hz** | ×2.70 |

The motor's centroid is **identical to four figures** across the whole load
range while its level nearly doubles — load moved the fader and nothing else.
`amp = 0.15 + 0.2·load` scaled the tonal hum and the broadband noise by the same
factor, so the spectrum could not change shape.

The engine was worse than flat: it got **duller** as load rose. `base_amp` scales
the low-frequency combustion thump along with everything else, and the intake bed
is additionally scaled by `load` *directly* — so it grows about 31× from load 0.1
to 0.9 while the broadband bed grows about 6×. The bass share wins.

That is backwards. A machine working harder is brighter.

### Diesel was a retuned petrol engine

`Diesel` differed from `Gasoline` in exactly three numbers: exhaust resonance
(80+15c vs 150+20c), intake resonance (250+20c vs 400+30c), and roughness base
(0.4 vs 0.15). Nothing modelled the thing that actually identifies a diesel from
across a car park — the sharp metallic clatter from near-instant pressure rise
after injection delay.

### Only the differential's ratio mattered, not its teeth

Ring runout and eccentricity modulate the whine once per **ring** revolution, but
nothing modelled it — so a 40/10 and an 80/20 axle, which are different hardware
with the same 4.0 ratio, rendered identically apart from their noise seed.

## Decision

### 1. A shared load tilt, applied to the broadband term only

```
tilt(load, k) = 1 + k·load
```

Applied to each load-taking synth's **broadband** term and *not* its tonal one —
that asymmetry is the whole mechanism, because scaling both equally is precisely
what made load a pure gain.

| synth | broadband term | k |
|---|---|---|
| engine | mechanical noise bed | **3.0** |
| motor | filtered noise bed | 1.0 |
| forced_induction | compressor hiss | 1.0 |

**At load 0 the tilt is exactly 1.0**, so an unloaded machine renders
bit-identically to 2.2.0 — the same bounded-review property ADR-005 used for the
reference RPM, and for the same reason.

The engine needs k = 3 rather than the default 1. That is not arbitrary tuning to
hit a number: with k = 1 the measured centroid still *fell*, because the engine's
own low-frequency terms grow so much faster (intake ~31×, broadband ~6×). k = 3
is what makes the direction correct, and it is defensible independently — piston
slap, valvetrain clatter and injector noise all rise steeply with cylinder
pressure.

Result:

| | load 0.1 | load 0.5 | load 0.9 |
|---|---|---|---|
| motor | 2077.7 Hz | 2157.3 Hz | **2252.8 Hz** |
| engine | 8892.1 Hz | 8914.0 Hz | **9031.1 Hz** |

Both now rise monotonically, and RMS still rises too — harder work is louder
*and* brighter, which is what it should be.

### 2. Diesel injection clatter

A second, much sharper impulse riding behind the main combustion pulse: it starts
after an injection delay (0.12 of the combustion window) and decays 8× faster,
putting its energy high in the spectrum where the rattle lives.

It is **point-sampled deliberately**, unlike the combustion pulse that ADR-005
had to integrate: this is broadband noise shaped by an envelope, not a narrow
deterministic envelope on its own, so sample-lattice alignment cannot notch it.

Measured at 2000 rpm, load 0.8, 6 cylinders: **diesel 9747.4 Hz centroid vs
gasoline 8531.3 Hz** — a 14% margin, far more than a resonance retune could
produce.

### 3. Differential ring-revolution modulation

```
ring_rev_freq = (rpm/60) · pinion/ring
```

A shallow modulation at that rate (`0.88 + 0.12·cos`) — the slow warble a worn or
badly-shimmed diff has. 40/10 and 80/20 now render differently.

This makes `sample_position` **live** in `differential.cyr`, where it had been
vestigial (written every block, read by nothing) since the port. The comment
recording that has been corrected.

## Consequences

- **Six goldens move**: all three engine types, motor, turbo (the load tilt) and
  differential (ring modulation). Diesel moves twice over, for the tilt and the
  clatter. gear, transmission, turbine, clock, belt and chain take no load and
  were not touched — they are bit-identical to 2.2.0.
- `tests/spectral.tcyr` grows to 41 assertions, now including centroid ordering
  by load for both motor and engine, `tilt(0) = 1.0` exactly, and the diesel
  margin being *substantial* (>1.05×) rather than merely present.

## Re-scoped, with one correction to ADR-006

**Source-body impulse responses — my earlier cost estimate was wrong.** ADR-006
said this "needs a convolution path and IR assets; larger than the rest of this
arc combined." The convolution path **already exists**: naad ships
`naad_convolution_from_ir`, `_process_sample`, `_process_block` and a full FFT
partition engine. The real obstacle is different and more interesting: ghurni
*already* models exhaust resonance with a bandpass filter, so adding convolution
on top would be a second, redundant mechanism. The genuine question is whether to
**replace** the bandpass with a convolution — an architectural change, and one
that needs its own ADR rather than being smuggled in here.

**Turbine rotor slap and multi-spool** moves to Arc 4 (breadth). Helicopter blade
slap is an impulsive phenomenon with its own physics, and multi-spool is a jet
architecture; both are closer to new machines than to a deeper turbine, which is
the question ADR-006 flagged and this is the answer to it.
