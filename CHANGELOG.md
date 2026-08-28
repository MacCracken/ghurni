# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.2.0] - 2026-08-28

**Arc 2 — "Depth."** Breadth stays frozen; five of the ten synths were missing
something structural about the machine they model. Recorded in
[ADR-006](docs/architecture/adr-006-acoustic-depth.md).

⚠ **This changes rendered audio.** Five goldens moved and two were *fixed* — see
below. Every claim was verified against the source before being accepted.

### Changed — mesh whine is no longer a pure sine

`gear`, `transmission` and `differential` all used a single `WAVEFORM_SINE`,
which is the one thing meshing gears never sound like. All three now use a
3-partial additive mesh (the pattern `motor` and `turbine` already used).

Gear's upper partials scale with **`brightness`** — the per-material constant
that until now only fed the resonance filter — so material finally changes
timbre:

| material | brightness | h2/h1 | h3/h1 |
|---|---|---|---|
| steel | 0.9 | 0.546 | 0.278 |
| brass | 0.7 | 0.426 | 0.219 |
| cast iron | 0.5 | 0.307 | 0.153 |
| nylon | 0.2 | 0.124 | 0.063 |

The fundamental is identical across all four, so this is a timbre change and not
a pitch change — `tests/spectral.tcyr` asserts the peak still lands at the mesh
frequency. The fundamental is clamped to **nyquist/3** so the third partial
cannot alias.

### Changed — `GH_ENGINE_HYBRID` is now actually electric

It is documented "electric-with-whine" and ran the full combustion path; the only
per-type differences were two resonance constants and a roughness factor. So the
one engine type advertised as electric was a quiet petrol engine.

The combustion loop no longer runs for HYBRID at all. In its place: a 2-partial
EM whine at `(rpm/60) × 8` poles — the relationship `motor.cyr` uses — with
amplitude driven by load, because traction motors whine louder under torque. The
exhaust and intake beds drop to 0.25; an EV still moves air but has no exhaust
note. Measured: **398.4 Hz @3000 rpm and 199.2 Hz @1500** (predicted 400/200,
within one bin), where gasoline and diesel peak at 102.3 Hz.

### Changed — turbo and supercharger are no longer the same sound

Both whined at plain shaft rate and differed *only* in spool lag, so at the same
spool speed they were spectrally identical. A turbo's centrifugal wheel has
roughly ten blades and whistles high; a Roots blower's three lobes give the low,
hard whine it is known for. Whine is now `(spool/60) × order`. Measured at
6000 rpm × ratio 2.0: **turbo 1798 Hz, supercharger 538 Hz** — a ratio of 3.34,
which is 10/3 as intended.

### Changed — chain link count is finally audible

`engagement_frequency` uses *sprocket_teeth*, so `links` was stored, clamped and
documented ("typically 100–120") while affecting **nothing but the noise seed**:
a 40-link and a 500-link chain rendered identically. A chain is a closed loop, so
link-to-link variation recurs once per lap (`engage_freq / links`); a shallow
modulation at that rate is the slow wobble a worn chain has.

### Changed — clock tick and tock differ

Every beat was identical and perfectly periodic, which is what made a clock read
as a metronome rather than a mechanism. An escapement alternates between two
pallets and the two impacts are not the same sound. Odd beats are now 0.82×
amplitude and 0.78× decay. Measured peaks: **0.205 / 0.172 / 0.216 / 0.177** — an
alternation, not a decay, and the tests assert that distinction explicitly.

### Fixed — two goldens that were blind

Both were exposed *by* this release rather than broken by it:

- **The clock golden ran for 0.5 s**, but a grandfather clock beats at 1 Hz — so
  it contained only beat 0 and **never covered a tock**. The tick/tock change did
  not move it. Widened to 3 s.
- **HYBRID had no golden at all** — the engine type whose behaviour changed most
  had nothing pinning it. Added.

### Testing

`tests/spectral.tcyr` grew 13 → 29 assertions: harmonic ratios strictly ordered
by material, the hybrid EM whine tracking RPM, gasoline and hybrid no longer
sharing a spectrum, and the turbo/blower ratio being *specifically* 10/3 rather
than merely different. `tests/oracle_pins.tcyr` pins the chain-lap audibility
(with a determinism control) and the clock alternation.

### Deferred — Arc 2 split into 2.2.0 and 2.3.0

Six of the original eleven Arc 2 items are **not** in this release, deliberately:
load→spectral tilt, diesel valvetrain content, differential ring modulation,
turbine rotor slap, and source-body impulse responses each need their own design
decision, and bundling them would have made this release unreviewable. They are
now **Arc 2b — `2.3.0` "Depth II"**.

Two items moved out of the timbral track entirely, to Arc 3: **per-component
stems** is an API change, and **differential drive/coast asymmetry** needs a
torque-direction input the API does not have. Downstream arcs renumbered
accordingly.

## [2.1.0] - 2026-08-28

**Arc 1 — "Rest State."** The first deliberate acoustic divergence from the
retired Rust oracle, and the first release since 2.0.0 that changes how ghurni
sounds. Recorded in [ADR-005](docs/architecture/adr-005-rpm-loudness-law.md).

⚠ **This changes rendered audio.** Six of eleven goldens moved; they were
updated in the same commit, each with its reason. See "What moved" below.

### The defect

ghurni's thesis is that RPM is the fundamental parameter. Measurement said
otherwise — **loudness was very nearly RPM-independent across the entire
operating range**, and three synths were non-monotonic:

| synth | @ 0 RPM | nominal | |
|---|---|---|---|
| motor | 0.184407 | 0.185256 | as loud stopped as running |
| belt_drive | 0.084913 | 0.069786 | **louder** stopped than running |
| turbine | 0.106632 | 0.251488 @8000 | 99.5% of nominal at 5% of nominal speed |
| gear | 0.026472 | 0.106961 @1500 | 94% of nominal at 3% of nominal speed |

Structural, not a bug: `gear.cyr` set `var amp = 0.3` and never varied it;
`belt_drive`'s `squeal_amp = (1 - tension) * 0.3` had no RPM term at all. These
were faithful ports — the oracle did the same — which is exactly why the fix
needed an ADR rather than a patch.

### Added — one loudness law, applied uniformly

`ghurni_rpm_gain(rpm, ref_rpm)` = `2r / (r + 1)` where `r = rpm / ref_rpm`.

Chosen for four properties, in this order: **exactly 1.0 at the reference RPM**
(so nominal audio is unchanged and the release stays reviewable), 0 at rest,
monotonic everywhere, and saturating at +6 dB so a 10× overspeed cannot produce
a 10× louder signal. Reference speeds are each mechanism's typical operating
point — engine/motor/transmission/differential/chain 3000, gear 1500, belt 2000,
turbine 8000.

**clock and forced_induction are deliberately exempt**: an escapement runs at a
fixed tick rate and is not RPM-driven, and forced_induction already scales
through `spool_rpm`, so a second gain would double-count.

### Fixed — the combustion pulse is integrated across each sample

The engine's envelope `e^(-a·t)` (a = 8·ln(10)² ≈ 42.4) decays to 1% within
about **three samples** at 7000 rpm — narrower than the sample grid resolves. So
the rendered peak depended on where the lattice fell, and at RPMs where
samples-per-cycle is an integer (`5292000/rpm` at 44.1 kHz) the lattice repeats
exactly: if no sample landed near the peak, none ever did.

| rpm | before | after |
|---|---|---|
| 6950 | 1.0388 | 0.7819 |
| **7000 (resonant)** | **0.3082** | **0.7896** |
| 7050 | 1.1127 | 0.7661 |

A 70% dropout in a notch ~50 rpm wide, at twenty RPMs between 2250 and 7000 —
audible as a hole during a sweep. Now the envelope is integrated across each
sample, `(e^(-a·lo) − e^(-a·hi)) / (a·dt)`, which is **energy-conserving**: the
integral over the cycle is invariant to lattice offset, so the notch cannot
exist. A 1000→9000 rpm sweep is smooth (0.99 → 0.68, no notches). Peaks are
lower than the old best case because that best case was an aliasing artefact.
**No measurable benchmark cost** — the extra exponential replaced a `pow` inside
the same guard.

### Fixed — NaN at the parameter boundary

`f64_clamp` **propagates** NaN where `f64_max`/`f64_min` absorb it, so every
public setter written in the oracle's idiom (`rpm.clamp(0.0, 50000.0)`) let a
NaN into synth state, where it poisoned oscillator phase and filter memory
permanently — one NaN `set_rpm` made every subsequent sample non-finite for the
life of the object. New `ghurni_finite_clamp` collapses non-finite input to the
"off" end; applied at all **18** public-parameter clamps. Measured: every NaN
entry point now renders 0/2205 non-finite samples, down from 2205/2205.

### Fixed — seed resolution unified

`turbine` folded `duct_resonance` into its noise seed with a bare `f64_to`,
keeping only whole Hz, so two turbines under 1 Hz apart shared a stream. Now
×1000, matching `belt_drive` and `forced_induction` — both fixed earlier for the
same defect. That convention is now stated once and applied everywhere.

### What moved, and what deliberately did not

The five unchanged goldens are the evidence the design did what it claimed:
**gear @1500, transmission @3000 and belt @2000 sit exactly at their reference
RPM, so their audio is bit-identical.** clock is exempt; forced_induction already
scaled. The six that moved: both engine goldens, motor @4000 (gain is 1.14
there, not 1), turbine @8000 (gain exactly 1 — it moved only because its seed
gained resolution), and differential/chain @3000, whose per-sample smoothers
converge to within an ULP of target so the gain is 0.9999999… rather than 1.

**`tests/spectral.tcyr` passed unchanged throughout**, which is the check that
this is a loudness change and not a pitch change: every peak still lands where
the RPM physics says.

### Testing

- The rest-state assertions added in 2.0.3 deliberately pinned the **wrong**
  behaviour so it could not be fixed silently. They are now the invariant:
  exact silence at rest for the six synths whose RPM floor is 0, rest under 5%
  of running for those with a floor (engine idles at 100 rpm, gear and turbine
  at 1), and monotonicity for the three that failed it.
- New groups covering the law itself, `ghurni_finite_clamp`, NaN at every entry
  point, and seed decorrelation. 509 assertions across 10 suites, up from 450.

## [2.0.4] - 2026-08-28

**Arc 0b — "Delete the Oracle."** `rust-old/`, the Rust 1.0.0 crate that served
as ghurni's parity oracle since the port, is removed from the tree.

**No API change, no audio change.** Every one of the 450 assertions stayed green
and **every golden checksum was unchanged** across the deletion — which is the
proof that the removal was purely documentary. Nothing built against the
directory; the coupling was 118 references in comments and docs.

### Removed

- `rust-old/` — 33 files, 3,934 lines (3,054 of them `src/`), 256 KB.

### Where the oracle went

It is not lost, and the citations that point at it still resolve:

| | |
|---|---|
| As `rust-old/` | tags `2.0.0` … `2.0.3` — `git show 2.0.3:rust-old/src/engine.rs` |
| As `src/*.rs` | tag `1.0.0`, the original crate layout |
| Relocation commit | `c9a0a02` |

Line citations of the form `rust-old/src/gear.rs:33` are **kept deliberately**
throughout `src/`, `tests/` and the docs. They are provenance — they record
where a pinned value came from — and they resolve against the tags above. What
changed is the framing: nothing now claims the directory is present.

### Changed — the correctness bar moved

This is the substance of the release, not the deletion itself. Until now
`CLAUDE.md` read *"Cross-check against `rust-old/` — the correctness bar is
'matches what the Rust naad-backend path did'"*. That stopped being checkable
the moment the directory went, so the bar is now the suite that 2.0.3 landed
precisely to replace it:

- **`tests/goldens.tcyr`** — rendered-audio checksums. **If a golden moves, it is
  not a patch release.** Work out *why* before touching the value; never "fix" a
  golden to make CI pass.
- **`tests/oracle_pins.tcyr`** (187 assertions) — every constant and contract the
  oracle proved, each citing its originating source line.
- **`tests/spectral.tcyr`** — rendered audio sits where the RPM physics says.

Deliberate audio changes still require an ADR and still belong in a MINOR.

> **This unblocks Arc 1 (`2.1.0` "Rest State").** While the oracle existed, every
> deliberate divergence had to argue against the project's own stated rule. The
> bar is now the goldens — which is the bar acoustic work actually wants.

### Documentation

Reframed everywhere the directory was described as present, rather than merely
cited: `CLAUDE.md` (identity, correctness bar, DO-NOT list, doc index),
`README.md`, `CONTRIBUTING.md` (prerequisites and the Parity section),
`docs/architecture/overview.md` (module tree), `adr-001`, `adr-004` (which now
records the recovery tags), `docs/development/state.md`, and
`docs/guides/testing.md` — whose suite table was also still missing the three
suites 2.0.3 added.

`docs/development/rust-old-retirement.md` is now a completed record rather than a
checklist: where the oracle went, what replaced it, and the audit trail.

`.gitignore` drops the now-meaningless `rust-old/target/` rule.

## [2.0.3] - 2026-08-27

**Arc 0 — "Pin the Oracle."** The deck-clearing patch that makes `rust-old/`
deletable and the coming audio work safe. Every behaviour the oracle currently
proves is now proved by an assertion instead.

**No public API change and no audio change.** Two guards were added at the
boundary (below); neither alters output for any valid input, and the new golden
suite proves it.

| | before | after |
|---|---|---|
| test suites | 7 | **10** |
| assertions | 224 | **450** |
| reference coverage | 49% | **66%** |
| benchmarks | 6 | **14** |
| fuzz checks | 0 (no harness) | **1631** |
| CI gates | build + test | **+ audit, deny, distlib --check, fuzz, coverage ratchet, examples RUN** |

### Added — test machinery

- **`tests/goldens.tcyr`** — rendered-audio regression goldens for all ten
  synths. This is the gate that makes ghurni's semver rule enforceable: a patch
  release may not change audio, but **2.0.1 and 2.0.2 both did and nothing
  caught it**. Verified against the real failures — reintroducing the 2.0.1
  `GHURNI_DB_SCALE` miscompile, the 2.0.2 belt-seed collapse, or a one-part-in-a-
  million gain change all fail the suite. Samples are quantized to 1e-6 before
  hashing so a different platform's libm cannot cause a false alarm, and four
  self-checks prove the hash reacts to magnitude, order and length.
- **`tests/oracle_pins.tcyr`** (166 assertions) — every value that lived only in
  `rust-old/`: all 12 preset parameter sets, the 36 per-material / per-type
  property constants, engine resonances and even-firing offsets, event
  durations, the auto-BOV trigger triple, the differential's ring/pinion
  asymmetry, the equal-power pan law, ChainDrive's sub-1 Hz silence and
  non-advancing RNG, the stereo tail contract, and the empty-ratios fallback.
  Every expected value was read from the oracle's source, never from what ghurni
  prints — a value transcribed from current output freezes today's bugs as
  "correct", which is exactly how `GHURNI_DB_SCALE` survived.
- **`tests/spectral.tcyr`** — FFT assertions that rendered audio actually sits
  at the frequency the physics predicts. Nothing checked this before: every
  audio assertion was finite / has-energy / an ordering, all of which hold for a
  synth emitting entirely the wrong pitch. Gear mesh, motor EM hum, turbine
  blade pass and differential whine each land within one 5.4 Hz bin of
  `(rpm/60) × count`, and frequency is asserted to *scale* with both rpm and
  tooth/pole/blade count.
- **`fuzz/ghurni.fcyr`** — 1631 adversarial checks over the public boundary,
  following naad's separated-sweep design (correlating an enum id with its
  numeric parameters makes the harness prove nothing — naad measured that).

### Fixed

- **The fuzz harness found a SIGSEGV on its first run.** `ghurni_mixer_add_channel`
  rejected error codes and null, but a small positive integer — a `GH_KIND_*`
  tag or a channel index passed where a synth pointer belongs — passed both
  guards and was dereferenced. Now rejected below `GHURNI_MIN_VALID_PTR` (the
  null page). This is a heuristic, not pointer validation; the contract's real
  limit is documented in SECURITY.md and the integration guide.
- **`ghurni_dcblocker_new` produced a non-finite pole for a NaN sample rate.**
  `f64_clamp` propagates NaN where `f64_max`/`f64_min` absorb it, so the filter
  and everything downstream went non-finite. A non-positive rate needed no fix —
  the oracle's expression already floors it at 0.9. `DcBlocker` was `pub(crate)`
  in Rust and unreachable from outside; Cyrius's flat namespace exposes it.

### Fixed — release machinery

- **`scripts/version-bump.sh` was Rust-era and would have corrupted this
  release.** It wrote `VERSION`, then `sed`'d a root `Cargo.toml` that does not
  exist and ran `cargo generate-lockfile`; with `set -e` it **half-bumped the
  repo and died**. Rewritten for Cyrius: validates semver, refuses a no-op bump,
  restamps the bundle, verifies the tree agrees, and prints the release checklist.
- **`.gitignore` was still the Rust crate's** and had no `/build/` — which is how
  **12 compiled ELF binaries (11 MB) came to be tracked and shipped inside every
  release tarball**, including 5 stale `build/err_*` from an old naming scheme.
  ⚠ The ignore rule is in place, but untracking them needs
  `git rm -r --cached build/`.
- **CI ran no quality gates.** Added `cyrius audit`, `cyrius deny`,
  `cyrius distlib --check` (nothing prevented `dist/ghurni.cyr` drifting from
  `src/`, and `dist/` is what consumers compile), `cyrius fuzz`, and a coverage
  ratchet. The examples are now **run**, not merely built — a demo that compiled
  and then faulted used to pass. The fuzz step guards against its own vacuity:
  `cyrius fuzz` exits 0 when it finds no harnesses, so the absence of `fuzz/` is
  now itself a failure.

### Fixed — documentation

- **README advertised a wastegate** that appears nowhere in `src/` or `dist/`.
- **`src/dsp.cyr` cited `tests/dsp.tcyr`, which does not exist** — and the
  comment shipped to consumers inside `dist/ghurni.cyr`.
- **ADR-004's serde justification was false.** It said the synth structs dropped
  deep serialization because "nothing meaningful survived the Rust
  `#[serde(skip)]`". `MixerChannel` skipped only `synth` and `scratch` —
  `name`, `gain`, `pan` and `muted` all serialized, so a Rust consumer could
  persist a whole mixer configuration. The drop is a real capability cut taken
  deliberately, not a no-op.
- **SECURITY.md claimed RPM/load were clamped to valid ranges.** They are, but
  NaN is not absorbed; that is now stated, with the workaround, and scheduled.
- **A missing `2.0.0` CHANGELOG entry** meant a re-cut of that tag shipped an
  empty release body. Written retroactively.
- The contracts that lived only in the oracle's doc comments are now in the
  source: `smooth_time_s` is **tau** (63%), not a settling time; `MechanicalEvent`
  cylinders are 0-indexed and `GearShift.from` 0 = neutral; the cross-plane V8
  offsets are uneven *on purpose* (that is the burble); the supercharger preset's
  `spool_inertia` argument is inert; `sample_position` is vestigial in three
  synths; and the clamps are the *legal* bounds, not the *typical* ones.

### Notes

- `docs/development/rust-old-retirement.md` now has **zero open items**. The
  deletion itself is 2.0.4.
- Coverage rose 49% → 65% as a side effect of the pinning suite; it is tracked
  as a CI **ratchet**, not a target. 2.0.1 shipped a 76.9×-wrong constant through
  a fully green suite — coverage is a floor, never a correctness proof.

## [2.0.2] - 2026-08-27

A P-1 audit / refactor / hardening / optimization / security sweep, plus the
repair of every Rust-era document left behind by the 2.0.0 port.

**Nothing here changes audio output.** Every functional fix is a rejection of
input that previously crashed, corrupted state, or lied about success; the two
refactors were proved bit-identical by checksumming rendered audio before and
after. Suite: **7 suites / 205 assertions** (was 6 / 135). `cyrius audit` exits
**0** for the first time.

### The two defect classes

Both come from Cyrius's single signed `i64`, and both are invisible in a
straight transliteration — the same two shapes naad 2.1.3 found:

1. **Rust `usize`/`u32`/enum guarantees erased.** Where the oracle typed a
   parameter `usize` or as an enum, a bad value is *unrepresentable*, so the
   Rust body validates nothing and a transliterated guard is only **half a
   bound**.
2. **`f64_to` truncates where Rust's `as` casts SATURATE.** Rust maps an
   over-range f64 to `usize::MAX`; `f64_to` overflows to `INT64_MIN`, which is
   neither `> MAX` nor `== 0`, so one-sided clamps let it straight through.

### Fixed — crashes

- **The mixer dereferenced a failed constructor's error code (SIGSEGV).**
  `ghurni_mixer_add_channel` stored whatever `i64` it was handed, and the tag
  dispatch then used it as a struct base pointer — confirmed under gdb faulting
  at `base + 0x90` with base `-1`. The oracle's field is
  `Option<Box<dyn Synthesizer>>`: a `Box` is always a live synth (`Engine::new`
  returns `Result`, so the caller must unwrap first) and all three use sites are
  guarded by `if let Some(synth)` (`rust-old/src/mixer.rs:101,123,156`).
  `add_channel` now rejects an error code, a null synth, and an out-of-range
  `GH_KIND_*` tag; the three dispatch loops skip an absent synth.
- **A caller-supplied `duration` could abort the process.** Every
  `*_synthesize` grew its output with `vec_push` past the stdlib's
  `VEC_CAP_MAX`, and `vec_push` calls `_vec_die()` → `exit(1)`. Above ~6087 s at
  44.1 kHz this killed the host process out of library code.
- **A desynchronised firing order aborted the process.** `set_firing_order`
  stored the *caller's* vec where Rust moved it, so a later push by the caller
  left `vec_len(firing_offsets)` larger than `misfire_flags`, and the per-sample
  loop indexed out of bounds into `_vec_die()`. The oracle is defensive at both
  spots (`.get(cyl).unwrap_or(false)`, `knock_remaining.iter_mut()`).
  `set_firing_order` and `ghurni_transmission_new` now **copy** their vec
  (restoring Rust's move semantics), and the engine's event loops are bounded by
  their own lengths.

### Fixed — silent corruption

- **`synthesize` returned an empty buffer and called it success** (CLASS 2). All
  ten one-shot functions computed `f64_to(sample_rate * duration)`; past 2^63
  that is `INT64_MIN`, the fill loop never ran, and the length-0 vec is a heap
  pointer so `ghurni_is_err` reported `GH_ERR_NONE`. New shared guard
  `ghurni_sample_count` (`src/dsp.cyr`) compares in f64 **before** the
  conversion, caps at `GHURNI_MAX_SAMPLES` (2^26 ≈ 25 min at 44.1 kHz), and
  rejects a count below 1. Wired into all ten, **before** the state-mutating
  setters — matching the oracle, which validates first.
- **Out-of-range enum ids fell through to a wrong default** (CLASS 1). All five
  enum constructors accepted any `i64`. `ghurni_forced_induction_new` was the
  sharpest: the `inertia = 0.01` default was only overridden inside the Turbo
  branch, so an unrecognised type **silently discarded the caller's
  `spool_inertia`**. Each constructor now range-checks its id.
- **`ghurni_transmission_shift_to` accepted a negative gear** (CLASS 1), setting
  `current_gear = -1` and firing the 200 ms shift transient. Rust typed the
  parameter `u32`, which made its one-sided `< ratios.len()` a complete bound.
- **Every Turbo shared one bit-identical noise stream.** The oracle seeds with
  `induction_type as u32 * 777 + drive_ratio.to_bits()`
  (`rust-old/src/forced_induction.rs:95`); the port dropped the ratio term
  entirely, collapsing the seed space to two values, so two layered turbos summed
  coherently (~+6 dB, phasey) instead of decorrelating. The seed now folds in
  `f64_to(ratio * 1000)` — the same behavioural-integer-seed convention
  `belt_drive` and `turbine` already document.

### Performance

Measured with interleaved A/B runs against the pre-fix bundle; run-to-run
variance on this machine is ±30%, so single measurements were not trusted.

- **The mixer leaked its scratch buffer on every block.** It built a fresh vec
  per channel per `process_block` and dropped the previous one; this library
  never frees. Measured: **278 MB of unreclaimable heap per 100 s of 4-channel
  audio → now 0**. The oracle reuses a grow-only buffer
  (`rust-old/src/mixer.rs:124-131`); the port now rebuilds only when the block
  size actually changes, since Cyrius vecs have no sub-slice and `process_block`
  must see a vec of exactly `len`. Also ~7% faster on the streaming path
  (1.239 ms → 1.155 ms, 4ch × 512).
- **The engine scanned its event vecs every sample even when idle.** Skipping
  the knock scan while nothing is armed is ~4% off engine synthesis
  (1.448 ms → 1.389 ms). Noise-stream consumption is unchanged — the old loop
  consumed no noise when nothing was armed either, so output is bit-identical.
- **belt_drive** hoisted `osc_set_frequency` and `belt_omega` out of the
  per-sample loop (both block-invariant; `osc_set_frequency` only validates and
  stores a field, so it cannot touch the phase). **Bit-identical**, but honestly:
  **no measurable time win**.

### Testing

- **New `tests/hardening.tcyr`** (70 assertions) pins every defect above.
  **Each assertion was mutation-checked**: the fix was reverted and the suite
  confirmed to fail. That pass caught one **vacuous test of my own** — the
  forced-induction seed test compared rendered audio, but `drive_ratio` also
  sets the compressor speed, so it passed with the defect reintroduced. It now
  pulls from the noise generator directly.
- **`GHURNI_DB_SCALE` is pinned** to its exact bit pattern and to the envelope
  value it produces. The 2.0.1 post-mortem showed the entire 135-assertion suite
  passed with that constant 76.9× wrong; that gap is now closed permanently.
- New benchmarks for the streaming 4-channel mixer path and belt_drive's block
  loop — the bench header had claimed mixer coverage it did not have.

### Documentation — the Rust-era backlog, cleared

Every one of these described the 1.x Rust crate and contradicted `CLAUDE.md`:

- **`README.md`** — sold a Rust crate (crates.io link, Rust quick start, a
  feature-flag table that does not exist, and a "~1,000× real-time" claim that
  benchmarks put at **~43×**). Rewritten against the real API.
- **`CONTRIBUTING.md`** — told contributors to run `cargo fmt/clippy/test/audit`
  and to write synthesizers "following the dual-impl pattern" with a
  `Synthesizer` trait. Rewritten around `cyrius audit`, plus the two defect
  classes above.
- **`SECURITY.md`** — the dependency table was `rust-old/Cargo.toml` verbatim
  (serde, thiserror, tracing, libm, naad "Optional"), none of which are in the
  graph. Now lists the real pinned set, and its attack-surface table reflects
  the guards this release actually added, with the known limitations stated.
- **`docs/architecture/overview.md`** — listed 20 `.rs` files under a
  `ghurni/src/` header, including the deliberately-unported `math.rs`/`rng.rs`,
  and documented a "Dual Code Paths" backend that does not exist.
- **`docs/architecture/adr-001-naad-backend.md`** — status `Accepted`, specifying
  naad as *optional* behind a feature flag with a libm/PCG32 fallback. Now marked
  **Superseded by ADR-004**, retained as history.
- **`docs/architecture/adr-002-scope-boundaries.md`** — promised consumers a
  `Synthesizer` trait; corrected to the `(GH_KIND_*, pointer)` tag dispatch.
- **`docs/development/integration-guide.md`** — all five code blocks were Rust.
  Rewritten in Cyrius, error handling first.
- **`docs/guides/testing.md`** — documented `cargo test` and a no-naad fallback
  path that cannot exist. Rewritten around the real gates, with the
  "a vacuous test is worse than no test" rules this sweep learned the hard way.
- **`docs/development/roadmap.md`** — planned `cargo-fuzz`, rayon and portable
  SIMD. Re-scoped to what the toolchain provides.

### Also

- Wrapped the 7 over-120-character lines that `cyrius lint` had always flagged,
  and documented the 7 public functions `cyrius doc --check` had always flagged
  (the logging wrappers, `main`, `ghurni_synth_set_rpm`). Both were pre-existing;
  with them fixed `cyrius audit` exits 0. The line wrapping was verified
  bit-identical by checksum.
- New helper `ghurni_vec_copy` (`src/error.cyr`) for restoring the oracle's
  move semantics at vec-taking boundaries.

### Port completion — items `rust-old/` still held alone

A module-by-module sweep of `rust-old/` against the port, to clear the oracle
for deletion. **83/83** portable public functions, 7/7 enums (29 variants),
40/40 trait accessors, 44/44 integration tests and 5/5 examples are ported, and
every constant, clamp bound, frequency formula and envelope shape was compared
with **zero numeric mismatches**. Four things were genuinely still missing:

- **`belt_drive`'s noise seed dropped its tension term** — the same defect
  class as forced_induction's, and unfixed. Tension is clamped to 0..1, so
  `f64_to(tens)` truncated to 0 for every tension below 1.0: every belt of a
  given integer diameter shared one bit-identical pink-noise realisation, and
  the documented slack-vs-tight layering summed perfectly correlated flap noise
  (~+6 dB and phasey) where the oracle decorrelated it. Now scaled ×1000 to
  match `forced_induction`. ⚠ **This changes belt_drive audio output**, toward
  the oracle's intent.
- **`ghurni_synth_rpm` / `ghurni_synth_sample_rate` added.** The tag dispatch
  covered only 2 of the `Synthesizer` trait's 4 methods
  (`rust-old/src/traits.rs:17,20`), so a consumer holding a heterogeneous
  `(GH_KIND_*, pointer)` collection could not read RPM or sample rate
  generically — the exact `Vec<Box<dyn Synthesizer>>` shape the oracle's
  `test_synthesizer_trait_dispatch` exercised.
- **`ghurni_err_from_name` added.** `GhurniError` was the only public enum with
  just the forward half, breaking CLAUDE.md's `_name`/`_from_name` rule, though
  the oracle derived `Deserialize` on it and round-tripped it.
- **`ghurni_fmod`'s doc comment was wrong, and dangerously so.** It claimed the
  operands are non-negative. `engine.cyr` passes a **negative** dividend on
  every sample for every cylinder whose firing offset exceeds the current crank
  angle — i.e. most cylinders, most of the time — and the floored behaviour is
  what makes that correct. The oracle spells the negative-safe idiom out by hand
  (`((a % m) + m) % m`); the port collapses it to one `ghurni_fmod` call. A
  "simplification" to a truncated remainder would silently drop most firing
  events on every multi-cylinder engine, and the one existing assertion
  (`370 mod 360`) would not have noticed. Four negative-dividend assertions now
  pin it.

Also recorded, not changed: `GHURNI_DB_SCALE`'s comment now states that the
oracle *divides* by `LOG10_E` where an exp→dB conversion would multiply, making
the naad decay 5.30× steeper than the `exp(-8t)` its own fallback arm used. That
asymmetry is the oracle's quirk, reproduced deliberately — without the note it
reads as a bug worth "fixing", which would rewrite every engine, gear and clock
envelope. And `belt_drive`'s RPM smoother is now documented as deliberately
inert (the oracle discards it with `let _smooth = ...` and uses raw RPM).

New: [`docs/development/rust-old-retirement.md`](docs/development/rust-old-retirement.md)
— the deletion checklist. `rust-old/` is **not yet clear to remove**: a body of
knowledge still lives only there (all 12 preset parameter sets, 32 per-material
property constants, the auto-BOV trigger triple, the differential's ring/pinion
asymmetry, the pan law's channel assignment), two claims in `state.md` are
actively wrong and become uncheckable once it goes, and 4 of the oracle's 10
benchmarks are unported.

### Known — still open

- **Allocation failure is unhandled.** No `alloc()` result is null-checked
  anywhere in the port, and no sibling library checks either; the contract wants
  deciding ecosystem-wide rather than patching here alone.
- **Presized output buffers.** `*_synthesize` still grows one `vec_push` at a
  time (~2× heap waste). The stdlib has no `vec_with_capacity`; this wants an
  upstream `vec` API, not a reach into vec internals.
- **Resonant-RPM combustion amplitude** is characterised but deliberately NOT
  changed — the fix would alter audio output and needs its own ADR. See
  `docs/development/state.md`.
- No fuzz harness (`cyrius fuzz` runs `fuzz/*.fcyr`; ghurni has none), and CI
  still does not run `audit`/`deny`.

## [2.0.1] - 2026-08-27

Toolchain and dependency refresh. **ghurni's public API is unchanged** — the
naad renames below are internal to ghurni's own call sites. Audio output *does*
change for gear / clock / engine, but only because the toolchain bump repairs a
silently miscompiled constant (see **Fixed**); the change moves output toward
`rust-old/` parity, not away from it.

### Changed

- **Toolchain**: cyrius `6.4.2` → `6.5.35`.
- **Dependencies** bumped to the coherent set naad 2.2.1 itself pins:

  | dep | from | to |
  |-----|------|----|
  | naad | 2.1.0 | 2.2.1 |
  | hisab | 2.6.7 | 2.11.2 |
  | goonj | 2.0.0 | 2.0.4 |
  | sakshi | 2.4.3 | 2.4.11 |

  naad 2.2.1 pins hisab 2.11.2 + goonj 2.0.4; goonj 2.0.4 pins hisab 2.11.2;
  hisab 2.11.2 pins sakshi 2.4.11 — so the vendored bundles are mutually
  consistent rather than merely individually latest.
- **Vendored `lib/` bundles refreshed.** This is the coordinated consumer
  refresh naad 2.2.0 called for; ghurni was one of the five named copies.
- **Migrated onto naad's namespace wave** (naad 2.1.3 + 2.2.0 renamed bare
  top-level symbols to escape Cyrius's flat distlib namespace). Values and
  behaviour are unchanged — these are pure renames:

  ```
  FILTER_BANDPASS  -> NAAD_FILTER_BANDPASS   (10 call sites)
  db_to_amplitude  -> naad_db_to_amplitude   ( 4 call sites)
  ```

  `naad_db_to_amplitude` is body-identical to the old `db_to_amplitude`; it is
  *not* the separate approximate `db_to_amplitude_lut`.
- `dist/ghurni.cyr` regenerated (2,904 lines, v2.0.1).

### Fixed — `GHURNI_DB_SCALE` was silently miscompiled (audible)

⚠ **This changes gear / clock / engine audio output**, in the direction of
`rust-old/` parity. It is a toolchain fix, not a naad one.

`src/error.cyr` declared `GHURNI_DB_SCALE = 46.051701859880914` (20 / LOG10_E).
Cyrius **≤ 6.5.27** packed a decimal float literal as a 32/32 rational,
`(denom << 32) | (numer & 0xFFFFFFFF)`; past ~9 fractional digits *both* fields
overflowed and the literal **compiled clean to a different number** — here
`0.598756873065743`, 76.9× too small. Fixed upstream in **cyrius 6.5.28**,
which this release's 6.4.2 → 6.5.35 bump crosses.

The constant is a live per-sample audio value, not a diagnostic. It feeds
`naad_db_to_amplitude(-x * GHURNI_DB_SCALE)` = `10^(-x·S/20)` at four sites —
`gear.cyr:196` (mesh ring), `clock.cyr:203` / `clock.cyr:207` (tick body ring),
`engine.cyr:312` (combustion pulse). At `x = 1` the envelope was **0.9334**
(essentially undamped) where it should be **0.00498**: a 187× difference, so the
ring/pulse decay shaping was effectively absent.

Scope — this is a property of the **compiling** toolchain, not of the shipped
source. `dist/ghurni.cyr` always carried the correct decimal *text*, so anyone
who compiled it with cyrius ≥ 6.5.28 already had the right value and has nothing
to regenerate. Affected artifacts are those built by cyrius ≤ 6.5.27, which
includes ghurni's own committed `build/*` binaries at 2.0.0. A consumer still on
an older toolchain pin should regenerate any stored reference renders.

`GHURNI_DB_SCALE` is now stored as the IEEE-754 bit pattern
`0x4047069E2AA2AA5B` — the idiom `GHURNI_EPSILON` / `GHURNI_POS_INF` /
`GHURNI_NEG_INF` in the same file already use — so the value cannot be re-broken
by a consumer compiling the bundle on an older toolchain. It was the only
decimal float literal past the threshold anywhere in the link unit (ghurni
sources, all four dep bundles, and the stdlib leaves were swept; every other hit
is a comment beside a hex pattern).

No test caught this in either direction: the suite asserts finiteness, non-zero
energy and one relative energy ordering, all of which hold for both values.

### Verified

- Old-vs-new top-level symbol diff across all four refreshed bundles,
  intersected against every identifier ghurni references: **zero** remaining
  references to a removed symbol.
- `cyrius build` · `cyrius test` (6 suites, 135 assertions + 6 from the smoke
  entry, 0 failures) · `cyrius bench` · all 5 `docs/examples/*.cyr` build and run.

## [2.0.0] - 2026-07-04

**The Cyrius port.** ghurni is rewritten from Rust to [Cyrius](https://github.com/MacCracken/cyrius)
for AGNOS, alongside its sibling libraries. The Rust crate (1.0.0) is preserved
at `rust-old/` as the parity oracle. See
[ADR-004](docs/architecture/adr-004-cyrius-port.md).

*(This entry was written retroactively in 2.0.3. The 2.0.0 tag shipped without
one, so `release.yml`'s changelog extraction produced an empty release body.)*

### Changed — the port

- **naad is the only backend.** The Rust crate was feature-gated: a default
  `naad-backend` path and a fallback using a local PCG32 (`rng.rs`) plus libm
  wrappers (`math.rs`). AGNOS always ships naad, so only the naad path is
  ported — 18 of the oracle's 20 modules. `rng.rs`, `math.rs` and every
  per-synth fallback loop are dropped, and naad's `NoiseGenerator` owns all
  stochastic content. [ADR-001](docs/architecture/adr-001-naad-backend.md),
  which specified the dual path, is superseded.
- **f32 → f64 throughout.** naad and hisab are f64-only, so the widening is
  forced. It is a precision improvement but a real behavioural difference in a
  few places; the divergence list lives in `docs/development/state.md`.
- **Integer error codes replace `GhurniError`.** Cyrius has no
  `Result<T, String>`. Fallible functions return a heap pointer on success or a
  negative `GH_ERR_*` code, distinguished by `ghurni_is_err`.
- **Tag dispatch replaces trait objects.** Cyrius has no vtables, so
  `Box<dyn Synthesizer>` becomes a `(GH_KIND_*, pointer)` pair plus a
  hand-written switch in `mixer.cyr` — the explicit equivalent of the trait's
  vtable.
- **Serde parity where it is meaningful.** Public enums map to `GH_<ENUM>_*`
  integers with `name` / `from_name` helpers using serde's externally-tagged
  variant names. POD structs (`GhDcBlocker`, `GhSmoothedParam`, `GhEvent`) get
  `#derive(Serialize)`. Deep serialization of the synth structs is dropped.
- **All symbols are `ghurni_` / `GHURNI_` / `Gh`-prefixed** so the distlib
  bundle coexists with naad, hisab, goonj and sakshi in one flat namespace.

### Ported

All ten synthesizers (engine, gear, motor, turbine, clock, transmission,
differential, forced induction, belt drive, chain drive), the mixer, the twelve
presets, the L0 foundations (error, logging, dsp, smooth, event, traits), all 44
Rust integration tests reproduced as behavioural-parity `.tcyr` suites, all five
examples, and the benchmark harness.

## [1.0.0] - 2026-03-28

### Changed

- **Breaking**: All constructors now take `sample_rate: f32` and return `Result<Self>`
- **Breaking**: `synthesize()` methods no longer take `sample_rate` (stored in struct)
- Replaced hand-rolled DSP with naad primitives (oscillators, filters, noise generators, additive synthesis)
- Engine exhaust uses `BiquadFilter` bandpass for resonance shaping
- Motor EM hum uses `AdditiveSynth` (3 partials) instead of manual sin loops
- Turbine blade pass uses `AdditiveSynth` (2 partials), duct uses `Oscillator`
- Gear resonance uses `BiquadFilter` for material coloring
- Clock tick uses `BiquadFilter` for body resonance

### Added

- **`Synthesizer` trait** — common interface (`process_block`, `set_rpm`, `rpm`, `sample_rate`) enabling generic composition, mixers, and wrappers. Implemented by all synthesizer types.
- **`MechanicalMixer`** — multi-component mixer with per-channel gain, pan, mute. Supports `process_block` (mono) and `process_block_stereo` (equal-power pan law).
- **`MechanicalEvent` enum** — discrete event triggers: Backfire, Misfire, Knock, Stall, RevLimiterHit, GearShift, Startup, Shutdown.
- **`SmoothedParam`** — one-pole exponential parameter smoother for click-free RPM/load transitions.
- **`DcBlocker`** — one-pole highpass DC blocker applied to all synthesis output.
- **Engine enhancements**:
  - Multi-cylinder firing order — per-cylinder crank-angle offsets (V8 burble vs inline-4 drone)
  - `set_firing_order()` for custom firing patterns (e.g., cross-plane V8)
  - Intake manifold Helmholtz resonance — separate BiquadFilter path
  - Deceleration crackle/pop — stochastic impulses on sharp load drop at high RPM
  - Load-dependent timbre — roughness, harmonic content scale with load
  - `trigger_event()` for backfire, misfire, knock events
  - Parameter smoothing on RPM and load
- **New synthesizer types**:
  - `ForcedInduction` — turbocharger (spool lag, wastegate) and supercharger (direct drive) with blow-off valve burst
  - `Transmission` — gear mesh at current ratio, synchronizer whine during shifts, `shift_to()` method
  - `Differential` — hypoid gear whine with housing resonance
  - `ChainDrive` — periodic link engagement rattle on sprocket teeth
  - `BeltDrive` — friction squeal and belt flap with tension control
- **Preset system** (`presets` module) — shipped factory presets: `v8_muscle_car`, `inline4_economy`, `diesel_truck`, `motorcycle_single`, `electric_vehicle`, `turbocharger`, `supercharger`, `manual_5speed`, `manual_6speed`, `steel_spur_gear`, `industrial_turbine`, `propeller`
- `naad-backend` feature flag (default on) — enables naad DSP primitives
- `process_block(&mut self, output: &mut [f32])` streaming API on all synthesizers
- Real-time parameter setters: `set_rpm()`, `set_load()`, `set_tension()`, `shift_to()`
- `sample_position` tracking for seamless streaming across `process_block()` calls
- `ComputationError` variant on `GhurniError`
- `dsp` module with `DcBlocker`, `validate_sample_rate`, `validate_duration`
- Fallback path when `naad-backend` is disabled (original math.rs + rng.rs)
- 44 integration tests: all types, events, presets, mixer, trait dispatch, parameter sweeps, continuity, serde roundtrips
- Benchmarks: block size sweep (64-4096), mixer, turbocharger, transmission

### Removed

- Direct `hisab` dependency (unused; naad depends on it transitively)

## [0.1.0] - 2026-03-27

### Added

- Initial scaffold of the ghurni crate
- **Engine**: 4 types (Gasoline, Diesel, TwoStroke, Hybrid) with combustion pulses, exhaust resonance, mechanical noise. RPM-driven firing frequency with cylinder count
- **Gear**: 4 materials (Steel, CastIron, Brass, Nylon) with tooth mesh frequency, resonant ringing, material-specific decay and brightness
- **Motor**: 4 types (DcBrushed, AcInduction, Brushless, Servo) with electromagnetic hum harmonics, commutator/bearing noise, pole-count-driven frequency
- **Turbine**: Blade pass frequency synthesis with harmonic content, whoosh noise, optional duct resonance
- **Clock**: 4 types (Wristwatch, WallClock, GrandfatherClock, PocketWatch) with escapement tick, resonant decay, type-specific frequency and amplitude
- `GhurniError` with serde roundtrip
- PCG32 PRNG for stochastic mechanical noise
- Integration tests: all engine/gear/motor/clock types, firing/mesh frequency verification, energy comparison, serde roundtrips
- Criterion benchmarks: V8 gasoline, diesel 6-cyl, steel gear, brushless motor, turbine, wristwatch
- `no_std` support via `libm` + `alloc`
- Strict `deny.toml` matching hisab production patterns
- Send/Sync compile-time assertions on all public types
