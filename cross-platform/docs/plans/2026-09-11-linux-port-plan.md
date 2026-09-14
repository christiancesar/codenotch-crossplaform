# Codenotch for Linux — Implementation Plan

> **Superseded.** Folded into `2026-09-11-cross-platform-plan.md` — the effort
> is now a single cross-platform codebase in this directory. Keep this file as
> historical notes; do not execute from it.

Spec: `docs/specs/2026-09-11-linux-port-spec.md`
Platform notes: `docs/notes/window-managers.md`
Started: 2026-09-11

Ordered so that something visible runs on a screen edge by the end of M2, every later
milestone can be cut without leaving the app broken, and the two platform questions that
carry the most risk (edge pinning, updates) are front-loaded enough to fail cheaply.
Each milestone lists what it depends on.

## M0 — Project skeleton

- [ ] Cargo workspace in `linux/` mirroring `windows/`: `codenotch/` (Tauri 2 app), no
      hook crate yet (a Claude Code hook messenger is not needed to read sessions on
      Linux — confirm against the Windows hook's scope before adding one)
- [ ] `codenotch/Cargo.toml`: `tauri` + `tauri-plugin-single-instance`, `serde`/`serde_json`,
      `dirs`, `notify`, `ureq`, `rusqlite` (bundled), `chrono`; `secret-service` and
      `notify-rust` deferred to M5/M6
- [ ] `tauri.conf.json`: transparent, undecorated, always-on-top, skip-taskbar `notch`
      window + a normal `settings` window; `productName` Codenotch, identifier
      `com.codenotch.linux`
- [ ] `ui/notch.html` + styles — single-file page, no framework, same as the Windows port
- [ ] `.gitignore` / `.gitattributes`, and a `cargo build --release` that runs clean on a
      stock Debian/Ubuntu host (note: parts of M2 need X11 running; keep CI headless-only
      for provider tests)

**Done when:** `cargo build --release` succeeds and launching puts a blank always-on-top
notch-sized pill on the right edge of an X11 desktop.

## M1 — The notch surface (window layer)

- [ ] Window positioning module: X11 struts (`_NET_WM_STRUT_PARTIAL`, via `x11rb`) so the
      edge is reserved and the pill sits flush on it
- [ ] Wayland: `zwlr_layer_shell` client for wlroots compositors and KDE KWin; probe for
      support at startup and fall back to a floating always-on-top window elsewhere
- [ ] Edge-state machine: collapsed pill ↔ open notch, fold/unfold springs, click-to-keep
      open, option-drag along the edge, edge positions persisted
- [ ] "The floating notch" lesson from the macOS port: round the window frame to whole
      pixels and read the real size back — no fractional-width content
- [ ] Click-through outside the notch body; hit region expands only while showing

**Done when:** on X11 the pill is flush on the edge and windows tile around it; on sway/
hyprland it docks via layer-shell; on GNOME Wayland it floats and still opens/animates
correctly.

## M2 — Provider cells (static data)

- [ ] `ProviderRing`, `ProviderCell`, colour bands from the design frame (green/yellow/
      orange, full ring + dimmed glyph at 100%)
- [ ] `UsageBand` / state rendering: `.stale` dims, `.derived`/`.manual` get `~`, a blank
      window shows a dash — copy the upstream drawing model
- [ ] Fixtures at 73% / 21% / 52% to match `frame-124-hover-tooltip.png`
- [ ] Snapshot/render check against the two design frames

**Done when:** the running app is pixel-close to `frame-124-hover-tooltip.png` minus the
tooltip, driven by hard-coded fixtures.

## M3 — Hover tooltip

- [ ] `TooltipCard` + tail aligned to the hovered cell; clamped to stay on screen for the
      top/bottom cells
- [ ] 180 ms spring in, 250 ms grace out; contents keep identity, only the mask animates
      (the macOS "fewer things move than you think" rule)
- [ ] Per-window rows: label + reset copy (relative under an hour, absolute beyond), 4 pt
      track bars, count-only rows render with an empty track

**Done when:** hovering each cell reproduces `frame-125-detail.png`, and moving from cell
to card does not dismiss it.

## M4 — Data layer and providers

Depends on the provider-path confirmations listed in `docs/notes/window-managers.md`.

- [ ] `UsageProvider` shape (Rust trait) + `Snapshot` / `LimitWindow` / `Status` model,
      mirroring `windows/codenotch/src/usage.rs`
- [ ] `UsageStore` equivalent — polling loop, persisted 429 back-off (60 s → 15 min cap,
      deadline persisted), `Refresh now`
- [ ] `UsageArchive` — last-good reading survives relaunch, comes back `.stale` + dimmed,
      never as live
- [ ] **Claude** — `~/.claude/.credentials.json` + `GET /api/oauth/usage`, one ring per
      `~/.claude-<slug>` profile; activity from `~/.claude/sessions/*.json`
- [ ] **Codex** — `~/.codex/auth.json` → `wham/usage`, rollout-log fallback, one ring per
      `~/.codex-<slug>` profile
- [ ] **Cursor** — editor `state.vscdb` (WAL: `mode=ro` first, `immutable` fallback) →
      `cursor.com/api/usage-summary`
- [ ] **GitHub Copilot** — `gh` session; **Ollama** and **LM Studio** local runtimes
      (11434/1234 + `~/.lmstudio/server-logs`)
- [ ] Remaining cloud providers (GLM, Grok, OpenCode, Command Code, Antigravity) —
      file-based, same order as upstream; **DeepSeek** web-session login via the app's own
      WebView (verify WebKitGTK WebView login works in Tauri before scheduling)
- [ ] Provider parser unit tests pinned to recorded bodies, following `Tests/`
      conventions from CONTRIBUTING.md

**Done when:** the Claude cell moves on its own as you use Claude Code, and an offline /
expired-credential machine shows dimmed stale rings instead of blanks or guesses.

## M5 — Settings, persistence, tray, autostart

- [ ] Settings window: provider list + drag-to-reorder (stored by id, unknown ids
      appended), per-provider connect/configure, appearance (size, accent), What's New
- [ ] Tray icon (AppIndicator) with the same right-click menu — refresh, keep open,
      sign in…, quit; app stays fully usable with no tray (stock GNOME)
- [ ] XDG autostart `.desktop`; read the launch state back from the filesystem, not a
      remembered preference
- [ ] XDG config/data layout via `dirs`; archived readings, order, overrides in
      `~/.local/share/codenotch/`

**Done when:** a provider can be added, removed, reordered and configured without a
rebuild, settings survive relaunch, and launch-at-login is honest about its own state.

## M6 — Notifications, session ends, threshold alerts

- [ ] Session-end peek/chime: notch opens for five seconds + sound when an agent stops
      working or waits on you (test per desktop shell; sound is best-effort)
- [ ] Threshold alerts 80% / 100% via `org.freedesktop.Notifications`, once per crossing,
      per-provider mute
- [ ] Activity indicator inside provider rings (busy spin, waiting pulse)

**Done when:** finishing a Claude Code turn opens the notch and warns once, and a crossed
threshold notifies exactly once until the window rolls over.

## M7 — Polish, packaging, updates

- [ ] Multi-monitor (X11 via randr; Wayland per-compositor as feasible)
- [ ] Reduced-motion handling; app icon; README screenshots
- [ ] Packaging: `.deb` / `.rpm` / AppImage
- [ ] **Updates decision** (see spec §"Packaging and updates" and
      `docs/notes/window-managers.md` §8): signed-feed self-update vs distro-managed vs
      link-out; with the same visible disclosure the macOS app makes

**Done when:** `make`-free installs on Debian/Ubuntu, Fedora and an AppImage build exist,
and the update story is written into the README.

---

## Open questions

1. **Depth of Wayland support.** Feature-detect layer-shell vs floating at runtime, or ship
   a compile-time default per compositor? (Recommendation: runtime probe, float fallback.)
2. **Stock GNOME tray.** Notch-only UX must be complete; is Settings still reachable without
   the AppIndicator extension? (Recommendation: yes — an orb/settings affordance attached
   to the floating notch, mirroring the macOS settings orb.)
3. **Cursor on Linux.** Confirmed installed? Session path under `~/.config/Cursor/`?
4. **DeepSeek web login** inside Tauri on WebKitGTK — verify before scheduling M4.
5. **Updates.** Self-update feed vs distro packaging — the one product-behaviour decision
   that isn't dictated by upstream.

## Risks

- **Data fidelity is the whole product.** Same rule as upstream: every adapter is pinned by
  recorded-body tests, and every failure degrades to a visible status. The parity risk is
  that a Linux file path differs silently — the provider-path table in the platform notes
  is the checklist.
- **Wayland fragmentation.** The single biggest scope risk. Mitigated by the X11-first +
  layer-shell-where-supported + float fallback ordering in M1.
- **WebView login** (DeepSeek) may not behave in WebKitGTK the way it does in macOS/Windows
  WebViews; cut it from v1 if it fights back, exactly as upstream keeps web-session
  providers behind an explicit opt-in.
- **No keyring in odd environments.** File-based stores on Linux mean the keyring is a
  fallback; never require it.

## Proposed directory layout

```
linux/
├── Cargo.toml            workspace: codenotch
├── codenotch/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── ui/notch.html     the pill + hover card (single file, no framework)
│   ├── glyphs/           provider marks
│   ├── src/              main, window-layer (x11_strut / layer_shell), tray,
│   │                     config, usage/store/archive, providers/*, session engine
│   └── tests/            recorded-body parser tests (mirror Tests/ by concern)
└── docs/                 this directory (spec, plan, platform notes)
```