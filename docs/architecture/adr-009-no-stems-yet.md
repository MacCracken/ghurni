# ADR-009: Output Path — why ghurni has no stems yet

**Status**: Accepted
**Date**: 2026-08-28
**Release**: 2.5.0 ("Output Path")

## Context

Arc 3b promised two things: **per-component stems** (render exhaust / intake /
mechanical into separate buffers) and a decision on **source-body separation**
(replace the resonant bandpass with convolution). The roadmap said they belonged
together — "a stem *is* a pre-body signal, so deciding the stem boundary and
deciding where the body filter sits are the same decision."

That instinct was right and the reason was wrong, which is why four separate API
designs failed before the actual obstacle surfaced.

**Neither ships. The blocker is the same for both, and it is not an API question.**

## The measurements

Everything below was measured against this tree, not estimated.

### 1. The engine has no exhaust path to hand out

A/B on source: render the gasoline and diesel golden configurations, then render
them again with the `exhaust` and `intake` terms forced to zero — RNG draws left
in place so the noise sequence is unchanged — and compare RMS over 1 s.

| case | full | both apertures muted | Δ |
|---|---|---|---|
| gasoline 4cyl @3000 load 0.6 | 0.03849662 | 0.03757428 | **−0.211 dB** |
| diesel 6cyl @1500 load 0.9 | 0.05177036 | 0.05118233 | **−0.099 dB** |

Deleting **both** aperture paths outright — a more violent occlusion than any
consumer could ever apply — moves the engine's level by about a fifth of a
decibel, against a level JND of roughly 1 dB.

The flagship use case for stems is "occlude the tailpipe as the car turns behind
a wall while the airbox stays visible." Measured: **the car goes behind the wall
and nothing happens.**

### 2. And the exhaust buffer would not even contain the exhaust note

Energy share and firing-period autocorrelation, gasoline 4cyl @3000 (firing
period 441 samples at 44.1 kHz):

| path | share of mean-square | autocorrelation @441 |
|---|---|---|
| structure | 95.3% | **+0.625** |
| exhaust | 3.6% | **−0.051** |
| intake | 1.3% | −0.005 |

The periodicity that *is* the exhaust note lives entirely in the unfiltered
structural sum. `engine.cyr:525-528` is the only feed of `exhaust_filter` and it
is **white noise**; `combustion_sum` is accumulated at `:483` and added at `:595`
and never touches the exhaust body at all.

So a consumer would wire occlusion to a buffer labelled EXHAUST, hear nothing
while driving, and then lose an entire backfire behind a wall. That is worse than
having no stems: it is a plausible API that silently does the wrong thing.

### 3. The obvious repair does not work either

Every failed design converged on the same remedy — route combustion through the
exhaust body. Prototyped, with a second Q = 1.5 bandpass at `exhaust_resonance`:

| routed | exhaust share | exhaust AC@441 | mono energy |
|---|---|---|---|
| 25% | 6.3% | 0.000 | 0.600× |
| 50% | 14.4% | 0.133 | 0.315× |
| 100% | 85.6% | 0.410 | 0.088× (**−10.6 dB**) |

Even at 100% the "exhaust" path is *less* periodic than today's mono (0.410 vs
0.597) and the engine loses 10.6 dB. **A resonant biquad rings and smears a pulse
train; it does not shape one.** A bandpass is a tone control. A pipe is a
resonant delay with standing waves at odd harmonics of c/4L. Those are different
objects, and ghurni owns the first while the roadmap has been describing the
second.

### 4. Convolution — the other route to a real body — is closed

Not on cost grounds first. On scope:
[ADR-002](adr-002-scope-boundaries.md) states "ghurni does NOT implement Doppler,
reverb, or spatialization"; reverb is **dhvani**'s and spatialization is
**goonj**'s. naad's type is literally `ConvolutionReverb`, and
`naad_convolution_from_room` builds a shoebox room from wall materials. It is a
room engine, not a body engine.

Three further findings, all verified, each independently sufficient:

- **Cost.** Measured: biquad 35 ns/sample; convolution 737 ns at 64 taps, 11.4 µs
  at 1024, 250 µs at 22050. The smallest IR that can represent a 170 Hz diesel
  exhaust mode is ~870 taps, and the engine has *two* bodies — roughly the entire
  44.1 kHz realtime budget for one voice.
- **Distribution.** `cyrius distlib` builds `dist/ghurni.cyr` by **concatenating
  `.cyr` text modules**. The format cannot carry a binary IR at all.
- **naad's block path is broken.** `naad_convolution_process_block` is not a
  streaming convolution: it writes only `[0, block_len)` from the IFFT and keeps
  no overlap-add state, so it truncates the ring at every block boundary. Proved
  with a 4-tap all-ones IR and one impulse — sample path `1,1,1,1`, block path
  `1, 1−1ULP, 0, 0`. The `_direct` fallback is correct; only the fast path is
  broken. Queued for a naad patch and a `2.5.x` here to pull it; **ghurni calls no
  convolution symbol today**, so nothing shipping is affected.

## Decision

**Ship no stems API, in any shape, on any synth.** Not the three-lane radiation
split, not the two-lane tone/noise split, not the engine-only passage split, not
the three earlier character designs. No `GH_STEM_*`, no `*_process_block_stems`,
no stem bus.

Four taxonomies were argued and all four fail, but they fail for two different
reasons and only one of them matters:

- Every taxonomy keyed on **signal character** structurally cannot separate
  exhaust from intake, because `engine.cyr:527` and `:532` are the same operation
  on the same noise source.
- Every taxonomy keyed on **radiation path** separates them correctly and then
  hands the consumer 3.6% and 1.3% of the signal with none of its periodicity.

The second is the real finding. The blocker was never the API shape — it is that
**ghurni does not model a radiation path.** It models a tone-shaping filter on a
noise bed.

### The acceptance criterion

So this cannot be deferred a fifth time by inattention, the condition that ends
it is a number, not a judgement:

> Before any stems symbol ships, the engine's exhaust path must carry **>25% of
> mean-square energy** *and* a **positive firing-period autocorrelation exceeding
> today's mono (+0.597)**.

> ⚠ **CORRECTED BY [ADR-010](adr-010-radiation-paths.md) (2.6.0). This criterion
> is passed by a wire.** Routing combustion straight into the exhaust term with
> no delay, no filter and no resonance measures 66.12% share and +0.743
> autocorrelation — both bars cleared with no duct present at all. Three
> independent controls (feedback = 0, reflection = 0, and the bare wire) confirm
> it. What it actually tests is *"the exhaust buffer now contains the combustion
> pulse train"*, which was this ADR's literal complaint — so it is a **routing
> test**, sound as the gate it was written to be and **worthless as evidence
> that a radiation path exists**. 2.6.0 met it, but was justified on the
> odd-harmonic mode series and the boom depth instead; see ADR-010. The
> measurement protocol also matters and was not stated here: these figures are
> for a fresh engine over 1 s with no warm-up. Discarding a 4410-sample warm-up
> gives mono +0.654 rather than +0.597.

`tests/oracle_pins.tcyr` pins the externally-observable proxy today: detuning both
bodies to 60 Hz moves the engine less than 0.45 dB (measured −0.152 dB). The day
the engine gets a real duct, that assertion fails loudly and the question reopens
on evidence.

> ⚠ **IT DID NOT.** 2.6.0 gave the engine a real quarter-wave duct and this
> assertion still passed, at −0.27 dB. The premise was wrong physics: a
> waveguide's throughput is roughly level-invariant under retuning, because the
> escape coefficient and the radiation filter do not care where the modes sit.
> Detuning a real pipe moves its **colour**, not its level. The tripwire was
> retired in 2.6.0 and replaced with assertions that discriminate — see ADR-010.

Only the downward direction is asserted. Detuning *up* raises the level (+2.6 dB
at 9999 Hz), but a constant-Q bandpass has bandwidth f0/Q, so a higher centre
simply passes more white noise — that is an artifact and evidence of nothing.

### What ships instead

Two real defects the investigation turned up, both of exactly the class 2.4.0
shipped a whole release of, and both bit-identical at every golden:

**1. `GhEngine_set_intake_resonance` was a dead store.** `engine.cyr:354` retunes
the exhaust body from its field every block; `intake_filter` was built once in
`_new` from a constructor *local* and never retuned. Cyrius has no visibility
control, so that accessor is public API — a public engine parameter that provably
did nothing. Measured before the fix: `intake_resonance = 9999 Hz` changed **0 of
4410 samples**; `exhaust_resonance` changed **4410 of 4410**. Fixed by mirroring
`:354`, reapplying the same nyquist clamp. Bit-identical by default, because the
field holds exactly the value the filter was built with and naad's
`filter_biquad_set_params` only recomputes coefficients — `filter_biquad_reset` is
the separate call that touches the delay line.

**2. Transmission's ADR-007 load tilt was never wired.** `transmission.cyr:163`
computed `tx_tilt` and **no line read it** — the variable occurred exactly once in
the file, while `gear.cyr:222` and `differential.cyr:188` both wire theirs.
Transmission gained `set_load` in 2.4.0 and got load's amplitude half with none of
its spectral half: a pure fader, which is the exact behaviour ADR-007 exists to
remove. Now applied to `hiss`, the broadband term. Exactly 1.0 at load 0, which is
the default, so the golden holds; audio changes only at load > 0, which is why
this is a MINOR.

The regression test is deliberately wider than the bug: `tests/spectral.tcyr` now
asserts **every** load-taking synth gets brighter under load — engine, motor,
forced_induction, gear, transmission, differential. Testing transmission alone
would have fixed the instance and left the class, and it is the absence of exactly
this check that let the defect sit through two releases.

## Consequences

- **No golden moves.** All 28 golden assertions hold byte-for-byte.
- 643 assertions across 10 suites, up from 600.
- **Zero new public symbols.** A release whose headline is "we are not shipping
  the API" must not grow the API by accident.
- Arc 3b is re-pointed: **2.6.0 "Radiation Paths"** — model the exhaust and intake
  as real resonant ducts. Stems ship as a *consequence* of that, not before it.
  Breadth moves to 2.7.0.

## The cost, stated plainly

Per-component stems has now been deferred four times: Arc 2 → Arc 3 (2.2.0) →
Arc 3b (2.4.0) → and now out of 2.5.0. Consumers have been told it is coming.

The difference is that this time we know why, the condition that ends it is a
number, and that number is pinned in the suite.

I would rather tell kiran and dhvani "ghurni cannot give you a tailpipe yet, here
is the measurement, here is the release that will" than hand them three buffers,
watch them build a bus layout and an occlusion curve on top, and then move 12 dB
of energy between those buffers in 2.6.0. The second failure is far more expensive
and it arrives after they have shipped.

## Corrections to the roadmap and earlier ADRs

The project treats correcting its own stale claims as part of each release.

- **roadmap.md — "every synth sums exhaust + intake + mechanical into one mono
  buffer."** False for nine of ten synths. No exhaust or intake synthesis term
  exists anywhere outside `src/engine.cyr`. This is the third error from the same
  survey that produced the two bullets ADR-008 already corrected.
- **roadmap.md — "a stem *is* a pre-body signal, so the stem boundary and the body
  filter placement are the same decision."** Right conclusion, wrong reason, and
  the wrong reason misdirected four designs. They are the same decision not
  because a stem is taken pre-body, but because there is no radiation path to take
  a stem *from*.
- **[ADR-006](adr-006-acoustic-depth.md) — "per-component stems is an API change,
  not a timbral one."** Exactly backwards, and this single line is why the item was
  routed to a control-surface arc and then to an output-shape arc instead of an
  acoustic one. It is a synthesis change first.
- **[ADR-008](adr-008-events-and-control.md) — "it needs a shape decision that
  applies to all ten synths at once."** That premise generated four failed designs:
  it forces a taxonomy, and a taxonomy partitions all ten whether or not all ten
  have anything to partition. Its sketched fixed-arity `process_block_stems(self,
  a, b, c)` is independently disproved — motor's entire output is two terms
  (`hum + flt`), so any K ≥ 3 uniform API leaves motor a permanently dead lane.
- **[ADR-007](adr-007-load-tilt-and-diesel.md) — "a shared load tilt, applied to
  the broadband term only."** Not true of transmission until this release. Live
  bug, fixed here.
- **roadmap.md — "`sample_position` is vestigial in two synths."** Wrong, and
  ADR-008 already narrowed it once from three. Nothing *inside* transmission or
  forced_induction reads it, but Cyrius has no visibility control, so the
  accessors are public API returning a meaningful value — samples rendered.
  Deleting the write would make a legitimate consumer read return 0 forever;
  deleting the accessor would be a breaking change a MINOR may not make. Both are
  now documented rather than removed. **The item is closed, not deferred.**
- **roadmap.md — "it grows the public surface by a third."** Overstated, and it
  steered the whole argument toward symbol-count minimisation. The designs cost
  +4, +11, +20 and +42 against ~185. Surface size was never the obstacle; signal
  content was.
