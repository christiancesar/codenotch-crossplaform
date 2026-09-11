# Ticket C.2 — Codex provider on Linux (adapt + verify)

- Epic: **C** · Depends on: Epic 0 · Lane: C · Headless-safe
- Host facts: `~/.codex/auth.json` exists; `codex` binary not on PATH (parser tests don't
  need it).

## Goal

The Codex cell shows the wham/usage reading on Linux with the same honesty as upstream:
fresh-until-5-min (`stale > 5 min`), rollout `rate_limits` tail fallback when the live
snapshot is missing, and every failure a visible status. `find_executable` Linux candidates
were added in Epic 0 — this ticket proves the parser and path table on Linux.

## Read first

1. `AGENTS.md` (provider rules), `.opencode/skills/provider-adapter/SKILL.md`.
2. `cross-platform/docs/plans/2026-09-11-cross-platform-plan.md` — Epic C, C.2.
3. `cross-platform/docs/notes/window-managers.md` §6 (Codex row: near-certain).
4. `codenotch/src/codex.rs` — auth read, `wham/usage` fetch, rollout snapshot parse (the
   `(windows, recorded-at ms, plan)` path ≈328 and the `snap.fetched_at = rec` discipline
   ≈437), and its existing tests.
5. `windows/codenotch/src/codex.rs` — reference only.

## Scope (exactly this)

1. Confirm Linux paths live: `~/.codex/auth.json`, rollout `*.jsonl` locations as the code
   computes them; fix only silently-diverging paths and list each in the report.
2. **Recorded-body pins** (fixtures, secrets-redacted): `codex-wham-usage.json` and one
   rollout-snapshot tail `codex-rollout.jsonl`. Pin the two parsers to them; assert the
   stale window (fetched-at older than 5 min → `.stale`), nothing-to-read →
   `nothingMetered`-style status, not a zero ring.
3. Confirm the `find_executable` candidates (Epic 0) are actually exercised only to *locate*,
   never to fabricate a reading when absent — bytes diff vs `windows/` if behaviour needed
   touching (must not).
4. `cargo test` on Linux green.

## Dependencies allowed

None.

## Must NOT

- Touch `windows/`; log/commit credentials; invent a window when the rollout has no tail;
  change the 5-min stale policy.

## Definition of Done

- [ ] Tests green incl. new pins; `cargo fmt --check` clean.
- [ ] Fixtures present under `codenotch/tests/fixtures/`; stale/NothingMetered probes pass.
- [ ] `codenotch doctor` on this host resolves the auth file without leaking it.

## Orchestrator acceptance

Compile + test gates; fixture review for secrets; confirm no Windows change.