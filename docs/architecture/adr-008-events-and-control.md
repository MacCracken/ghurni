# ADR-008: The Control Surface Stops Lying

**Status**: Accepted
**Date**: 2026-08-28
**Release**: 2.4.0 ("Events & Control")

## Context

The previous three releases changed how ghurni *sounds*. This one is about what
it lets a consumer *do* — and specifically about API that was advertised and did
nothing.

| Verified | Finding |
|---|---|
| `src/event.cyr` declares 8 `GH_EVENT_*` kinds; `src/engine.cyr` handles 3 | **Five events were accepted and silently discarded**: `STALL`, `REV_LIMITER_HIT`, `GEAR_SHIFT`, `STARTUP`, `SHUTDOWN`. Constructors existed for every one of them (`ghurni_event_stall`, …), so a consumer could build and fire an event that provably did nothing. |
| Only `engine` has a `trigger_event` | `GEAR_SHIFT` had nowhere to land even in principle — a shift is not the engine's business. |
| `grep 'fn ghurni_.*_set_load'` | Only engine, motor and forced_induction took **load**. gear, transmission and differential — the three synths whose entire purpose is transmitting torque — took none. |
| `src/clock.cyr` | A clock's beat was fixed at construction. `set_rpm` is a deliberate no-op and nothing else touched `tick_rate`, so **a clock had no speed control at all**. |
| `grep 'fn ghurni_.*_from_name'` | `GH_KIND_*` had **neither** `name` nor `from_name`; `MechanicalEvent` had only the forward half. CLAUDE.md requires the pair on every public enum — 2.0.2 fixed `GhurniError` for exactly this reason. |

The oracle ignored those five events too (`_ => {}` at `engine.rs:202`), so this
was an inherited product gap rather than a port defect. But the oracle retired in
2.0.4, and "the Rust did it too" stopped being an answer.

## Decision

### 1. The four engine sequences

Each is a sample counter, following the `backfire_remaining` / `decel_pop_remaining`
pattern already in the module. **All four are zero unless an event is running**,
and every term collapses to a no-op then — which is why this release adds four
event sequences and moves *no goldens*.

| event | behaviour |
|---|---|
| `STALL` | RPM target → 0, and combustion sputters out **gated by noise** over 0.6 s. The irregularity is the point: a smooth fade reads as a fade-out, not a failure. |
| `SHUTDOWN` | The same collapse, smooth and over 1.6 s, so it reads as deliberate. |
| `STARTUP` | RPM target → the engine's 100 rpm idle floor, with a starter whir over the top for 1.1 s while combustion ramps **in**. |
| `REV_LIMITER_HIT` | Combustion **chopped** on and off at 18 Hz for 0.35 s. A limiter does not mute an engine, it bounces it. |

`STALL`, `SHUTDOWN` and `STARTUP` drive the collapse or climb through the
**existing `smooth_rpm` smoother** rather than a bespoke envelope — that is
`set_rpm`'s own path, so the ramp is already click-free and already the right
shape.

The starter whir reuses `whine_synth`, which until now was built only for
HYBRID. It is built for every type as of 2.4.0 and stays silent unless the engine
is a hybrid or starting.

### 2. `GEAR_SHIFT` goes to the transmission

`ghurni_transmission_trigger_event` routes it to `shift_to(arg1)` — the payload
is `(from, to)` with 0 meaning neutral on the `from` side. An out-of-range target
is ignored rather than rejected, matching `shift_to`'s own contract.

### 3. Load on gear, transmission and differential

`set_load` / `load` on all three, clamped 0..1 through `ghurni_finite_clamp`.
Load raises the tonal amplitude (`1 + 0.5·load`) and applies the ADR-007
broadband tilt.

**Both terms are exactly 1.0 at load 0, which is the default**, so an unloaded
synth renders bit-identically to 2.3.0. Same bounded-review property ADR-005 used
for the reference RPM and ADR-007 for the tilt — and the reason the goldens hold.

### 4. Differential drive/coast

`set_coasting(0|1)`. A hypoid gearset meshes on the opposite tooth flank on
overrun, and the coast flank is cut differently from the drive flank — which is
why a worn diff often whines in one direction only. Coasting drops the whine to
0.55 and raises the broadband share by 1.6.

This needed a torque **direction**, which the API could not express: `load` is
0..1 and cannot be negative. A separate flag keeps `load`'s meaning intact rather
than overloading it with a sign.

### 5. Clock speed control

`ghurni_clock_set_tick_rate` / `ghurni_clock_tick_rate`, clamped 0.1..50 Hz —
below that the beat is inaudible, above it the ticks merge into a tone.
`set_rpm` stays a deliberate no-op: a clock's beat is set by its escapement, not
by a driven shaft.

### 6. `name` / `from_name` completed

`ghurni_kind_name` / `_from_name` (new, `traits.cyr` had no functions at all) and
`ghurni_event_from_name`. Both return `-1` for an unknown name, and that sentinel
is safe to feed straight into the API — `ghurni_mixer_add_channel` range-checks
its tag, which the tests assert.

## Consequences

- **No golden moves.** Everything here is additive or defaults to a no-op. That
  was a design constraint, not a happy accident, and `tests/goldens.tcyr` passing
  unchanged is the evidence.
- 600 assertions across 10 suites, up from 556.
- New public symbols: `ghurni_gear_set_load` / `_load`,
  `ghurni_transmission_set_load` / `_load` / `_trigger_event`,
  `ghurni_differential_set_load` / `_load` / `_set_coasting` / `_coasting`,
  `ghurni_clock_set_tick_rate` / `_tick_rate`, `ghurni_kind_name` / `_from_name`,
  `ghurni_event_from_name`. Additive only — hence MINOR.

## Corrected: two roadmap items were already stale

Both were carried forward from a survey and are no longer true:

- **"`ghurni_differential_ratio` is dead API."** It is not — `tests/oracle_pins.tcyr`
  has called it since 2.0.3, when the ring/pinion asymmetry was pinned.
- **"`sample_position` is vestigial in three synths."** Two, not three:
  `differential` became a live reader in 2.3.0 when ring-revolution modulation
  landed. Only `transmission` and `forced_induction` still write it without
  reading it.

## Deferred: per-component stems

Rendering exhaust / intake / mechanical into separate buffers is a real API gap
and is the one Arc 3 item not done here. It needs a shape decision that applies
to all ten synths at once — a `process_block_stems(self, a, b, c)` per synth
multiplies the public surface by a third, and the component split is not the same
across synths (a clock has no intake). That deserves its own ADR rather than a
per-synth improvisation, and it is the natural companion to a source-body
convolution decision, which ADR-007 also left open.
