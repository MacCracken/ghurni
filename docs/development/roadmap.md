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

## Arc 0 — `2.0.3` "Pin the Oracle"

**Theme: make every behaviour `rust-old/` currently proves provable without it.**
Nothing here changes audio or adds public API. This is the deck-clearing patch
that makes the oracle deletable and the audio arcs safe.

### Pin what only the oracle checks
- [ ] **All 12 preset parameter sets.** Only 4 are touched by any test, and only
      for "has energy" — no assertion reads a stored value back. A typo
      (`3.8` → `3.6` in `manual_6speed`) is caught by nothing today.
- [ ] **The 36 per-material / per-type property constants.** Clock's 16 and
      motor's 8 are reachable through pure `#must_use` fns
      (`ghurni_clock_tick_rate_for` / `_resonance_for` / `_decay_for` / `_amp_for`,
      `ghurni_motor_noise_cutoff` / `_noise_level`) — one-line assertions each.
      Gear's 12 are inline literals in `ghurni_gear_new`, so they need the
      `GhGear_resonance` / `_decay` / `_brightness` accessor route.
      *(The retirement doc says "32"; the actual enumeration is 36.)*
- [ ] Engine even-firing offsets, per-type exhaust/intake resonances, event durations.
- [ ] The auto-BOV trigger triple (`prev_load > 0.5`, `load < 0.2`, `spool > 5000`).
- [ ] The differential's **ring/pinion asymmetry** — mesh frequency uses *pinion*
      teeth. A swap is completely silent today.
- [ ] The equal-power pan law's **left=cos / right=sin** assignment and its
      −3 dB centre attenuation. A left/right swap passes the suite today.
- [ ] `ChainDrive` silence below 1 Hz engagement **and its non-advancing RNG**.
- [ ] `process_block_stereo` leaves the longer buffer's tail **untouched**.
- [ ] `Transmission::new` accepts an **empty** ratios vec; `current_ratio()` → 1.0.

### New test machinery
- [ ] **Golden-file regression** — checksum every synth's output for fixed
      parameters. This is the gate that makes the semver rule above enforceable:
      after this lands, an unintended audio change fails CI instead of shipping.
      **Highest-value item in the arc.**
- [ ] **Spectral validation** — assert the peak bin matches the expected
      firing / mesh / blade-pass frequency. Cheaper than the old roadmap assumed:
      naad already ships `fft_magnitudes` (`lib/naad.cyr:499`), so this is a
      direct call, not an FFT project. Input length must be a power of two, so
      use 4096/8192 rather than the current 4410/2205 block sizes.
- [ ] **Fuzz harness** — `cyrius fuzz` runs `fuzz/*.fcyr`; ghurni has none, and
      the command currently **exits 0 while finding nothing**, which is a vacuous
      gate. Drive the public boundary with `INT64_MIN`, NaN, ±inf, subnormals,
      `f64::MAX`, non-power-of-two lengths. Sweep enum ids at valid parameters
      and values at a valid id — correlate them and the harness proves nothing
      (naad's lesson).
- [ ] **Rest-state invariant** — one cheap check that collapses three findings:
      no synth should be as loud stopped as running. See Arc 1 for the fix; the
      *test* can land here as a characterisation (recording today's values), then
      flip to the invariant in 2.1.0.

### Release hygiene — blocks a clean 2.0.3
- [ ] **12 compiled ELF binaries (11 MB) are tracked in git** and ship inside
      every release tarball, including 5 stale `build/err_*` from an old naming
      scheme. `git rm -r --cached build/`.
- [ ] **`.gitignore` is still the Rust crate's** (`/target`, `Cargo.lock`,
      `*.rs.bk`, `*.pdb`) and has no `/build/` — which is how the binaries got in.
- [ ] **`scripts/version-bump.sh` is Rust-era**: it writes `VERSION`, then seds a
      root `Cargo.toml` that does not exist and runs `cargo generate-lockfile`.
      It half-bumps the repo and fails. It would corrupt the 2.0.3 release.
- [ ] **CI runs no quality gates.** `.github/workflows/ci.yml` is
      deps → build → test → build-examples. Add `cyrius audit` (exits 0 today),
      `cyrius deny` (exits 0), `cyrius distlib --check` (nothing currently
      prevents `dist/ghurni.cyr` drifting from `src/` — and `dist/` is what
      consumers compile), and **actually run** the examples: CI builds all five
      and never executes them, so a compiling-but-crashing example passes.

### Documentation defects (several ship inside `dist/ghurni.cyr`)
- [ ] **README advertises a wastegate.** `grep -rn wastegate src/ dist/` returns
      nothing. Either implement it (Arc 2) or stop claiming it.
- [ ] `src/dsp.cyr:12` cites `tests/dsp.tcyr`, **which does not exist** — and the
      comment ships to consumers at `dist/ghurni.cyr:196`.
- [ ] ADR-004 still carries the serde justification `state.md` already corrected
      as false (`MixerChannel` serialized `name`/`gain`/`pan`/`muted`).
- [ ] Stale measured numbers across `state.md`, `overview.md`, `README.md`.
      Current truth: **82/167 functions referenced (49%)**, **7 suites /
      224 assertions**, `dist/ghurni.cyr` 3,161 lines.
- [ ] The retirement checklist's own `rust-old` reference counts are wrong.
- [ ] No `2.0.0` CHANGELOG entry — a re-cut of that tag still ships an empty
      release body. Release machinery, not a nicety.
- [ ] Record the undocumented contracts: `smooth_time_s` is **tau** (63%), not a
      settling time; `MechanicalEvent` cylinder is 0-indexed and `GearShift.from`
      0 = neutral; the cross-plane V8 offsets exist because uneven intervals make
      the burble; the supercharger preset's `spool_inertia` argument is inert.

### Small correctness items found by the sweep
- [ ] `ghurni_dcblocker_new` is the **only** public constructor that does not
      validate its sample rate.
- [ ] `ghurni_gear_new` clamps teeth only from below; every sibling
      integer-count constructor clamps both ends.
- [ ] `GhSmoothedParam`'s derived serde roundtrip is untested while both sibling
      POD structs are.
- [ ] Two setters silently no-op and return success — the exact shape 2.0.2 hunted.
- [ ] `f64_clamp` propagates NaN where `f64_max`/`f64_min` absorb it. Several NaN
      entry points render 100% non-finite audio, and **SECURITY.md claims
      otherwise**. Fix the doc here; fix the behaviour in Arc 1 (it is an audio
      change at the boundary).
- [ ] **Benchmark gap** — 6 of the oracle's 10, plus its 64→4096 block-size sweep.
      Missing: motor, turbine, clock, transmission, turbocharger.

---

## Arc 0b — `2.0.4` "Delete the Oracle"

**Theme: retire `rust-old/`.** Its own release so the deletion is a revertable
commit rather than noise inside a feature arc.

- [ ] Everything in [`rust-old-retirement.md`](rust-old-retirement.md) is checked.
- [ ] `git rm -r rust-old/`; update the 61 references across 34 files.
- [ ] **Rewrite CLAUDE.md's correctness bar.** It currently reads "matches what
      the Rust naad-backend path did". That stops being checkable the moment the
      directory goes; it becomes "matches the frozen goldens in `tests/`" — which
      is only true if Arc 0 actually landed them. **This is why Arc 0 comes first.**
- [ ] Note in ADR-004 which tag the oracle is recoverable from.

> **Sequencing constraint.** Both audio arcs below are blocked on this one. While
> `rust-old/` exists, the stated correctness bar is "match the oracle", so a
> deliberate divergence has to argue against the project's own rule every time.
> Delete it first and the bar becomes the goldens, which is the bar you actually
> want for acoustic work.

---

## Arc 1 — `2.1.0` "Rest State"  ⚠ audio-changing

**Theme: RPM actually drives loudness.** Ships **all** deliberate audio changes
at once, behind one ADR, so there is exactly one release where goldens move.

Measured today (0.2 s, RMS, rest vs running):

| synth | @ 0 RPM | running | |
|---|---|---|---|
| motor | 0.184407 | 0.184407 | **identical stopped and running** |
| belt_drive | 0.084913 | 0.069786 | **louder stopped** |
| turbine (ducted) | 0.106632 | 0.251488 | 42% of running while stationary |
| gear | 0.026472 | 0.106961 | 25% |
| differential | 0.019027 | 0.140681 | 14% |
| transmission | 0.016178 | 0.106189 | 15% |
| engine | 0.008977 | 0.026329 | 34% |
| chain_drive · forced_induction | 0.000000 | — | correct |

For a library whose thesis is *"RPM is the fundamental parameter"*, seven of ten
synths contradict it. The causes are structural, not bugs: `motor`'s amplitude
path has no RPM term at all, and `belt_drive`'s `squeal_amp = (1 - tension) * 0.3`
has none either. Both are faithful ports — the oracle does the same — which is
exactly why this needs an ADR rather than a patch.

- [ ] **ADR-005: acoustic divergence from the oracle.** The umbrella for this arc.
- [ ] An RPM-dependent loudness law for the seven synths that lack one.
- [ ] **Resonant-RPM combustion amplitude.** At RPMs where samples-per-cycle is an
      integer (`5292000/rpm ∈ ℤ` at 44.1 kHz) the engine lands off-peak every
      cycle and the thump loses up to two thirds of its amplitude; the f32
      oracle's coarser rounding accidentally snapped the lattice onto the cycle.
      Characterised in [state.md](state.md). Same ADR, same release.
- [ ] **NaN at the parameter boundary.** `f64_clamp` propagates it; guarding is
      itself an audio change, so it belongs here.
- [ ] Seed aliasing the port introduced in `belt_drive` / `turbine` /
      `forced_induction` (behavioural integer seeds that lose parameter
      resolution) — decide the convention once and apply it uniformly.

**Patch line `2.1.x`** — regressions the new goldens catch; per-synth loudness-law
tuning that consumers report as too aggressive or too subtle.

---

## Arc 2 — `2.2.0` "Depth"  ⚠ audio-changing

**Theme: the ten synths get acoustically richer.** Breadth stays frozen; this arc
is about the machines already modelled sounding more like themselves.

- [ ] **Load is a pure gain control** — it produces no spectral tilt anywhere.
- [ ] **Every gear-mesh whine in the library is a single pure sine** — no
      harmonics, no sidebands, no tooth-profile error content. Affects gear,
      transmission and differential together.
- [ ] **Turbo and supercharger are spectrally identical**; forced-induction whine
      sits at shaft rate rather than blade-pass rate; no surge or flutter; and the
      **wastegate the README advertises does not exist**.
- [ ] **`GH_ENGINE_HYBRID` is documented "electric-with-whine" and is a gasoline
      engine** with different constants.
- [ ] Diesel is a retuned Gasoline; no valvetrain, injector or timing-drive content.
- [ ] Clock tick and tock are the same sound, perfectly periodic, with no strike.
- [ ] Chain drive: link count is audibly inert; no slack or polygon effect.
- [ ] Differential: no ring-revolution modulation, no drive/coast asymmetry.
- [ ] Rotor slap and multi-spool turbines — turbine has the frequency but not the
      character of helicopters or jets.
- [ ] **Per-component stems.** Every synth sums exhaust + intake + mechanical into
      one mono buffer. Rendering those separately is a real API gap — and it is
      the *source-side* half of the old "multi-channel output" item, which was
      otherwise already satisfied (mono stems positioned downstream by goonj).
- [ ] Source-body impulse responses (exhaust pipe, engine bay). This is **not**
      goonj's: ghurni already owns exhaust resonance as a constructor field.

**Patch line `2.2.x`** — per-synth spectral tuning; golden refreshes.

---

## Arc 3 — `2.3.0` "Events & Control"

**Theme: the control surface stops lying.**

- [ ] **Five of the eight declared `GH_EVENT_*` kinds are accepted and silently
      discarded** — `STALL`, `REV_LIMITER_HIT`, `GEAR_SHIFT`, `STARTUP`,
      `SHUTDOWN`. Constructors exist for all of them
      (`ghurni_event_stall`, `ghurni_event_rev_limiter_hit`, …); only `engine` has
      a `trigger_event` at all, and it implements three. The oracle ignored them
      too, so this is an inherited product gap, not a port defect — but the
      oracle's cover disappears in 2.0.4.
- [ ] **Engine start / stop chain** — the sound consumers reach for first and the
      one `STARTUP`/`SHUTDOWN` imply.
- [ ] **Clock has no speed control at all** — its tick rate cannot be set.
- [ ] **Gear, transmission and differential accept no load input**, though load is
      what makes a drivetrain audible under strain.
- [ ] `GH_EVENT_*` and `GH_KIND_*` still violate the `name`/`from_name` rule that
      2.0.2 fixed for `GhurniError`.
- [ ] `ghurni_differential_ratio` is dead API; `sample_position` is vestigial in
      three synths.

**Patch line `2.3.x`** — event-timing tuning; consumer-reported control gaps.

---

## Arc 4 — `2.4.0` "New Mechanisms"

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
- [ ] Weapon actions / rotary cannon — large genre, zero coverage. Check the
      garjan boundary first (the *impact* is theirs; the *action* is ours).

**Patch line `2.4.x`** — per-synth tuning for the new mechanisms.

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
