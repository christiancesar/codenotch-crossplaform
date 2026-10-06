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

## Phase 2: foundations (done, Windows build pending CI)

- `support/`: `now_ms`, `sleep_interruptible` (wake flag passed in), `parse_iso`, base64,
  `cap`. SQLite `open_ro` arrives with the Cursor/OpenCode providers.
- `storage/`: config dir, atomic write, versioned load/save with migrations, quarantine.
- `config/`: model vs on-disk DTO, unknown keys preserved, v1 -> v2 migration. Gate tests
  pass for every golden config fixture (findings 1 and 2 of the gate are fixed).
- `platform/`: `Processes`, `Focus`, `Autostart`, `Locale` with Windows and Linux
  implementations; Linux autostart is new (XDG entry). The rest of the OS concerns
  (window, input, pty, credentials, icons, console) land with their consumers in phases 3
  and 4. Opening files and URLs goes through `tauri-plugin-opener` instead of a trait.
- Windows test binaries now embed the Common Controls manifest (they failed with
  `STATUS_ENTRYPOINT_NOT_FOUND`).
- CI fails if an OS `cfg` appears outside `platform/`.

## Phase 3: providers (done)

- Model: `ProviderId` (frozen ids and v0.3 file names), `ProviderStatus` (adds `none`, which
  v0.3's page already handled), `LimitWindow`, `UsageSnapshot`, `Reading`, `FetchError`.
- Snapshots keep the v0.3 format via a stored DTO; gate finding 3 fixed. All six real v0.3.1
  snapshot files load and round-trip.
- One scheduler for all providers: 429 backoff (60 s doubling to 15 min, Retry-After only
  raises it, persisted), stale-not-blank, atomic save, change callback, refresh flag,
  `should_fetch` for costly reads, hover opt-in.
- Claude, Codex (live + rollout fallback), Cursor, OpenCode (Go + token tally), Antigravity
  (CLI with TTL, bridge, Google credential, request count). Every v0.3 parser test moved.
- New platform pieces: `Pty` (ConPTY on Windows, process group on Linux), `command_lines`,
  `listening_ports` (/proc on Linux, no lsof), `Credentials`.
- Live check on Ubuntu 26.04: `agy --print /usage` read through the new Linux runner.
- Behaviour changes on purpose: Codex now uses the doubling backoff too; a needsAuth or
  failed read keeps the windows already shown (stale) instead of blanking them; a UTF-16LE
  credential blob is now actually decoded.

## Open decisions

- **Frontend styling** (plain CSS, CSS Modules, styled-components or Tailwind): the user wants
  to discuss it before phase 6. Placeholders use inline styles until then.

## Next

Phase 4: sessions (store, transcript watcher, hook server, hooks install, sweep), activity per
provider, notch placement / hit-test / drag / pointer watchdog with the Linux window pieces
(XWayland, input shape, no-activate), tray menu and icon rendering, glyphs, diagnostics
(`doctor`), CLI subcommands.
