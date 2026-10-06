# AGENTS.md

Personal rebuild of Codenotch for Linux and Windows (Tauri 2, Rust). It started as a fork
of vinzdg/codenotch but no longer has to stay compatible with upstream. The macOS app
and the old `windows/` port are gone; tag `archive/pre-rebuild-cleanup` has them if
something needs to be looked up.

## Where things are

- `cross-platform/codenotch` is the running app at v0.3.x. Treat it as the reference
  implementation while the rebuild happens: read it, port from it, fix critical bugs in
  it, don't restructure it in place.
- `cross-platform/codenotch-hook` is the tiny client Claude Code's hooks call. It talks to
  the app over `POST 127.0.0.1:<port>/event`, relaunches the app if it is down, and
  stays silent when the user quit from the tray (`user-quit` marker).
- `cross-platform/docs/specs/2026-09-15-cross-platform-architecture-spec.md` is the plan:
  a `create-tauri-app` scaffold (React + TypeScript + Vite) at the repository root, with
  `package.json`, `backend/` and `frontend/`, built beside `cross-platform/` until
  cutover. Follow its phases in order.
- All rebuild work goes on the single branch `feat/codenotch-next`, committed as it
  advances, merged into `development` only when the app fully works. Don't open extra
  branches for it.

## Build & test

- Reference app: `cd cross-platform && cargo test && cargo build`. Linux needs
  `libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev`.
- CI (`.github/workflows/cross-platform-ci.yml`) runs `cargo check` and `cargo test` on
  Ubuntu and Windows.

## Linux constraints worth knowing

- Ubuntu 26.04 / GNOME 50 is Wayland-only. The app forces `GDK_BACKEND=x11` (XWayland)
  because native Wayland ignores window positioning and GNOME has no layer-shell. Under
  XWayland the pointer position freezes off-window, so hover depends on the X input
  region being shaped to the notch's hot rectangles, not on polling the cursor.
- Hooks in `~/.claude/settings.json` point at `codenotch-hook` by name.

## Persisted data

Everything lives under `dirs::config_dir()/codenotch/`. Never overwrite a file that
failed to parse; the spec's Compatibility section is the gate for any change to stored
formats.

## Style

Comments explain why, not what. No premature abstraction.
