# Ticket H.1 — OpenCode as a 5th provider: research + decision (no code)

- Epic: **H** (OpenCode, exploratory) · Depends on: nothing · Lane: research only
- Queued 2026-09-14, end of the roadmap, operator-authorized exception to
  requirement 2 ("exactly the current 4 providers").
- **This ticket produces a decision and a follow-up ticket (H.2), not an
  adapter.** Do not write `opencode.rs` here.

## Why this exists

The ask: track the OpenCode CLI "the same way Claude is done" — read what
OpenCode already writes to disk, no OAuth flow of Codenotch's own, no
copying a web session/cookie. That non-negotiable (rule 4 in the port spec)
applies here exactly as it does to the other 4. What's genuinely open is
*what* to show, because OpenCode isn't shaped like Claude/Codex/Cursor.

## What's already confirmed (read-only, no writes) — start from here, don't re-derive

- `~/.local/share/opencode/auth.json` and `account.json`: per-service
  credentials (`google`, `github-copilot`, `deepseek`, `sakana-fugu`,
  `opencode-go`), each `{type: "api", key}` or `{type: "oauth", access,
  refresh, expires}`. These are OpenCode's *own* BYOK keys for calling
  models — not a Codenotch-relevant quota. OpenCode is a routing/BYOK tool,
  not a metered subscription like Claude Pro/ChatGPT Plus/Cursor — **there
  is no natural "% used" for a ring** the way the other 4 have.
- `~/.local/share/opencode/opencode.db`: a real SQLite 3 database
  (confirmed via `file` + `sqlite3`), ~1 GB on this host. Tables include
  `session`, `message`, `part`, `project`, `event`, `workspace`, `todo`,
  `credential`, `session_share`, `account`, `account_state`,
  `event_sequence`, `session_message`, `permission`, `project_directory`,
  plus migration bookkeeping tables.
  - `session` columns: `id, project_id, parent_id, slug, directory, title,
    version, share_url, summary_additions, summary_deletions,
    summary_files, summary_diffs, revert, permission, time_created,
    time_updated, time_compacting, time_archived, workspace_id, path,
    agent, model, cost, tokens_input, tokens_output, tokens_reasoning,
    tokens_cache_read, tokens_cache_write, metadata`. This is rich enough
    for a session-activity view (recency via `time_updated`, cost/token
    totals per session) — the same shape of information `watcher.rs`
    derives from Claude's `~/.claude/projects/*.jsonl` files, just already
    structured in SQL instead of JSONL.
  - `event` columns: `id, aggregate_id, seq, type, data`. Observed types on
    this host: `session.created.1`, `session.updated.1`,
    `message.updated.1`, `message.part.updated.1` — a genuine append-only
    event log, tailable by `seq`. This is the closest analogue to Claude
    Code's hook events (`server.rs`'s `/event` POSTs), except pulled instead
    of pushed.
  - `project` columns include `worktree, vcs, name, time_created,
    time_updated` — maps a session to a repo, useful for a "which project"
    label the way Claude sessions show `cwd`.
- Same technical pattern the port already uses for Cursor: SQLite opened
  read-only (`rusqlite`, `SQLITE_OPEN_READ_ONLY` — see `cursor.rs`'s
  `open_ro()`). No new dependency needed; `rusqlite` is already in
  `Cargo.toml`.
- OpenCode's *config* (not runtime state) lives separately at
  `~/.config/opencode/` (`opencode.json`, `AGENTS.md`, `plugins/`,
  `prompt/`) — not relevant to usage/activity tracking, only mentioned here
  so it isn't confused with `~/.local/share/opencode/` during
  implementation.

## Open questions to resolve in this ticket (produce a written decision, not code)

1. **Shape**: given there's no quota, is the right model —
   (a) a `usage.rs`/`cursor.rs`-style provider with `usedCount` and no
       `usedFraction` (the model already supports a fidelity-tagged count
       with an empty progress track — see `docs/specs/2026-09-11-linux-port-spec.md`
       "What a provider must provide"), showing e.g. accumulated cost or
       token count over some window;
   (b) a session-engine integration (`state.rs`/`watcher.rs`) that lists
       OpenCode sessions in the notch's session list the way Claude/Codex
       sessions appear, using `session.time_updated` for recency and
       `event` for busy/idle detection;
   (c) both.
   The original phrasing ("the same way Claude is done") leans toward (b)
   being the core ask — Claude's own session list comes from watching
   `~/.claude/projects`, not from its usage ring — with (a) as an optional
   extra if a meaningful "cost so far" number is wanted on the ring too.
   **Confirm this reading with the operator before committing to a shape**
   if it's still ambiguous once the schema is fully mapped.
2. **Multi-account**: `account.json`'s `active` map suggests OpenCode can
   have multiple accounts per service; check whether `opencode.db` is
   itself per-user-profile (single DB for the whole machine, as observed
   here) or whether multiple profiles are possible (mirroring the
   `~/.claude-<slug>` multi-login pattern) before assuming a single DB path.
3. **Liveness**: is a session "busy" derivable from `opencode.db` alone
   (e.g. `time_updated` within the last N seconds, or a specific `event`
   type), or does OpenCode need a companion signal the way Codex's
   `state_5.sqlite` polling works in `activity.rs`? Check `activity.rs`'s
   `codex_activity`/`codex_turns_in_progress` for the closest existing
   pattern to mirror if so.
4. **WAL/locking**: `cursor.rs`'s `open_ro()` comment and the port notes
   mention SQLite WAL-mode read races; confirm whether `opencode.db` runs in
   WAL mode (`PRAGMA journal_mode`) and whether OpenCode itself needs to be
   idle for a read to see fresh data, same due-diligence as the Cursor
   adapter did.

## Must NOT

- Write `opencode.rs`, touch `main.rs`'s provider wiring, or add a UI
  element in this ticket — H.1 is research + a written decision only.
- Touch `windows/`.
- Read or log actual OpenCode credential values from `auth.json`/
  `account.json` — schema and key *names* only, never values, matching the
  same rule every other provider ticket follows.

## Definition of Done

- [ ] Full schema dump (via `sqlite3 opencode.db .schema` or equivalent,
      read-only) attached to the report for `session`, `event`, `project`,
      `workspace`, `account`, `account_state`, `credential` — confirms or
      corrects the column lists above (they were read from a live `pragma
      table_info` on this host on 2026-09-14; a newer OpenCode version may
      have migrated the schema since).
- [ ] Written answer to each of the 4 open questions above.
- [ ] A follow-up ticket `H2-opencode-<shape>.md` written (not implemented)
      with the same anatomy as `C1-claude-provider.md` (Read first / Scope /
      Must NOT / Definition of Done / Verify), scoped to whatever shape was
      decided.

## Orchestrator acceptance

Confirm no code was written (`git status` clean except the new H.2 ticket
file and this ticket's own updates); confirm no credential values appear
anywhere in the report or the schema dump; confirm the open questions were
actually answered, not restated.
