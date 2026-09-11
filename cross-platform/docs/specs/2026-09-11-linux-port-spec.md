# Codenotch for Linux — Port Spec

Source of truth for the UI remains the upstream design spec and its frames:
`docs/specs/2026-08-28-usage-notch-design.md`, `docs/design/frame-124-hover-tooltip.png`,
`docs/design/frame-125-detail.png`. This document describes only what the Linux port
**keeps** and what it **changes** to run on Linux — everything not listed here is
inherited verbatim.

Scope: a Rust + Tauri 2 port following the `windows/` tree's approach (no shared code with
`Sources/`; providers reimplemented from documented behaviour). Status of this port at
writing: planning.

## Non-negotiables (carried from CONTRIBUTING.md)

These are the project's rules and they port unchanged, because they are the product:

1. **Fidelity is declared.** Every provider states `.official` / `.derived` / `.manual`.
   A `.derived` or `.manual` number gets a `~` in the UI. Never present a guessed number
   as official.
2. **Every failure is a visible status.** `stale`, `needsAuth`, `accessDenied`,
   `credentialExpired`, `signedOutByOwner`, `nothingMetered`, `rateLimited`… — never an
   unrenderable error, never a made-up percentage, never a missing window defaulted to
   zero.
3. **Archived readings come back dimmed.** The last good reading survives relaunch and is
   presented `.stale` with its age, never as live.
4. **No OAuth flow of its own.** Codenotch borrows the owning tool's credential; the only
   real sign-in is a `WebSessionProvider` modal (DeepSeek).
5. **i18n is key-based and per-port.** The English string is the key. The Linux port has
   its own dictionary (like `windows/codenotch/src/i18n.rs`) and must never be merged with
   `Sources/Localizable.xcstrings`.

## What a provider must provide

The Rust analogue of `Sources/Providers/UsageProvider.swift` — this shape is what couples
every provider to the model, port it as-is:

```
Provider   { id, displayName, glyph, account(), signInRoute, signOut() }
Snapshot   { providerID, windows: [LimitWindow], status, fetchedAt }
LimitWindow{ id, label, usedFraction?, usedCount?, resetsAt?, fidelity }
```

- `usedFraction` and `resetsAt` are optional exactly like upstream — a provider that
  reports only a count (no denominator, no reset) renders the count with an empty track,
  never an invented percentage.
- The ring always means the **headline window by name** (`headlineID`), not "the biggest
  window" — a session window that rolls over must not silently promote the weekly into its
  place. Copy the upstream discipline here.

## The notch surface

Same visuals, same interaction model:

- Inverse-rounded pill on a screen edge, pure black, no border.
- One provider cell per tracked account; ring drawn from 12 o'clock, clockwise; colour
  bands from the design frame (green / yellow / orange, full ring at 100%).
- Hover card to the left of the notch with a tail aligned to the cell, clipped contents,
  and the same "fewer things move than you think" morphing rule (contents keep their
  identity, only masks animate).
- Click a ring → refetch that provider only. Click the body → keep open. Right-click →
  menu (Refresh now, Keep open, Sign in…, Quit). Option-drag to move along the edge.
- Activity: the thin inner arc spins while a session is busy, pulses amber when it is
  waiting on you.

### The Linux delta: the edge

The only real visual change is **how "welded to the edge" is achieved**, and it is not one
answer on Linux — it depends on the display server:

- **X11:** `_NET_WM_STRUT_PARTIAL` reserves the edge the way a panel does, so the notch
  truly owns it and windows tile around it. Achievable with always-on-top + struts.
- **Wayland:** a plain window cannot be positioned onto an edge by the client. The native
  mechanism is the `zwlr_layer_shell` protocol (used by bars like waybar) — supported on
  wlroots compositors (sway, hyprland…) and KDE KWin, **not** on GNOME Mutter. On Mutter,
  a notch falls back to a floating always-on-top window, or to a Mutter extension.

Full matrix, the recommended strategy and the fallbacks are in
[`docs/notes/window-managers.md`](docs/notes/window-managers.md). The spec decision is:
**X11 always exact; Wayland exact where layer-shell exists; everywhere else a floating
always-on-top pill.**

## Settings, persistence, durability

- Settings window with provider list, drag-to-reorder, per-provider connect/configure,
  appearance (size, accent), What's New — the same set as upstream, behind a tray icon on
  Linux (see notes: GNOME's tray requires the AppIndicator extension).
- XDG layout via the `dirs` crate (as the `windows/` port already uses): config in
  `~/.config/codenotch/`, archived readings + local glyph overrides in
  `~/.local/share/codenotch/`. Restored readings must obey rule 3.
- Local runtime adapters (Ollama, LM Studio) poll the same localhost ports (11434 / 1234);
  1 s or 2 s timers are fine because they are local, but keychain-held credentials still
  go through a cache + modified-at probe, exactly to avoid purpose-built prompt loops.

## Notifications, chimes, session ends

- The notch opens for five seconds and warns when an agent stops working or waits on you —
  same events, fired through `org.freedesktop.Notifications` (libnotify) on Linux.
- Threshold alerts at 80% / 100%, once per crossing, per-provider mute.

## Packaging and updates

- Bundle targets: `.deb` / `.rpm` / AppImage are the three Linux expectations; Tauri 2
  supports all three. Deb/RPM integrate with the distro's own updater.
- Self-update is the **open question**: Sparkle does not exist on Linux. Options to weigh:
  (a) AppImage + AppImageUpdate-style self-update against a signed feed (closest to the
  macOS model); (b) ship to distro repos / Flatpak and let the platform update; (c) check
  for updates from a feed and long-link to packages. Decide before M6; write it into the
  plan.

## Compatibility targets (test matrix)

- Display server: X11 (GNOME, KDE, any), Wayland compositors — at least one wlroots
  (sway/hyprland) and one non-layer-shell (GNOME Mutter) to prove the fallback.
- Session: the two Claude Code login profiles and Codex profiles must behave as upstream
  documents them.
- Headless CI for provider parsers (recorded bodies), with home-dir-level plumbing so a CI
  host has no keyring, no running editor, no live accounts.

## Non-goals

- No window/tab focusing via tty scripting — the macOS app raises the app, and Linux has
  even less general per-terminal scripting; keep the "clicking brings the owning app
  forward" behaviour and the tooltip names the session.
- No migration of macOS or Windows state files; per-port keyed stores stay per-port.
- No merging of the Rust i18n files with the Swift catalog.