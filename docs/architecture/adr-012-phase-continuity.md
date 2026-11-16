# ADR-012: Phase Continuity — phase is integrated, not derived from position

**Status**: Accepted
**Date**: 2026-11-16
**Release**: 2.8.0 ("Phase Continuity")

## Context

The 2.7.2 P(-1) sweep found a family of defects it could not fix in a patch,
because every fix changes rendered audio. Three auditors found the central one
independently.

**Phase was position × current speed.** Six synths computed a periodic phase as
`f(absolute sample position × the rate in force now)`: the engine's crank angle,
the gear's ring envelope, the clock's beat grid, the differential's ring
revolution, the chain's engagement and lap, and the belt's loop rotation. That is
correct only for a rate that has never changed. Once the rate moves, the phase
jumps by `position × Δrate`, which grows without bound as the stream ages. The
sweep measured this a minute into a stream:

- the engine's firing slots changed 3,462 times during a 200 ms ramp where 19
  firings were due;
- the differential's 16 Hz ring warble ran at 96 Hz;
- the chain's impact train ran 6× too fast.

The engine also ran its crank backward while it slowed. A gasoline 4-cylinder
gliding from its 800 rpm default to its 100 rpm floor really turns about 241° in
the first 0.1 s, so a second cylinder fires at about 65 ms. The old crank never
reached 180°.

Measured again for this ADR, black-box, on 2.7.2 against 2.8.0:

| Measure | 2.7.2 | 2.8.0 |
|---|---|---|
| Wall clock ramped 1→2 Hz over 10 s, 100 s in: ticks counted (16 due), blocks of 4410 / 441 / 1000 | 19 / 25 / 19 | 16 / 16 / 16 |
| Chain impacts in a 200 ms ramp, 1 s in / 60 s in | 119 / 94 | 125 / 126 |

The same sweep found six neighbours, each audio-changing:

- **A clock tick was a share of the beat.** Decay and both tick windows scaled
  with the period. A wristwatch tick stayed above 0.01 for 89 samples at its 8 Hz
  and for 1,201 at 0.1 Hz.
- **A misfire removed one sample.** The flag cleared after the first sample of
  the combustion window, and the other 140 samples of the firing still fired.
- **The hybrid's STARTUP drove one synth at two pitches.** The starter whir
  borrowed `whine_synth`, so each sample retuned it to the drive's pitch, then to
  220 Hz, and advanced it twice.
- **Block-stepped terms.** The transmission read its loudness from the raw rpm,
  and its load once per block. The differential read its load and coast flank
  once per block. The belt advanced its smoother and then discarded the value,
  computing everything from the raw rpm. Measured as mean |x| over the 5 ms after
  a step at a block edge, against the 5 ms before:

  | Step | 2.7.2 | 2.8.0 |
  |---|---|---|
  | transmission, rpm 3000 → 0 | 0.025 | 0.982 |
  | belt, rpm 2000 → 0 | 0.004 | 0.826 |
  | differential, load 1 → 0 | 0.713 | 0.975 |
  | differential, power → coast | 0.604 | 0.960 |

- **Partials did not survive low sample rates.** naad zeroes an additive partial
  at or above Nyquist, in `additive_new`, `additive_set_partial` and
  `additive_set_fundamental` alike, and never restores it. Every synth built its
  additive tone at 100 Hz, so below a 612 Hz rate the top partial was gone for the
  life of the object. Four filters and oscillators sat at fixed centres too: the
  transmission's synchro at 4 kHz, the blow-off valve at 2 kHz, the belt's flap at
  500 Hz and the turbo whine at 100 Hz. Those constructors failed at 8 kHz, 4 kHz,
  1 kHz and 200 Hz.
- **A negative ratio was a 1 Hz mesh.** `rpm / -3.2` gave a negative output speed,
  which the mesh clamp turned into a 1 Hz tone. Every downstream `set_rpm` clamps
  a negative rpm to silence.

## Decision

### 1. Phase is anchored

Each of the six phases is now held as an **anchor**: a phase value, the sample
position it belongs to, and the rate in force there. Each sample computes

```
phase = anchor + (position - anchor_pos) × rate
```

in the expression's original form and units. The synth re-anchors whenever the
rate changes, at the sample where the new rate takes effect, by evaluating that
expression with the old rate and reducing it into one period. The rate in force
over a sample interval drives that interval, so the phase is the integral of the
speed history.

This is a plain accumulator once the rate changes every sample, as it does while
a smoother glides. It was chosen over a plain accumulator for two properties:

- **A rate held from the first sample on renders the old bits.** Before the first
  sample the stored rate is 0, so the first sample anchors at its own position and
  adds `0 × 0`. With the anchor at 0, `0 + x` is `x`, and each expression is the
  2.7.2 expression term for term. Every gear, motor, turbine and default-rate
  clock case is bit-identical, and so is every engine case at its 800 rpm start
  speed. That bounds what a reviewer must check: audio differs only where a rate
  changed, which is where the old phase was wrong.
- **No drift.** During a steady stretch the phase is one product from the anchor,
  not millions of rounded additions.

Because the first sample anchors at its own position, **no synth's audio depends
on its `sample_position` any longer**. Two otherwise identical synths, one told it
is a minute into its stream, render the same bits through any sequence of speeds
and block sizes. `sample_position` still counts rendered samples and is still
public API (ADR-009).

Per synth:

| Synth | Anchored quantity | Re-anchored when |
|---|---|---|
| engine | crank angle, degrees in `[0, cycle)` | rpm per sample changes |
| gear | mesh cycles in `[0, 1)` | the mesh frequency changes, once per block |
| differential | ring revolutions in `[0, 1)` | the ring step changes |
| chain | lap fraction; engagement phase is `x mod period` | the engagement period changes; a stopped chain holds |
| belt | loop rotation, radians in `[0, 2π)` | the rotation rate changes |
| clock | samples since the last beat, and the beat count | see below |

**The clock counts time, not phase.** `beat_pos` is the number of samples since
the last beat instant, which may be fractional. A beat falls when it reaches the
current period. A rate change keeps the time since the last tick, so the next tick
comes one new period after the last, as an escapement's does, and a tick that is
still sounding keeps its own length. At the four default rates every quantity
involved is exact at the common sample rates (`sr / rate` is 5512.5, 22050, 44100
or 8820 at 44.1 kHz), so the result matches the old position-derived phase bit for
bit.

### 2. A clock tick has a duration

The attack window, the ring window and the decay are measured in the type's
**default** beat rather than the current one: `u = beat_pos / (sr / default_rate)`.
At the default rate that divisor is the period itself, so nothing moves there.
Elsewhere the tick keeps its length: 0.02 and 0.15 default beats for the windows,
and a decay of 0.375 ms (wristwatch), 5 ms (wall clock), 30 ms (grandfather) or
1 ms (pocket watch).

### 3. A misfire skips a firing

The per-cylinder flag is a small state machine:

| State | Meaning |
|---|---|
| 0 | none |
| 1 | armed while the cylinder's window was open; that firing completes |
| 3 | armed, and the cylinder has been seen outside its window |
| 2 | skipping the firing in progress |
| 4 | skipping, and armed again for the next firing |

The window entry is detected by state, not by a `cyl_norm` threshold, because a
threshold can be stepped over at high rpm or a low sample rate. At 3000 rpm on one
cylinder the skip now lasts the whole 141-sample window.

### 4. The starter has its own synth

`starter_synth` is built exactly as `whine_synth` is. A non-hybrid engine never
advanced `whine_synth` except for the starter, so its STARTUP keeps its bits. Only
the hybrid's STARTUP moves.

### 5. Block-stepped terms ramp

- **Transmission:** a 50 ms smoother on the input rpm, for the loudness law, and
  a 20 ms smoother on load. The input rpm is used, not the output rpm, so loudness
  stays independent of the selected gear (ADR-005).
- **Differential:** 20 ms smoothers on load and on the coast flag. The coast
  factors blend as `(1 − c) + c × k`, so `c = 0` and `c = 1` give exactly 1.0 and
  exactly `k`.
- **Belt:** the smoother it always advanced now drives its loudness, flap
  amplitude and loop rate.

Under power at load 0, every factor is exactly 1.0, as before.

Gear, motor and turbine still step their loudness at block edges when rpm
changes. They have no smoother at all, by the oracle's design, so their pitch
steps too, and the two stay consistent. Giving them one changes their character
and is a separate decision.

### 6. Every frequency stays under Nyquist

Three helpers in `dsp.cyr`:

- `ghurni_additive_new(n, sr)` builds at `min(100, 0.49·sr/n)`.
- `ghurni_below_nyquist(f, sr)` caps at `0.49·sr`.
- `ghurni_fund_clamp(f, hi)` keeps the 1 Hz floor from overriding the ceiling.

At any common rate each returns its input unchanged. Every synth now builds and
renders finite audio at every rate from 1 Hz up, and keeps every partial.

### 7. A negative ratio is a reverse gear

`ghurni_transmission_output_rpm` returns `rpm / |ratio|`, a speed, which is what
every rpm in this library is.

### 8. Refactors that needed new names

- `ghurni_zeros(n)` replaces ten copies of the zero-fill loop in `synthesize`.
  It is the one place 2.10.0's presized buffers will change.
- `ghurni_mixer_mix` is the one channel loop for mono and stereo.
- The four tag dispatchers read a table. `ghurni_kind_method(kind, method)`
  returns a function pointer from it, and the dispatchers call it with `callptr`.
  The table is filled on first use, because a library's top-level statements run
  in the consumer's program before anything it initialises. `&fn` and `callptr`
  are compiler builtins (cyrius 6.0.70+), so the bundle needs no new stdlib leaf.
  `ghurni_synth_process_block` now returns the synth's own code.

## Consequences

**Seventeen goldens moved**, each in a synth this release changes on purpose, and
`tests/goldens.tcyr` records why. Level changes run from −2.65 dB (the
transmission, which fades in alongside its pitch instead of sounding at full
level from sample 0) to +0.77 dB (SHUTDOWN). Twelve goldens were added for the
paths that changed: mid-stream ramps on six synths, set clock rates, the hybrid's
STARTUP, the reverse gear, and two low rates.

A raw-bit render of 1,344 cases (about 16.8M samples) against 2.7.2 confirms the
blast radius:

- **Unchanged:** every case at a speed held from the first sample. That is all
  140 gear, 140 motor and 105 turbine cases, all 20 default-rate clock cases, the
  engine at its 800 rpm start speed, every hybrid case except STARTUP, and the
  synths at rpm 0.
- **Changed:** every case where a rate changed.
- **Newly working:** 54 cases whose constructors failed at 8 kHz and 1 kHz now
  render.

One oracle pin measured a transient that 2.7.2 rendered wrong: "engine at rest is
well under running" compared 0.1 s one-shots whose "rest" was the glide from
800 to 100 rpm. It now measures the settled 100 rpm floor over a full cycle.

**Behaviour a consumer can notice:**

- `ghurni_clock_set_tick_rate` changes the beat, not the length of each tick.
- MISFIRE silences a whole firing.
- A fresh transmission or belt fades in over its smoother instead of starting at
  full level, and a step to rpm 0 glides out.
- `ghurni_transmission_output_rpm` is positive for a reverse gear.
- `sample_position` no longer affects audio. A consumer who rewound it to
  resynchronise a phase was relying on the defect.

**Cost:**

- Per object: one more AdditiveSynth per engine, and two more smoothers each on
  the transmission and the differential.
- New fields: three anchor fields on each phased synth, and two on the clock.
  Each is a public `#derive(accessors)` pair.

## Alternatives considered

- **A plain accumulator, `phase += rate` per sample.** Simpler, and it differs
  from the anchored form only in rounding. It moves every golden in all six
  synths, including the constant-speed ones, so a reviewer could not tell a real
  change from a rounding change. It also accumulates rounding error over a long
  steady stretch. Rejected.
- **Keep the clock's beat fraction across a rate change.** This is a phase
  accumulator in beats. It makes a tick that is still sounding jump forward or
  back when the rate changes, and it cannot reproduce the default-rate bits.
  Rejected for time since the last tick.
- **Anchor at position 0, not at the first rendered sample.** This keeps the old
  bits for a constant rate too, but leaves audio dependent on `sample_position`.
  Rejected.
- **Smoothers for gear, motor and turbine.** They have none, so their pitch glides
  would be new behaviour, not a repair. Left for a separate decision.
