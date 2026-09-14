# Ticket A.2 — Wayland layer-shell docking (`zwlr_layer_shell`)

- Epic: **A** · Depends on: Epic 0, A.1 (strut module lands the `window_layer` shape) · Lane: A
- Verifies on: wlroots compositor (sway/hyprland) or KWin 6.2+ — **none installed on this
  host** (no sway/hyprland, no Wayland session). This ticket is compile + probe-logged on
  the host; the real-compositor run is an operator item explicitly listed in DoD.

## Goal

Under a Wayland session the notch docks to the edge as a *layer* with an exclusive zone —
the strut equivalent — on compositors that speak `zwlr_layer_shell` (wlroots: sway,
hyprland, river; KWin 6.2+). Tiling WMs that fight floating always-on-top windows get the
honest behaviour here.

## Read first

1. `AGENTS.md`, `.opencode/skills/edge-pinning/SKILL.md` (Wayland section).
2. `cross-platform/docs/notes/window-managers.md` §1 (Wayland) — the whole "no client can
   position or raise its own window" reasoning.
3. `cross-platform/docs/plans/2026-09-11-cross-platform-plan.md` — Epic A, and the open
   decision "runtime probe vs compile-time default".
4. `codenotch/src/window_layer/` from A.1 (the module home you extend) and `main.rs`
   `apply_visibility`/window builder.

## Scope (exactly this)

1. In `window_layer/`: `wayland_layer.rs`, `#[cfg(not(windows))]`. When the session is
   Wayland, ask the compositor for a `zwlr_layer_shell` surface for the notch window:
   anchor right, `exclusive_zone` = the notch span, desired size equal to the pill
   (reuse the same size logic `place_notch` uses on Windows).
2. Preferred crate: `gtk-layer-shell` (the mechanism GTK bars like waybar use — turns the
   existing gtk window into a layer surface in place). If it fights the tauri/wry window
   (undecorated gtk quirks), fall back to `wayland-client` + `wayland-protocols` with the
   `layer-shell` protocol on a **separate** connection — and in that case also say in the
   report why the window content attach strategy is sound. Pick one; document the choice.
3. Do **not** break the compiled-on-this-host property: no Wayland compositor here means the
   module must degrade to the floating fallback (A.3) at runtime and never panic on connect
   failure.
4. Startup probe (this ticket lands the probe hook A.3 fills): `WAYLAND_DISPLAY`/session
   type → try layer-shell → log `layer-shell: engaged/compositor refused → float` to the
   unified log (`applog`).
5. Unit test: the probe/reject decision logic (pure function: session vars + connect result
   → engaged | float | x11) — CI-safe.

## Dependencies allowed

`gtk-layer-shell` (recommended) **or** `wayland-client` + `wayland-protocols`
(with `layer-shell` feature) + `wayland-backend`(s) as needed. Add exactly one route.

## Must NOT

- Touch `windows/`; pretend a Wayland verification happened on this host; ship a panic on
  layer-shell refusal; block startup when no compositor negotiates.

## Definition of Done

- [ ] Compiles on this host (`cargo check --workspace`, `cargo test` green).
- [ ] Probe degrades cleanly here: run the app under `:1` (X11) — it must take the X11/float
      path and not attempt layer-shell.
- [ ] Pinned unit test for the probe decision; generate only what the crate offers.
- [ ] Report (operator item): one sway or hyprland session where the notch docks with an
      exclusive zone and other windows tile around it. Do **not** mark this ticket done by
      claiming that verification.

## Orchestrator acceptance

Compile gates + diff review. The docked-on-compositor proof is recorded as a follow-up once
an operator provides/toys with a wlroots session; a purple "unverified live" marker stays in
the plan until then.