# Linux platform notes — the hard parts

The providers and the model port almost mechanically from `windows/`. The things that
make Linux genuinely different are all about the **desktop environment**: how a window
gets pinned to a screen edge, where the tray icon lives (and whether it exists), where a
notification is delivered, and what a "keychain" means. This file is the working notes an
agent or contributor should read before touching the window layer.

## 1. Edge pinning — the core difficulty

The macOS app welds itself to the physical screen edge with an `NSPanel` at the status-bar
level, joining all spaces, over fullscreen apps. There is no single Linux equivalent; the
answer depends on the display server, and a port has to pick per-environment.

### X11 — exact, panel-like

- The standard mechanism for "this window owns a screen edge" is the EWMH property
  `_NET_WM_STRUT_PARTIAL`: it *reserves* the edge, so maximized windows tile around the
  notch exactly like a panel. This is what system trays use.
- Combined with an always-on-top, undecorated, skiptaskbar window, X11 gives parity with
  macOS: the notch sits flush on the edge and keeps space.
- Implementable in Rust through `x11rb`/`xcb`, or by a small GTK helper setting the strut
  on the window's XID. Feasible, contained, well-documented surface.

### Wayland — where it gets real

A Wayland client **cannot position or raise its own window**. Two routes exist:

- **`zwlr_layer_shell`** (wlr-layer-shell protocol): the compositor-native way to ask for a
  *layer* docked to an edge (bars, overlays). Supports exclusive zones (`_NET_WM_STRUT`
  equivalent) and keyboard/mouse constraints. Supported by wlroots-based compositors
  (sway, hyprland, river…) and by KDE KWin (6.2+). This is the honest implementation for
  those.
- **GNOME Mutter** offers no edge-docking surface to ordinary clients and no layer-shell.
  A notch there is a floating always-on-top window, or requires a Mutter extension
  (fragile, version-sensitive — same trade as a browser extension, and not something to
  build v1 on).
- Recommendations: implement X11 struts first (works everywhere on X), then layer-shell
  for the compositors that speak it, and accept "floating always-on-top, drag to edge"
  as the universal fallback. Framereless/transparent/always-on-top windows are all
  supported by Tauri on Linux out of the box.

### Tiling window managers

- tiling WMs (i3, sway, dwm, bspwm…) may ignore or fight an always-on-top floating window.
  layer-shell is the *right* shape here, which is a reason to bias the fallback ordering
  toward it. This is a per-WM test item, not a design blocker.

## 2. Tray icon / status area

- Linux has no macOS menu bar. The equivalent is the **StatusNotifierItem / AppIndicator**
  protocol (`org.kde.StatusNotifier`), provided by `libayatana-appindicator`.
- Tauri 2's `tray-icon` feature builds on it and ships a working tray on KDE, XFCE, and
  GNOME **with** the "AppIndicator/KStatusNotifierItem Support" extension enabled.
- Default GNOME (default Ubuntu, default Fedora Workstation) hides it without the
  extension. Consequence: on a stock GNOME setup the only guaranteed entry point is the
  notch itself; the app must stay usable with no tray at all (right-click menu + settings
  reachable from the notch, matching the macOS "the notch *is* the UI" rule).

## 3. Notifications and sounds

- Desktop notifications: `org.freedesktop.Notifications` (libnotify semantics) — the `notify-rust`
  crate is the standard binding, and it is a good fit for the "once per crossing, muted per
  provider" threshold alerts.
- The macOS app learned the hard way that a system *sound* channel can exist while staying
  silent (`NSSound` returns success and nothing plays). On Linux the analogous trap is that
  notification *urgency* and *sound* are compositor/distro-dependent — assume sound is
  unreliable, and keep the chime optional and non-critical.
- Session-end chime/peek: same events as upstream, but test per desktop shell; GNOME and
  KDE differ in how a notification is shown and whether it can be accompanied by sound.

## 4. "Keychain" on Linux = Secret Service

- macOS Keychain maps to the **Secret Service API** (`org.freedesktop.secrets`) — the
  `secret-service` crate, or libsecret. It is *not always available*: headless hosts, X11
  sessions with no gnome-keyring/keepassxc unlocked, and CI have no secrets service.
- The upstream discipline (read the borrowed credential only when the owning tool changed
  it, cache aggressively, degrade to `needsAuth` rather than prompt-loops) maps directly.
- **Most providers are already file-based on Linux** because the owning tools are
  cross-platform Node/Go CLIs: Claude Code's `~/.claude/.credentials.json`, Codex's
  `~/.codex/auth.json`, Grok's `~/.grok/auth.json`, OpenCode's own key, `gh`'s stored
  session — the same files the `windows/` port reads. That means the keyring is, at most, a
  **fallback** on Linux, not the primary store. Prefer the file the owning tool uses.
- `~/.claude-<slug>` and `~/.codex-<slug>` multi-login discovery: port as-is; the
  directory conventions are identical across platforms.

## 5. Config, data, autostart

- Use the `dirs` crate (the `windows/` port already does): config →
  `~/.config/codenotch/`, data/archive → `~/.local/share/codenotch/`.
- Autostart = an XDG `desktop` file in `~/.config/autostart/` (`.desktop` with
  `X-GNOME-Autostart-enabled=true`). No `SMAppService` analogue; write the file yourself,
  and read back *from the filesystem* rather than from a remembered preference — the same
  honesty rule the macOS app applies to its own launch-at-login.
- Multi-monitor: X11 can walk screens via `randr`; Wayland again depends on the compositor.
  Slide the notch-follows-active-screen behaviour to a later milestone and treat it as
  per-environment.

## 6. Provider paths to confirm on Linux

Facts stated here that are not yet verified against a live Linux install — the port must
confirm each before coding the adapter (check the `windows/` port's README and the
upstream tests for the canonical file shapes):

| Provider | Expected Linux location | Status |
|---|---|---|
| Claude Code | `~/.claude/.credentials.json`, sessions `~/.claude/sessions/*.json` | near-certain (cross-platform CLI) |
| Codex | `~/.codex/auth.json`, rollout logs | near-certain |
| Grok | `~/.grok/auth.json` | to confirm |
| OpenCode | own key, as stored by OpenCode | to confirm |
| GitHub Copilot | `gh` auth (`~/.config/gh/`) | near-certain |
| Cursor | `~/.config/Cursor/User/globalStorage/state.vscdb` | confirmed |
| LM Studio | `~/.lmstudio/server-logs` + SDK socket on 1234 | near-certain (cross-platform app) |
| Ollama | localhost 11434; relay for speed/thinking | certain |
| Antigravity | `agy` CLI if shipped for Linux; else local language-server bridge | to confirm |

## 7. Processes, liveness, session monitoring

- `ProcessLiveness` maps to `/proc/<pid>` + start-time comparison (`/proc/<pid>/stat`
  field 22) — same pid-reuse concerns as macOS; the UTC-timezone lesson from the macOS
  port applies to any file that carries a ctime string.
- Directory watching: `notify` crate (already used by the `windows/` port) replaces
  `DispatchSource`. SQLite reads of a running tool's WAL database need the same
  `mode=ro`-then-`immutable` fallback the macOS port fought for.

## 8. Updates without Sparkle

- Sparkle is macOS-only. Options, to be decided in the plan before M6:
  - AppImage + a self-update path against a signed feed (closest to the current model,
    but AppImage has no universal packaged-update convention).
  - Distro packaging (deb/rpm/Flatpak) and let the platform's own updater own the channel —
    the most "Linux-native", least Codenotch-controlled.
  - Feed the version and link out to packages (start simplest, upgrade later).
- Whatever is chosen must preserve the user-facing promise: *visible* disclosure in
  settings, and nothing installs that wasn't signed by the maintainer.

## 9. Where the port keeps parity

The pieces that port 1:1 and need no special casing: polling cadence (60 s active / 5 min
idle), the persisted 429 back-off deadline (floor-raiser: 60 s → double → cap 15 min),
stale-dimmed archived readings, per-provider refetch on click, drag-to-reorder persisted
by id, and the English-key i18n dictionary.