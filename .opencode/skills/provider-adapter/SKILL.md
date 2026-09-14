---
name: provider-adapter
description: Codenotch provider adapters — the four providers (Claude, Codex, Cursor, Antigravity), their data sources, fidelity rules, and how to change or add one without breaking the "never invent a number" contract. Use when a provider response shape changes, when adding a provider, or when debugging a provider readout.
---

# Provider adapters

## The four providers

| Cell | Data source | Notes |
|---|---|---|
| **Claude** | `GET https://api.anthropic.com/api/oauth/usage` with the token Claude Code keeps in `~/.claude/.credentials.json` | Session/weekly windows; 429 back-off with a persisted deadline; stale readings dimmed with age; session-activity arc + amber "waiting on you" pulse via Claude Code hooks. |
| **Codex** | `https://chatgpt.com/backend-api/wham/usage` with the session Codex keeps in `~/.codex/auth.json` (read only, never refreshed); fallback to `rate_limits` in the newest rollout log | Live 5h/weekly (paid) or monthly window (free); otherwise the last snapshot marked stale by its own timestamp. |
| **Cursor** | The editor's own session from `state.vscdb` → `cursor.com/api/usage-summary` | Included/API/on-demand usage, reset at billing-cycle end. Borrows the editor's session — no sign-in of its own, one account only. |
| **Antigravity** | Official `agy` CLI `/usage` print when installed; else the legacy local `language_server` bridge, Google Cloud Code API, or transcript model count | Four official quota rows (Gemini & Claude/GPT 5h/weekly). CLI spawns in a hidden pseudo-console with a 70s timeout and process-tree cleanup; refresh gated to ≥5 min. |

Providers that are not installed simply have no cell.

## Non-negotiables

- **Fidelity**: every adapter declares `.official`, `.derived`, or `.manual`. Never present a
  guessed number as official.
- **No invented numbers**: never default a missing window to zero. A failed or missing reading
  becomes a rendered status (`stale` dimmed with age, `needsAuth`, `accessDenied`,
  `credentialExpired`, `signedOutByOwner`, `nothingMetered`) — never a fabricated value.
- `account()` and `signInRoute`/`signOut()`/`forgetCachedCredential()` are protocol
  requirements, not extension members — extension members dispatch statically through
  `any UsageProvider` and silently no-op.
- No OAuth flow of its own: the app borrows the owning tool's credential.

## Changing or adding a provider

1. **Pin with a recorded-body test**: capture the real (sanitized) response body, save it under
   the fixtures, and assert the parser output. If a vendor changes a response shape, that test
   fails first — add the same pin for any new provider parser.
2. Every failure path maps to a `ProviderStatus`/`UsageProviderError`, never a throw the UI can't
   render, and never a hoisted HTTP error code.
3. i18n keys on the Rust/tray side must stay identical to the page's dictionary.

## Where each lives

- macOS Swift app: `Sources/Providers/` (one adapter per provider, each with a `Fidelity`).
- Rust port (`cross-platform/codenotch/src/`): `usage.rs` (engine/aggregation), `codex.rs`,
  `cursor.rs`, `antigravity.rs`, `agy_cli.rs` (official CLI path), plus `state.rs` and
  `watcher.rs` around them.
- The two sides do **not** share code; keep the behavior aligned, not duplicated.