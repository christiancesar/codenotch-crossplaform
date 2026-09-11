# Ticket C.5 — Session engine on Linux (watcher, state, hook server)

- Epic: **C** · Depends on: C.1 (Claude session reads come first) · Lane: C — keep last in
  the lane: it touches the shared `watcher.rs`/`server.rs` surface.
- Verifies on: this host (live Claude present) for the watcher paths; headless-safe for the
  state machine.

## Goal

Session monitoring parity on Linux: `watcher.rs` watches `~/.claude/projects` (and the
desktop-app `local-agent-mode-sessions` dirs) and drives the 4-state machine in `state.rs`
(the constants stay unchanged); Claude Code hook events still reach the app through
`server.rs` on the same port (48666). The Windows process-focus raise in `focus.rs` is
already no-op'ed on non-Windows — verify, don't rewrite.

## Read first

1. `AGENTS.md` (session engine; never hit a provider endpoint from tests).
2. `cross-platform/docs/notes/window-managers.md` §7 (processes/liveness; UTC-timezone
   lesson) — relevant to any ctime-parsing in watcher.
3. `cross-platform/docs/plans/2026-09-11-cross-platform-plan.md` — Epic C, C.5.
4. `codenotch/src/watcher.rs` (~18K), `state.rs` (~9K, the 4-state machine), `server.rs`
   (~2.5K, port 48666), `focus.rs` (raise-on-click: Windows `#[cfg]`, non-Windows noop).
5. `codenotch-hook/src/` — the messenger (already platform-renamed in Epic 0).
6. `windows/codenotch/src/watcher.rs` / `state.rs` / `server.rs` — reference only.

## Scope (exactly this)

1. **Confirm Linux watch paths** vs a live `~/.claude` on this host: `projects/`,
   `local-agent-mode-sessions` as actually created. Fix only diverging paths; list them.
2. Watch-loop behaviour on Linux: `notify` crate already used; confirm recursive-watch
   config and debounce survive on Linux inotify semantics; no polling rewrites.
3. Port: `server.rs` must bind/parse the same 48666 contract on Linux; the hook messenger
   reports events and the app reads them (a loopback integration test is allowed — real
   port, ephemeral free port, no provider endpoints).
4. State machine: unchanged constants, and a state-transition unit test that runs on CI.
5. Focus: assert the non-Windows noop raise still returns the *desired* honest result
   ("takes you to the owning app") without tty scripting — no new focus machinery.
6. `cargo test` green, including `codenotch-hook`.

## Dependencies allowed

None.

## Must NOT

- Change the 4-state constants; add tty/`wmctrl` scripting for focus; touch
  `local-agent-mode-sessions` semantics outside path confirmation; touch `windows/`.

## Definition of Done

- [ ] Tests green (workspace incl. hook); `cargo fmt --check` clean.
- [ ] Watcher path confirmation table in the report; transition tests pinned.
- [ ] Hook→server loopback works on Linux; report states what was run vs compiled-only.

## Orchestrator acceptance

Compile + test gates incl. hook crate; diff shows only Linux-path/diff-gated changes.