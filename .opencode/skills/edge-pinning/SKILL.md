---
name: edge-pinning
description: How the Codenotch notch attaches to a Linux screen edge — X11 struts, Wayland layer-shell, fallbacks, tray visibility, notifications, and Secret Service. Use when working on window positioning, always-on-top/tiling behavior, multi-monitor, tray, or anything in the window layer under cross-platform/.
---

# Edge pinning on Linux

The macOS app welds itself to the physical screen edge with an `NSPanel` at status-bar level
(joins all spaces, floats over fullscreen apps). Linux has no single equivalent — the answer
is per display server.

## X11 — exact, panel-like

- `_NET_WM_STRUT_PARTIAL` (EWMH) *reserves* the edge: maximized windows tile around the notch
  exactly like a panel. This is the system-tray mechanism.
- Combined with always-on-top, undecorated, skip-taskbar, this gives macOS parity on X.
- Implement with `x11rb`/`xcb`, or a small GTK helper that sets the strut on the window's XID.

## Wayland

A Wayland client cannot position or raise its own window. Two routes:

- **`zwlr_layer_shell`** — compositor-native "docked layer" with exclusive zones (the strut
  equivalent) and input constraints. wlroots compositors (sway, hyprland, river…) and KDE
  KWin 6.2+ support it.
- **GNOME Mutter** has no edge-docking surface and no layer-shell for ordinary clients: the
  notch there is a floating always-on-top window, or a fragile Mutter extension — not v1
  material.

## Recommended fallback order

1. X11 struts — works on all X.
2. layer-shell — for compositors that speak it (tiling WMs too; they may fight an
   always-on-top float).
3. Universal fallback — floating always-on-top window, drag-to-edge.

## The window layer around the notch

- **Tray** = StatusNotifierItem/AppIndicator (`libayatana-appindicator`). Stock GNOME hides the
  tray without the extension → the notch itself must stay the guaranteed entry point
  (right-click menu + settings reachable from the notch). The app must work with no tray at all.
- **Notifications**: `org.freedesktop.Notifications` via `notify-rust`; notification sound is
  unreliable across shells (like macOS `NSSound` silently no-op'ing) — keep the chime optional
  and non-critical.
- **"Keychain"** = Secret Service (`org.freedesktop.secrets`) — not always present (headless
  hosts, X11 without gnome-keyring, CI). Cache aggressively, degrade to `needsAuth`, never
  prompt-loop.

## Sources of truth

- `cross-platform/docs/notes/window-managers.md` — the full working notes.
- `cross-platform/docs/plans/2026-09-11-cross-platform-plan.md` — Epic A (edge pinning), B
  (tray/autostart/settings).