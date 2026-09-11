# Tickets — Codenotch cross-platform port

One file per unit of work, dispatched by the orchestrator to the `executor` agent pool.
The orchestrator owns this list, dispatches, reviews (optionally via the `reviewer`
agent), and merges. `windows/` stays read-only; the executor never runs `git add/commit`.

## Status

- Epic 0 (foundation) — **done**, gate re-verified 2026-09-11 on the Linux host:
  `cargo check --workspace` + `cargo test --workspace` green (`rustc 1.98.1`).
- Epic A (edge pinning) — ticketized below; A.1 is the first dispatch (X11 edge visible at
  the end of the epic). Live desktop available: `DISPLAY=:1`, a WM is running, no sway/
  hyprland yet (A.2 = compile + probe only on this host; real compositor run is an operator
  item).
- Epic C (providers) — ticketized below. Host has live Claude + Codex + Cursor credentials;
  no Antigravity install (C.4 is fixture-only). Headless-safe: the pinned suite runs on CI.
- Epics B/D/E/F — written at epic level in
  `../2026-09-11-cross-platform-plan.md`; ticketized when their dependency epic lands.

## Lanes / sequencing

```
Epic 0 (done)
 ├── Epic A (A.1 → A.2 → A.5 → A.3 → A.4)   # A.5 first if fractional-pixel ships in A.1's window
 └── Epic C (C.1 → C.6 → C.2 → C.3 → C.4 → C.5)  # C.5 touches watcher, keep last in the lane
```

Epics A and C are independent and may run in parallel lanes; the orchestrator keeps a
single lane unless the files touched are provably disjoint. `cargo` locks the target dir — two executors
building concurrently block each other, not corrupt.

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