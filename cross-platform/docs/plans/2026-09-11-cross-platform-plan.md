# Codenotch Cross-Platform (Windows + Linux) — Implementation Plan

- Supersedes: `2026-09-11-linux-port-plan.md` (folded in below)
- Spec: `docs/specs/2026-09-11-linux-port-spec.md`
- Platform notes: `docs/notes/window-managers.md`
- Started: 2026-09-11 · Status: Epic 0 done (gate re-verified 2026-09-11 on the
  Linux host), Epics A + C ticketized in `docs/plans/tickets/`, dispatch underway.
- Working mode: orchestrated — the orchestrator dispatches tickets to executor
  agents (the oikos-style executor pool) and validates their output.

## Goal / product

Turn the Windows Tauri 2 port into **one cross-platform codebase** that compiles
and runs on Windows **and** Linux, living in this `cross-platform/` directory
(= the former `linux/` + the Windows port copied in). The upstream `windows/`
tree stays untouched as the shipping/reference port until the maintainer reworks.

Requirements agreed with the maintainer-equivalent (this repo's operator):

1. **Compiler parity first** — the tree must build and unit-test on Linux, and
   must keep building on Windows (verified later on a Windows host/CI).
2. **Providers**: exactly the current 4 (Claude, Codex, Cursor, Antigravity).
   No Linux-only providers (Copilot/Ollama/LM Studio/DeepSeek) in this pass.
3. **Single Tauri config.** No `tauri.windows.conf.json`/`tauri.linux.conf.json`
   split: `tauri.conf.json` covers both. The only needed touch is adding
   `icons/icon.png` (256×256) to `bundle.icon` (Windows uses the `.ico`, Linux
   the `.png`; the list accepts both). Packaging target per OS is decided in
   Epic E (`--bundles` flags vs platform merge files).
4. The app deliberately runs no OAuth of its own and never invents a number:
   failures degrade to a visible status (`stale`/`needsAuth`/…), never a guess.
5. Docker-true parity with the design frames; layout changed only per
   `Design.px` and the macOS port's lessons (whole-pixel rounding, real-size
   readback).

## Structure

```
cross-platform/
├── Cargo.toml            workspace: codenotch, codenotch-hook
├── codenotch/            Tauri 2 app (copied from windows/, adapted)
│   ├── src/              main, window-layer (a1_*), tray, config, usage/store,
│   │                     providers/*, session engine, glyphs, doctor
│   ├── ui/notch.html     the pill + hover card (single file, no framework)
│   ├── glyphs/           provider marks (+ NOTICE.md)
│   └── icons/            icon.ico + icon.png
├── codenotch-hook/       <5 ms hook messenger (binary name per platform)
├── docs/                 spec, notes, plans, tickets
└── README.md             adapted per-OS pre-reqs
```

## Epics and tickets

### Epic 0 — Foundation: tree + compile parity

| Ticket | Delivery | Acceptance |
|---|---|---|
| T0.1 Toolchain & deps | Install rustup + apt per official Tauri v2 pre-reqs: `libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev` (no `libgtk-3-dev` needed — pulled transitively; `patchelf` deferred to Epic E / .deb bundling) | `pkg-config` finds all; `cargo`/`rustc` present |
| T0.2 Restructure | `cross-platform/` = this dir + Windows workspace copied in (Cargo.toml, Cargo.lock, codenotch/, codenotch-hook/, LICENSE, README adapted); merge `.gitignore`/`.gitattributes`; plan + tickets in `docs/plans/` | tree mirrors `windows/` + docs; `windows/` untouched |
| T0.3 Linux runtime behaviour | `main.rs` → `xdg-open` for `open_data_dir`/`open_provider_page`/`open_usage_page` (keep `explorer`/`cmd /C start` + `CREATE_NO_WINDOW` on Windows); `hooks_install.rs` → platform hook name; `codenotch-hook` → `read_port()` on `~/.config/codenotch/config.json` and `spawn_main()` without `.exe`; `i18n::resolve_auto()` from `$LANG`/`LC_ALL`; `agy_cli::find_agy()` and `codex::find_executable()` Linux candidates | `cargo check` clean; no Windows branch touched |
| T0.4 Icons | Add `icons/icon.png`; extend `bundle.icon` | `generate_context!` runs on Linux |
| T0.5 Verify | `cargo build` + `cargo test` headless on the Linux host | green; Windows build verified on host/CI as follow-up |

**Done.** Gate re-run 2026-09-11: `cargo check --workspace` + `cargo test --workspace`
green on the Linux host (`rustc 1.98.1`, X11 `:1` with a WM). Notes: the tray feature
compiles without the `appindicator3-0.1` pkg-config name (`libappindicator v0.9.0` was
pulled); `cargo tauri` CLI not installed — only needed for packaging (Epic E).

**Done:** `cross-platform/` compiles and tests on Linux; nothing Windows changed.

### Epic A — Edge pinning (was M1)

- A.1 X11 `_NET_WM_STRUT_PARTIAL` (x11rb) — reserve the right edge; flush
  placement; re-place on monitor change.
- A.2 Wayland `zwlr_layer_shell` — dock on wlroots (sway/hyprland/river) and
  KWin 6.2+.
- A.3 Runtime probe + floating fallback (GNOME Mutter) — always-on-top, drag to
  edge, persist position (reuse `place_notch`/`drag_begin`/`notch_y`).
- A.4 Click-through / hit regions on Linux — validate `set_ignore_cursor_events`
  on X11 vs Wayland.
- A.5 Whole-pixel rounding + real-size readback (macOS lesson).

**Done:** X11 tiles windows around the notch; sway/hyprland dock; GNOME floats
and still works.

### Epic B — Tray, autostart, settings, persistence (was M5)

- B.1 TrayIcon = StatusNotifierItem (tauri `tray-icon`) with the same menu; the
  app stays fully usable without a tray on stock GNOME (settings reachable from
  the notch, mirroring the macOS orb).
- B.2 XDG autostart: `~/.config/autostart/codenotch.desktop`
  (`Exec=<exe> --silent`, `X-GNOME-Autostart-enabled=true`); `is_enabled()`
  reads the filesystem, never a remembered preference.
- B.3 Settings window parity: provider drag-to-reorder by id (unknown ids
  appended), slot pickers, real-icon preview.
- B.4 Config/data layout via `dirs`: config `~/.config/codenotch`, data
  `~/.local/share/codenotch`.

**Done:** provider added/removed/reordered without rebuild; launch-at-login
honest; settings survive relaunch.

### Epic C — Providers & engine parity (was M4 core; all file-based already)

Adapt-and-verify tickets, each with a pinned recorded-body test:
- C.1 Claude: `~/.claude/.credentials.json` → `/api/oauth/usage`; one ring per
  `~/.claude-<slug>`; sessions from `~/.claude/sessions/*.json`.
- C.2 Codex: `~/.codex/auth.json` → `wham/usage`; rollout-*.jsonl fallback;
  stale >5 min.
- C.3 Cursor: confirm `~/.config/Cursor/User/globalStorage/state.vscdb` (WAL ro)
  → `usage-summary`.
- C.4 Antigravity: `agy` in `~/.local/share/agy/bin` / PATH + local bridge +
  transcript fallback; Credential Manager stays Windows-only.
- C.5 Session engine: `watcher.rs` (`~/.claude/projects` + desktop-app
  `local-agent-mode-sessions`), `state.rs` (4-state machine, unchanged
  constants), hook events via `server.rs` (port 48666).
- C.6 Archive/honesty: archived readings come back `.stale` + dimmed; 429
  backoff persisted (60 s → double → 15 min cap); pinned suite runs on CI.

**Done:** the Claude cell moves on its own; offline/expired shows dimmed stale,
never blank or guessed.

### Epic D — Notifications, sessions, activity (was M6)

- D.1 Threshold alerts 80%/100% via `notify-rust` (org.freedesktop.Notifications),
  once per crossing, per-provider mute.
- D.2 Session-end peek/chime: notch opens 5 s + best-effort sound (test per
  shell; sound optional).
- D.3 Activity indicator parity (busy spin / waiting pulse).
- D.4 Liveness via `/proc/<pid>/stat` field 22; priority/activity polling per
  platform.

**Done:** ending a Claude turn opens the notch and warns once; a crossed
threshold notifies exactly once until rollover.

### Epic E — Polish, packaging, updates (was M7)

- E.1 Multi-monitor (X11 randr; Wayland per-compositor best effort).
- E.2 Reduced-motion; app icon; README screenshots.
- E.3 Packaging deb/rpm/AppImage — decide the bundle-config strategy here
  (`--bundles` per OS vs platform merge files).
- E.4 **Updates**: signed-feed self-update vs distro-managed vs link-out;
  visible disclosure.

**Done:** `make`-free installs on Debian/Ubuntu, Fedora, AppImage; update story
written into README.

### Epic F — Governance

- F.1 CI Linux job (headless `cargo check`/`cargo test`) — connects to open
  upstream issues #147/#151.
- F.2 Draft upstream issue describing the cross-platform tree — **not submitted
  before local validation**.
- F.3 Documented sync rules `windows/` ↔ `cross-platform/`.

## Sequencing & dependencies

```
T0.1 → T0.2 → T0.3 → T0.4 → T0.5
                ├──→ Epic A ─────→ Epic B
                └──→ Epic C ─────→ Epic D ─→ Epic E
F.1 after Epic 0 · F.2 after Epic 0 validated · F.3 continuous
```

Each epic is cuttable without breaking the ones before it (a notch is visible
on an X11 edge by the end of Epic A). Nothing in Epic 0 depends on a graphical
session; CI stays headless for provider tests.

## Orchestration model and agent pool

This port uses the same orchestrated pool as the `mobile` and `api` projects:

- **Model routing**: primary lane is **OpenRouter**; fallback to **local opencode
  (big-pickle)** when routing, policy or cost requires it. For the cross-platform
  Rust/Tauri surface, executor work is currently handled by the local opencode lane.
- **Orchestrator**: **opencode (Go binary)** with a high-context model such as
  **Kimi 2** or **GLM**. The orchestrator owns this plan, the ticket list, diff
  review, and merge decisions.
- **Executor lane**: opencode executor subagents receive one ticket at a time,
  read the listed files first (AGENTS.md, CONTRIBUTING.md, the matched Windows
  sources in `windows/`, plus the ticket's plan/notes), self-check via compile +
  pinned tests + real-desktop probes, and report back.
- **Reviewer lane**: optional opencode reviewer subagent gates scope, platform
  gating, tests, and the AGENTS.md invariants before the orchestrator merges.
- **Merge**: the orchestrator runs compile gates and commits only validated
  output.

Workflow:

1. Orchestrator dispatches one ticket to a fresh executor.
2. Executor edits + self-checks + returns a report (per
   `.opencode/agent/executor.md`).
3. Orchestrator (or a reviewer) validates: `windows/` untouched, compile gates
   green, diff review, real-desktop claims verified or honestly reported.
4. Orchestrator merges and updates the ticket index.

*Note:* In this session the orchestrator is running on the local opencode
(big-pickle) lane while the OpenRouter/Kimi/GLM routing is recorded as the
target operating model. A concrete local-agent registry example (same pool
pattern as mobile/api, but scoped to this repo) lives in
`agent-pool-example.md` and in the updated `opencode.json` +
`.opencode/prompt/` files.

Work items land in `docs/plans/tickets/` (one file per ticket; see
`docs/plans/tickets/README.md` for the index, lanes and ticket anatomy).
Epics A and C are ticketized; B/D/E/F stay at epic level here until their
dependency lands.

## Open decisions

1. **Wayland depth** — runtime probe vs compile-time default (recommendation:
   probe + float fallback).
2. **Stock GNOME** — settings reachable without the AppIndicator extension
   (recommendation: notch-attached affordance, mirroring macOS).
3. **Cursor on Linux** — confirm `~/.config/Cursor/User/globalStorage/state.vscdb`
   on a real install.
4. **DeepSeek web login in WebKitGTK** — deferred/cut; stays out (4 providers
   only).
5. **Updates (E.4)** — the one product decision not dictated by upstream.
6. **Identifier** — keep `com.immidi.codenotch`.
7. **Windows verification** of `cross-platform/` — Windows host or CI.

## Risks

- **Data fidelity is the whole product** — every adapter pinned by recorded-body
  tests; the provider-path table in `docs/notes/window-managers.md` §6 is the
  checklist.
- **Wayland fragmentation** — biggest scope risk; mitigated by X11-first →
  layer-shell → float ordering.
- **WebKitGTK vs WebView2** (DPI, zoom, transparency) — validate early (T0.5
  shows the real window).
- **No keyring / no tray** in odd environments — file-based stores and a
  notch-only UX; never require either.