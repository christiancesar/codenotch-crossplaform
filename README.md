# Codenotch (personal rebuild)

A usage notch for coding assistants on Linux and Windows: a small strip pinned to the
screen edge showing how much of each assistant's usage limit is left and whether a
session is still working.

This repository started as a fork of [vinzdg/codenotch](https://github.com/vinzdg/codenotch)
and is now a personal project. It no longer tracks or stays compatible with upstream.
The original macOS app and the separate Windows port were removed; the last commit that
still contains them is tagged `archive/pre-rebuild-cleanup`.

## Layout

- `crossplatform-v2/` is the app, from 0.4.0 on: a stock `create-tauri-app` layout with
  `src-tauri/` (Rust, Tauri 2) and `src/` (React + TypeScript + Vite), a generated, typed IPC
  contract (`src/libs/ipc/bindings.ts`), and a Storybook workshop for every component. Its docs
  are in `crossplatform-v2/docs/` (`progress.md`, `design-system.md`).
  It also builds `codenotch-hook` (`src-tauri/hook/`), the client Claude Code's hooks call,
  which the installers ship next to the app.
- `cross-platform/` is the v0.3 app, kept as the reference the rebuild was checked against.

## Build

Linux needs the WebKitGTK toolchain and Node 24:

```sh
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev patchelf
cd crossplatform-v2
npm ci
npm run tauri dev                              # run it
npm run storybook                              # the component workshop
npm run tauri build -- --bundles deb,appimage  # installers
(cd src-tauri && cargo test)                   # also regenerates the IPC bindings
```

## Releases

GitFlow: work lands on `development` through pull requests, then `development` goes to
`release` and `release` to `main`. A `v*` tag on `main` builds the Linux and Windows installers
and opens a draft GitHub Release for review.

## License

MIT, see `LICENSE`. The original copyright notice is kept as the license requires.
