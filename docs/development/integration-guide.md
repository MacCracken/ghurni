# Integration Guide

ghurni ships as a single Cyrius bundle, `dist/ghurni.cyr`. A consumer includes
the dependency bundles first, then ghurni:

```cyrius
include "lib/sakshi.cyr"
include "lib/hisab.cyr"
include "lib/goonj.cyr"
include "lib/naad.cyr"
include "dist/ghurni.cyr"
```

Call `alloc_init()` once before any ghurni call. Every runnable example in
[`docs/examples/`](../examples/) follows this prologue.

## Error handling comes first

Constructors return a heap pointer on success or a **negative `GH_ERR_*` code**
on failure. Check every one with `ghurni_is_err` before use:

```cyrius
var engine = ghurni_engine_new(GH_ENGINE_DIESEL, 6, f64_from(44100));
if (ghurni_is_err(engine) == 1) {
    println("engine init failed: ");
    println(ghurni_err_name(engine));
    return 1;
}
```

This is not optional politeness. `ghurni_engine_new` returns `-1` for a bad
sample rate or an unknown engine type, and `-1` is not a pointer. Library
functions that take a synth reject an error code rather than dereferencing it,
but the value is still useless as a synth.

## One-Shot Synthesis

Allocates and returns a fresh vec of f64:

```cyrius
# 1 s at 3000 RPM, 60% load.
var samples = ghurni_engine_synthesize(engine, f64_from(3000), 0.6, F64_ONE);
if (ghurni_is_err(samples) == 1) { return 1; }
print_num(vec_len(samples));    # 44100
```

`duration * sample_rate` is bounded by `GHURNI_MAX_SAMPLES` (2^26 ≈ 25 min at
44.1 kHz). An over-range or sub-sample duration returns
`GH_ERR_INVALID_PARAMETER` rather than allocating unboundedly or returning an
empty buffer.

## Streaming (Real-Time)

Fill a caller-owned vec repeatedly; state carries across calls:

```cyrius
var engine = ghurni_engine_new(GH_ENGINE_GASOLINE, 4, f64_from(44100));
if (ghurni_is_err(engine) == 1) { return 1; }
ghurni_engine_set_rpm(engine, f64_from(3000));
ghurni_engine_set_load(engine, 0.6);

# ~11.6 ms at 44.1 kHz
var buffer = vec_new();
var i = 0;
while (i < 512) { vec_push(buffer, 0); i = i + 1; }

var block = 0;
while (block < 100) {
    ghurni_engine_process_block(engine, buffer);
    # ... hand `buffer` to the audio output ...

    # Update parameters between blocks.
    ghurni_engine_set_rpm(engine, new_rpm);
    ghurni_engine_set_load(engine, new_load);
    block = block + 1;
}
```

Reuse one buffer across blocks. `process_block` writes exactly `vec_len(out)`
samples in place and never resizes it.

## Parameter Changes

`ghurni_*_set_rpm` / `_set_load` are smoothed internally by `GhSmoothedParam`
(a one-pole exponential approach), so changes take effect gradually over the
next block instead of clicking. Read the smoothed value back with
`ghurni_smooth_current`, and test for arrival with `ghurni_smooth_is_settled`.

**Load** is 0..1 and drives both level and brightness — a machine working harder
is louder *and* brighter ([ADR-007](../architecture/adr-007-load-tilt-and-diesel.md)).
Six synths take it: `engine`, `motor`, `forced_induction`, and — as of 2.4.0 —
`gear`, `transmission` and `differential`. **The default is 0, where the tilt is
exactly 1.0**, so a synth you never load renders exactly as it did before 2.4.0.

Out-of-range and non-finite values are clamped, never propagated:

```cyrius
ghurni_gear_set_load(gear, f64_from(5));    # -> 1.0
ghurni_gear_set_load(gear, f64_div(0, 0));  # NaN -> 0.0, never NaN in your audio
```

Two controls are not RPM or load:

```cyrius
# A differential meshes on the opposite tooth flank on overrun, and that flank
# is cut differently -- which is why a worn diff often whines one way only.
ghurni_differential_set_coasting(diff, 1);   # 0 = driving (default), non-zero = coasting

# A clock's beat is set by its escapement, so `set_rpm` on a clock is a no-op by
# design. Use the tick rate instead (clamped 0.1..50 Hz).
ghurni_clock_set_tick_rate(clock, f64_from(4));
```

## Event Triggers

Discrete events are queued and consumed by the next `process_block`. All eight
kinds do something as of 2.4.0 — before that, five were accepted and silently
discarded ([ADR-008](../architecture/adr-008-events-and-control.md)).

Seven go to the engine:

```cyrius
ghurni_engine_trigger_event(engine, ghurni_event_backfire());
ghurni_engine_trigger_event(engine, ghurni_event_misfire(0));   # cylinder 0
ghurni_engine_trigger_event(engine, ghurni_event_knock(2));     # cylinder 2
ghurni_engine_trigger_event(engine, ghurni_event_startup());          # ~1.1 s
ghurni_engine_trigger_event(engine, ghurni_event_shutdown());         # ~1.6 s
ghurni_engine_trigger_event(engine, ghurni_event_stall());            # ~0.6 s
ghurni_engine_trigger_event(engine, ghurni_event_rev_limiter_hit());  # ~0.35 s
```

`STARTUP`, `SHUTDOWN` and `STALL` move the engine's RPM through its **existing
smoother**, so they compose with your own `set_rpm` calls rather than fighting
them — whichever you issue last wins, and the transition is click-free either
way. `REV_LIMITER_HIT` chops combustion at 18 Hz without touching RPM, so it
layers over whatever the engine is doing.

`GEAR_SHIFT` belongs to the **transmission**, not the engine — a shift is not the
engine's business:

```cyrius
# payload is (from, to); 0 on the `from` side means neutral
ghurni_transmission_trigger_event(tx, ghurni_event_gear_shift(1, 2));
```

An out-of-range target gear is ignored, matching `ghurni_transmission_shift_to`.

Events round-trip by name in both directions, which is what you want for a
save file or a level script:

```cyrius
var e = ghurni_event_from_name("RevLimiterHit");   # -> GH_EVENT_REV_LIMITER_HIT
var n = ghurni_event_name(GH_EVENT_STALL);         # -> "Stall"
```

`ghurni_event_from_name` returns `-1` for an unknown name — check it before use.
`ghurni_kind_name` / `ghurni_kind_from_name` do the same for `GH_KIND_*`, and the
`-1` there is safe to hand straight to `ghurni_mixer_add_channel`, which
range-checks its tag and returns an error rather than dispatching on garbage.

## Custom Firing Order

Crank-angle offsets in degrees, one per cylinder. The vec is **copied**, so you
may reuse or modify your own vec afterwards:

```cyrius
var order = vec_new();
vec_push(order, f64_from(0));   vec_push(order, f64_from(90));
vec_push(order, f64_from(270)); vec_push(order, f64_from(180));
vec_push(order, f64_from(540)); vec_push(order, f64_from(630));
vec_push(order, f64_from(450)); vec_push(order, f64_from(360));
ghurni_engine_set_firing_order(engine, order);   # cross-plane V8
```

The call is ignored unless `vec_len(order)` equals the engine's cylinder count.

## Multi-Component Mixing

`GhMixer` combines synths through the `GH_KIND_*` tag dispatch. `add_channel`
returns the channel index, or a negative `GH_ERR_*` if the synth is a failed
constructor result / null or the kind tag is out of range:

```cyrius
var mixer = ghurni_mixer_new();
var idx = ghurni_mixer_add_channel(mixer, "engine", GH_KIND_ENGINE, engine);
if (ghurni_is_err(idx) == 1) { return 1; }
ghurni_mixer_add_channel(mixer, "turbo", GH_KIND_FORCED_INDUCTION, turbo);

ghurni_mixer_set_channel_gain(mixer, idx, 0.8);
ghurni_mixer_set_channel_pan(mixer, idx, f64_neg(0.5));   # -1 = hard left
ghurni_mixer_set_master_gain(mixer, 0.9);

ghurni_mixer_set_rpm(mixer, f64_from(3000));   # broadcast to every channel

ghurni_mixer_process_block(mixer, mono);
ghurni_mixer_process_block_stereo(mixer, left, right);
```

Each channel keeps a scratch buffer sized to the block length and reuses it, so
a fixed-block-size callback allocates once per channel, not once per block. Keep
the block size stable to keep it that way. For stereo, `left` and `right` must
be distinct vecs; the shorter of the two bounds the work, and the tail of the
longer one is left untouched rather than zeroed.

> ⚠ **The `(GH_KIND_*, pointer)` pair must match.** The tag declares which struct
> the pointer addresses. `add_channel` range-checks the tag and rejects error
> codes, null, and small integers — but it **cannot** verify that a pointer is of
> the type the tag claims. Registering a gear under `GH_KIND_ENGINE` makes the
> dispatcher read engine fields off a smaller allocation, which is undefined
> behaviour. This is inherent to the design: Rust's `Box<dyn Synthesizer>`
> carried its own vtable and a raw `i64` does not. Always pass the tag that goes
> with the constructor you called.

## Presets

Twelve shipped configurations, each returning a ready synth (or a `GH_ERR_*`):

```cyrius
var engine = ghurni_preset_v8_muscle_car(f64_from(44100));
var trans  = ghurni_preset_manual_5speed(f64_from(44100));
```

`v8_muscle_car` · `inline4_economy` · `diesel_truck` · `motorcycle_single` ·
`electric_vehicle` · `turbocharger` · `supercharger` · `manual_5speed` ·
`manual_6speed` · `steel_spur_gear` · `industrial_turbine` · `propeller`

## Determinism

Identical parameters and an identical call sequence produce bit-identical
output ([ADR-003](../architecture/adr-003-deterministic-synthesis.md)). Noise
seeds are derived from constructor parameters, so two instances that differ in
any seeded parameter decorrelate — layer two turbos with different drive ratios
and their hiss sums incoherently, as it should.

## Logging

Diagnostics go through sakshi. Set the verbosity at runtime:

```cyrius
sakshi_set_level(0);   # quiet; raise for WARN/INFO/DEBUG
```

ghurni logs a WARN on every rejected parameter, which is the fastest way to find
out *why* a constructor returned `-1`.
