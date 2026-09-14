# Tickets — Codenotch cross-platform port

One file per unit of work, dispatched by the orchestrator to the `executor` agent pool.
The orchestrator owns this list, dispatches, reviews (optionally via the `reviewer`
agent), and merges. `windows/` stays read-only; the executor never runs `git add/commit`.

## Status (updated 2026-09-14 — see plan's "Reassessment" section)

- Epic 0 (foundation) — **done**, gate re-verified 2026-09-11 on the Linux host:
  `cargo check --workspace` + `cargo test --workspace` green (`rustc 1.98.1`).
- **A.1 done and live-verified**: X11 struts (`_NET_WM_STRUT_PARTIAL`) confirmed working
  on stock GNOME/Mutter via `xprop _NET_WORKAREA` (reserved width reflected). Realization-race
  bug fixed in `5c8d4e7`. Kept on by default.
- **A.2–A.5 deferred** until Phase 1 (below) closes — not blocking, cuttable without
  regressing anything already working.
- **Epic C is mostly path-confirmation, not new adapters** — the state engine
  (`state.rs`/`server.rs`) is already 100% cross-platform via hook events. `codenotch doctor`
  on this host already shows a valid Claude credential and working session reads; C.1–C.4 are
  now "confirm + add the recorded-body pin", not "port the provider".
- **New: Phase-1 tickets (P1.1–P1.3)** cover Linux stubs that block actually *using* the app
  (drag, no-activate, seen-clears-it, click-to-focus) — found by comparing every
  `#[cfg(not(windows))]` branch against its Windows counterpart. These are the real gate before
  any more Epic A/B/D/E work; a green `cargo check` was never proof the app was usable.
- Epics B/D/E/F — written at epic level in
  `../2026-09-11-cross-platform-plan.md`; ticketized when their dependency epic lands.
- **Known issue for B.1 (tray)**: the two-row digit bitmap in `trayicon.rs` (32x32,
  Windows-tuned — see its own doc comment on scaling assumptions) renders with too
  little vertical gap on a Linux panel/AppIndicator, so two readings (e.g. Claude "26",
  Codex "100") visually run together into one number ("26100"). Confirmed live via a
  real desktop screenshot on 2026-09-14. Not a data bug — needs the row gap/contrast
  tuned for Linux's rendering, deferred until B.1 is picked up.
- CI: `.github/workflows/cross-platform-ci.yml` now runs `cargo check` + `cargo test` on
  `ubuntu-latest` and `windows-latest` for every push/PR touching `cross-platform/`.

## Lanes / sequencing

```
Epic 0 (done)
 ├── Phase 1 (P1.1 → P1.2 → P1.3) — unblocks real desktop use, do this first
 ├── Epic A (A.1 done) → A.5 → A.2 → A.3 → A.4   # deferred to Phase 3
 └── Epic C (confirm C.1 → C.6 → C.2 → C.3 → C.4 → C.5)  # C.5 touches watcher, keep last in the lane
```

P1.1 and P1.2 touch disjoint files (`main.rs` vs `focus.rs`) and can run in parallel; P1.3
depends on P1.2 landing first (it wires `ack_scan` in `main.rs` to the new focus detection).
Epic C tickets are independent of Phase 1 and may run in parallel with it.

## Orchestration model (revised 2026-09-14)

The opencode multi-agent routing (OpenRouter → big-pickle → opencode-go/Kimi/GLM,
separate executor/reviewer agents) described below and in `agent-pool-example.md` added more
planning/config overhead than code for a diff this size (6 of the first 8 commits on this
branch were plan/agent-config, not source). Tickets now dispatch directly to Claude Code
subagents (the `Agent` tool, `general-purpose` type) running in the same sandbox as the real
X11 desktop, so a ticket's "Verify" step can actually be run live instead of reported
second-hand. The ticket anatomy (read first / scope / must not / definition of done / verify)
is unchanged — only the dispatch target is simpler.

## Ticket anatomy

Every ticket is self-contained:

- **Read first** — the exact files (plan, notes, skill, module) to read before touching code.
- **Scope** / **Must NOT** — what exactly to do and the guardrails (never `windows/`, no
  deps outside the ticket, `#[cfg(not(windows))]` not `#[cfg(linux)]`).
- **Definition of Done** — checklist the executor self-checks.
- **Verify** — commands + real-desktop probes; anything unverifiable is *reported, not
  claimed* (the honesty rule from AGENTS.md).
- **Orchestrator acceptance** — what the orchestrator (and optionally the `reviewer`) checks
  before the ticket counts as done.

## Flow

1. Orchestrator writes/approves the ticket.
2. Dispatches exactly one ticket to a fresh `executor` on the local opencode lane.
3. Executor edits + self-checks + returns `--outcome`.
4. Orchestrator (or the optional `reviewer` lane) compiles gates + diff review; on
   disagreement, `reviewer` gives a verdict.
5. Orchestrator merges (git) and updates this index + the plan.

Target operating model (same pool as `mobile`/`api`): primary model routing via
OpenRouter, fallback to local opencode (big-pickle), orchestrator driven by
opencode-go with Kimi 2 / GLM for planning/review/merge decisions.

## Invariants from AGENTS.md that every executor re-reads

- Never invent a number; every failure is a visible `ProviderStatus`; archived readings come
  back `.stale` + dimmed.
- `account()` / `signInRoute` / `signOut()` are protocol requirements, never extension
  members.
- Comments explain *why*, never *what*. No premature abstraction.