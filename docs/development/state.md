# ghurni — Live Port State

> Volatile state (version, module sizes, coverage). Durable rules live in
> [`CLAUDE.md`](../../CLAUDE.md). Refreshed at each milestone.

## Version

**2.0.1** — full-parity Cyrius port, on the refreshed dependency set
(cyrius 6.5.35 · naad 2.2.1 · hisab 2.11.2 · goonj 2.0.4 · sakshi 2.4.11).
(1.0.0 was the final Rust crate, preserved at `rust-old/`.)

## Port status: COMPLETE

All 20 Rust source modules ported; all 44 Rust integration tests reproduced as
behavioural-parity `.tcyr` suites; smoke binary + benchmarks green.

## Modules (`src/*.cyr`, 2,967 lines)

| Layer | Modules |
|-------|---------|
| L0 foundations | `error` `logging` `dsp` `smooth` `event` `traits` |
| L1 synths | `engine` `gear` `motor` `turbine` `clock` `transmission` `differential` `forced_induction` `belt_drive` `chain_drive` |
| L2 composites | `mixer` `presets` |
| entry | `main` (smoke) |

Bundle: `dist/ghurni.cyr` (2,904 lines, `cyrius distlib`).

## Tests (`tests/*.tcyr`, 6 suites, 135 assertions, 0 failures)

| Suite | Covers |
|-------|--------|
| `foundations` | error codes/helpers, validation, DcBlocker, SmoothedParam, MechanicalEvent + serde |
| `engine` | all engine integration tests + firing frequency + param sweep + EngineType serde |
| `synths` | gear/motor/turbine/clock/transmission/differential/forced_induction/belt/chain + gear sweep |
| `serde` | EngineType/GearMaterial/MotorType/ClockType/InductionType + GhurniError name mapping |
| `mixer` | mono/stereo/mute, trait dispatch, process-block continuity, presets |
| `smoke_all` | full-unit integration: every synth + mixer + preset produce finite audio |

`cyrius bench` (`benches/ghurni.bcyr`): DcBlocker ~20 ns/sample, smoother ~15 ns/sample, plus full engine/gear synthesis paths.

## Deliberate divergences from `rust-old/`

- **naad-only backend.** The Rust `#[cfg(not(feature="naad-backend"))]` fallback
  (`rng.rs`, `math.rs`, per-synth fallback loops) is not ported — naad's
  NoiseGenerator owns the randomness. See ADR-004.
- **f32 → f64 throughout** (naad/hisab are f64-only). Test tolerances loosened
  where f32 bit-exactness is not meaningful; audio parity is behavioural
  (finite / has-energy / energy-ordering), matching what the Rust tests asserted.
- **Integer error codes** replace the `GhurniError` String-payload enum;
  diagnostic text via `ghurni_err_name`.
- **Tag dispatch** replaces `Box<dyn Synthesizer>` trait objects (ADR-004).
- **Serde**: enums roundtrip via name↔code helpers; POD structs (DcBlocker,
  SmoothedParam, Event) via `#derive(Serialize)`; synth structs holding opaque
  naad pointers drop deep serialization (nothing meaningful survived the Rust
  `#[serde(skip)]` on the backend fields either).
- **RNG seeds** derived from parameters are behavioural, not bit-identical to the
  Rust `to_bits()` seeds (tests don't depend on exact sample values).

## Examples (`docs/examples/*.cyr`, 5 runnable programs)

`simple_engine`, `vehicle_scene`, `mixer_demo`, `error_handling`, `logging` —
each ports the matching `rust-old/examples/*.rs`, includes the dep bundles +
`dist/ghurni.cyr`, and builds/runs standalone:

```sh
cyrius build docs/examples/simple_engine.cyr build/ex_simple_engine && ./build/ex_simple_engine
```

## CI

`.github/workflows/{ci,release}.yml` use the cyrius toolchain (install from the
`cyrius.cyml [package].cyrius` pin → `cyrius deps` → `cyrius build` → `cyrius
test` → build examples; release verifies VERSION/tag consistency + ships a
source tarball + SHA256SUMS). Mirrors the prani / naad sibling CI.

## Known follow-ups

- None outstanding for parity.
- `CHANGELOG.md` still has **no `2.0.0` entry** for the port itself. 2.0.1 now
  has one, so `release.yml`'s changelog extraction populates for this tag, but
  a re-cut of the 2.0.0 tag would still produce an empty release body.

### Rust-era documentation staleness (pre-dates 2.0.1; surfaced auditing it)

None of these affect the build — they are docs that still describe the Rust
crate rather than the Cyrius port, and each contradicts `CLAUDE.md`:

- `docs/architecture/overview.md` — the module tree lists 20 `.rs` files under a
  `ghurni/src/` header, including `math.rs` / `rng.rs`, which ADR-004 says were
  deliberately **not** ported. Actual `src/` holds 19 `.cyr` modules.
- `docs/architecture/adr-001-naad-backend.md` — status still `Accepted`, and it
  specifies naad as *optional* behind a `naad-backend` flag with a libm/PCG32
  fallback and "dual implementation in every synthesizer". `CLAUDE.md` states
  naad is the ONLY backend. Should be marked superseded by ADR-004.
- `SECURITY.md` — the dependency table is `rust-old/Cargo.toml` verbatim
  (serde / thiserror / tracing / libm, naad "Optional"). None of those exist in
  the port; the real deps (naad, hisab, goonj, sakshi) are absent from it.
- `docs/guides/testing.md` — documents `cargo test` / `cargo bench` and a
  "Fallback path (no naad)" that does not exist. Should be `cyrius test/bench`.
- `docs/development/roadmap.md` — plans `cargo-fuzz` targets; the toolchain
  ships `cyrius fuzz` (`.fcyr`).
- CI runs **none** of the quality gates cyrius 6.5.35 provides
  (`cyrius audit` / `deny` / `fuzz` / `lint` / `fmt`) — only deps/build/test +
  examples. naad's CI enforces all three of the first group.

### Float-literal hardening

`GHURNI_DB_SCALE` is stored as an IEEE-754 bit pattern, not a decimal literal —
cyrius <= 6.5.27 miscompiled decimal literals past ~9 fractional digits (see the
2.0.1 CHANGELOG entry). It was the only such literal in the link unit. Any new
high-precision float constant should follow the same hex idiom, because
`dist/ghurni.cyr` is compiled by the **consumer's** toolchain, not ghurni's.
