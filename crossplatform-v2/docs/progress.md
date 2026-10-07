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

## Phase 6: frontend (in progress)

Decided with the user (2026-10-06):

- **Styling:** Tailwind v4 + shadcn/ui (radix base, preset `b1vtaYiolM`, neutral theme). Colours
  from the Tailwind palette; Codenotch roles in `src/styles/tokens.css`.
- **Fonts:** Raleway (headings) and Space Grotesk (text), bundled via `@fontsource-variable`.
- **Motion:** the official app's springs through Motion (`src/libs/motion.ts`); continuous
  indicators on composited layers with the official smooth timings.
- **Charts:** shadcn charts for card detail and history; notch rings stay SVG.
- **Marks:** each provider's official brand mark (Lobe Icons 1.95.1), colour where the brand
  has colours.
- **Workshop:** Storybook 10 (`npm run storybook`), design system pages plus one story per
  component state. Components are built and reviewed there before screens are wired to IPC.

Done in Storybook: design system pages (Introduction, Colors, Typography, Geometry, Motion) and
the notch components: UsageRing, ProviderGlyph, ActivityArc, PercentLabel, ProviderCell,
NotchShell, HoverCard, CardHeader, UsageBar, LimitWindowBlock, StatusDot, SessionList,
ScaleSlider, NoticeToast, and `Notch/Notch` composing the window with hover.

Review notes applied (2026-10-07): `NotchShell` is one SVG silhouette (the idle tab's corners
did not fit and the outline broke at the fillets); `HoverCard` stays inside the window, only the
tail follows the cell, and scrolls past the window height.

Settings window in Storybook (2026-10-07): the five v0.3 panes (tray icon, notch, behaviour,
Claude Code, about) as presentational components under `src/components/settings/`, the v0.3
rules in `src/libs/settings.ts`, and `Settings/SettingsWindow` composing them with every
control live. Linux and Windows copy differ where the systems do.

Translation (2026-10-07): the whole UI in en, pt, zh, ja and ko through i18next, following the
backend's resolved language; Storybook has a Language toolbar menu. zh, ja and ko were written
without a native reviewer and should get one.

Wired to the IPC (2026-10-07): `app/settings` loads every pane from the `get_*` commands, saves
through the `set_*` ones (toast on save, strip on failure, Result commands roll the switch back)
and follows the `scale`, `notch_slots`, `lang` and `usage` events. `app/notch` draws the rings
from `get_usage` and the events, the card per hovered provider, reports the hot rectangles to
`set_hot` while anything moves, hands drags to `drag_begin`, opens the provider page on click,
fits the viewport to 340 px and reports the DPR. Not yet run on the desktop: v0.3 was running
(same identifier, single-instance).

Notch height (2026-10-07, user's choice): the window grows with the pill instead of clipping it.
460 px stays the minimum; the page asks `set_notch_height` for the open pill plus fillets at the
current scale (five rings at 100 % need ~605 px), the backend clamps it to the monitor and
re-places the window around the saved centre.

## 0.4.0

The rebuild becomes the released app: `package.json` at 0.4.0 (Cargo and tauri.conf already
were), the package and release workflows build `crossplatform-v2/`, and CI runs the tests
through cargo-nextest with a per-test limit (the PR's Ubuntu runner had starved twice). Checked
locally before the PR: the CI steps, and `npm run tauri build -- --bundles deb,appimage`
(3 min, deb 5.4 MB, AppImage 87 MB).

`codenotch-hook` moved in unchanged as a second binary of the package (`src-tauri/hook/`, outside
`src/` so the platform rule holds); the installers ship it next to the app (`/usr/bin` in the
deb), where the hooks switch looks for it. v0.3 never packaged it: its hooks pointed at a debug
build in the repository.

## Open decisions

- Design-system open table (`design-system.md`, end): frame sizes vs v0.3 web sizes, card
  body text floor of 11 px. Components currently follow the frame with the 11 px floor.

## Next

1. Run v2 on the desktop (v0.3 closed) and check the notch and settings against the stories.
2. Phase 7 cutover: parity checklist on Linux and Windows.
