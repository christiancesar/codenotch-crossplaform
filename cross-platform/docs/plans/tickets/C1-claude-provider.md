# Ticket C.1 — Claude provider on Linux (adapt + verify)

- Epic: **C** · Depends on: Epic 0 · Lane: C · Headless-safe (pinned suite)
- Host facts: `~/.claude/.credentials.json` exists, `claude` on PATH — live recording possible.

## Goal

The Claude cell moves on its own on a Linux machine — same fidelity contract as upstream:
the ring shows a *real* reading declared by its provider, `401/403` re-reads the credential
once and retries, failures degrade to a visible status. Nothing here should need rewriting;
the deliverable is proof-on-Linux plus the missing recorded-body pins.

## Read first

1. `AGENTS.md` (provider rules — the footguns), `.opencode/skills/provider-adapter/SKILL.md`.
2. `cross-platform/docs/plans/2026-09-11-cross-platform-plan.md` — Epic C, C.1.
3. `cross-platform/docs/notes/window-managers.md` §4 (file-based credentials) and §6 (path
   table — near-certain column for Claude).
4. `codenotch/src/usage.rs` — the Claude core (`probe_credentials` ≈122, polling `start`
   ≈259, the 401 re-read at ≈286, expiry copy at ≈295) and its existing unit tests.
5. `codenotch/src/main.rs` — profile discovery (`~/.claude-<slug>`) and session reads
   (`~/.claude/sessions/*.json`).
6. `windows/codenotch/src/usage.rs` — read-only reference if a behaviour needs diffing.

## Scope (exactly this)

1. **Confirm every Linux path** against the live install: credential file, `~/.claude-<slug>`
   multi-login, sessions directory. Fix silently-diverging paths only (there may be none);
   every path gets a line in the report.
2. **Recorded-body pin**: capture one real `/api/oauth/usage` response (via the app's own
   fetch path with a local `tiny_http`-style recorder, or a raw copy of the response) into
   `codenotch/tests/fixtures/claude-usage.json`, and pin the parser test to it — the record
   is opaque, no secrets (tokens in the real response are the caller's, never commit them;
   if a capture contains a bearer token, redact and note it).
3. Failure mapping: every parser/network failure that the code can hit maps to a
   `ProviderStatus` the UI renders (stale/needsAuth/…). No zero-default, no invented
   percentage, and the sigil rules (`~` for derived/manual) hold.
4. Run the existing suite on Linux; add the one pin test if none exists for the usage parse.

## Dependencies allowed

None. A recorder harness may reuse the existing `tiny_http`/`ureq` code paths with no new
crate.

## Must NOT

- Read or log the live credential contents; commit any captured secret; touch Windows
  provider code paths; add a provider not already in the port.

## Definition of Done

- [ ] `cargo test --workspace` green incl. the new pinned test
- [ ] `./target/debug/codenotch doctor` on this host reports the real Claude credential
      path as valid/present.
- [ ] Report: the confirmed path table + what was verified live vs fixture-only.

## Orchestrator acceptance

Compile + test gates; fixture reviewed for leaked secrets; diff vs `windows/` import
regression.