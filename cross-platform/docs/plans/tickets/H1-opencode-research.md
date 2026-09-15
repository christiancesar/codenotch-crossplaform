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

- [x] Full schema dump (via `sqlite3 opencode.db .schema` or equivalent,
      read-only) attached to the report for `session`, `event`, `project`,
      `workspace`, `account`, `account_state`, `credential` — confirms or
      corrects the column lists above (they were read from a live `pragma
      table_info` on this host on 2026-09-14; a newer OpenCode version may
      have migrated the schema since).
- [x] Written answer to each of the 4 open questions above.
- [x] A follow-up ticket `H2-opencode-<shape>.md` written (not implemented)
      with the same anatomy as `C1-claude-provider.md` (Read first / Scope /
      Must NOT / Definition of Done / Verify), scoped to whatever shape was
      decided.

## Orchestrator acceptance

Confirm no code was written (`git status` clean except the new H.2 ticket
file and this ticket's own updates); confirm no credential values appear
anywhere in the report or the schema dump; confirm the open questions were
actually answered, not restated.

---

## Report (2026-09-15)

All queries below were read-only (`sqlite3 -readonly`). `account` and
`credential` were schema-dumped (`.schema`, column names only) but never
row-queried — see "Must NOT" — so nothing from those two tables' actual
values appears anywhere below, only their column names.

### Schema dump (live, this host, opencode 2026-09-15)

```sql
CREATE TABLE `session` (
	`id` text PRIMARY KEY,
	`project_id` text NOT NULL,
	`parent_id` text,
	`slug` text NOT NULL,
	`directory` text NOT NULL,
	`title` text NOT NULL,
	`version` text NOT NULL,
	`share_url` text,
	`summary_additions` integer,
	`summary_deletions` integer,
	`summary_files` integer,
	`summary_diffs` text,
	`revert` text,
	`permission` text,
	`time_created` integer NOT NULL,
	`time_updated` integer NOT NULL,
	`time_compacting` integer,
	`time_archived` integer, `workspace_id` text, `path` text, `agent` text, `model` text,
	`cost` real DEFAULT 0 NOT NULL, `tokens_input` integer DEFAULT 0 NOT NULL,
	`tokens_output` integer DEFAULT 0 NOT NULL, `tokens_reasoning` integer DEFAULT 0 NOT NULL,
	`tokens_cache_read` integer DEFAULT 0 NOT NULL, `tokens_cache_write` integer DEFAULT 0 NOT NULL,
	`metadata` text,
	CONSTRAINT `fk_session_project_id_project_id_fk` FOREIGN KEY (`project_id`) REFERENCES `project`(`id`) ON DELETE CASCADE
);
CREATE INDEX `session_project_idx` ON `session` (`project_id`);
CREATE INDEX `session_parent_idx` ON `session` (`parent_id`);
CREATE INDEX `session_workspace_idx` ON `session` (`workspace_id`);

CREATE TABLE `event` (
	`id` text PRIMARY KEY,
	`aggregate_id` text NOT NULL,
	`seq` integer NOT NULL,
	`type` text NOT NULL,
	`data` text NOT NULL,
	CONSTRAINT `fk_event_aggregate_id_event_sequence_aggregate_id_fk` FOREIGN KEY (`aggregate_id`) REFERENCES `event_sequence`(`aggregate_id`) ON DELETE CASCADE
);
CREATE INDEX `event_aggregate_type_seq_idx` ON `event` (`aggregate_id`,`type`,`seq`);
CREATE UNIQUE INDEX `event_aggregate_seq_idx` ON `event` (`aggregate_id`,`seq`);

CREATE TABLE `project` (
	`id` text PRIMARY KEY,
	`worktree` text NOT NULL,
	`vcs` text,
	`name` text,
	`icon_url` text,
	`icon_color` text,
	`time_created` integer NOT NULL,
	`time_updated` integer NOT NULL,
	`time_initialized` integer,
	`sandboxes` text NOT NULL,
	`commands` text, `icon_url_override` text
);

CREATE TABLE IF NOT EXISTS "workspace" (
	`id` text PRIMARY KEY,
	`type` text NOT NULL,
	`name` text DEFAULT '' NOT NULL,
	`branch` text,
	`directory` text,
	`extra` text,
	`project_id` text NOT NULL, `time_used` integer NOT NULL DEFAULT 0,
	CONSTRAINT `fk_workspace_project_id_project_id_fk` FOREIGN KEY (`project_id`) REFERENCES `project`(`id`) ON DELETE CASCADE
);

CREATE TABLE `account` (
	`id` text PRIMARY KEY, `email` text NOT NULL, `url` text NOT NULL,
	`access_token` text NOT NULL, `refresh_token` text NOT NULL,
	`token_expiry` integer, `time_created` integer NOT NULL, `time_updated` integer NOT NULL
);
CREATE TABLE `account_state` (
	`id` integer PRIMARY KEY NOT NULL,
	`active_account_id` text, `active_org_id` text,
	FOREIGN KEY (`active_account_id`) REFERENCES `account`(`id`) ON UPDATE no action ON DELETE set null
);
CREATE TABLE `credential` (
	`id` text PRIMARY KEY, `integration_id` text, `label` text NOT NULL, `value` text NOT NULL,
	`connector_id` text, `method_id` text, `active` integer,
	`time_created` integer NOT NULL, `time_updated` integer NOT NULL
);
```

Corrections to the 2026-09-14 read: `project` gained `icon_url`,
`icon_color`, `commands`, `icon_url_override` since the ticket was filed;
`workspace` gained `time_used`. Everything else in the "already confirmed"
section matches.

### Q1 — Shape: **(c) both**, decided with the operator

`session` already carries `cost`, `tokens_input/output/reasoning/cache_*`
and `model` as plain columns — a "cost so far" or "tokens today" number for
the ring is a `SUM()`/`MAX(time_updated)` query, not a new data source.
Given that, gating the session-list integration (the actual ask, "the same
way Claude is done") behind a separate follow-up bought nothing, so H.2
below scopes both: OpenCode sessions in the session list (via
`state.rs`/`watcher.rs`), and an aggregate on the ring using the same
`session` rows already being read for the list.

### Q2 — Multi-account

Only one `opencode.db` exists on this host, at the standard XDG path
(`~/.local/share/opencode/opencode.db`); no per-profile sharding (no
`~/.opencode-<slug>` sibling directories, unlike Claude's
`~/.claude-<slug>` pattern) was found. `workspace` (0 rows on this host)
and `session.workspace_id` (NULL on every one of the 331 sessions here) are
schema support for *grouping sessions inside one DB*, not for multiple
accounts or multiple DBs — don't build a workspace-scoping code path for
v1, there's nothing live to scope. Separately, `account`/`account_state`/
`credential` (schema only, not queried for values here) look like
OpenCode's own hosted/zen login, not the same thing as `auth.json`'s
per-service BYOK map — don't conflate the two when H.2 reads `auth.json`
for provider *names* (§ next ticket's "Read first"). No evidence either
way of a supported multi-DB-per-machine setup; treat the single fixed path
as the v1 assumption, same posture the port already takes with Claude
before multi-login existed.

### Q3 — Liveness: derivable from `opencode.db` alone, no companion signal

Two options, both DB-only:
- **Coarse** (mirrors `codex_activity`'s rollout-mtime path in
  `activity.rs`): `session.time_updated` within a short window (e.g. last
  60–120 s) = busy, matching the "recency + timeout" shape already used
  for Codex's CLI/extension fallback (`activity.rs` ≈327-354).
  `time_compacting`/`time_archived` being non-null are cheap extra signals
  (compacting = busy, archived = never busy).
- **Fine** (mirrors `codex_turns_in_progress`'s inProgress+freshness
  pattern, `activity.rs` ≈255-318): `event` is a genuine append-only log
  keyed by `aggregate_id` (= session id) and `seq`; the latest row per
  `aggregate_id` with `type='message.part.updated.1'` inside a short
  window is a much finer "still streaming a reply right now" signal than
  session-level `time_updated`, at the cost of one more query per
  candidate session. On this host: 101205 `message.part.updated.1`, 39031
  `message.updated.1`, 10930 `session.updated.1`, 215 `session.created.1`
  rows — high volume, so query it by `aggregate_id` with the existing
  index (`event_aggregate_type_seq_idx`), never a full-table scan.
- Recommendation for H.2: start coarse (`session.time_updated`), same
  cost/complexity as the Claude session list already has; only reach for
  the `event` table if the coarse signal proves too laggy in practice.

### Q4 — WAL/locking: confirmed WAL, `cursor.rs`'s `open_ro()` pattern applies directly

`PRAGMA journal_mode` reports `wal`; `opencode.db-shm` and `opencode.db-wal`
sidecar files are present and non-empty on this host while OpenCode is
running. This is the exact situation `cursor.rs`'s `open_ro()` already
handles (`SQLITE_OPEN_READ_ONLY` first, verified with a real query since a
missing `-shm` can let `open` succeed while every query fails; falls back
to the `immutable=1` URI form). No new locking pattern needed — H.2 reuses
`open_ro()` (or a copy of it) as-is; `rusqlite` is already a dependency,
confirmed no new crate needed.

### Follow-up

`H2-opencode-session-and-cost.md` written alongside this report, scoped to
the (c) decision above.
