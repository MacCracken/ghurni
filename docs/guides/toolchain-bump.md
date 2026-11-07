# Moving the toolchain pin

How to move `[package].cyrius` and the dependency set together. Written down after
2.6.2, 2.7.0 and 2.7.1, each of which found something a green `cyrius test` did not.

## 1. Pick the set

naad decides it. Find the naad tag that pins the target cyrius, then take what it pins:

```bash
git -C ../naad tag --sort=-v:refname | head
```

```bash
git -C ../naad show 2.3.3:cyrius.cyml | grep -E '^cyrius|^tag'
```

Read the tags in the sibling clone, not `~/.cyrius/deps`, which holds only tags
something has already resolved. hisab pins sakshi, and goonj pins hisab, so check
that the chain agrees. naad's `CHANGELOG.md` names the set in its entry.

Then read naad's changelog from the current tag to the new one. Look for **new
failure returns** in anything ghurni calls: naad 2.3.2 made `delay_line_new` fallible,
and ghurni held it unchecked until 2.7.1.

Do not reuse naad's review of language changes. It rests on what naad's source does
not contain, and ghurni uses float literals and type annotations that naad does not.

## 2. Move the pins and re-vendor

Edit `cyrius` and the four `tag` lines in `cyrius.cyml`, and the comment above
`[deps.naad]`. Then:

```bash
rm cyrius.lock
```

```bash
cyrius lib sync
```

```bash
cyrius deps
```

```bash
cyrius deps --verify
```

```bash
cyrius deps --locked
```

Remove the lock first. From 6.7.18 `lib sync` writes nothing while the lock records
the old pin, and `--relock` keeps dead `commit` rows for old tags. If `cyrius deps`
notes a file outside the declared footprint, remove it and run `cyrius deps --lock`.

Traps:

- `cyrius lib sync --help` does not print help. It runs the sync.
- `cyrius build --no-deps` also skips the stdlib prelude, so it fails with undefined
  `F64_ONE` and proves nothing.
- A plain `cyrius build` re-vendors `lib/`, so a patched `lib/` file in a scratch tree
  is silently restored. To test one, move `cyrius.cyml` aside and include the stdlib
  explicitly.

## 3. Prove what moved

Green goldens do not mean bit-identical, because they truncate to 1e-6. Render the
same cases on both toolchains and compare raw f64 bits, and put that number in the
changelog. The golden rule still decides the version: if a golden moves, it is not a
patch.

Run every gate in `.github/workflows/ci.yml`, not only `cyrius test`. At 2.6.2 the
tests were green while `cyrius audit` exited 1.

Compare `cyrius bench` against the old tree in alternating runs on a quiet machine.
Numbers taken inside `cyrius audit` are not comparable.

## 4. Check consumers

`dist/ghurni.cyr` is compiled by the **consumer's** toolchain. Measure two things
separately:

- **The bundle on the oldest consumer pin.** Copy the consumer to a scratch directory,
  point its `[deps.ghurni]` at `path = "<this checkout>"`, run `cyrius deps --relock`
  there, and run its tests. Run the control first, on the tag it pins, or a failure
  proves nothing.
- **The floor of the full set.** In scratch copies of this tree, set the pin to each
  candidate, then remove the lock, sync, resolve and build. Report the lowest that
  builds and runs.

## 5. Finish

```bash
scripts/version-bump.sh <version>
```

Update `CHANGELOG.md`, `docs/development/state.md`, the dependency tables in
`README.md` and `SECURITY.md`, the pin in `CONTRIBUTING.md`, and the shipped table in
`docs/development/roadmap.md`.
