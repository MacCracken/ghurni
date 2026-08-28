# ADR-006: Acoustic Depth — the machines sound like themselves

**Status**: Accepted
**Date**: 2026-08-28
**Release**: 2.2.0 ("Depth")

## Context

[ADR-005](adr-005-rpm-loudness-law.md) fixed *how loud* each synth is. This one
is about *what it sounds like*. Breadth stays frozen — no new synths — because
five of the ten already shipped were missing something structural about the
machine they model. Each was verified against the source before being accepted
as a defect:

| Verified | Finding |
|---|---|
| `src/gear.cyr:109`, `transmission.cyr:73`, `differential.cyr:73` | All three mesh whines were a **single `WAVEFORM_SINE`** — no harmonics, no sidebands. A pure sine is the one thing meshing gears never sound like. |
| `src/engine.cyr` | **`GH_ENGINE_HYBRID` is documented "electric-with-whine" and ran the full combustion path.** The only per-type differences were two resonance constants and a roughness factor — so the one engine type advertised as electric was a quiet petrol engine. |
| `src/forced_induction.cyr:214` | Turbo and supercharger whined at plain **shaft rate** and differed *only* in spool lag. At the same spool speed they were spectrally identical. |
| `src/chain_drive.cyr` | `engagement_frequency` uses **sprocket_teeth**, so `links` was stored, clamped and documented ("typically 100–120") but **audibly inert**. A 40-link and a 500-link chain rendered identically apart from their noise seed. |
| `src/clock.cyr` | Every beat was **identical and perfectly periodic** — no tick/tock asymmetry, which is what made a clock read as a metronome rather than a mechanism. |

## Decision

### 1. Mesh tone becomes a 3-partial additive synth

`gear`, `transmission` and `differential` replace their pure-sine oscillator with
`additive_new(f, 3, sr)` — the pattern `motor` and `turbine` already use.

Gear's upper partials scale with **`brightness`**, the per-material constant that
until now only fed the resonance filter. So material finally changes *timbre*:

| material | brightness | measured h2/h1 | h3/h1 |
|---|---|---|---|
| steel | 0.9 | 0.546 | 0.278 |
| brass | 0.7 | 0.426 | 0.219 |
| cast iron | 0.5 | 0.307 | 0.153 |
| nylon | 0.2 | 0.124 | 0.063 |

The fundamental is unchanged across all four (0.0575), so this is a timbre change
and not a pitch change — `tests/spectral.tcyr` asserts the peak still lands at the
mesh frequency. Transmission and differential use fixed 1.0 / 0.4 / 0.2 partials;
neither has a material parameter to key off.

The fundamental is clamped to **nyquist/3**, not nyquist, so the third partial
cannot alias.

### 2. HYBRID is an electric drive

For `GH_ENGINE_HYBRID` the combustion loop does not run at all (`n_fire = 0`).
In its place is a 2-partial EM whine at `(rpm/60) × 8` poles — the same
relationship `motor.cyr` uses — with amplitude driven by load, because traction
motors whine louder under torque. The exhaust and intake beds are pulled to 0.25
of their level: an EV in electric mode still moves air but has no exhaust note.

Measured: hybrid peaks at 398.4 Hz @3000 rpm and 199.2 Hz @1500 rpm (predicted
400 / 200, within one 5.4 Hz bin), where gasoline and diesel peak at 102.3 Hz.

The whine synth follows the optional-oscillator pattern `turbine.cyr` uses for
`duct_osc`: built only for HYBRID, `0` otherwise, guarded at the use site.

### 3. Turbo and supercharger whine at their own orders

A turbo's centrifugal wheel has roughly ten blades and whistles high; a Roots
blower's three lobes give the low, hard whine it is known for. So the whine
frequency becomes `(spool/60) × order`, with `GHURNI_TURBO_BLADES = 10` and
`GHURNI_SUPERCHARGER_LOBES = 3`.

Measured at 6000 rpm × ratio 2.0: turbo 1798 Hz, supercharger 538 Hz — a ratio of
3.34, which is 10/3 as intended.

### 4. Chain link count becomes audible

A chain is a closed loop of N links, so any variation between links (wear, a
stiff link, the master link) recurs once per **lap**:

```
lap_freq = engage_freq / links
```

A shallow amplitude modulation at that rate — `0.85 + 0.15·cos(2π·lap_phase)` —
is what makes the link count audible, and it is the slow wobble a worn chain has.

### 5. Clock tick and tock differ

Odd beats are rendered at 0.82× amplitude and 0.78× decay, as the lighter pallet
impact is. Measured peaks alternate 0.205 / 0.172 / 0.216 / 0.177 — an
alternation, not a decay, which `tests/oracle_pins.tcyr` asserts explicitly.

## Consequences

- **Five goldens moved** (gear, transmission, differential, turbo, chain), all
  updated in the same commit with reasons recorded in `tests/goldens.tcyr`.
- **Two goldens were fixed rather than moved**, and both were blind spots the
  change exposed:
  - The clock golden ran for **0.5 s**, but a grandfather clock beats at 1 Hz —
    so it contained only beat 0 and *never covered a tock*. The tick/tock change
    did not move it. Widened to 3 s.
  - **HYBRID had no golden at all** — the engine type whose behaviour changed
    most had nothing pinning it. Added.
- `tests/spectral.tcyr` grew from 13 to 29 assertions, covering harmonic
  ordering by material, the hybrid EM whine tracking RPM, and the turbo/blower
  ratio being *specifically* 10/3 rather than merely "different".

## Deferred to a later arc, with reasons

The original Arc 2 listed eleven items. Six are not in this release, and the
split is deliberate rather than a shortfall — each of these is a design problem
of its own rather than a missing structural feature:

- **Load produces no spectral tilt.** Load already drives roughness and gain, so
  this is a refinement rather than an absence; doing it properly means deciding a
  filter-tracking law for every synth, which is its own ADR.
- **Diesel is a retuned Gasoline** — no valvetrain, injector or timing-drive
  content. That is additive synthesis work of the same size as this whole
  release.
- **Differential ring-revolution modulation and drive/coast asymmetry.**
  Drive/coast needs a torque-direction input the API does not have — it belongs
  with Arc 3's control-surface work, not here.
- **Turbine rotor slap and multi-spool.** A helicopter and a jet are arguably
  different machines rather than a deeper turbine; this may belong in breadth.
- **Per-component stems** (exhaust / intake / mechanical rendered separately) is
  an **API** change, not a timbral one. Moved to Arc 3 "Events & Control", where
  the rest of the control-surface work lives.
- **Source-body impulse responses.** Needs a convolution path and IR assets;
  larger than the rest of this arc combined.

These become **Arc 2b — `2.3.0` "Depth II"**, with the arcs after it renumbered.
