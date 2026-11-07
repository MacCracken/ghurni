# ghurni Roadmap — 2.x

Organised as **arcs**: each minor (`2.N.0`) is one themed body of work, and each
carries a **patch line** (`2.N.x`) for the follow-up that predictably lands after
it ships. Nothing here is scheduled by date; the ordering is by dependency.

**Semver contract for this project** — the spine of the whole plan:

| | Allowed | Forbidden |
|---|---|---|
| `2.0.x` **patch** | bug fixes, tests, docs, CI, release hygiene, internal perf proved bit-identical | any new public `ghurni_*` symbol; **any change to rendered audio** |
| `2.N.0` **minor** | new synths, new public functions, new parameters, deliberate audio changes | breaking an existing signature |

> A deliberate audio change also needs an **ADR**, because it diverges from the
> parity oracle (CLAUDE.md). Both 2.0.1 and 2.0.2 shipped audio changes inside a
> patch — justified each time (a miscompiled constant; a collapsed noise seed),
> but the rule was bent twice. From 2.0.3 on it is enforced: if `cyrius test`
> golden checksums move, it is not a patch.

---

## Shipped

Full detail lives in [`CHANGELOG.md`](../../CHANGELOG.md) and the ADRs; this is
the index. Everything below is closed — nothing here is outstanding work.

| release | arc | what it was | record |
|---|---|---|---|
| `2.0.1` | — | toolchain + dependency catch-up | CHANGELOG |
| `2.0.2` | — | P-1 audit / hardening / security sweep | CHANGELOG |
| `2.0.3` | Arc 0 "Pin the Oracle" | golden checksums, spectral assertions, the fuzz harness and the CI gate chain — the deck-clearing that made the oracle deletable and the audio arcs safe | CHANGELOG |
| `2.0.4` | Arc 0b "Delete the Oracle" | `rust-old/` retired; the correctness bar moved to the frozen suite | [retirement doc](rust-old-retirement.md) |
| `2.1.0` | Arc 1 "Rest State" | one shared RPM loudness law (`2r/(r+1)`), plus the sample-integrated combustion pulse that closed the resonant-RPM notch | [ADR-005](../architecture/adr-005-rpm-loudness-law.md) |
| `2.2.0` | Arc 2 "Depth" | 3-partial mesh tone, HYBRID as a real electric drive, turbo/blower orders, audible chain links, tick/tock asymmetry | [ADR-006](../architecture/adr-006-acoustic-depth.md) |
| `2.3.0` | Arc 2b "Depth II" | load tilts the spectrum instead of moving a fader; diesel injection clatter; differential ring-revolution modulation | [ADR-007](../architecture/adr-007-load-tilt-and-diesel.md) |
| `2.4.0` | Arc 3 "Events & Control" | five dead `GH_EVENT_*` kinds made real, drivetrain load inputs, clock speed control, the missing `name`/`from_name` halves | [ADR-008](../architecture/adr-008-events-and-control.md) |
| `2.5.0` | Arc 3b "Output Path" | stems **deferred on measurement** (the engine had nothing to split); convolution **closed** on ADR-002 scope; two dead-API defects fixed | [ADR-009](../architecture/adr-009-no-stems-yet.md) |
| `2.5.1` | — | pulled naad 2.2.2 (its FFT convolution dropped its tail); corrected the assertion counts reported for 2.3.0/2.4.0/2.5.0 | CHANGELOG |
| `2.6.0` | Arc 3c "Radiation Paths" | the exhaust becomes a quarter-wave waveguide and the combustion pulse train goes down it; cyrius 6.5.36 | [ADR-010](../architecture/adr-010-radiation-paths.md) |
| `2.6.2` | — | cyrius 6.6.20 and the naad 2.2.5 set; no golden moved, and audio differs by at most 8.9e-16; 28 doc comments; envelopes 12-32% slower because `f64_pow` got correct | CHANGELOG |
| `2.7.0` | — | cyrius 6.7.9 and the naad 2.3.0 set; audio byte-identical to 2.6.2; ghurni's own set now needs cyrius 6.7.2 or later | CHANGELOG |
| `2.7.1` | — | cyrius 6.7.18 and the newest tag of every dep; audio byte-identical to 2.7.0, floor stays 6.7.2; most synths 5-10% faster through naad; duct delay line now checked | CHANGELOG |

**Three corrections worth remembering**, because each was a claim this project
made and later disproved with its own measurements:

- **ADR-009's acceptance criterion is passed by a wire** (66.12% share, +0.743
  autocorrelation with no delay, filter or resonance). It is a *routing* test and
  must never be cited as evidence a radiation path exists. Corrected in ADR-010.
- **ADR-009's tripwire never fired.** It was written so that a real duct would
  break it; 2.6.0 built one and it still passed at −0.27 dB. A waveguide's
  throughput is level-invariant under retuning — detuning a pipe moves its
  *colour*, not its level. Retired and replaced.
- **ADR-006 called stems "an API change, not a timbral one."** Exactly backwards,
  and that single line routed the item through two wrong arcs before ADR-009
  measured it.

### Closed out of the shipped arcs

- ~~12 compiled ELF binaries tracked in git~~ — untracked; `/build/` is in
  `.gitignore` and `git ls-files build/` is empty.
- ~~Source-body impulse responses need their own ADR~~ — **decided in ADR-009**:
  keep the bandpass. ADR-002 forbids reverb (dhvani's) and spatialization
  (goonj's), naad's type is a room model, cost is 21×–7161× a biquad, and
  `cyrius distlib` concatenates `.cyr` **text** so the format cannot carry a
  binary IR at all.
- ~~Re-check the ADR-005 loudness law against the new duct~~ — **done during the
  2.6.0 architecture choice**, and it decided it. The rejected inverting-comb
  swung mono level −2.16 to +1.99 dB across 800–7000 rpm, which modulates the
  loudness law; the shipped waveguide holds −0.13 to +0.78 dB.

### Open patch line: `2.6.x`

The only patch line from a shipped arc with work still in it. Earlier ones
(`2.1.x`–`2.5.x`) are closed.

- [ ] **Duct tuning per engine type**, and golden refreshes if it moves them.
      `GHURNI_DUCT_RAD_HZ = 520` in particular is an argued value, not a measured
      one — it is what makes the model read as an exhaust *system* near the
      ground rather than a bare pipe in free field, and it costs ~4.5 dB in the
      45–180 Hz bands. Reviewed by ear and accepted for 2.6.0; revisit with a
      second opinion rather than a second measurement.
- [ ] **Two-stroke expansion chamber** — characterised, not papered over. A
      two-stroke's chamber is a divergent-then-convergent cone pair whose whole
      purpose is a positive reflection followed by a negative one; the shipped
      quarter-wave model is the wrong device for it, and the two-stroke misses
      the ADR-009 share bar at 21.9%.

---

## Arc 3d — `2.7.0` "Intake & Stems"

**Theme: finish the radiation model, then hand out the lanes.**

- [ ] **Intake duct** — Helmholtz plenum + runner, to the same bar the exhaust
      cleared. An airbox is a duct too; that is what airbox resonance IS.
- [ ] **Per-component stems**, keyed on radiation path, once there are two real
      lanes. ADR-009's taxonomy is the starting point — it was the correct
      definition applied to a model that could not feed it.
- [ ] Let the duct energy split (`GHURNI_DUCT_DRIVE` / `_STRUCT`) survive one
      release in consumers' hands before an occlusion curve is built on it.

**Patch line `2.7.x`** — stem-level tuning.

---

## Arc 4 — `2.8.0` "New Mechanisms"

**Theme: breadth.** Deliberately last: a new synth built before Arc 1's loudness
law and Arc 0's fuzz harness would inherit both problems on day one.

Ranked by what a consumer hits first. The old roadmap's five are re-scoped:

- [ ] **Brake system** — the largest single source gap, and the old roadmap
      under-scoped it to *squeal only*. Pad contact, disc scrape, ABS pulsing.
- [ ] **Tyre / wheel contact** — the other half of the vehicle contact patch,
      absent from code and roadmap alike. Check the ADR-002 line against garjan
      before starting (surface interaction may be shared).
- [ ] **Ratchet / pawl / detent click train** — highest reuse-per-line item in the
      whole list, and on no roadmap. Feeds handbrakes, winches, gear selectors,
      clockwork.
- [ ] **Pneumatic release / valve exhaust** — outranks the old roadmap's impact
      wrench, and `forced_induction`'s blow-off valve is already the primitive.
- [ ] **Hydraulic actuator** — the old roadmap picked the *pump*; consumers reach
      for the *actuator* (extend, stall, relief).
- [ ] **Bearings — as a wear/defect modifier, not a standalone synth.** BPFO/BPFI/
      BSF/FTF sidebands belong on top of an existing rotating source. The old
      roadmap had this in the wrong section.
- [ ] **Compressors** — reciprocating chug, rotary/screw whine. Keep; note this is
      the one item that layering nearly solves already.
- [ ] **Stepper motor** — a fifth motor family whose sound is categorically unlike
      the four that ship.
- [ ] **Turbine rotor slap and multi-spool** — moved here from Arc 2b in 2.3.0.
      Helicopter blade slap is impulsive and multi-spool is a jet architecture;
      both are new machines rather than a deeper turbine.
- [ ] Weapon actions / rotary cannon — large genre, zero coverage. Check the
      garjan boundary first (the *impact* is theirs; the *action* is ours).

**Patch line `2.8.x`** — per-synth tuning for the new mechanisms.

---

## Dropped

Removed with reasons, so they are not re-proposed:

- **Doppler wrapper** — ADR-002 assigns it to goonj, and the old roadmap line
  already conceded "or defer to goonj". Dropped.
- **Parameter automation curves** — ADR-002 puts RTPC mapping in dhvani/kiran.
- **Per-sample accessor hoisting** — measured in 2.0.2 on belt_drive: bit-identical
  and **no measurable time win**. Not worth scheduling.
- **"Raise reference coverage" as a goal** — kept as a CI *ratchet*
  (`cyrius coverage --min`), not a target. Coverage is a floor, and 2.0.2's
  post-mortem is the standing proof that a green suite can pin nothing.
- **`cyrius vet` UNTRUST on `dist/ghurni.cyr`** — structural (it is the project's
  own generated bundle, not a `[deps]`-resolved one) and there is no trust-list
  mechanism in the manifest. Not a defect.

## Blocked upstream — not in any arc

- **Presized output buffers.** `*_synthesize` grows one `vec_push` at a time
  (~2× heap waste). The stdlib has no `vec_with_capacity`; this wants an upstream
  `vec` API, not a reach into vec internals.
- **The allocation-failure contract.** 15 unguarded `alloc()` sites, and no
  sibling library checks either. Wants an ecosystem-wide decision.

## Architectural notes

- `sample_position` is an `i64` sample counter — no practical overflow.
- Deterministic output depends on naad's `NoiseGenerator` seeding. Seeds derive
  from constructor parameters; if you add a synth, make sure every seeded
  parameter actually **reaches** the seed with enough resolution to matter — a
  bare `f64_to` of a sub-unity parameter truncates to a constant, which is the
  defect fixed twice already (forced_induction, belt_drive).
- `dist/ghurni.cyr` is compiled by the **consumer's** toolchain, so anything
  sensitive to compiler behaviour (long float literals in particular) must be
  written in a form an older toolchain cannot get wrong.
- Adding a synth touches eight places: the module, `traits.cyr`, both mixer
  dispatchers, `add_channel`'s range check, `cyrius.cyml [lib].modules`, the test
  suite, and the bench. Worth writing down as a checklist before Arc 4.

---

## Moving the cyrius pin to 6.6.6

**Current pin: `cyrius = "6.6.3"`.** Nothing must change first — pin bump and rebuild.

**What was checked** (19 `.cyr` under `src/`; vendored `lib/` excluded):

- **Windows write corruption: not a ghurni exposure.** `O_APPEND` / `O_TRUNC` appear **33
  times, all inside the vendored `lib/` fold, zero in `src/`**. ghurni opens no files —
  no `file_open` / `sys_open` / `file_read_all` / `file_exists` call site in `src/`.
- **Struct copies / by-value struct params: nothing to change.** ghurni declares **15
  `Gh*` structs** (`GhEngine`, `GhTransmission`, `GhMixerChannel`, …) and **not one** is a
  by-value fn parameter, a fn return type, or a `var x: GhT = …` declaration — all are
  heap-offset layouts reached by pointer. The two mixer dispatchers and `add_channel` pass
  handles, not values, so the new deep-copy semantics change nothing.
- **Also clean:** no `async fn`, no `operator` fn, no SIMD-returning fn, no fn mixing pair
  and scalar returns, no `var` inside a top-level block (zero top-level `{` / `if (` /
  `while (` at column 0), no raw `SYS_STATFS`, no `lib/regression.cyr` consumer, no `vec_*`
  of ghurni's own, no duplicate top-level global, no symlink in `lib/`.
- **Platform:** CI is `ubuntu-latest` only, no `CYRIUS_TARGET_*` branch in `src/`.

**What it gains:** 6.6.5's aggregate-layout fix (silently wrong since 5.8.17) and the
corrected ENTRY stack bases, plus 6.6.6's nine new refusals. **Not** a gain here: 6.6.4's
≥64 KB string-literal fix — ghurni's longest literal is 64 bytes (`src/dsp.cyr`), measured.

**Verify after bumping:** `cyrius deps` → `cyrius build` → `cyrius test` → `cyrius bench`
→ **regenerate `dist/ghurni.cyr`**. ⚠ The *"compiled by the consumer's toolchain"* note in
the section above is the live constraint on this bump: verify the regenerated bundle still
compiles under the **oldest** pin in ghurni's consumer set — long float literals in
particular — before tagging, not just under 6.6.6. Add the bump to the eight-place
"adding a synth" checklist's sibling list, since `cyrius.cyml` is one of the places it touches.
