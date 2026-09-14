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
- CI: `.github/workflows/cross-platform-ci.yml` now runs `cargo check` + `cargo test` on
  `ubuntu-latest` and `windows-latest` for every push/PR touching `cross-platform/`.
- **Phase 1 closed 2026-09-14** (P1.1–P1.3 done + live desktop checklist passed — see the
  plan's Phase 1 section). Two new epics queued at the end of the roadmap, no dependency on
  anything above: **Epic G** (pt-BR i18n, `G1-ptbr-i18n.md`) and **Epic H** (OpenCode as an
  exploratory 5th provider, `H1-opencode-research.md` — research/decision ticket first, no
  code yet).

## Lanes / sequencing

```
Epic 0 (done)
 ├── Phase 1 (P1.1 → P1.2 → P1.3) — done
 ├── Epic A (A.1 done) → A.5 → A.2 → A.3 → A.4   # deferred to Phase 3
 ├── Epic C (confirm C.1 → C.6 → C.2 → C.3 → C.4 → C.5)  # C.5 touches watcher, keep last in the lane
 └── Phase 4, end of the queue, no dependency on the above: Epic G (i18n) ∥ Epic H (OpenCode)
```

Epic C tickets are independent of Phase 2/3 work and may run in parallel with it. Epic G and
Epic H are independent of everything and of each other (disjoint files: `i18n.rs`/
`settings.html` vs. a new `opencode.rs`) and can run in parallel whenever picked up.

## Orchestration model (revised 2026-09-14, twice)

The opencode multi-agent routing originally described here (OpenRouter → big-pickle →
opencode-go/Kimi/GLM, separate executor/reviewer *agent-definition files*) added more
planning/config overhead than code for a diff this size (6 of the first 8 commits on this
branch were plan/agent-config, not source) — that framing is what got simplified first.

What's actually in use now, proven across P1.1–P1.3: the orchestrator (Claude Code, in the
same sandbox as the real X11 desktop) dispatches each ticket headlessly via
`opencode run --agent codenotch-exec-opencode-go-kimi --auto "Read <ticket>.md and execute
it exactly as written... do NOT git add/commit"`, one ticket per `opencode run` process,
disjoint-file tickets launched in parallel in the background. The executor edits + self-checks
+ reports back in its own reply (no git writes); the orchestrator then reviews the real diff,
re-runs `cargo check`/`cargo test` itself, confirms `windows/` untouched, and commits. Only
if an opencode dispatch fails, is unavailable, or its output can't be trusted does the
orchestrator fall back to a Claude Code subagent (the `Agent` tool) for that ticket instead.

One real failure mode to guard against every time two opencode runs touch overlapping ground:
a P1.1 dispatch once ran a broad `git checkout -- <file list>` "cleanup" that included a file
P1.2 (running in parallel) had uncommitted changes in, destroying them (recovered by replaying
the destroyed session's own edit history via `opencode export`; see the plan's commit history
around `d1635d9`). Every dispatch prompt since explicitly forbids `git checkout`/`reset`/
`stash` on anything outside the ticket's own scope — keep that instruction in every future
dispatch, not just when it's remembered.

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

Superseded — see "Orchestration model" above for what's actually running. The
`mobile`/`api`-style multi-lane pool (OpenRouter primary, local big-pickle
fallback, a separate opencode-go/Kimi/GLM orchestrator) is still documented
in `agent-pool-example.md` as a reference pattern, but this branch's own
orchestrator is Claude Code itself, dispatching straight to a single
opencode executor agent per ticket.

## Invariants from AGENTS.md that every executor re-reads

- Never invent a number; every failure is a visible `ProviderStatus`; archived readings come
  back `.stale` + dimmed.
- `account()` / `signInRoute` / `signOut()` are protocol requirements, never extension
  members.
- Comments explain *why*, never *what*. No premature abstraction.