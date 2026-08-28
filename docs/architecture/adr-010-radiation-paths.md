# ADR-010: Radiation Paths — the engine gets a pipe

**Status**: Accepted
**Date**: 2026-08-28
**Release**: 2.6.0 ("Radiation Paths")

## Context

[ADR-009](adr-009-no-stems-yet.md) deferred per-component stems after measuring
that ghurni had nothing to split: the engine modelled its exhaust as a bandpass
on a *separate white-noise bed*, the combustion pulse train never passed through
it, and the buffer labelled EXHAUST carried 3.6% of the energy with **negative**
firing-period autocorrelation. It set a numeric condition for reopening the
question.

This release builds the thing that was missing. It is the largest deliberate
audio change in the project's history.

## Decision

**The exhaust becomes a quarter-wave waveguide, and the combustion pulse train
goes down it.**

A pipe closed at the exhaust valve and open at the tailpipe is a quarter-wave
resonator: standing waves at `f_n = (2n-1)·c/(4L)` — odd harmonics of a
fundamental fixed by the *pipe*, while the excitation sweeps with RPM. One naad
`DelayLine` per engine, used bidirectionally:

```
closed (engine) end : pressure reflects with +1
open (tailpipe) end : pressure INVERTS; -R reflects, (1-R) escapes

fwd[n] = in[n] - R · loss(fwd[n-D]),   D = fs / (2·f1)
```

The **negative** loop gain is the whole point: it puts resonances on the odd
series, where a positive-feedback comb would put them on every multiple. A
one-pole lowpass inside the loop makes each successive reflection duller (wall
and viscous loss, plus the muffler a reflected wave traverses twice per round
trip); without it the comb rings equally to Nyquist and reads as a metallic buzz.

The mouth tap is read after the write at `D/2` — the one-way transit — and what
escapes is differentiated by a first-order highpass, because an acoustically
small monopole radiates the *time derivative* of its volume flow.

**`exhaust_resonance` is reinterpreted, not replaced**: it was a bandpass centre,
it is now the quarter-wave fundamental `f1 = c/4L`. Same field, same units, same
public accessors — and for the first time it sets something a listener would call
the exhaust note. The shipped per-type constants turn out to imply sensible
geometry at `c ≈ 550 m/s` (a hot pipe): gasoline 4cyl 230 Hz = 0.60 m, diesel
6cyl 170 Hz = 0.81 m. A bigger engine really does carry a longer manifold.

The duct sits **after** the rev-limiter, stall, shutdown and startup gates, so a
fuel cut and a stall now reach the tailpipe. Until now the exhaust term could not
hear any of them, because it was pure noise.

Not one `noise_next_sample` draw was added, removed or reordered. The old
bandpassed bed survives as the duct's *secondary* excitation rather than as the
exhaust term itself.

## The correction that matters most: ADR-009's criterion is passed by a wire

ADR-009 wrote a number so the question could not be deferred by inattention:

> exhaust path must carry **>25%** of mean-square energy AND a **positive
> firing-period autocorrelation exceeding today's mono (+0.597)**.

**That criterion cannot detect a duct.** Measured on this tree, routing
combustion straight into the exhaust term with *no delay, no filter, no
resonance, no pipe at all*:

| | 2.5.1 | **a literal wire** | bar |
|---|---|---|---|
| exhaust share | 3.59% | **66.12%** | >25% — passes |
| exhaust AC@441 | −0.0401 | **+0.7430** | >+0.597 — passes |

Three independent verifiers reached the same conclusion by three different
routes — a `feedback = 0` control (61.9% / +0.709), a `reflection = 0` control
(58.6% / +0.718), and the wire above. A fourth demonstration: the exhaust
autocorrelation is bit-for-bit identical while the share is swung from 64% to 44%
by a routing constant.

The criterion measures **"the exhaust buffer now contains the combustion pulse
train"** — which was ADR-009's literal complaint, so it is not the wrong
question, merely a much smaller one than *"does ghurni model a radiation path?"*
It is a **routing test**. It is fine as the gate it was written to be, and it
must **never be cited as evidence that a duct exists**.

It is met, and comfortably: share 3.59% → **64.3%**, exhaust AC −0.040 → **+0.717**,
with mono autocorrelation *rising* 0.654 → 0.664 (the engine did not get less
periodic to buy a periodic exhaust) and level +0.21 dB. But that is not why this
release is justified.

### What actually carries content

`tests/spectral.tcyr` now asserts the two things a bandpass cannot fake, and both
were verified to **fail against 2.5.1 and pass against 2.6.0**:

1. **The series is odd.** At 3300 rpm the firing rate is 110 Hz, so with
   `f1 = 250 Hz` neither `2·f1 = 500` (500/110 = 4.55) nor `3·f1 = 750`
   (750/110 = 6.82) is a firing harmonic — the source puts no line on either.
   Any difference between them comes from the pipe, and `3·f1` is louder. A
   bandpass has one peak and no series.
2. **It booms where the physics says, and deeper than a bandpass can.** With the
   shipped `f1 = 230 Hz`, a firing harmonic lands on the mode at `rpm = 6900/m`
   — 2300 rpm for m=3, with 2000 rpm between harmonics. Nothing is fitted;
   6900/m falls out of a constant that already shipped.

The bar on the second is **1.6, chosen to discriminate rather than to be true**:
a bare "boom > no-boom" assertion passes on 2.5.1 as well, because the source's
own harmonic lands on 230 Hz at 2300 rpm whether or not a pipe exists. Only the
*depth* separates them — measured 1.318 for the bandpass and 1.937 for the duct.

## It was listened to

Roughly two hundred measurements across seven agents could not answer the only
question that finally mattered, and it was the top risk on the release: **does it
sound like an exhaust?** A/B audio was rendered at a steady 3000 rpm and over a
900 → 6500 rpm sweep, both members of each pair sharing one gain so the level
difference was audible rather than normalised away, and reviewed by ear:
*"doesn't sound hollow or thin, slight frequency changes but sounds pretty close
to exhaust."*

That is what authorised the tuning, and it is worth recording that no metric in
this document would have caught the alternative. The octave-band rebalance is
real and larger than the +0.21 dB level change suggests:

| band | change |
|---|---|
| 45–90 Hz | −4.55 dB |
| 90–180 Hz | −4.55 dB |
| 180–355 Hz | −0.20 dB |
| 355 Hz and up | +0.4 to +0.8 dB |

and the first five firing harmonics fall from 8.1% to 4.2% of total energy. The
engine trades some bottom end for a radiated character. `GHURNI_DUCT_RAD_HZ` is
520 Hz rather than the ~1.8 kHz free-field figure for a bare 60 mm pipe, because
what is modelled is a whole exhaust *system* heard close to the ground; at the
free-field value the duct is technically more correct and audibly a hiss.

## Consequences

- **Three of 28 golden assertions move** — gasoline, diesel and hybrid. All nine
  non-engine synths are bit-identical, which is the correct blast radius for a
  change confined to `src/engine.cyr`. Reasons are recorded beside each value in
  `tests/goldens.tcyr`.
- **The hybrid golden moves for an inaudible reason** and will be misread as an
  electric-drive regression if this is not said plainly: `duct_drive` is 0 for
  HYBRID, so the EM whine is untouched; only the small `bed_gain = 0.25`
  flow-noise term now traverses the duct. Measured level change: **−0.004 dB**.
  The checksum moves; the sound does not.
- **Cost is negligible.** Measured A/B on `cyrius bench`: +24 ns/sample on
  `synthesize`, +10 ns on 64-sample blocks, and *within noise* at 512 and 4096.
  Against a 22.68 µs realtime budget that is about 0.1%.
- 640 assertions across 10 suites, up from 633.
- **14 new public symbols** (8 `GHURNI_DUCT_*` constants, 3 fields and their
  accessors). Cyrius has no visibility control, so struct fields are public API.
- Memory: a 2048-slot delay line is 16 KB per `GhEngine`, allocated
  unconditionally including for HYBRID where the duct is silent. Sizing it from
  `exhaust_resonance` at construction is **not** safe, because the parameter is
  settable afterwards and would silently truncate.

### The 2.5.0 tripwire is retired, because it was wrong

ADR-009 pinned: *"detuning both bodies to 60 Hz moves the engine less than
0.45 dB — they are not ducts,"* with the stated intent that a real duct would
make it fail. **2.6.0 built that duct and the assertion still passed** — measured
−0.27 dB, comfortably inside the bracket. The premise was wrong physics: a
waveguide's throughput is roughly level-invariant under retuning, because the
escape coefficient and the radiation filter do not care where the modes sit.
Detuning a real pipe moves its **colour**, not its level.

Shipping a false assertion is worse than shipping none, so it is replaced — with
a test that retuning changes the *render*, and an explicit pin that the level is
*not* what moves, so the retired premise cannot be reintroduced by someone
reading only its old message.

## Deferred, with reasons

- **The intake duct** (Helmholtz plenum + runner). Arc 3c's second modelling
  item; nobody prototyped it. It carries 1.3% of the energy and the exhaust was
  the whole point. Adding a second, unmeasured resonator to the largest audio
  change in the project's history is how you get a golden you cannot explain.
- **Stems.** ADR-009's criterion is met and **is no longer the blocker** — say
  that plainly, because four deferrals have made it hard to hear. The occlusion
  use case now works: muting the exhaust term moves the engine **−4.39 dB**,
  where 2.5.1 moved −0.11 dB. What remains are two different blockers, each with
  an exit condition. (1) The intake lane is still 1.3% of noise, and ADR-009's
  taxonomy is keyed on radiation path — shipping it now hands consumers two real
  lanes and one fake, the exact failure ADR-009 refused. (2) The energy split
  between duct and structure is set by hand and is one release old; it should
  survive a release in consumers' hands before an occlusion curve is built on it.
- **Two-stroke expansion chamber.** A divergent-then-convergent cone pair whose
  purpose is a positive reflection followed by a negative one — a different
  device, and this model is wrong for it. Characterised, not papered over: the
  two-stroke misses the ADR-009 share bar at 21.9%.

## Alternatives rejected

- **Modal bank** (8 parallel biquads at the odd harmonics). Measured a
  **stationary 92 Hz drone that does not track firing rate** — at 6000 rpm, where
  no firing harmonic can reach 92 Hz, a discrete peak sat 17.5 dB above its local
  floor. Also 383 ns/sample, and its level *fell* 3.38 dB at 6000 rpm where
  2.5.1's rose monotonically, which breaks the ADR-005 loudness law this arc was
  explicitly told to protect.
- **Inverting comb without a radiation stage.** Meets every number and is
  cheaper, but its comb's feedforward path *is* its input, so 85% of the raw
  combustion pulse sits in a buffer labelled EXHAUST having passed through one
  5 kHz one-pole. That is the pressure at the closed valve, not the radiated
  field — the wrong object with the right numbers, for a release whose
  consequence is a stem taxonomy keyed on radiation path. Its mono level also
  swings −2.16 to +1.99 dB across 800–7000 rpm, modulating ADR-005.
- **Convolution.** Closed in ADR-009 and unchanged: ADR-002 forbids reverb,
  naad's type is a room model, cost is 21×–7161× a biquad, and `cyrius distlib`
  concatenates `.cyr` text so the format cannot carry a binary IR.
