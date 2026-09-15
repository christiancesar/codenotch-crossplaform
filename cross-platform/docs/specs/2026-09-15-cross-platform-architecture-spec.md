# Architecture Spec: Rebuilding Codenotch on the create-tauri-app Scaffold

Companion to `docs/specs/2026-09-11-linux-port-spec.md`. That document covers what the
Linux port keeps and changes at the product level. This one covers how the code is
organized. `cross-platform/codenotch` grew without a structure on either side of the IPC
boundary, and this spec proposes rebuilding it on the stock `create-tauri-app` scaffold,
with an explicit layout for the backend, the frontend and the contract between them.

The code samples are illustrative. They show the shape of the solution, not a finished
implementation, and they are not a review of every call site.

## Problem statement

State of `cross-platform/codenotch` at v0.3.0.

**Backend**

- `src/` is 20 modules in one flat directory. `main.rs` alone is 1527 lines and holds the
  CLI subcommands, the Tauri builder, notch placement, hit-testing, drag, the pointer
  watchdog, tray rendering and updating, the background sweep threads and all 37
  `#[tauri::command]` functions.
- Each usage source (`usage.rs` for Claude, `codex.rs`, `cursor.rs`, `antigravity.rs` plus
  `agy_cli.rs`) implements the same pieces again: credential reading, HTTP fetch,
  parsing, persistence to a JSON file, a polling loop and `app.emit(...)`. Activity
  detection for those same providers lives in `activity.rs`, their icon lookup in
  `glyphs.rs`, their diagnostics in `doctor.rs`.
- Helpers are duplicated: `now_ms` exists in 8 modules, `sleep_interruptible` in 3,
  base64 code in 3, `parse_iso`, `cap` and `open_ro` in 2 each.
- OS-specific code is spread over 11 modules as inline `#[cfg(windows)]` /
  `#[cfg(not(windows))]` pairs, 27 of them in `main.rs`. The same function has two bodies
  in the same file, and nothing shows which OS concerns a module touches without
  reading all of it.

**Frontend**

- The UI is two static HTML files, `ui/notch.html` (538 lines) and `ui/settings.html`
  (1273 lines), with inline `<script>` blocks. No components, no shared state, no build
  step, Tauri reached through `window.__TAURI__` (`withGlobalTauri: true`).
- Rendering is string templates assigned to `innerHTML`. `settings.html` escapes provider
  labels because they come from remote servers; `notch.html` interpolates `${w.label}`
  unescaped (`ui/notch.html:286`). Two files, two conventions.
- Every call goes through the untyped core API, a command name string plus a loose args
  object:

  ```js
  invoke('set_scale', { scale: scalePct / 100 })
  invoke('get_usage').then(u => { usage = u || usage; renderRing(); })
  ```

  The notch also listens to 13 events (`usage`, `state`, `codex`, `notch_slots`, ...)
  with the same lack of typing.

- Nothing ties `'set_scale'` to `fn set_scale(app: AppHandle, scale: f64)`. The payload
  shape, the return type and whether the call can fail are facts a contributor has to
  recover from the Rust source, one command at a time, 37 times. That cognitive load does
  not go away when an LLM does the reading instead of a person; the model still has to
  load and cross-reference the same files to avoid guessing a shape.

This is normal for an early project and is not a criticism of the current code. The
proposal is to rebuild both sides into a deliberate structure, not to move files around.

## Scope

In scope:

- A new project generated with `create-tauri-app` (React + TypeScript + Vite). Code from
  the current tree is ported into it and reorganized as it moves. This is not a 1:1
  move: each module is split by responsibility on the way in.
- The backend module layout: providers, platform isolation, sessions, notch, tray,
  storage, diagnostics.
- A rewrite of the `notch` and `settings` windows as React trees in one Vite project.
- A generated, typed IPC contract, snake_case end to end.
- Compatibility with existing installs: persisted files, the hook, autostart, binary
  names.

Out of scope:

- Product changes. The parity bar is v0.3.0: same visuals, same behaviour, on Windows and
  Linux.
- Adopting the provider contract of the Linux port spec (`fidelity`, `usedFraction`,
  ...). That changes the stored data format, so it follows the migration path in
  [Compatibility with existing installs](#compatibility-with-existing-installs), after the
  rebuild.
- `codenotch-hook`. It stays a separate small crate; only the contracts it depends on are
  listed here.

## Starting point: create-tauri-app

The base is the stock scaffold. Anyone can reproduce it:

```
cargo create-tauri-app
# or: npm create tauri-app@latest
```

Answers for this project:

```
Project name        codenotch
Identifier          com.immidi.codenotch   (must match the current one, see Compatibility)
Frontend language   TypeScript / JavaScript
Package manager     npm
UI template         React
UI flavor           TypeScript
```

The scaffold it produces:

```
codenotch/
├── src/                          # frontend (React, runs in the webview)
│   ├── main.tsx                  # React entry point, mounts <App />
│   ├── App.tsx
│   ├── App.css
│   ├── vite-env.d.ts
│   └── assets/
├── index.html                    # Vite entry HTML
├── vite.config.ts
├── tsconfig.json
├── package.json
└── src-tauri/                    # Tauri (Rust, runs on the OS side)
    ├── src/
    │   ├── main.rs                # binary entry point, calls lib.rs's run()
    │   └── lib.rs                 # Builder, plugins, invoke_handler![...]
    ├── capabilities/
    │   └── default.json           # per-window IPC permission grants
    ├── Cargo.toml
    ├── build.rs
    ├── icons/
    └── tauri.conf.json
```

Its only command already shows the problem at scale 1: nothing checks `"greet"` and
`{ name }` against `fn greet(name: &str) -> String`.

```rust
// src-tauri/src/lib.rs
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
```

```tsx
// src/App.tsx
import { invoke } from "@tauri-apps/api/core";
setGreetMsg(await invoke("greet", { name }));
```

The split exists because of the Tauri process model: the core process owns the OS,
windows and system access; the webview renders and calls back over IPC.
https://v2.tauri.app/concept/process-model/

What gets adapted after generating it:

1. **Layout.** `src-tauri/` becomes `backend/`. `src/`, `index.html`, `vite.config.ts` and
   `tsconfig*.json` move to `frontend/`. `package.json` stays at the project root, so the
   Tauri CLI run from the root still finds `backend/tauri.conf.json`. The exact wiring
   (Vite root, `devUrl`, `frontendDist`, `beforeDevCommand`, `beforeBuildCommand`) is
   settled and verified in the scaffold PR.
2. **`tauri.conf.json`.** `productName`, `version`, `identifier`, both windows with their
   current properties (`transparent`, `decorations`, `alwaysOnTop`, `skipTaskbar`,
   sizes, `visible: false`), bundle settings and icons are carried over from the current
   file. `withGlobalTauri` becomes `false` because the frontend imports
   `@tauri-apps/api`. The CSP, `null` today, gets a real policy now that all assets come
   from the bundle.
3. **`Cargo.toml`.** Current dependencies, including the target-specific ones, the
   `tray-icon` and `image-png` features and `tauri-plugin-single-instance`. The scaffold's
   `[lib]` plus thin `main.rs` pattern is kept. The binary must still be named
   `codenotch`.
4. **Capabilities.** `default.json` keeps granting `core:default` to `notch` and
   `settings`.
5. **CI.** `actions/setup-node` and `npm ci` run before `tauri-action`. Paths change at
   cutover (see [Migration](#migration)).

A naming note: "window" always means a Tauri OS-level window (`notch` and `settings` in
`app.windows`), never a page in the SPA-routing sense. One React tree per window, under
`frontend/src/app/notch` and `frontend/src/app/settings`.

## Project layout

```
cross-platform/codenotch/
├── package.json                  # vite, react, @tauri-apps/api, @tauri-apps/cli
├── backend/                      # Rust / Tauri (core process)
└── frontend/                     # React + Vite (webview)
```

The top level splits by process, not by language. `ui/` was dropped as a name because it
usually means a component library, which is one thing inside `frontend/`, not the whole
frontend.

## Backend structure

### Why not entities / repositories / use_cases

The first draft of this spec proposed Clean Architecture layers. They were dropped. Most
of this backend is integration code: OS APIs, vendor HTTP endpoints, local files, SQLite.
There is little business logic between input and output. Layering by technical role would
spread each provider over four directories and add a use case per command for one-liners
like `get_lang` or `nub_capable`.

The layout groups by feature instead, and gets single responsibility from three rules:

1. **OS-specific code lives only in `platform/`.** No `#[cfg(windows)]` anywhere else.
2. **Providers, sessions and diagnostics don't know Tauri.** They take no `AppHandle`
   and emit nothing. `app/` wires them to the runtime.
3. **Commands are adapters.** A `#[tauri::command]` reads or updates `AppState`, calls one
   module function and returns. No logic of its own.

### Tree

```
backend/
├── Cargo.toml
├── build.rs
├── tauri.conf.json
├── capabilities/default.json
├── icons/
├── assets/glyphs/                     # bundled SVG marks (today's glyphs/)
├── tests/fixtures/
│   ├── vendors/                       # API responses, SQLite dumps (today's fixtures)
│   └── persisted/v0.3.0/              # files from a real v0.3.0 install, see Compatibility
└── src/
    ├── main.rs                        # argv: CLI subcommand or lib::run()
    ├── lib.rs                         # module tree, run()
    ├── cli.rs                         # install-hooks, uninstall-hooks, autostart, doctor
    ├── app/                           # the only place that holds AppHandle
    │   ├── mod.rs                     # Builder, plugins, setup()
    │   ├── state.rs                   # AppState
    │   ├── events.rs                  # typed events sent to the webview
    │   └── workers.rs                 # starts scheduler, watcher, hook server, sweepers
    ├── commands/                      # thin #[tauri::command] adapters
    │   ├── usage.rs                   # get_usage, get_codex, get_cursor, get_antigravity, refresh_usage
    │   ├── sessions.rs                # get_state, get_activity, focus_session, dismiss_session
    │   ├── notch.rs                   # drag_begin, set_hot, report_dpr, scale, notch slots
    │   ├── tray.rs                    # tray options, config, preview
    │   ├── settings.rs                # lang, ui flags, autostart, hooks, reset_notch_position
    │   └── system.rs                  # open_data_dir, open_provider_page, open_usage_page, log_js
    ├── providers/                     # one directory per usage source
    │   ├── mod.rs                     # UsageProvider trait, registry
    │   ├── model.rs                   # UsageSnapshot, LimitWindow, ProviderId, FetchError
    │   ├── scheduler.rs               # polling, backoff, persistence, change notification
    │   ├── claude/                    # today: usage.rs, claude part of activity.rs
    │   │   ├── mod.rs                 # impl UsageProvider
    │   │   ├── credentials.rs
    │   │   ├── api.rs
    │   │   ├── parse.rs
    │   │   └── activity.rs
    │   ├── codex/                     # today: codex.rs, codex part of activity.rs
    │   │   ├── mod.rs
    │   │   ├── credentials.rs
    │   │   ├── api.rs
    │   │   ├── parse.rs
    │   │   ├── rollout.rs             # local rollout files fallback
    │   │   └── activity.rs
    │   ├── cursor/                    # today: cursor.rs, cursor part of activity.rs
    │   │   ├── mod.rs
    │   │   ├── credentials.rs         # state.vscdb
    │   │   ├── api.rs
    │   │   ├── parse.rs
    │   │   └── activity.rs
    │   └── antigravity/               # today: antigravity.rs, agy_cli.rs, part of activity.rs
    │       ├── mod.rs
    │       ├── discovery.rs           # local language server endpoint
    │       ├── bridge.rs
    │       ├── direct.rs
    │       ├── cli.rs                 # agy CLI runner and quota parsing
    │       ├── transcripts.rs         # requests counted from transcripts
    │       └── activity.rs
    ├── sessions/                      # Claude Code session tracking
    │   ├── store.rs                   # today: state.rs
    │   ├── watcher.rs                 # transcript watcher
    │   ├── hook_server.rs             # today: server.rs, POST /event from codenotch-hook
    │   ├── hooks_install.rs           # merge into ~/.claude/settings.json
    │   └── sweep.rs                   # stale-session sweep, seen-clears-it scan
    ├── notch/
    │   ├── placement.rs               # place_notch_sized, target size, expand/collapse
    │   ├── hit_test.rs                # cursor_in_hot and its tests (pure)
    │   ├── drag.rs
    │   └── pointer_watchdog.rs
    ├── tray/
    │   ├── menu.rs                    # today: tray.rs
    │   ├── render.rs                  # today: trayicon.rs (pure pixel drawing)
    │   └── updater.rs                 # start_tray_updater, paint_tray, reading_for_slot
    ├── glyphs/                        # sanitize and cache; candidates come from each provider
    ├── config/
    │   ├── model.rs                   # Config, TraySlot
    │   └── migrate.rs                 # the upgrades config::load() does inline today
    ├── storage/                       # every file Codenotch writes
    │   ├── paths.rs                   # config_dir()/codenotch/*
    │   ├── atomic.rs                  # temp file + rename
    │   └── versioned.rs               # schema version, migrations, quarantine
    ├── diagnostics/                   # today: doctor.rs + diag.rs, see below
    ├── i18n/
    ├── platform/                      # see Platform isolation
    └── support/                       # now_ms, sleep_interruptible, base64, sqlite open_ro
```

### Providers

Every provider does the same job with different sources, so the shared part becomes a
trait and the repeated loop becomes one scheduler.

```rust
// backend/src/providers/mod.rs (sketch)
pub trait UsageProvider: Send + Sync {
    fn id(&self) -> ProviderId;
    /// Whether the tool is installed or signed in at all.
    fn is_present(&self) -> bool;
    /// One read. No sleeping, no emitting, no disk writes.
    fn fetch(&self, previous: &UsageSnapshot) -> Result<UsageSnapshot, FetchError>;
    /// Whether the tool is working right now (the notch's working state).
    fn activity(&self) -> Vec<Activity>;
    /// One line for `codenotch doctor`.
    fn probe(&self) -> String;
    /// Where to look for the vendor's own icon.
    fn glyph_candidates(&self) -> Vec<PathBuf>;
}
```

- `FetchError` unifies the per-module enums that exist today (`FetchErr` in `usage.rs`
  and `cursor.rs`, `LiveErr` in `codex.rs`): needs auth, rate limited with a retry-after,
  network, parse.
- `scheduler.rs` owns what each module repeats today: the interval, backoff (only Claude
  has it now, in `usage.rs::backoff_secs`), persistence through `storage/`, and a change
  callback that `app/` turns into an event. The scheduler never sees `AppHandle` either.
- Antigravity keeps its two runtimes (CLI and legacy language server) as an internal
  strategy of its provider.
- Parsing functions move unchanged with their fixture tests (`parse_response`,
  `windows_from_usage`, `parse_summary`, `parse_quota`, the bridge fixture). They are the
  most valuable tests in the tree and the proof that the port did not change behaviour.

### Platform isolation

What is OS-specific today:

| Concern | Functions today | Where |
| --- | --- | --- |
| Window behaviour | `noactivate`, `expand_notch`, `collapse_notch`, `target_logical_size`, `attach_console` | `main.rs` |
| Pointer input | `left_button_down` | `main.rs` |
| Focus | `focus_terminal`, `focus_claude_desktop`, `fg_pid`, `ack_scan` | `focus.rs`, `main.rs` |
| Processes | `proc_maps`, `claude_net_pid`, `claude_io_bytes`, `lower_thread_priority`, `listening_ports`, `run_hidden` | `focus.rs`, `activity.rs`, `antigravity.rs` |
| Pseudo-terminal | `run_cmd_conpty` | `agy_cli.rs` |
| Credentials | `read_credential_raw` | `antigravity.rs` |
| Opening things | `open_data_dir`, `open_provider_page`, `open_usage_page` | `main.rs` |
| Autostart | `reg.exe` Run key only; on Linux the calls fail | `autostart.rs` |
| Executable lookup | `find_agy`, `find_executable`, hook binary name | `agy_cli.rs`, `codex.rs`, `hooks_install.rs` |
| Icons | `from_exe` | `glyphs.rs` |
| Locale | system language | `i18n.rs` |

Choosing between two implementations by OS is normal. The problem is where the choice
is made. After the rebuild it is made once, in `platform/mod.rs`:

```
platform/
├── mod.rs              # small traits + the implementation for the target OS
├── windows/
│   ├── mod.rs          # pub struct Platform; impl of every trait
│   ├── window.rs
│   ├── input.rs
│   ├── focus.rs
│   ├── process.rs
│   ├── pty.rs
│   ├── credentials.rs
│   ├── open.rs
│   ├── autostart.rs    # HKCU Run key
│   ├── icon.rs
│   ├── locale.rs
│   └── console.rs
└── linux/              # same files: x11rb, GTK, /proc, XDG
    └── ...
```

```rust
// backend/src/platform/mod.rs (sketch)
pub trait Focus {
    fn focus_terminal(&self, claude_pid: u32) -> bool;
    fn focus_claude_desktop(&self) -> bool;
    fn foreground_pid(&self) -> u32;
}

pub trait Autostart {
    fn is_enabled(&self) -> bool;
    fn enable(&self) -> Result<(), String>;
    fn disable(&self) -> Result<(), String>;
}

// ...Window, Input, Processes, Pty, Credentials, Open, Icons, Locale

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::Platform;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use self::linux::Platform;
```

- Traits are small on purpose. A consumer depends only on what it uses
  (`fn focus_session(focus: &impl Focus, ...)`), and tests pass a fake.
- Each OS module has one `Platform` struct implementing every trait. A missing method on
  one OS is a compile error on that OS's CI job, not a runtime surprise.
- `self::` in the `pub use` matters: a bare `windows::` would be ambiguous with the
  `windows` crate.
- The cross-platform tree targets Windows and Linux, so the code says
  `target_os = "linux"` instead of today's `not(windows)`.
- Autostart gets a Linux implementation (XDG autostart entry) as part of this, since the
  trait requires one.
- CI fails if `cfg(windows)` or `cfg(not(windows))` appears outside `platform/`.

### Diagnostics

`doctor.rs` today is one function building a `String`, importing `usage`, `codex`,
`cursor`, `antigravity`, `glyphs`, `activity` and `watcher` directly. `diag.rs` is the
`doctor deep` variant. Both become a directory:

```
diagnostics/
├── mod.rs              # run(), run_deep(), writes doctor.log through storage/
├── report.rs           # Report made of sections, rendered to text at the end
├── checks/
│   ├── config.rs       # config path and values
│   ├── port.rs         # hook server port free or in use
│   ├── sessions.rs     # watch roots, newest transcripts, tail parse
│   ├── providers.rs    # iterates the registry and calls probe()
│   ├── glyphs.rs
│   └── watch_log.rs
└── deep/               # today: diag.rs
    ├── recent_files.rs
    ├── sqlite_dump.rs
    └── json_dump.rs
```

Each check is independent and testable. `providers.rs` iterates the registry, so a new
provider shows up in `doctor` without editing diagnostics.

## The typed IPC layer

### snake_case end to end

Command names, argument keys, response fields and event names are all snake_case.

- serde already serializes Rust fields as snake_case, the persisted JSON files are
  snake_case, and today's JS already reads `fetched_at` and `resets_at`. No conversion
  layer anywhere.
- The identifier is the same in Rust and TypeScript, so one search finds both sides.
- Tauri converts command **arguments** to camelCase by default: `settings.html` sends
  `{ notchVisible, trayVisible }` to `set_ui_flags(notch_visible, tray_visible)`. Every
  command is declared with `#[tauri::command(rename_all = "snake_case")]` to turn that
  off. Return values are not affected; they follow serde.

### Generated from Rust, not hand-written

The first draft proposed hand-written types first and codegen later. That draft is the
argument against it: its `UsageSnapshot` (`providerID`, `fetchedAt: string`,
`usedFraction`, `fidelity`) did not match what `get_usage` returns (`fetched_at: u64`,
`used`, `count`, `derived`). `invoke<T>` is a cast, so it would have compiled and produced
`undefined` at runtime.

Types are generated with `tauri-specta`. Model types derive `specta::Type`, commands get
`#[specta::specta]`, and the builder exports commands and events to
`frontend/src/libs/ipc/bindings.ts`. The file is committed; CI regenerates it and fails on
any diff.

```rust
// backend/src/app/mod.rs (sketch, confirm against the pinned version)
let builder = tauri_specta::Builder::<tauri::Wry>::new()
    .commands(tauri_specta::collect_commands![
        crate::commands::usage::get_usage,
        crate::commands::notch::set_scale,
        // ...
    ])
    .events(tauri_specta::collect_events![crate::app::events::UsageChanged])
    .function_casing(tauri_specta::Casing::SnakeCase)
    .dangerously_cast_bigints_to_number();

#[cfg(debug_assertions)]
builder
    .export(
        specta_typescript::Typescript::default(),
        "../frontend/src/libs/ipc/bindings.ts",
    )
    .expect("failed to export IPC bindings");
```

What the generated file looks like for today's types (illustrative):

```ts
// frontend/src/libs/ipc/bindings.ts (generated, never edited)
export type LimitWindow = {
  id: string;
  label: string;
  used: number;
  resets_at: number | null;
  count: number | null;
  derived: boolean;
};

export type UsageSnapshot = {
  status: string;
  windows: LimitWindow[];
  fetched_at: number;
  note: string;
  backoff_until: number;
};
```

Call sites:

```tsx
const usage = await commands.get_usage();
await commands.set_scale(scalePct / 100);
```

instead of:

```js
invoke('get_usage').then(u => { usage = u || usage; renderRing(); })
```

A spike in phase 1 confirms, before model types start deriving `specta::Type`:

- **Version.** `tauri-specta` is still a release candidate (`2.0.0-rc.25` at the time of
  writing). Pin `tauri-specta` and `specta` with `=`.
- **Argument casing.** `rename_all = "snake_case"` together with `#[specta::specta]`
  produces snake_case keys in the generated wrappers.
- **64-bit integers.** `fetched_at`, `resets_at`, `backoff_until` (`u64`) and `count`
  (`i64`) need `dangerously_cast_bigints_to_number`. These are epoch milliseconds and
  counters, far below 2^53, so `number` is exact for them.
- **Event names.** The names generated from event types map to the current snake_case
  names, or are renamed once on both sides.

If the spike fails, the fallback is to generate types only (`specta` or `ts-rs`) and keep
a thin hand-written `commands.ts` that maps names to generated types, reviewed in the same
PR as the Rust command.

### Runtime validation (Zod): considered, not adopted as the contract

- Zod validates at runtime against a schema someone writes. It turns a silent `undefined`
  into a loud error, but only on the code path that runs, and the schema is still a
  second copy of the Rust type that can drift.
- Frontend and backend ship in the same binary. There is no version skew between them in
  production. A mismatch is a build-time mistake, and codegen catches it at compile time.
- Data that really comes from outside (vendor APIs, files on disk) is parsed by serde in
  Rust before it reaches the webview.

Revisit if the webview starts consuming data the backend did not produce. If runtime checks
are wanted in dev builds, generate the schemas from `bindings.ts` (for example with
`ts-to-zod`) instead of writing them.

## Frontend structure

```
frontend/
├── index.html                      # the one Vite entry
├── src/
│   ├── main.tsx                    # picks the tree from the window label
│   ├── app/
│   │   ├── notch/                  # replaces notch.html
│   │   │   ├── Notch.tsx
│   │   │   ├── Ring.tsx
│   │   │   ├── HoverCard.tsx
│   │   │   └── notch.css
│   │   └── settings/               # replaces settings.html
│   │       ├── Settings.tsx
│   │       ├── panes/              # one component per section (tray, notch, behaviour, ...)
│   │       └── settings.css
│   ├── components/                 # shared UI pieces
│   ├── contexts/                   # shared React context providers
│   ├── libs/
│   │   ├── ipc/
│   │   │   ├── bindings.ts         # generated by tauri-specta
│   │   │   └── index.ts            # re-exports + shared failure handling
│   │   └── ...                     # formatting, time, other pure helpers
│   └── vite-env.d.ts
├── vite.config.ts
└── tsconfig.json
```

Both OS windows load the same `index.html`. A Tauri window is a webview with a label,
and nothing stops two of them from loading the same bundle. `main.tsx` picks the tree:

```tsx
// frontend/src/main.tsx
import { lazy, Suspense } from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";

const Notch = lazy(() => import("./app/notch/Notch"));
const Settings = lazy(() => import("./app/settings/Settings"));

const Root = getCurrentWindow().label === "settings" ? Settings : Notch;

ReactDOM.createRoot(document.getElementById("root")!).render(
  <Suspense fallback={null}>
    <Root />
  </Suspense>
);
```

- One entry keeps the stock Vite setup. A multi-entry build (`rollupOptions.input`) also
  works but buys nothing here, since both windows share the IPC layer and most
  components.
- `lazy` keeps the settings code out of the notch window, which is always on screen.
  Each tree exports its component as `default`.
- Each tree imports its own stylesheet. The notch window is transparent, so page-level
  styles (`body` background, margins) belong to each tree, never to a shared global CSS.
- React escapes interpolated text by default, which removes the unescaped `innerHTML` in
  `notch.html`. Provider SVG glyphs are the one place that needs raw markup, and they are
  already sanitized in Rust (`sanitize_svg`).
- `libs/ipc/index.ts` keeps the behaviour `settings.html` has today in `call()`: a failed
  command paints the error strip and resolves to a fallback instead of leaving a
  half-rendered window.

The original HTML files are reference material during the rewrite. They are not kept or
served alongside the new frontend.

## Compatibility with existing installs

**This section is a gate.** No rebuild PR merges while the checks below fail.

### What is on the user's machine

Under `dirs::config_dir()/codenotch/` (`%APPDATA%\codenotch` on Windows,
`~/.config/codenotch` on Linux):

| File | Written by | Content |
| --- | --- | --- |
| `config.json` | `config.rs` | port, language, notch position and scale, tray and notch slots, visibility |
| `usage.json` | `usage.rs` | Claude snapshot, including `backoff_until` |
| `codex.json`, `cursor.json`, `antigravity.json` | provider modules | snapshot per provider |
| `glyphs/` | `glyphs.rs` | cached vendor icons |
| `quota-work/` | `agy_cli.rs` | working directory of the Antigravity CLI |
| `run.log`, `install.log`, `doctor.log`, `watch.log` | several | logs |

Outside that directory: hook entries in `~/.claude/settings.json` (`hooks_install.rs`) and,
on Windows, the `Codenotch` value under `HKCU\...\Run` (`autostart.rs`).

The Claude snapshot file is `usage.json`, not `claude.json`. A store keyed by provider id
has to map the legacy name.

### Why this is dangerous today: failures are silent and destructive

Every loader follows the same pattern:

```rust
std::fs::read_to_string(path)
    .ok()
    .and_then(|t| serde_json::from_str(&t).ok())
    .unwrap_or_default()
```

If one field is renamed or changes type during the rebuild, the whole file fails to
parse and the app starts on defaults without logging anything.

For `config.json` it is worse. `setup()` saves the config right after startup (the
"Persist the config (codenotch-hook reads the port from it)" block in `main.rs`), so on
the first launch of the new version the defaults **overwrite** the user's file. Notch
position, scale, tray and notch slots, language and visibility are gone for good, with no
error anywhere.

For provider snapshots the loss is smaller but still real: losing `backoff_until` makes
Claude polling ignore a rate-limit backoff it was in.

Writes are not atomic either (`std::fs::write` directly, only `agy_cli.rs` writes through
a temp file). A crash mid-write leaves a truncated file, which then hits the same silent
reset.

### Solution

1. **Stored format separate from the model.** Each store has its own DTO with today's
   exact field names (`StoredConfig`, `StoredSnapshot`) and `From` conversions to the
   model types. Model and IPC types can be renamed freely; the disk format only changes
   through step 2.
2. **Versioned files with migrations.** Files carry `"schema": N`. No key means v1, the
   v0.3.0 format. The loader reads a `serde_json::Value`, runs the migrations in order and
   then deserializes. The upgrades `config::load()` does inline today (`tray_providers` to
   `tray_slots`, `notch_providers` to `notch_slots`, pinning `tray_mode` for configs
   older than that setting) become the first migrations. Adopting `fidelity` from the
   Linux port spec later is a v1 to v2 migration (`derived: true` becomes
   `fidelity: "derived"`).
3. **Never overwrite a file that could not be read.** A file that exists but fails to
   parse is renamed to `<name>.unreadable-<timestamp>` and logged in `run.log`. The app
   runs on defaults, and the original stays on disk. The unconditional save in `setup()`
   goes away; config is written when it changes or when the file is missing.
4. **Atomic writes everywhere.** `storage/atomic.rs` (temp file plus rename, from
   `agy_cli.rs::atomic_write`) is the only way to write a persisted file.
5. **Golden fixtures from v0.3.0.** Before the first backend PR, real files from a v0.3.0
   install (anonymized) go into `backend/tests/fixtures/persisted/v0.3.0/`, together with
   the legacy shapes `config::load()` still handles (no `tray_mode`, only
   `tray_providers`). Tests load each one and assert every value survives. These tests
   stay after the rebuild.

```rust
// backend/src/storage/versioned.rs (sketch)
pub enum Loaded<T> {
    /// No file: first run, defaults are safe to save.
    Missing,
    Ok(T),
    /// The file existed but could not be read. It was moved aside, never overwritten.
    Quarantined(PathBuf),
}

/// `migrations[i]` upgrades a document from schema `i + 1` to `i + 2`.
pub fn load<T: DeserializeOwned>(path: &Path, migrations: &[fn(Value) -> Value]) -> Loaded<T> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Loaded::Missing,
        Err(_) => return Loaded::Quarantined(quarantine(path)),
    };
    let parsed = serde_json::from_str::<Value>(&text).ok().and_then(|mut doc| {
        let from = doc.get("schema").and_then(Value::as_u64).unwrap_or(1) as usize;
        for migrate in migrations.iter().skip(from.saturating_sub(1)) {
            doc = migrate(doc);
        }
        serde_json::from_value::<T>(doc).ok()
    });
    match parsed {
        Some(value) => Loaded::Ok(value),
        None => Loaded::Quarantined(quarantine(path)),
    }
}
```

### Contracts that do not change

- **Binary names** `codenotch` and `codenotch-hook`. The hook launches the main app by
  name from its own directory (`codenotch-hook/src/main.rs`, `spawn_main`), and the
  entries in `~/.claude/settings.json` point at `codenotch-hook` and are recognized by
  that name (`hooks_install.rs`, `is_ours`).
- **The `"port"` key at the top level of `config.json`.** The hook finds it with a text
  scan, not serde (`read_port`).
- **The hook route** `POST /event?e=...&ppid=...` on `127.0.0.1`.
- **`identifier` and `productName`.** Installers and the single-instance plugin derive
  the app's identity from them.
- **Autostart**: the `Codenotch` Run value name and the `--silent` flag.
- **CLI subcommands**: `install-hooks`, `uninstall-hooks`, `autostart on|off`,
  `doctor [deep]`.

## Migration

The new project is built beside the current one, in `cross-platform/codenotch-next/`,
until cutover. The current tree keeps shipping bug fixes during the rebuild; every fix
merged there is ported to the new tree in a follow-up PR. The new tree stays out of the
Cargo workspace (`exclude` in `cross-platform/Cargo.toml`) because both packages are named
`codenotch`.

0. **Compatibility gate.** Golden fixtures of the persisted files, the list of frozen
   contracts, and a parity checklist of v0.3.0 behaviour on Windows and Linux (notch,
   hover card, drag, scale, tray modes, every settings pane, hooks, autostart, doctor).
1. **Scaffold.** Run `create-tauri-app`, adapt the layout to `backend/` and `frontend/`,
   carry over `tauri.conf.json`, `Cargo.toml`, capabilities and icons. CI builds on
   Windows and Linux with both windows opening empty. The `tauri-specta` spike runs here
   with one command and one event.
2. **Foundations.** `support/`, `storage/` (versioned load, atomic write, quarantine),
   `config/` with migrations passing the golden fixtures, `platform/` traits with both
   implementations.
3. **Providers.** Trait, registry and scheduler, then one provider per PR, Claude first as
   the reference. Parsing code and fixture tests move unchanged.
4. **Sessions, notch, tray, glyphs, diagnostics, CLI.**
5. **Commands and bindings.** Thin adapters, `rename_all = "snake_case"`, generated
   `bindings.ts` committed, CI diff check.
6. **Frontend.** `notch` and `settings` rewritten in React against `bindings.ts`. Can
   start alongside phases 3 and 4 once the first commands exist.
7. **Cutover.** Parity checklist passes on both OSes. Installing over a v0.3.0 install
   keeps the user's config and snapshots. The old tree is removed, `codenotch-next/` is
   renamed to `codenotch/`, workspace members become `codenotch/backend` and
   `codenotch-hook`, and `projectPath` and `working-directory` in the workflows are
   updated.
