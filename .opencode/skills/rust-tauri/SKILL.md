---
name: rust-tauri
description: How the Codenotch Rust/Tauri 2 codebase works — workspace layout, build and verify commands, platform gating, tauri.conf.json, capabilities, the vanilla UI, i18n keys, and diagnosis. Use when implementing, debugging, or reviewing anything under cross-platform/ (cargo, Cargo.toml, #[cfg], tauri, WebKitGTK, hook).
---

# Rust + Tauri 2 in Codenotch

## Workspace map

`cross-platform/` is a `cargo` workspace with two members:

- `codenotch/` — the Tauri 2 app (Rust backend + vanilla web UI).
- `codenotch-hook/` — a tiny companion binary (Claude Code session hook) that tells the app
  when a session starts, stops, or is waiting on you, over a localhost HTTP port.

`windows/` is the reference Windows port: the read-only source of truth. **Never edit it.**
Anything you change under `cross-platform/` should keep the Windows code paths byte-for-byte
identical — add `#[cfg(not(windows))]` branches, don't rewrite existing ones.

## Dependencies (codenotch/Cargo.toml)

- `tauri` v2 with `tray-icon` + `image-png`; `tauri-plugin-single-instance`.
- HTTP: `ureq` (blocking, json + native-tls). native-tls is deliberate: the Antigravity local
  bridge's self-signed cert is only accepted for 127.0.0.1.
- Local server: `tiny_http` (the hook talks to the app on a localhost port).
- Data sources: `rusqlite` (bundled SQLite C sources — the first build is 1–2 min slower) reads
  Cursor's `state.vscdb`; `dirs` for config/home paths; `notify` for file watching; `chrono`
  for timestamps; `png` for embedding provider marks as the app icon.
- `windows` v0.58 is a **Windows-gated** dependency
  (`[target.'cfg(windows)'.dependencies]`); its `features` list enumerates exactly the Win32
  surfaces used. Linux builds never pull it.

## Build / verify

```bash
cargo check                 # type-check both members
cargo build                 # debug; first build is slow (bundled SQLite)
cargo test                  # a real app launch, but tests never hit provider endpoints —
                            # they read recorded bodies; use ./target/debug/codenotch doctor
cargo fmt --check
```

Linux system deps before building: `libwebkit2gtk-4.1-dev libxdo-dev libayatana-appindicator3-dev
librsvg2-dev libssl-dev build-essential`. `tauri.conf.json` still declares `targets: ["nsis"]`
(Windows packaging) — flip it before any `tauri build`.

## Platform gating idiom

- Windows-only code: `#[cfg(windows)]` block, with `use std::os::windows::...` inside the block.
- The rest: `#[cfg(not(windows))]`, never `#[cfg(linux)]` — the code must keep compiling for other
  non-Windows targets too.
- Paths: on non-Windows use `dirs::config_dir()` → `~/.config/codenotch/` and
  `dirs::data_local_dir()` → `~/.local/share/`; on Windows `%APPDATA%\codenotch`.

## Config & capabilities

- `codenotch/tauri.conf.json`: two windows — `notch` (transparent, undecorated, alwaysOnTop,
  skipTaskbar, 340×460, starts hidden) and `settings` (520×620). `frontendDist: ui`,
  `withGlobalTauri: true`, `bundle.icon = ["icons/icon.ico", "icons/icon.png"]`.
- `codenotch/capabilities/default.json`: which Tauri commands and env the UI may invoke.
  **Any new `#[tauri::command]` must be allowed in a capability** or the UI cannot call it.

## UI

One vanilla HTML page per window: `codenotch/ui/notch.html`, `codenotch/ui/settings.html`.
Provider marks are SVG glyphs (`codenotch/glyphs/`) embedded from `@lobehub/icons-static-svg`
(MIT); users can override per-provider via the glyphs dir in the config folder.

## i18n

Rust-side (tray) strings in `i18n.rs` must use the same keys as the page; the English source
string is the key. Missing translations fall back to English. Auto locale: Windows reads
`GetUserDefaultLocaleName`; elsewhere `LC_ALL` → `LC_MESSAGES` → `LANG` (zh/ja/ko mapped, else en).

## Diagnosis

- `codenotch doctor` self-diagnoses credentials, data sources, icons, hooks.
- Logs live in the config dir (`%APPDATA%\codenotch` on Windows, `~/.config/codenotch` on Linux).
- Never log secrets; read borrowed credentials only when the owning tool changed them.