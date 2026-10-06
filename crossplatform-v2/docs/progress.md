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

## Phase 4: sessions, notch, tray, glyphs, diagnostics, CLI (done)

- `sessions/`: typed four-state store, hook server on the frozen route, transcript watcher
  (pure `Tracker` + thread), sweep and seen-clears-it (one function over the platform
  traits, v0.3 had it twice), hooks installer as a pure settings.json transform.
- Provider activity per provider plus one 2 s loop; `LiveQuery` re-queries only when a
  database or its WAL changes. `providers::all()` is the registry.
- `notch/`: placement and drag ratio as pure geometry, hit test and input region, drag and
  watchdog over a WebviewWindow with callbacks.
- `platform::Window` / `Input`: XWayland + input shaping on Linux, click-through toggling
  and console attach on Windows, no-activate, left button, focus signature.
- `tray/`: renderer moved unchanged, slot readings shared with the notch rule, menu with the
  main-thread swap. Slots keep v0.3's provider lists in step (downgrade-safe).
- `glyphs/`: override then built-in; v0.3's unreachable .exe-icon step dropped.
- `diagnostics/`: doctor as independent checks in a Report; doctor deep; `cli.rs`.
- 105 tests.

## Phase 5: app wiring, commands, bindings (done)

- `app/`: state, typed events, workers, ui actions, setup; the only holder of AppHandle.
- 34 commands, 10 events, generated `bindings.ts`; CI checks it is current.
- Deliberate surface changes from v0.3: `get_usage` returns all providers with their id (was
  five getters), one `usage` event with the provider id (was five event names),
  `get_state` -> `get_sessions`, `nub_capable` -> `starts_collapsed`, `open_usage_page`
  folded into `open_provider_page`, manual refresh no longer clears a 429 deadline.
- An unreadable config is reported on the notch (Notice) instead of only in run.log.
- Smoke run on Ubuntu 26.04: notch at (1580,155) from the saved ratio, hook route answered,
  config untouched, Claude 429 persisted as a backoff.

## Open decisions

- **Frontend styling** (plain CSS, CSS Modules, styled-components or Tailwind): the user wants
  to discuss it before phase 6. Placeholders use inline styles until then.

## Next

Phase 6: the notch and settings windows in React against `bindings.ts`. Blocked on the
styling decision above.
