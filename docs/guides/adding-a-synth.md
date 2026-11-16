# Adding a synth

A new synth touches more of the tree than it looks, because Cyrius has no trait
objects (ADR-004) and each test suite lists its own includes. Miss one and the synth
usually still compiles, then fails somewhere quieter.

## The module

1. **`src/<synth>.cyr`.** A `Gh<Name>` struct with `#derive(accessors)`. Document the
   fields in a comment above it, because an inline `#` inside a struct body breaks the
   parser. Provide `ghurni_<synth>_new`, `_set_rpm`, `_process_block`, `_synthesize`,
   `_rpm` and `_sample_rate`.
2. **Constructor.** Call `ghurni_validate_sample_rate` first and return its code.
   Range-check every enum id on both sides, because `i64` cannot make a bad value
   unrepresentable the way Rust did.
3. **Seeds.** Every seeded parameter must reach the seed with enough resolution to
   matter. A bare `f64_to` of a sub-unity value truncates to a constant, which has been
   fixed twice already.
4. **Setters.** Use `ghurni_finite_clamp`, and check that its `lo` really is the off
   end of that range. For belt tension and pan it is not.
5. **`_process_block`.** Open with the `out < GHURNI_MIN_VALID_PTR` check. Allocate
   nothing: the allocator never frees, so a per-block allocation is a permanent leak.
   End with `ghurni_dcblocker_process_buffer`.
6. **`_synthesize`.** Call `ghurni_sample_count` before touching any state, then
   `ghurni_zeros(n)` for the output.
7. **Loudness.** Apply `ghurni_rpm_gain` unless an ADR says otherwise. Take it from a
   smoothed rpm if the synth has a smoother, and ramp any term a setter changes, or
   it steps at the block edge (ADR-012).
8. **Phase.** Never compute a periodic phase as absolute position x the current rate.
   Anchor it, as `gear.cyr` does for a block-constant rate and `differential.cyr` for
   a per-sample one: keep the phase, the position and the rate at an anchor, and
   re-anchor whenever the rate changes (ADR-012). `tests/continuity.tcyr` checks that
   no synth's audio depends on its `sample_position`.
9. **Low rates.** Build additive tones with `ghurni_additive_new`, and every filter
   or oscillator centre through `ghurni_below_nyquist`. naad zeroes a partial at or
   above Nyquist for good, and refuses a filter there.

## The wiring

10. **`src/traits.cyr`.** Add a `GH_KIND_*`, and an arm in both `ghurni_kind_name` and
    `ghurni_kind_from_name`.
11. **`src/mixer.cyr`.** Add one `ghurni_kind_row` to `ghurni_kind_method`, raise
    `GHURNI_KIND_COUNT`, and grow `GHURNI_KIND_TABLE` by 32 bytes. The four dispatchers
    and `ghurni_mixer_add_channel` all read the table, so there is no other range check
    to move.
12. **`cyrius.cyml [lib].modules`.** Add the file in dependency order, after L0 and
    before `mixer`. Then run `cyrius distlib`, because `dist/ghurni.cyr` is what
    consumers compile.
13. **`src/presets.cyr`.** Add a preset if the synth has an obvious stock form.

## The gate

14. **The include lists.** Seven suites and the fuzz harness name their modules
    explicitly: `tests/goldens`, `hardening`, `mixer`, `oracle_pins`, `smoke_all`,
    `spectral`, `synths`, and the newer suites that copy that list.
15. **`tests/goldens.tcyr`.** One golden per type, and one per load or event path.
    Default configurations alone leave constants unpinned: doubling a level must fail.
16. **`tests/spectral.tcyr`.** Assert the audio sits where the RPM physics says.
17. **`tests/boundaries.tcyr`.** The constructor's rate code, the over-long `synthesize`,
    and both ends of each clamp, read back.
18. **`tests/recovery.tcyr`.** A NaN `set_rpm` must not silence the synth for life.
19. **`tests/mixer_contract.tcyr`.** Its dispatch arms, compared bit for bit against the
    synth rendered alone.
20. **`fuzz/ghurni.fcyr`.** Add it to the constructor and render sweeps.
21. **`benches/ghurni.bcyr`.** Add a one-shot case.

## The record

22. **`README.md`** feature list, and the module table in
    `docs/architecture/overview.md` and `docs/development/state.md`.
23. **ADR**, if the synth changes the scope in ADR-002.

A new public function belongs in a minor, never a patch.
