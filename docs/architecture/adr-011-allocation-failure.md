# ADR-011: A refused allocation is fail-stop

**Status**: Accepted
**Date**: 2026-11-07
**Release**: 2.7.2 (P(-1) sweep)

## Context

No `alloc()` or `vec_new()` result in ghurni is checked, and the roadmap carried that
as "blocked upstream" for many releases, waiting on an ecosystem-wide decision. naad
has since made one, in its ADR-0006, and goonj made the same call in its ADR-0004:
ending the process is acceptable, as long as the refusal never returns a silently
wrong answer.

The risk that matters is not the crash. It is a refusal that does not crash: wrong
audio returned as if it were right, or a store through 0 plus a large offset that
lands inside the process image instead of the null page. That is a measurement, so
the 2.7.2 sweep measured it rather than borrow naad's numbers.

## Measurement

A scratch build whose `alloc` refuses the Nth call, singly and from then on, across
every synth, the mixer and two presets. Each case ran in its own process, and each
fault address was read under gdb.

| Sweep | Cases | Outcome |
|---|---|---|
| Refuse the Nth allocation, 13 pipelines | 690 | 540 fault, 124 stop at the stdlib's vec abort, 26 harmless, **0 silent, 0 hangs** |
| Every public function, all arguments 0 | 177 | 87 fault, 90 return, **0 hangs** |

Every fault landed in the null page, at offset 16 at most.

## Decision

**A refused allocation ends the process, and ghurni adds no checks for it.** Rust's
allocator aborts here too.

Allocations whose size depends on input are bounded before they are made, as they
were already: `ghurni_sample_count` caps every `synthesize` at `GHURNI_MAX_SAMPLES`, and
`ghurni_engine_compute_even_firing` clamps its count. Those are input validation, not
allocation-failure handling, and they stay.

## Consequences

- **Positive**: no error path that nothing can test, and no new failure returns.
- **Negative**: a consumer cannot survive heap exhaustion inside ghurni. It could not
  in Rust either.
- **Neutral**: this rests on a measurement, and a new buffer whose first store sits
  at a large offset would invalidate it. Rerun both sweeps at every P(-1).

## Amendment, 2.8.0: naad reports some refusals now

naad 2.3.4 (its ADR-0010) made five constructors report a refused allocation as
`NAAD_ERR_ALLOCATION` (-6) instead of stopping inside naad. ghurni calls one of them
on every construction, `noise_new`, and stored its result unchecked on the old
premise that it never fails. A refusal there built a synth holding -6 as its noise
generator, which then stopped at its first render instead of in the constructor.

ghurni already passed on the code of every other naad constructor, so it now checks
`noise_new` too, and the constructor returns `GH_ERR_SYNTHESIS_FAILED`.
`tests/alloc_refused.tcyr` refuses exactly that request in all ten constructors.
This passes on a report naad makes; it is not an allocation check of ghurni's own.
The decision above stands for those.

## Alternatives considered

- **Check every constructor.** About 40 sites, each a new error return that no test
  could reach without the same patched allocator. Rejected for the reason above: the
  measured outcome is already fail-stop.
- **Route `synthesize` through a fallible push.** `vec_push_a` can report a refusal,
  which would turn a 25-minute `synthesize` on a small heap into an error. Deferred:
  it is the one place where the requested size is large and caller-chosen, but it is
  also bounded, and the change touches all ten synths.
