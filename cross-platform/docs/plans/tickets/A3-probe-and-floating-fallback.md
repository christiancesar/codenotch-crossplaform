# Ticket A.3 — Runtime probe + floating fallback (GNOME Mutter / no layer-shell)

- Epic: **A** · Depends on: A.1 (strut path), A.2 (layer-shell path) · Lane: A
- Verifies on: `:1` here (X11) and, by report, on a Wayland session without layer-shell
  (GNOME Mutter) as an operator item.

## Goal

One startup probe picks the honest mechanism per session: **layer-shell → struts → float**,
with the universal fallback (floating always-on-top window, drag-to-edge) always available.
GNOME Mutter has no layer-shell for ordinary clients — the notch there floats, still opens,
animates and works. Nothing in this ticket invents behaviour: it selects among the three.

## Read first

1. `AGENTS.md` + `.opencode/skills/edge-pinning/SKILL.md` ("Recommended fallback order").
2. `cross-platform/docs/notes/window-managers.md` §1 (Wayland/Mutter) and §2 (the notch is
   the guaranteed entry point — relevant to no-tray stock GNOME).
3. `cross-platform/docs/plans/2026-09-11-cross-platform-plan.md` — Epic A + open decision 1.
4. `codenotch/src/main.rs` `place_notch` (72), `drag_begin` (152), `toggle_drag` (203),
   `notch_y` config, `reset_notch_position` (865) — the existing float machinery already
   works; keep it as the fallback.
5. The `window_layer/` modules from A.1/A.2.

## Scope (exactly this)

1. Centralize the probe: one `window_layer::Session` enum (`X11Struts | WaylandLayer |
   Float`) resolved from session vars + connect results. Expose `pub fn resolve()` used by
   the startup path and by `apply_visibility`.
2. Float path: reuse the current `place_notch`/`drag_begin`/`notch_y` behaviour **as-is**;
   ensure it is the branch taken when no layer-shell engaged and no strut could be set
   (e.g. bare X, Mutter, XWayland under Mutter). No new drag logic.
3. Persist nothing new: `notch_y` already persists; the mechanism is re-resolved at each
   placement, not remembered (same honesty rule as autostart).
4. Unit tests: `resolve()` decision matrix (fake env + fake connect results) pinned; and a
   "struts refused → float, no panic" test path.

## Dependencies allowed

None beyond A.1/A.2's.

## Must NOT

- Add a persistent "mode" preference; hard-code a compositor name — decision must come from
  the probe; touch the Windows placement path.

## Definition of Done

- [ ] Compile + `cargo test` green; probe matrix unit-tested.
- [ ] On `:1` (X11 + WM): app starts on the strut path (A.1 verified); `applog` records the
      chosen `Session`.
- [ ] Simulated refusal (XGD_SESSION_TYPE unset / fake broken layer result) → Float chosen,
      app still opens and places.
- [ ] Report the GNOME-Mutter-float verification as the operator item it is — not claimed.

## Orchestrator acceptance

Compile gates + diff review; run the refusal simulation; confirm nothing Windows changed.