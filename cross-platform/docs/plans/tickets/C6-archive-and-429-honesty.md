# Ticket C.6 — Archive honesty + persisted 429 back-off

- Epic: **C** · Depends on: C.1 (Claude usage core that consumes the archive) · Lane: C
- Headless-safe: the whole behaviour is testable from fixtures, no endpoints.

## Goal

The two "don't lie to the user" guarantees survive the Linux port: (1) the last-good
reading is persisted and comes back **`.stale` + dimmed** on relaunch — never presented as
live; (2) a `429`/`Retry-After: 0` from a provider only ever *raises* the poll floor
(60 s → double → 15 min cap) and the deadline is persisted so it survives relaunch. Both
behaviours already exist in `windows/` — this is adapt + pin on Linux.

## Read first

1. `AGENTS.md` — "Archived readings come back dimmed"; the 429 floor-raiser rule.
2. `cross-platform/docs/notes/window-managers.md` §9 (the parity list — polling cadence,
   persisted 429 deadline, stale-dimmed archive).
3. `cross-platform/docs/plans/2026-09-11-cross-platform-plan.md` — Epic C, C.6.
4. `codenotch/src/usage.rs` — `load_persisted` (≈78), the polling loop `start` (≈259),
   persistence writes, the current back-off math; and `codenotch/src/config.rs` (where a
   persisted deadline would live next to the archive you already write).
5. `windows/codenotch/src/usage.rs` — reference.

## Scope (exactly this)

1. Archive round-trip: persist a synthetic snapshot, relaunch-path reload, assert it comes
   back `.stale` with its age (`fetchedAt` preserved, not now) — a fixture-driven unit test.
2. Back-off: extract the floor-raiser decision (current attempt count + deadline → next
   poll time; 60 s start, doubling, 15 min cap) into a pure function if it isn't already,
   and pin: `Retry-After: 0` does not shrink the floor, double/cap math, and the persisted
   deadline surviving a reload.
3. Where the deadline is stored on Linux (under `~/.local/share/codenotch/`), make sure the
   file layout matches the archive's and is atomic-written (mirror the existing archive
   write discipline, e.g. the atomic-write pattern seen in `agy_cli`).
4. `cargo test` green on Linux.

## Dependencies allowed

None.

## Must NOT

- Change polling cadence (60 s / 5 min); shrink the back-off floor below 60 s; touch
  `windows/`; log deadline internals.

## Definition of Done

- [ ] Pins for archive-stale-reload and 429 math pass on Linux
- [ ] Report the on-disk layout of the persisted deadline and that it round-trips a reload.

## Orchestrator acceptance

Compile + test gates; diff review (pure extraction only, Windows byte-identical); notes
check that the archive/back-off table (notes §9) is unchanged.