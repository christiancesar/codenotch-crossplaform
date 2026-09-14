# Ticket C.4 — Antigravity provider on Linux (fixture-based verify)

- Epic: **C** · Depends on: Epic 0 · Lane: C · Headless-safe
- Host facts: no `~/.local/share/agy`, no `agy` binary → **fixture-only**; do not claim a
  live Antigravity read. The Credential Manager path stays Windows-only.

## Goal

The Antigravity cell degrades honestly on a machine with no Antigravity install (`stale`/
`nothingMetered`, never blank, never a guess) and parses real readings correctly where the
binary exists. Epic 0 added Linux candidates for `agy` discovery — this ticket proves the
discovery, the local-bridge/transcript fallback parse, and the degraded state, all from
fixtures.

## Read first

1. `AGENTS.md` (provider rules), `.opencode/skills/provider-adapter/SKILL.md`.
2. `cross-platform/docs/notes/window-managers.md` §6 (Antigravity row: `agy` if shipped for
   Linux, else local language-server bridge — *to confirm*) and §4 (native-tls 127.0.0.1
   bridge note).
3. `cross-platform/docs/plans/2026-09-11-cross-platform-plan.md` — Epic C, C.4.
4. `codenotch/src/agy_cli.rs` — `find_agy` (Linux candidates from Epic 0), the bridge
   (`native-tls` for 127.0.0.1), transcript fallback, `quote_arg`/`encode_png` dead-code
   warnings to ignore-not-fix, existing tests.
5. `codenotch/src/antigravity.rs` — `#[cfg(windows)]` Credential-Manager sections that must
   stay Windows-only, and the rest that already parses cross-platform.
6. `windows/codenotch/src/agy_cli.rs` / `antigravity.rs` — reference only.

## Scope (exactly this)

1. **Recorded-body pins** built from the *documented* bridge/transcript shapes (no live
   install here): `agy-bridge.json` (usage-summary) + one transcript sample + one
   empty-first-install case (`agy_install_dirs` with nothing present). Pin the parsers.
2. Discovery: on this host, absence must produce the honest degraded status — write the
   pin that proves `find_agy` returning nothing does not fabricate a zero ring.
3. Bridge security: keep the native-tls-that-trusts-127.0.0.1-only contract untouched and
   gated correctly for non-Windows; the local bridge must never dial off-loopback.
4. `cargo test` green on Linux.

## Dependencies allowed

None.

## Must NOT

- Add a fake local bridge; make the Credential-Manager path non-Windows; invent install
  roots; claim a live Antigravity verification that did not happen; touch `windows/`.

## Definition of Done

- [ ] Tests green incl. the three pins
- [ ] The no-install degraded case is pinned and passes on `cargo test`.
- [ ] Report: which paths/bridge shapes are fixture-derived vs confirmed on a live binary
      (honesty rule — this ticket is fixture-derived).

## Orchestrator acceptance

Compile + test gates; fixture review; note in plan that Antigravity live verification is
deferred until an install exists.