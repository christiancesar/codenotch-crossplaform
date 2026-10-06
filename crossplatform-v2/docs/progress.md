# Rebuild progress

Branch `feat/codenotch-next`. Phases from
`cross-platform/docs/specs/2026-09-15-cross-platform-architecture-spec.md`.

## Phase 0: compatibility gate (done)

See `phase0-compat-gate.md`.

## Phase 1: scaffold (done, visual check pending)

- Stock `create-tauri-app` 4.7.4 output committed unchanged first, then adapted.
- `tauri.conf.json`: `Codenotch` 0.4.0, identifier kept, `notch` and `settings` windows with
  the v0.3 properties, both loading `index.html`; `withGlobalTauri` off; CSP set.
- Icons generated from the v0.3 Linux icon; the v0.3 `icon.ico` kept for Windows; tray PNGs
  under `src-tauri/icons/tray/`.
- Dependencies: only what phase 1 uses (tauri with `tray-icon`/`image-png`, opener,
  single-instance, serde, specta trio). The rest arrive with the module that needs them.
- `src/main.tsx` picks the React tree from the window label.
- CI: `.github/workflows/crossplatform-v2-ci.yml` (Ubuntu + Windows, npm build, cargo test,
  bindings diff).
- Smoke run on Ubuntu 26.04: starts and stays up 8 s with no panic. Visual check of both
  windows still to be done by the user.

### tauri-specta spike results (`2.0.0-rc.25`, pinned with `=`)

- Commands and events export to `src/libs/ipc/bindings.ts`; `cargo test` regenerates it.
- u64 / i64 arrive as `number` with `dangerously_cast_bigints_to_number`.
- Event names are set per event with `#[tauri_specta(event_name = "...")]`, so the v0.3
  snake_case names (`usage`, `state`, ...) can be kept.
- **Deviation from the spec:** argument keys are always generated in camelCase
  (hard-coded in `tauri-specta/src/lang/js_ts.rs`), and rc.25 has no builder option for
  accessor casing. So commands use Tauri's default argument casing (no
  `rename_all = "snake_case"`) and the accessors are camelCase (`commands.appVersion()`).
  Wrappers are positional, so call sites never write the keys. Response fields and event
  payloads stay snake_case through serde.

## Open decisions

- **Frontend styling** (plain CSS, CSS Modules, styled-components or Tailwind): the user wants
  to discuss it before phase 6. Placeholders use inline styles until then.

## Next

Phase 2: `support/`, `storage/` (versioned load, atomic write, quarantine), `config/` with
migrations passing the golden fixtures, `platform/` traits with Windows and Linux
implementations.
