# Ticket A.4 — Click-through / hit regions on Linux

- Epic: **A** · Depends on: A.5 (true frame readback first) · Lane: A
- Verifies on: `:1` (X11). Wayland click-through is compositor-dependent; scope here is
  X11 + non-regression of the float path.

## Goal

Outside the notch body the window is transparent and click-through: the cursor falls to
whatever is beneath (this is what makes a notch usable). The hit region expands only while
the notch is showing (`set_hot`, `main.rs:362`, feeds the pointer watchdog at 483).
Today `set_click_through` (375) is the Windows approach; this ticket ports the same *effect*
to Linux.

## Read first

1. `AGENTS.md` (the design-frame rule; the notch itself *is* the UI — no tray on stock
   GNOME, so hover must be right).
2. `cross-platform/docs/notes/window-managers.md` §2 (GNOME no-tray → hover is the entry).
3. `codenotch/src/main.rs`:
   - `set_click_through` (375), `set_hot` (362), `cursor_in_hot` (436 + `HOT_PAD`
     ≈432), `start_pointer_watchdog` (483) and how the UI calls them.
   - `codenotch/ui/notch.html` (what the cursor actually hovers).
4. Tauri 2 docs for `set_ignore_cursor_events` (the cross-platform API meant for this) —
   check what it does on Linux (X11 input shapes) vs `WS_EX_TRANSPARENT` on Windows.

## Scope (exactly this)

1. On Linux use Tauri's `set_ignore_cursor_events(true)` (or, if it proves unusable here,
  an X11 input-shape over the transparent margin only — chose and justify). Wire it into
  `set_click_through` behind `#[cfg(not(windows))]`, keeping the Windows branch intact.
2. The hot/expanded region must still win: hovering the body keeps the notch responsive on
  X11 — verify the cursor watchdog and `cursor_in_hot` still fire with click-through on.
3. No change to `HOT_PAD`, leave timings (`WATCHDOG_MS` 50 / `LEAVE_MS` 300) alone.
4. Unit test only if you extract a pure decision (which region is hot for a given cursor
  point) — one function, CI-safe.

## Dependencies allowed

None expected. If X11 input-shape is needed and Tauri's API can't reach it, add
`xfixes`/`x11rb` on top of A.1's `x11rb` — allowed but justify in the report.

## Must NOT

- Change the Windows click-through path; alter hover timing/geometry; rewrite the watchdog.

## Definition of Done

- [ ] Compile + `cargo test` green; `cargo fmt --check` clean.
- [ ] On `:1`: with the notch visible, clicking the transparent margin next to the pill
      activates the window beneath (e.g. opens/raises a terminal) while clicking the pill
      body does not leak through; hover over the body still expands/holds the notch.
- [ ] Report what was verified live vs compiled (X11 only; Wayland deferred).

## Orchestrator acceptance

Compile gates + diff review; confirm Windows branch byte-identical.