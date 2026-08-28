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

## Arc 0 — `2.0.3` "Pin the Oracle"  ✅ SHIPPED

> Released 2026-08-27. 10 suites / 450 assertions (was 7 / 224), coverage
> 49% → 66%, 14 benchmarks (was 6), a 1631-check fuzz harness, and the full CI
> gate chain. See the 2.0.3 CHANGELOG entry.

**Theme: make every behaviour `rust-old/` currently proves provable without it.**
Nothing here changes audio or adds public API. This is the deck-clearing patch
that makes the oracle deletable and the audio arcs safe.

### Pin what only the oracle checks
- [x] **All 12 preset parameter sets.** Only 4 are touched by any test, and only
      for "has energy" — no assertion reads a stored value back. A typo
      (`3.8` → `3.6` in `manual_6speed`) is caught by nothing today.
- [x] **The 36 per-material / per-type property constants.** Clock's 16 and
      motor's 8 are reachable through pure `#must_use` fns
      (`ghurni_clock_tick_rate_for` / `_resonance_for` / `_decay_for` / `_amp_for`,
      `ghurni_motor_noise_cutoff` / `_noise_level`) — one-line assertions each.
      Gear's 12 are inline literals in `ghurni_gear_new`, so they need the
      `GhGear_resonance` / `_decay` / `_brightness` accessor route.
      *(The retirement doc says "32"; the actual enumeration is 36.)*
- [x] Engine even-firing offsets, per-type exhaust/intake resonances, event durations.
- [x] The auto-BOV trigger triple (`prev_load > 0.5`, `load < 0.2`, `spool > 5000`).
- [x] The differential's **ring/pinion asymmetry** — mesh frequency uses *pinion*
      teeth. A swap is completely silent today.
- [x] The equal-power pan law's **left=cos / right=sin** assignment and its
      −3 dB centre attenuation. A left/right swap passes the suite today.
- [x] `ChainDrive` silence below 1 Hz engagement **and its non-advancing RNG**.
- [x] `process_block_stereo` leaves the longer buffer's tail **untouched**.
- [x] `Transmission::new` accepts an **empty** ratios vec; `current_ratio()` → 1.0.

### New test machinery
- [x] **Golden-file regression** — checksum every synth's output for fixed
      parameters. This is the gate that makes the semver rule above enforceable:
      after this lands, an unintended audio change fails CI instead of shipping.
      **Highest-value item in the arc.**
- [x] **Spectral validation** — assert the peak bin matches the expected
      firing / mesh / blade-pass frequency. Cheaper than the old roadmap assumed:
      naad already ships `fft_magnitudes` (`lib/naad.cyr:499`), so this is a
      direct call, not an FFT project. Input length must be a power of two, so
      use 4096/8192 rather than the current 4410/2205 block sizes.
- [x] **Fuzz harness** — `cyrius fuzz` runs `fuzz/*.fcyr`; ghurni has none, and
      the command currently **exits 0 while finding nothing**, which is a vacuous
      gate. Drive the public boundary with `INT64_MIN`, NaN, ±inf, subnormals,
      `f64::MAX`, non-power-of-two lengths. Sweep enum ids at valid parameters
      and values at a valid id — correlate them and the harness proves nothing
      (naad's lesson).
- [x] **Rest-state invariant** — landed as a CHARACTERISATION in
      `oracle_pins.tcyr`: chain_drive and forced_induction are asserted exactly
      silent at rest (already correct), while motor's stopped-equals-running and
      belt_drive's louder-stopped are pinned as *known defects* so Arc 1's fix
      cannot land silently. Those assertions must be rewritten as the invariant
      when 2.1.0 introduces the loudness law.

### Release hygiene — blocks a clean 2.0.3
- [ ] **12 compiled ELF binaries (11 MB) are tracked in git** and ship inside
      every release tarball, including 5 stale `build/err_*` from an old naming
      scheme. `.gitignore` covers `/build/` as of 2.0.3, but untracking is a git
      operation the maintainer runs: **`git rm -r --cached build/`**. Carried
      into 2.0.4.
- [x] **`.gitignore` is still the Rust crate's** (`/target`, `Cargo.lock`,
      `*.rs.bk`, `*.pdb`) and has no `/build/` — which is how the binaries got in.
- [x] **`scripts/version-bump.sh` is Rust-era**: it writes `VERSION`, then seds a
      root `Cargo.toml` that does not exist and runs `cargo generate-lockfile`.
      It half-bumps the repo and fails. It would corrupt the 2.0.3 release.
- [x] **CI runs no quality gates.** `.github/workflows/ci.yml` is
      deps → build → test → build-examples. Add `cyrius audit` (exits 0 today),
      `cyrius deny` (exits 0), `cyrius distlib --check` (nothing currently
      prevents `dist/ghurni.cyr` drifting from `src/` — and `dist/` is what
      consumers compile), and **actually run** the examples: CI builds all five
      and never executes them, so a compiling-but-crashing example passes.

### Documentation defects (several ship inside `dist/ghurni.cyr`)
- [x] **README advertises a wastegate.** `grep -rn wastegate src/ dist/` returns
      nothing. Either implement it (Arc 2) or stop claiming it.
- [x] `src/dsp.cyr:12` cites `tests/dsp.tcyr`, **which does not exist** — and the
      comment ships to consumers at `dist/ghurni.cyr:196`.
- [x] ADR-004 still carries the serde justification `state.md` already corrected
      as false (`MixerChannel` serialized `name`/`gain`/`pan`/`muted`).
- [x] Stale measured numbers across `state.md`, `overview.md`, `README.md`.
      Truth as shipped in 2.0.3: **111/167 functions referenced (66%)**,
      **10 suites / 450 assertions**, 14 benchmarks, 1631 fuzz checks,
      `dist/ghurni.cyr` 3,242 lines.
- [x] The retirement checklist's own `rust-old` reference counts are wrong.
- [x] No `2.0.0` CHANGELOG entry — a re-cut of that tag still ships an empty
      release body. Release machinery, not a nicety.
- [x] Record the undocumented contracts: `smooth_time_s` is **tau** (63%), not a
      settling time; `MechanicalEvent` cylinder is 0-indexed and `GearShift.from`
      0 = neutral; the cross-plane V8 offsets exist because uneven intervals make
      the burble; the supercharger preset's `spool_inertia` argument is inert.

### Small correctness items found by the sweep
- [x] `ghurni_dcblocker_new` is the **only** public constructor that does not
      validate its sample rate.
- [x] ~~`ghurni_gear_new` clamps teeth only from below~~ — **investigated, not a
      defect.** The oracle does the same (`teeth.max(4)`, `rust-old/src/gear.rs:72`,
      documented "Number of teeth (4+)"), and a huge tooth count is safe because
      mesh frequency is Nyquist-clamped before it reaches the oscillator (verified
      by execution at 100000 teeth). Adding an upper bound would DIVERGE from the
      oracle, not fix anything. The lower clamp and the huge-teeth path are now
      pinned in `oracle_pins.tcyr` instead.
- [x] `GhSmoothedParam`'s derived serde roundtrip is untested while both sibling
      POD structs are.
- [x] Two setters silently no-op and return success — the exact shape 2.0.2 hunted.
- [x] `f64_clamp` propagates NaN where `f64_max`/`f64_min` absorb it. Several NaN
      entry points render 100% non-finite audio, and **SECURITY.md claims
      otherwise**. Fix the doc here; fix the behaviour in Arc 1 (it is an audio
      change at the boundary).
- [x] **Benchmark gap** — 6 of the oracle's 10, plus its 64→4096 block-size sweep.
      Missing: motor, turbine, clock, transmission, turbocharger.

---

## Arc 0b — `2.0.4` "Delete the Oracle"  ✅ SHIPPED

> Released 2026-08-28. 33 files / 3,934 lines removed; 118 references across 39
> files reframed as historical citations. All 450 assertions green and **every
> golden unchanged**, proving the removal was purely documentary. The oracle is
> recoverable at tag `2.0.3`.

**Theme: retire `rust-old/`.** Its own release so the deletion is a revertable
commit rather than noise inside a feature arc.

- [x] Everything in [`rust-old-retirement.md`](rust-old-retirement.md) is checked. *(zero open items as of 2.0.3)*
- [x] `git rm -r rust-old/`; update the 61 references across 34 files.
- [x] **Rewrite CLAUDE.md's correctness bar.** It currently reads "matches what
      the Rust naad-backend path did". That stops being checkable the moment the
      directory goes; it becomes "matches the frozen goldens in `tests/`" — which
      is only true if Arc 0 actually landed them. **This is why Arc 0 comes first.**
- [x] Note in ADR-004 which tag the oracle is recoverable from.

> **Sequencing constraint — now satisfied.** Both audio arcs below were blocked
> on this one: while `rust-old/` existed the stated correctness bar was "match
> the oracle", so a deliberate divergence had to argue against the project's own
> rule every time. The bar is now the goldens, which is the bar acoustic work
> actually wants. **Arc 1 is unblocked.**

---

## Arc 1 — `2.1.0` "Rest State"  ✅ SHIPPED

> Released 2026-08-28 behind [ADR-005](../architecture/adr-005-rpm-loudness-law.md).
> One loudness law applied uniformly; the combustion pulse integrated across each
> sample (killing a 70% amplitude notch at resonant RPMs); NaN closed at all 18
> parameter clamps; seed resolution unified. Six of eleven goldens moved —
> gear/transmission/belt were **bit-identical**, which is the ADR's central claim
> holding. 509 assertions.

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

- [x] **ADR-005: acoustic divergence from the oracle.** The umbrella for this arc.
- [x] An RPM-dependent loudness law for the seven synths that lack one.
- [x] **Resonant-RPM combustion amplitude.** At RPMs where samples-per-cycle is an
      integer (`5292000/rpm ∈ ℤ` at 44.1 kHz) the engine lands off-peak every
      cycle and the thump loses up to two thirds of its amplitude; the f32
      oracle's coarser rounding accidentally snapped the lattice onto the cycle.
      Characterised in [state.md](state.md). Same ADR, same release.
- [x] **NaN at the parameter boundary.** `f64_clamp` propagates it; guarding is
      itself an audio change, so it belongs here.
- [x] Seed aliasing the port introduced in `belt_drive` / `turbine` /
      `forced_induction` (behavioural integer seeds that lose parameter
      resolution) — decide the convention once and apply it uniformly.

**Patch line `2.1.x`** — regressions the new goldens catch; per-synth loudness-law
tuning that consumers report as too aggressive or too subtle.

---

## Arc 2 — `2.2.0` "Depth"  ✅ SHIPPED

> Released 2026-08-28 behind [ADR-006](../architecture/adr-006-acoustic-depth.md).
> Five structural gaps closed; five goldens moved and two were **fixed** — the
> clock golden never covered a tock, and HYBRID had no golden at all.

**Theme: the ten synths get acoustically richer.** Breadth stays frozen; this arc
is about the machines already modelled sounding more like themselves.

- [x] **Every gear-mesh whine in the library was a single pure sine.** gear,
      transmission and differential now use a 3-partial additive mesh, and gear's
      upper partials scale with the existing per-material `brightness`, so
      material finally changes timbre (measured h2/h1: steel 0.546 → nylon 0.124).
- [x] **`GH_ENGINE_HYBRID` was a gasoline engine.** It now renders an electric
      drive: no combustion at all, a 2-partial EM whine at `(rpm/60) × 8` poles,
      and the exhaust/intake beds pulled to 0.25.
- [x] **Turbo and supercharger were spectrally identical.** Whine now sits at
      blade-pass (10) and lobe-pass (3) respectively — measured ratio 3.34.
- [x] **Chain link count was audibly inert.** A once-per-lap modulation at
      `engage_freq / links` makes it audible.
- [x] Clock tick and tock were the same sound. Odd beats are now softer and
      shorter-decaying, an alternation rather than a decay.

**Patch line `2.2.x`** — per-synth spectral tuning; golden refreshes.

---

## Arc 2b — `2.3.0` "Depth II"  ✅ SHIPPED

> Released 2026-08-28 behind [ADR-007](../architecture/adr-007-load-tilt-and-diesel.md).
> Load now tilts the spectrum instead of moving a fader; diesel got its clatter;
> the differential's tooth counts became individually audible. Six goldens moved;
> the six synths that take no load are bit-identical to 2.2.0.

- [x] **Load produced no spectral tilt** — and on the engine it went the *wrong
      way* (centroid fell 8882 → 8690 Hz as load rose). A shared
      `tilt(load, k) = 1 + k·load` now scales each synth's broadband term and not
      its tonal one. Measured: motor 2078 → 2253 Hz, engine 8892 → 9031 Hz,
      both monotonic. `tilt(0) = 1.0` exactly, so unloaded audio is unchanged.
- [x] **Diesel was a retuned Gasoline** — three constants apart. It now has
      injection clatter: a sharper impulse behind the main pulse, decaying 8×
      faster. Measured centroid 9747 Hz vs gasoline's 8531 Hz.
- [x] **Differential ring-revolution modulation.** 40/10 and 80/20 axles — same
      4.0 ratio, different hardware — no longer render identically. Also makes
      `sample_position` live in that module, where it had been vestigial.
- [ ] ~~Turbine rotor slap and multi-spool~~ → **moved to Arc 4 (breadth).**
      Blade slap is an impulsive phenomenon with its own physics and multi-spool
      is a jet architecture; both are closer to new machines than to a deeper
      turbine. That was the open question in ADR-006 and this is the answer.
- [ ] ~~Source-body impulse responses~~ → **own ADR, not yet scheduled.**
      ⚠ ADR-006's cost estimate was **wrong**: naad already ships a full
      convolution engine (`naad_convolution_from_ir` / `_process_block`), so the
      path exists. The real obstacle is that ghurni *already* models exhaust
      resonance with a bandpass, so convolution would be a second, redundant
      mechanism. The genuine question is whether to **replace** the bandpass —
      an architectural change deserving its own ADR.

**Patch line `2.3.x`** — load-tilt strength tuning; golden refreshes.

---

## Arc 3 — `2.4.0` "Events & Control"

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
- [ ] **Per-component stems** — every synth sums exhaust + intake + mechanical
      into one mono buffer; rendering them separately is a real API gap. Moved
      here from Arc 2 in 2.2.0: it is a control-surface change, not a timbral one.
- [ ] **Differential drive/coast asymmetry** — needs a torque-direction input the
      API does not have. Moved here from Arc 2 for the same reason.

**Patch line `2.4.x`** — event-timing tuning; consumer-reported control gaps.

---

## Arc 4 — `2.5.0` "New Mechanisms"

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

**Patch line `2.5.x`** — per-synth tuning for the new mechanisms.

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
