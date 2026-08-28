# ADR-005: RPM-Dependent Loudness, and Deliberate Acoustic Divergence

**Status**: Accepted
**Date**: 2026-08-28
**Release**: 2.1.0 ("Rest State")

## Context

ghurni's stated thesis is that **RPM is the fundamental parameter** — every
sound derives from rotational speed. Measurement says otherwise. Rendering each
synth at rest, at a low speed, and at its nominal speed (RMS, 0.2 s, 44.1 kHz):

| synth | @ 0 RPM | low | nominal | |
|---|---|---|---|---|
| motor | 0.184407 | 0.144094 @100 | 0.185256 @1500 | rest ≈ nominal, and **non-monotonic** |
| belt_drive | 0.084913 | 0.080266 @100 | 0.069786 @2000 | **louder stopped than running** |
| turbine | 0.106632 | 0.250295 @400 | 0.251488 @8000 | 99.5% of nominal at 5% of nominal speed |
| gear | 0.026472 | 0.099980 @50 | 0.106961 @1500 | 94% of nominal at 3% of nominal speed |
| differential | 0.019027 | 0.110695 @100 | 0.140681 @3000 | |
| transmission | 0.016178 | 0.098088 @200 | 0.106189 @3000 | |
| engine | 0.008977 | 0.030087 @300 | 0.026329 @3000 | **non-monotonic** |
| chain_drive | 0.000000 | 0.052312 @100 | 0.045019 @3000 | silent at rest, but **non-monotonic** |
| forced_induction | 0.000000 | 0.003849 @500 | 0.044083 @5000 | correct — scales via spool |
| clock | 0.023267 | 0.023267 | 0.023267 | correct — a fixed-rate escapement, not RPM-driven |

So the defect is broader than "audible when stopped". **Loudness is very nearly
RPM-independent across the entire operating range**, and three synths are
non-monotonic — speeding them up makes them quieter.

The cause is structural, not a bug. `gear.cyr` sets `var amp = 0.3` and never
varies it. `belt_drive`'s `squeal_amp = (1 - tension) * 0.3` has no RPM term at
all. `motor`'s noise path is gated on `noise_level` and load, never on speed.

**These are faithful ports.** The Rust oracle did exactly the same, which is
precisely why this needs an ADR: it is a deliberate divergence from the
now-retired parity oracle, not a transliteration fix.

## Decision

Introduce **one** shared loudness law, applied uniformly, rather than a bespoke
curve per synth.

```
r    = rpm / ref_rpm          (rpm < 0 or non-finite -> 0)
gain = 2r / (r + 1)
```

| r | gain | dB |
|---|---|---|
| 0 | 0.0000 | −∞ |
| 0.033 | 0.0639 | −23.9 |
| 0.5 | 0.6667 | −3.5 |
| **1.0** | **1.0000** | **0.0** |
| 2.0 | 1.3333 | +2.5 |
| ∞ | 2.0000 | +6.0 |

Properties that motivated this exact form:

- **Exactly 1.0 at the reference RPM.** Audio at each synth's nominal operating
  point is *unchanged in steady state*. That is what keeps this release
  reviewable: consumers' existing mixes are not re-levelled, and only off-nominal
  speeds move. Measured A/B at each reference RPM, with the law disabled vs
  enabled, after settling: gear 0.106802 / 0.106802, motor 0.111296 / 0.111296,
  turbine 0.252224 / 0.252224, transmission 0.106644 / 0.106644, belt 0.069252 /
  0.069252, chain 0.041150 / 0.041150 — identical. Engine (0.027552 → 0.027551)
  and differential (0.141890 → 0.141885) differ in the 6th digit because their
  per-sample smoothers converge to within an ULP of the target rather than
  exactly, so the gain is 0.9999999… and not literally 1.
- **0 at rest**, so a stopped machine is silent — the headline defect.
- **Monotonic everywhere**, which fixes motor, engine and chain_drive.
- **Saturates at +6 dB**, so a 10× overspeed cannot produce a 10× louder signal.
- Smooth and cheap — one divide, no branch, no transcendental in the hot path.
- **Absorbs NaN.** A non-finite RPM yields gain 0 rather than propagating, which
  closes the `f64_clamp`-propagates-NaN hole recorded in SECURITY.md.

### Reference RPMs

Chosen as each mechanism's typical operating speed, **not** to minimise golden
movement:

| synth | ref | | synth | ref |
|---|---|---|---|---|
| engine | 3000 | | transmission | 3000 |
| gear | 1500 | | differential | 3000 |
| motor | 3000 | | belt_drive | 2000 |
| turbine | 8000 | | chain_drive | 3000 |

**Excluded, deliberately:**
- **clock** — an escapement runs at a fixed tick rate. Its `set_rpm` exists only
  to satisfy the trait shape. Applying the law would be wrong.
- **forced_induction** — already scales correctly through `spool_rpm`, which is
  itself RPM-driven. Applying a second gain would double-count.

## Consequences

- **Six of eleven golden checksums move**, and the five that do not are the
  clearest evidence the design did what it claimed: gear @1500, transmission
  @3000 and belt @2000 sit exactly at their reference RPM, so their gain is
  exactly 1.0 and their audio is **bit-identical**. clock is exempt from the law,
  and forced_induction already scaled through `spool_rpm`. The six that moved are
  the two engine goldens (loudness plus the pulse change below), motor @4000
  (gain 1.14 there, not 1), turbine @8000 (gain exactly 1 — it moved only because
  its noise seed gained resolution), and differential/chain @3000 (the ULP effect
  above). All six were updated in the same commit, with reasons recorded in
  `tests/goldens.tcyr`.
- **`tests/spectral.tcyr` passes unchanged**, which is the check that this is a
  loudness change and not a pitch change: every peak still lands where the RPM
  physics says.
- Audio at each synth's reference RPM is unchanged, so the change is bounded and
  auditable: the diff is confined to off-nominal speeds.
- The rest-state assertions in `tests/oracle_pins.tcyr`, which deliberately
  pinned the *wrong* behaviour so it could not be fixed silently, are rewritten
  as the invariant: **rest < running, for every RPM-driven synth.**
- `ghurni_rpm_gain` is public, so consumers can apply the same law to their own
  layered sources and stay consistent with the library.
- This does **not** address spectral change with load or speed — a gear at high
  speed is louder but not brighter. That is Arc 2 ("Depth"), deliberately kept
  separate so this release changes exactly one thing about the sound.

## Also decided: the combustion pulse is integrated across each sample

Shipped in the same release because it is the other deliberate audio change,
and bundling them means the goldens move exactly once.

The engine's combustion envelope is `e^(-a·t)` with `a = 8·ln(10)² ≈ 42.415`. It
decays to 1% by `t ≈ 0.11`, which at 7000 rpm is about **three samples wide** —
narrower than the sample grid can resolve. Point-sampling it made the rendered
peak depend on where the lattice happened to fall, and at RPMs where
samples-per-cycle is an integer (`5292000/rpm` at 44.1 kHz) the lattice repeats
exactly: if no sample landed near `t = 0`, none ever did.

Measured, 2-stroke 1-cylinder at load 0.9, peak amplitude:

| rpm | before | after |
|---|---|---|
| 6950 | 1.0388 | 0.7819 |
| **7000 (resonant)** | **0.3082** | **0.7896** |
| 7050 | 1.1127 | 0.7661 |

A 70% dropout in a notch ~50 rpm wide — audible as a hole during an RPM sweep,
and there are twenty such RPMs between 2250 and 7000 alone.

The fix integrates the envelope across each sample instead of sampling it at a
point:

```
avg = ∫(lo..hi) e^(-a·u) du / dt  =  (e^(-a·lo) − e^(-a·hi)) / (a·dt)
```

`lo` snaps to 0 when the pulse fired inside this sample's span, which recovers
the onset energy a point-sample drops. Dividing by `dt` — the sample's duration
— rather than by `(hi − lo)` is what makes it **energy-conserving**: the total
∫ over the cycle is invariant to lattice offset, which is precisely why the notch
disappears. A 1000→9000 rpm sweep is now smooth (0.99 → 0.68, no notches).

Peaks are lower everywhere than the old best case, because the old best case was
an aliasing artefact: a pulse three samples wide cannot legitimately reach full
amplitude at this sample rate. Benchmarks show **no measurable cost** — the extra
exponential replaced a `pow` inside the same guard.

## Alternatives rejected

- **Per-mechanism exponents** (amplitude ∝ speed^k, k varying by aerodynamic vs
  impact source). More physically nuanced, but the exponents would be invented
  rather than measured — there are no listening tests to calibrate against, and
  eight tunable constants is a worse starting point than one law. Revisit in
  Arc 2 with spectral work, where the exponents can be justified together.
- **A low-RPM fade only** (gain 1 above a knee, ramp below). Fixes silence at
  rest without touching the rest of the range — but leaves the measured
  non-monotonicity and the "94% of nominal at 3% of speed" problem untouched,
  which is most of the defect.
- **Re-levelling each synth from scratch.** Correct in principle, but it would
  change every operating point at once and make the release impossible to review
  against the goldens.
