# ghurni Roadmap — 2.x

Organised as **arcs**: each minor (`2.N.0`) is one themed body of work, and each
carries a **patch line** (`2.N.x`) for the follow-up that predictably lands after it.
Nothing here is scheduled by date; the order is by dependency. Shipped work is not
listed here: it lives in [`CHANGELOG.md`](../../CHANGELOG.md) and the
[ADRs](../architecture/).

**Semver contract for this project**, the spine of the whole plan:

| | Allowed | Forbidden |
|---|---|---|
| `2.N.x` **patch** | bug fixes, tests, docs, CI, release hygiene, internal perf proved bit-identical | any new public `ghurni_*` function; **any change to rendered audio on valid input** |
| `2.N.0` **minor** | new synths, new public functions, new parameters, deliberate audio changes | breaking an existing signature or the meaning of an argument |
| `3.0.0` **major** | the backlog at the end of this file | — |

> A deliberate audio change also needs an **ADR**. The gate is mechanical: if a
> golden checksum in `tests/goldens.tcyr` moves, it is not a patch. Goldens truncate
> to 1e-6, so a patch also states its raw-bit difference against the last release
> ([toolchain guide](../guides/toolchain-bump.md) §3).

---

## Now: `2.8.0` "Phase Continuity"

**The defects 2.7.2 could not fix in a patch, because fixing them changes audio.**
Phase is anchored to the speed in force instead of derived from absolute position, a
clock tick has a duration, a misfire skips a whole firing, the hybrid's starter has its
own synth, block-stepped terms ramp, every synth builds and keeps its partials at any
valid rate, and a negative ratio is a reverse gear. A speed held from the first sample
renders 2.7.2's bits; audio differs only where a rate changed.
[ADR-012](../architecture/adr-012-phase-continuity.md) records it, and the full record
is in the [CHANGELOG](../../CHANGELOG.md). Also moves to naad 2.3.5, hisab 3.5.0 and
goonj 2.3.2, with sakshi now a stdlib leaf.

**Patch line `2.8.x`**: anything the new goldens and A/B renders turn up that does not
move audio on valid input. A change that does move it goes to a minor below.

---

## `2.9.0` "Intake & Stems"

**Theme: finish the radiation model, then hand out the lanes.** Carried from the old
Arc 3d and the 2.6.x patch line.

- [ ] **Intake duct**: Helmholtz plenum plus runner, to the same bar the exhaust
      cleared. An airbox is a duct too; that is what airbox resonance is.
- [ ] **Per-component stems**, keyed on radiation path, once there are two real lanes.
      ADR-009's taxonomy is the starting point.
- [ ] **Duct tuning per engine type**, and golden refreshes if it moves them.
      `GHURNI_DUCT_RAD_HZ = 520` is an argued value, not a measured one; revisit with
      a second opinion rather than a second measurement.
- [ ] **Two-stroke expansion chamber**. A divergent-then-convergent cone pair whose
      purpose is a positive reflection followed by a negative one. The quarter-wave
      model is the wrong device for it, and the two-stroke misses the ADR-009 share
      bar at 21.9%.
- [ ] Let the duct energy split (`GHURNI_DUCT_DRIVE` / `_STRUCT`) survive a release in
      consumers' hands before an occlusion curve is built on it.
- [ ] **STARTUP fades a running engine's combustion out instead of cutting it.** Found by
      2.8.0's A/B review: the gate is `1 - ufrac`, about 0 on the first sample, so STARTUP
      on a running hybrid stops its electric whine instantly (a 0.097 step at the shared
      gain). The shared starter synth masked it until 2.8.0 gave the whine its own.

**Patch line `2.9.x`**: stem-level tuning.

---

## `2.10.0` "Throughput"

**Theme: take what naad 2.3.x offers, once consumers can follow.** Every item here
raises the consumer floor to naad 2.3.3 and cyrius 6.7.18. **dhvani reached both in
2.3.1** (checked at 2.8.0), so this arc is unblocked. jalwa still reaches ghurni through
dhvani 2.2.4 on cyrius 6.6.3, so check it before raising the floor.

- [ ] **naad block forms**: `noise_fill_buffer`, `filter_biquad_process_buffer`,
      `additive_fill_buffer` and `osc_fill_buffer`, wherever the noise draw order and
      per-sample parameters allow a block. naad holds each to its per-sample path
      bit for bit, so this is a speed change only, and must be proved so here too.
- [ ] **Presized output buffers.** `synthesize` grows its result one `vec_push` at a
      time, so 2,205 samples allocate 65 KB for 17 KB of audio. `naad_vec_zeros`
      (naad 2.3.2) is the presized vec this was blocked on, and `ghurni_zeros` (2.8.0)
      is the one place to change.
- [ ] **Settled smoothers cost a call per sample.** 2.8.0 added two per-sample
      smoothers to the transmission (+13% on its one-shot) and two to the differential.
      A settled smoother returns its target; skipping the arithmetic then is
      bit-identical except for a target of -0.0, which no ghurni setter produces.
- [ ] **Envelope cost.** `naad_db_to_amplitude` is `f64_pow` per sample in clock, gear
      and engine, and `f64_pow` doubled at cyrius 6.6.10. Any replacement must hold
      the goldens.

---

## `2.11.0` "Vehicle Contact"

**Theme: the largest single source gap.** Deliberately after the loudness law and the
fuzz harness, so a new synth does not inherit old problems on day one.

- [ ] **Brake system**: pad contact, disc scrape, ABS pulsing. The old roadmap
      under-scoped this to squeal only.
- [ ] **Tyre and wheel contact**: the other half of the contact patch. Check the
      ADR-002 line against garjan before starting, since surface interaction may be
      shared.

**Patch line `2.11.x`**: per-synth tuning.

---

## `2.12.0` "Small Mechanisms"

Ranked by reuse.

- [ ] **Ratchet, pawl and detent click train**: highest reuse per line in the list.
      Feeds handbrakes, winches, gear selectors and clockwork.
- [ ] **Pneumatic release and valve exhaust**: `forced_induction`'s blow-off valve is
      already the primitive.
- [ ] **Hydraulic actuator**: extend, stall and relief. The old roadmap picked the
      pump; consumers reach for the actuator.
- [ ] **Stepper motor**: a fifth motor family, categorically unlike the four that ship.

---

## `2.13.0` "Machine Families"

- [ ] **Bearings, as a wear and defect modifier** on an existing rotating source:
      BPFO, BPFI, BSF and FTF sidebands. Not a standalone synth.
- [ ] **Compressors**: reciprocating chug, rotary and screw whine. Layering nearly
      solves this already.
- [ ] **Turbine rotor slap and multi-spool**: helicopter blade slap is impulsive and
      multi-spool is a jet architecture; both are new machines, not a deeper turbine.
- [ ] **Weapon actions and rotary cannon**: a large genre with no coverage. The
      impact is garjan's; the action is ours.

---

## `3.0.0` backlog

Each of these changes an existing signature or the meaning of an argument, so none
can go in a minor.

- **GEAR_SHIFT numbering.** `from` treats 0 as neutral, but `to` is a 0-based index
  into the ratios, so the documented `gear_shift(1, 2)` selects third gear, top gear
  is unreachable by event, and "shift to neutral" selects first. Fixing `to` changes
  what every existing call means.
- **Accessor visibility.** `#derive(accessors)` setters are public API (ADR-009) and
  bypass every validation a real setter does: `GhEngine_set_sample_rate(e, 0)` renders
  NaN for good. Hiding them needs language support, and it breaks callers.
- **Checked handles.** A synth method given a failed constructor's code faults at the
  null page. Checking `self` everywhere is the ADR-004 caller contract today, and
  typed handles would replace it.
- **Rust's serde wire form.** The JSON codecs write `{"kind":5,...}` with raw f64 bit
  patterns, not Rust's externally tagged form. Matching Rust changes the format.
- **Mixer configuration serde.** Rust serialized a channel's name, gain, pan and mute;
  the port dropped it, which ADR-004 records as a scope cut.
- **Leftover public names**: the ten `*_process_block_naad` halves of the Rust feature
  split, and `ghurni_map_naad_error`, which ignores its argument.

---

## Upstream

- **cyrius `#derive(Serialize)` reads past the NUL** on an unterminated key: its key
  scan is `while (load8(json + p) != 34)`. ghurni stopped using the derive in 2.7.2.
  Filed with cyrius as
  `docs/development/issues/2026-11-07-derive-serialize-key-scan-reads-past-nul.md`.
  ghurni can go back to the derive once that ships, if the strict reader is kept for
  its validation.

## Dropped

Removed with reasons, so they are not re-proposed:

- **Doppler wrapper**: ADR-002 assigns it to goonj.
- **Parameter automation curves**: ADR-002 puts RTPC mapping in dhvani and kiran.
- **Per-sample accessor hoisting as its own goal**: measured in 2.0.2 on belt_drive
  as no win. 2.7.2 hoisted where it was measured to pay, in the engine.
- **"Raise reference coverage" as a goal**: kept as a CI ratchet
  (`cyrius coverage --min`), not a target. A green suite can pin nothing.
- **`cyrius vet` UNTRUST on `dist/ghurni.cyr`**: structural, since it is the project's
  own generated bundle, and the manifest has no trust list.
- **Allocation-failure handling**: decided in
  [ADR-011](../architecture/adr-011-allocation-failure.md) as fail-stop, on measurement.

## Standing notes

- **Gear, motor and turbine have no smoother**, by the oracle's design, so their pitch
  and loudness both step at block edges. Giving them one changes their character, not
  just their continuity, so ADR-012 left them alone. It is open for an ADR in any minor.
- **Three corrections to remember before the stems arc**, each a claim this project
  made and later disproved: ADR-009's acceptance criterion is passed by a wire, so it
  must never be cited as evidence a radiation path exists; ADR-009's tripwire never
  fired, because a waveguide's throughput is level-invariant under retuning; and
  ADR-006 called stems "an API change, not a timbral one", which is exactly backwards.
- `sample_position` is an `i64` sample counter, with no practical overflow.
- Deterministic output depends on naad's `NoiseGenerator` seeding. Every seeded
  parameter must reach the seed with enough resolution to matter, and seeds that add
  terms can collide, as the integration guide records.
- `dist/ghurni.cyr` is compiled by the **consumer's** toolchain, so anything sensitive
  to compiler behaviour, long float literals in particular, must be written in a form
  an older toolchain cannot get wrong. Check the bundle on the oldest consumer pin
  before every release.
- Adding a synth: [docs/guides/adding-a-synth.md](../guides/adding-a-synth.md).
  Moving the toolchain pin: [docs/guides/toolchain-bump.md](../guides/toolchain-bump.md).
