# AGENTS.md

Personal rebuild of Codenotch for Linux and Windows (Tauri 2, Rust). It started as a fork
of vinzdg/codenotch but no longer has to stay compatible with upstream. The macOS app
and the old `windows/` port are gone; tag `archive/pre-rebuild-cleanup` has them. The v0.3
app, its architecture spec, plans and notes are gone too; tag `archive/v0.3-reference` has
them if something needs to be looked up.

## Where things are

- `crossplatform/` is the app, in the stock `create-tauri-app` layout: `src-tauri/` (Rust
  backend), `src/` (React + TypeScript + Vite frontend), `docs/` (`progress.md`,
  `design-system.md`, `phase0-compat-gate.md`).
- OS-specific code lives only in `src-tauri/src/platform/` (CI greps for `cfg(windows)` and
  friends anywhere else).
- `src/libs/ipc/bindings.ts` is generated from the Rust commands by `cargo test`; commit it
  with the Rust change, CI fails on a diff.
- `src-tauri/hook/` is `codenotch-hook`, the tiny client Claude Code's hooks call. It talks
  to the app over `POST 127.0.0.1:<port>/event`, relaunches the app if it is down, and
  stays silent when the user quit from the tray (`user-quit` marker).
- Upstream (vinzdg/codenotch) still has a maintained Windows port under `windows/`; its
  issues and PRs are worth searching before solving a Windows problem from scratch.

## Build & test

- `cd crossplatform && npm ci && npm run tauri dev`; tests with
  `cd crossplatform/src-tauri && cargo test`.
- Linux needs `libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev`.
  Windows needs the Visual Studio Build Tools (C++ workload, Windows SDK) and WebView2.
  Both need Node 24.
- CI (`.github/workflows/crossplatform-ci.yml`) builds the frontend and runs the tests on
  Ubuntu and Windows.
- Work goes on a branch with a pull request into `development`; `development` goes to
  `release` and `release` to `main`, where a `v*` tag cuts the release.

## Linux constraints worth knowing

- Ubuntu 26.04 / GNOME 50 is Wayland-only. The app forces `GDK_BACKEND=x11` (XWayland)
  because native Wayland ignores window positioning and GNOME has no layer-shell. Under
  XWayland the pointer position freezes off-window, so hover depends on the X input
  region being shaped to the notch's hot rectangles, not on polling the cursor.

## Claude Code

- The app wires its hooks into `~/.claude/settings.json` at start; entries are recognized
  by `codenotch-hook` in the command. They fire from the terminal CLI and from the Claude
  desktop app.
- Usage is read with the token in `~/.claude/.credentials.json`, which only the
  standalone `claude` CLI writes (the desktop app keeps its own); the app renews it by
  running that CLI.

## Persisted data

Everything lives under `dirs::config_dir()/codenotch/`. Never overwrite a file that
failed to parse; `crossplatform/docs/phase0-compat-gate.md` (fixtures and frozen
contracts) is the gate for any change to stored formats.

## Style

Comments explain why, not what. No premature abstraction.
