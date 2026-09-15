# Ticket H.2 — OpenCode as a 5th provider: activity rows + cost ring

- Epic: **H** (OpenCode) · Depends on: **H.1** (research + decision, done) · Lane: H
- Decided shape (H.1, confirmed with the operator): **both** — OpenCode sessions show up
  as activity rows in the hover card the same way Codex/Cursor/Antigravity do, and the
  ring shows an accumulated cost/token count, because both come from the same `session`
  row this ticket already has to read for the activity rows.
- Host facts: `~/.local/share/opencode/opencode.db` exists (SQLite, WAL mode, ~1 GB on
  the research host), 331 sessions, high `event` volume. No credential values were ever
  read for H.1 and none should be read here either — see "Must NOT".

## Correction vs. H.1's original phrasing

H.1 pointed at `state.rs`/`watcher.rs` (Claude's own hook-driven session-list engine) as
the shape-(b) target. That's wrong for this provider: `state.rs`'s `Store`/`Session` is
Claude-exclusive (PID tracking, hook events from `server.rs`, single-provider by
construction — see `ui/notch.html` ≈287, `if(p.id==='claude'){ ... stateSnap.sessions ...
}`). Codex, Cursor and Antigravity are **not** in that list; they show up as
`activity.rs::Activity` rows filtered by provider in the *same* hover card (`ui/notch.html`
≈296-306, `else{ const acts=activity.filter(a=>a.provider===p.id); ... }`). OpenCode's data
(materialized SQL rows, no hook events to inject) is a much closer match to how
Codex/Cursor read their own on-disk state than to Claude's hook model — this ticket
targets `activity.rs`, not `state.rs`.

## Read first

1. `src/activity.rs`:
   - `Activity` struct (`provider, state, name, detail, since`).
   - `Presence` struct and `presence()` (≈517-527) — per-provider install detection,
     gates which `_activity()` functions even run.
   - `read_all()` (≈529-542) — where a new provider's function gets added.
   - `cursor_activity()` (≈148) and `codex_activity()`/`codex_turns_in_progress()`
     (≈255-354) — the two closest shapes: Cursor reports its own state directly, Codex
     infers busy/waiting from on-disk recency + a status/type field. OpenCode's `event`
     table (append-only, keyed by `aggregate_id`+`seq`) is closer to Codex's shape.
2. `src/usage.rs`: `LimitWindow` (`id, label, used, resets_at, count, derived`) and
   `UsageSnapshot` (`status, windows, fetched_at, note, backoff_until`) — Antigravity's
   existing `count`/`derived` usage (no `used` fraction, no `resets_at`) is the exact
   precedent for OpenCode's ring value; read whichever of `antigravity.rs`/`cursor.rs`
   builds a `UsageSnapshot` most directly as the template.
3. `src/cursor.rs`: `open_ro()` (≈88) — the read-only SQLite open pattern (verifies a real
   query works, falls back to the `immutable=1` URI form for a missing `-shm`). Confirmed
   applicable as-is: `opencode.db` is WAL-mode with live `-shm`/`-wal` sidecars.
4. `ui/notch.html`: `providers()` (≈172-184, the `agSnap.status!=='absent'` gate is the
   pattern a `openCodeSnap` follows for the ring), and the `renderCard()` split at ≈287
   quoted above for the activity-rows side.
5. `H1-opencode-research.md`'s Report section (schema dump + the 4 answered questions) —
   don't re-derive the schema, it's already there.

## Scope (exactly this)

1. **`src/opencode.rs`** (new file): `present()` (does
   `~/.local/share/opencode/opencode.db` exist — mirrors `cursor::present()`/
   `codex::present()`); a read-only connection helper reusing/copying `cursor.rs`'s
   `open_ro()`; a function returning recent-session rows (`id, title, time_updated,
   time_compacting, time_archived, cost, tokens_input, tokens_output`) ordered by
   `time_updated DESC LIMIT` some small N — mirror whatever limit `cursor_activity`/
   `codex_turns_in_progress` already use.
2. **Activity rows**: `opencode_activity()` in `activity.rs`, following H.1's Q3 answer —
   start with the coarse signal (`time_updated` within a short freshness window = busy;
   `time_compacting` non-null = busy; `time_archived` non-null = never busy), same
   recency-window shape as `codex_activity`'s CLI/extension fallback. Do not reach for the
   `event` table in this ticket unless the coarse signal proves too laggy in manual
   testing — H.1 already flagged that as a "only if needed" escalation, not a default.
   Wire into `Presence`/`read_all()` exactly like Cursor/Codex/Antigravity.
3. **Cost ring**: a `UsageSnapshot` for OpenCode using `LimitWindow.count` (e.g. accumulated
   `cost` over some window, or a token count — pick whichever reads better once real
   numbers are in front of you, both are equally cheap from the same query) and
   `derived: true` (it's Codenotch's own aggregation, not a vendor-published number, same
   fidelity tag Antigravity uses for its count). No inverted percentage, no invented
   denominator — this provider has none, same as the H.1 research established.
4. **Wire the provider into `main.rs`/`AppState`** the same way Antigravity is wired
   (a `Mutex<UsageSnapshot>` field, a poll/refresh entry point, an `emit` to the page),
   and into `ui/notch.html`'s `providers()` list (`agSnap.status!=='absent'`-style gate)
   and glyph lookup.
5. **Multi-account**: none needed per H.1 Q2 — single fixed DB path, no workspace/profile
   scoping.

## Dependencies allowed

None. `rusqlite` is already a dependency (confirmed in H.1); no new crate.

## Must NOT

- Read or log values from `account`, `account_state`, or `credential` tables, or from
  `auth.json`/`account.json` — schema/presence checks only, same rule as every other
  provider ticket. Provider *names* from `auth.json` (which BYOK keys exist) may inform
  future work but are out of scope here — this ticket only needs `opencode.db`.
- Touch `state.rs`/`watcher.rs` (that's Claude's own engine, not this provider's shape —
  see "Correction" above) or `windows/`.
- Add a `workspace`/multi-profile scoping layer (H.1 Q2: nothing live to scope on this
  host; revisit only if a real multi-workspace OpenCode install turns up).
- Query the `event` table by anything other than an indexed `aggregate_id` lookup (it's
  a six-figure-row table on a real install; no full scans).

## Definition of Done

- [ ] `cargo test --workspace` green.
- [ ] `./target/debug/codenotch doctor` (or equivalent) reports OpenCode presence/absence
      correctly on a host with and without `~/.local/share/opencode/opencode.db`.
- [ ] Manual verification: OpenCode cell appears in the pill only when
      `~/.local/share/opencode/opencode.db` exists; hovering it shows recent
      busy/waiting rows when a session is actually active; the ring shows a count, never
      a fabricated percentage.
- [ ] Report: which of the two Q3 liveness signals (coarse `time_updated` vs. the `event`
      table) ended up used, and why, if it changed from the coarse default above.

## Orchestrator acceptance

Compile + test gates; confirm no credential/account values appear anywhere in code,
logs, or the report; confirm `state.rs` was not touched; confirm the new provider fully
disappears (no empty cell, no crash) on a host without OpenCode installed.
