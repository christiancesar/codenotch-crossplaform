# Codenotch (personal rebuild)

A usage notch for coding assistants on Linux and Windows: a small strip pinned to the
screen edge showing how much of each assistant's usage limit is left and whether a
session is still working.

This repository started as a fork of [vinzdg/codenotch](https://github.com/vinzdg/codenotch)
and is now a personal project. It no longer tracks or stays compatible with upstream.
The original macOS app and the separate Windows port were removed; the last commit that
still contains them is tagged `archive/pre-rebuild-cleanup`.

## Layout

- `cross-platform/` is the current Tauri 2 app (`codenotch`) and the hook client Claude
  Code calls (`codenotch-hook`). It is the reference while the app is rebuilt.
- `cross-platform/docs/specs/2026-09-15-cross-platform-architecture-spec.md` describes the
  rebuild: a `create-tauri-app` scaffold with a `backend/` (Rust) and `frontend/`
  (React + TypeScript + Vite) split and a generated, typed IPC contract.

## Build

Linux needs the WebKitGTK toolchain:

```sh
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev patchelf
cd cross-platform
cargo test
cargo build
```

## License

MIT, see `LICENSE`. The original copyright notice is kept as the license requires.
