# Retiring `rust-old/`

`rust-old/` is the Rust 1.0.0 crate, kept as the **parity oracle** for the
Cyrius port ([ADR-004](../architecture/adr-004-cyrius-port.md)). This document
is the checklist for deleting it.

**Status: NOT YET CLEAR.** The port is functionally complete — but a body of
knowledge still exists *only* in the oracle, and deleting it before that is
written down would make several load-bearing decisions unverifiable.

## What the sweep verified

A module-by-module comparison of all 18 portable Rust modules against the
Cyrius port (`math.rs` and `rng.rs` are deliberately unported — ADR-004).

| Check | Result |
|-------|--------|
| Public functions | **83 / 83** portable Rust `pub fn`s have a Cyrius counterpart (22 more are `math.rs`/`rng.rs`, excluded by ADR-004) |
| Enums + variants | 7 / 7 enums, all 29 variants → `GH_*` constants |
| `Synthesizer` trait | 4 methods × 10 synths = 40 / 40 concrete accessors |
| Integration tests | **44 / 44** Rust `#[test]`s reproduced as `.tcyr` assertions |
| Examples | 5 / 5 ported and running |
| Benchmarks | **6 / 10** — see gaps below |
| Numeric fidelity | Every constant, clamp bound, frequency formula, envelope shape and noise-draw ordering checked. **Zero mismatches found.** |

Nothing in the repo *builds* against `rust-old/`; the coupling is entirely
documentary (61 references across 34 files, listed at the end).

## Closed during this sweep

These were found by the sweep and are already fixed:

- **`belt_drive`'s noise seed dropped its tension term.** `f64_to(tens)` with
  tension clamped to 0..1 truncated to 0 for every tension below 1.0, so every
  belt of a given integer diameter shared one bit-identical pink-noise stream —
  the documented slack-vs-tight layering summed perfectly correlated noise.
  Now scaled ×1000, matching `forced_induction`. Pinned and mutation-checked.
- **`ghurni_fmod`'s doc comment was wrong and dangerous.** It claimed the
  operands are non-negative; `engine.cyr` passes a *negative* dividend on every
  sample for every cylinder whose firing offset exceeds the crank angle. The
  floored behaviour is load-bearing — a "simplification" to a truncated
  remainder would silently drop most firing events. Comment corrected, and four
  assertions now pin the negative-dividend cases.
- **`GHURNI_DB_SCALE`'s comment** now records that the oracle *divides* by
  `LOG10_E` where the exp→dB conversion would multiply, making the naad decay
  5.30× steeper than the `exp(-8t)` its fallback used. That asymmetry is the
  oracle's own quirk, reproduced deliberately; without the note a future
  maintainer would "correct" it and rewrite every engine/gear/clock envelope.
- **`ghurni_err_from_name` added.** `GhurniError` was the one public enum with
  only the forward half, breaking CLAUDE.md's `_name`/`_from_name` rule.
- **`ghurni_synth_rpm` / `ghurni_synth_sample_rate` added.** The tag dispatch
  covered only 2 of the `Synthesizer` trait's 4 methods, so a consumer holding a
  heterogeneous `(GH_KIND_*, ptr)` collection could not read RPM or sample rate
  generically.
- **`belt_drive`'s smoother** is now documented as deliberately inert (the
  oracle writes `let _smooth = ...` and uses raw RPM).

## Blockers — must land before deletion

### 1. Preserve the oracle-only knowledge

Each of these exists *only* in `rust-old/` today. Write it into a `.cyr`
comment, a test assertion, or `state.md` — whichever the item names.

**Constants and parameters nothing pins**
- [ ] **All 12 preset parameter sets.** Verified identical to the oracle, but
      only 4 are called by any test on *either* side, and every assertion is
      finite-or-has-energy — none reads a value back. A typo (`3.8` → `3.6` in
      `manual_6speed`) is currently caught only by diffing against `presets.rs`.
      → assert the stored ratios / teeth / blade count / rpm / load for all 12.
- [ ] **The 32 per-material / per-type property constants** (gear resonance,
      decay, brightness ×4 materials; motor cutoff and noise level ×4; clock
      tick rate, resonance, decay, amplitude ×4). None pinned.
- [ ] **The engine's even-firing offsets**, exhaust/intake resonance per
      `EngineType`, and event durations.
- [ ] **The auto-BOV trigger triple** (`prev_load > 0.5`, `load < 0.2`,
      `spool > 5000`) — pinned by no test on either side.
- [ ] **The differential's ring/pinion asymmetry** — mesh frequency uses *pinion*
      teeth, not ring. A swap is completely silent today.
- [ ] **The equal-power pan law's left=cos / right=sin assignment** and its
      −3 dB centre attenuation. A left/right swap passes the suite today.

**Contracts and rationale**
- [ ] `smooth_time_s` is **tau** (63% of the way to target), not a settling time.
- [ ] `MechanicalEvent` payloads: cylinder is 0-indexed; `GearShift.from` 0 = neutral.
- [ ] The cross-plane V8 offsets exist because uneven intervals create the burble.
- [ ] The supercharger preset's `spool_inertia` argument is inert (always 0.01).
- [ ] `ChainDrive` is exactly silent below 1 Hz engagement **and its RNG does not
      advance** during that silence — so its noise is a function of RPM history.
- [ ] `process_block_stereo` leaves the tail of the longer buffer **untouched**,
      not zeroed. Untested on both sides.
- [ ] `sample_position` is vestigial in transmission / differential /
      forced_induction (its only readers were the dropped fallback arms).
- [ ] `Transmission::new` accepts an **empty** ratios vec; `current_ratio()` then
      returns 1.0.
- [ ] Typical-vs-legal parameter ranges from the Rust rustdoc (pulley diameter
      "typically 50–200 mm", chain links "typically 100–120").

### 2. Correct two claims in `state.md` that are wrong today

Both become uncheckable the moment the oracle goes:

- [ ] **The RNG-seed bullet is over-broad.** It says seeds are "behavioural, not
      bit-identical" without scope. Only `belt_drive`, `turbine` and
      `forced_induction` fold in a float. `engine`, `gear`, `motor`, `clock`,
      `transmission`, `differential` and `chain_drive` seed from integers and are
      **bit-identical to the oracle** — a maintainer "harmonising" them would
      silently change seven synths' audio.
- [ ] **The serde justification is factually wrong.** It says the synth structs
      dropped deep serialization because "nothing meaningful survived the Rust
      `#[serde(skip)]`". For `MixerChannel` that is false: only `synth` and
      `scratch` were skipped — `name`, `gain`, `pan` and `muted` all serialized,
      and a Rust consumer could persist a whole mixer config as a template.

### 3. Close the benchmark gap

`rust-old/benches/benchmarks.rs` has 9 named benchmarks plus a block-size sweep;
`benches/ghurni.bcyr` has 6. Missing: **motor**, **turbine**, **clock**,
**transmission**, **turbocharger**, and the **64→4096 block-size sweep**.

## Deliberately NOT carried over

Decisions, not gaps — record them in `state.md` and move on:

- `#[derive(Clone)]` on every synth (deep-copying live DSP state). No Cyrius
  equivalent; not currently listed as a divergence, so it reads as an oversight.
- Full serde round-trip of synth structs, including `MixerChannel` config.
- `#[inline]` on the `process_block` bodies.
- Sub-ULP sum re-association in engine / gear / drivetrain / mixer stereo gain.
- `ghurni_event_new` does not range-check its `kind` (harmless: an unknown kind
  falls through every arm exactly as Rust's `_ => {}` did).
- A sub-sample duration is now an error where the oracle returned an empty
  success — a second, unrecorded half of the `ghurni_sample_count` guard.

## Mechanical deletion steps

Once the above is landed:

1. `git rm -r rust-old/`
2. Update the **61 references across 34 files**:
   - `src/*.cyr` — 17 refs in 15 module headers. **These propagate into
     `dist/ghurni.cyr`**, the shipped consumer bundle. Rewrite as "ported from
     the Rust 1.0.0 crate" without a path, or keep the path and note it is
     historical (tag `1.0.0` still has it).
   - `tests/*.tcyr` — 5 refs in 4 suites.
   - `docs/examples/*.cyr` — 5 refs, one per example header.
   - `CLAUDE.md` — 4 refs, including the "correctness bar" rule and the
     DO-NOT-MODIFY entry. **This is the important one**: the project's stated
     correctness bar is "matches what the Rust naad-backend path did", which
     stops being checkable. Replace it with "matches the frozen assertions in
     `tests/`" and say the oracle is retired as of tag `<x>`.
   - `README.md`, `CONTRIBUTING.md`, `docs/architecture/{overview,adr-001,adr-004}.md`,
     `docs/development/{state,roadmap}.md`, `docs/guides/testing.md`.
   - `CHANGELOG.md` — 6 refs. **Leave these alone**; a changelog is a historical
     record and its references were accurate when written.
   - `lib/*.cyr` references to `rust-old` belong to the vendored naad / goonj /
     hisab bundles and are none of ghurni's business.
3. `cyrius distlib` to refresh the bundle headers.
4. Note in `ADR-004` that the oracle was retired, and at which tag it can still
   be recovered from git history.

> The oracle is never truly gone — `git show 1.0.0:src/engine.rs` recovers it.
> The point of this checklist is that *nobody will think to look*, so anything
> load-bearing must live in the port before the directory does not.
