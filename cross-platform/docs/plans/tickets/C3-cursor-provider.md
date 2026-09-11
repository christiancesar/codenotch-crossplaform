# Ticket C.3 — Cursor provider on Linux (path confirm + verify)

- Epic: **C** · Depends on: Epic 0 · Lane: C · Headless-safe
- Host facts: `~/.config/Cursor` exists and `cursor` on PATH → live path confirmation possible.

## Goal

The Cursor cell reads `state.vscdb` on Linux. The Linux location
(`~/.config/Cursor/User/globalStorage/state.vscdb`) is **unconfirmed** — this ticket's first
deliverable is the confirmation (a real install exists on this host), then the pinned
parser over the WAL database (`mode=ro`-first, `immutable` fallback — the fight the macOS
port had).

## Read first

1. `AGENTS.md` (provider rules), `.opencode/skills/provider-adapter/SKILL.md`, plus the
   macOS WAL/Ro lesson it documents.
2. `cross-platform/docs/notes/window-managers.md` §6 (Cursor row: *to confirm*) and §7
   (SQLite WAL note).
3. `cross-platform/docs/plans/2026-09-11-cross-platform-plan.md` — Epic C, C.3; open
   decision 3.
4. `codenotch/src/cursor.rs` — the `state.vscdb` reader (`rusqlite` bundled), open-mode
   strategy, `usage-summary` model, existing tests.
5. `windows/codenotch/src/cursor.rs` — reference only.

## Scope (exactly this)

1. **Confirm on this host**: find the real vscdb path under `~/.config/Cursor` (it may be
   `User/globalStorage/state.vscdb` or similar — prove it), set the Linux candidate in the
   code, and put the confirmed path + a redacted dump of the relevant table rows in the
   report (no secrets).
2. WAL strategy: on Linux open `mode=ro` first and fall back to `immutable=1` when the
   editor holds an open connection — both in tests.
3. **Recorded-body pin**: transpose the confirmed DB's relevant rows into a fixture
   `cursor-state.sql`/`cursor-state.json` (values only, no tokens) and pin the parser.
4. `usage-summary` endpoint: confirm the parse shape against the current fixture pair; keep
   `.derived`/`.manual` sigil behaviour honest.
5. Update the path table in `docs/notes/window-managers.md` §6 for Cursor (to:
   confirmed/false) — the only docs edit allowed in this ticket.

## Dependencies allowed

None (rusqlite already bundled).

## Must NOT

- Copy `state.vscdb` wholesale (may carry secrets) — extract only needed rows; touch
  `windows/`; invent a denom when the DB only has a count.

## Definition of Done

- [ ] Confirmed path lands in the code + notes; tests incl. the new pin green on Linux;
      `cargo fmt --check` clean.
- [ ] `cargo test` scoped to `cursor` passes with both open modes exercised.

## Orchestrator acceptance

Compile + test gates; fixture review for secrets; notes diff approved.