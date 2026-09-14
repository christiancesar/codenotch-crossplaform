# Ticket B.2 — XDG autostart on Linux

- Epic: **B** (tray, autostart, settings, persistence) · Depends on: Epic 0 (done) · Lane:
  `autostart.rs` only
- Verifies on: real X11 desktop (`DISPLAY=:1`, GNOME/Mutter, this host).

## Goal

`autostart.rs` today is a byte-for-byte copy of `windows/`'s version: `is_enabled()`,
`enable()`, `disable()` all shell out to `reg.exe` against
`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`. On Linux there is no registry — none of
these three functions do anything meaningful, so the "Start with Windows" toggle in Settings
(`settings.html`'s Behaviour tab, already screenshotted this session) silently no-ops on
Linux. Port it to the XDG autostart mechanism: a `.desktop` file in
`~/.config/autostart/`.

## Read first

1. `AGENTS.md`, the honesty rule: `is_enabled()` must read the filesystem, never a remembered
   preference — same rule this ticket already follows on Windows (`reg query`, not a cached
   bool).
2. `cross-platform/docs/notes/window-managers.md` §5 ("Config, data, autostart") — the exact
   XDG spec already researched: `~/.config/autostart/codenotch.desktop`,
   `X-GNOME-Autostart-enabled=true`, `Exec=<exe> --silent`.
3. `cross-platform/docs/plans/2026-09-11-cross-platform-plan.md` — Epic B, B.2.
4. `codenotch/src/autostart.rs` in full (the Windows implementation — the contract to match:
   three functions, same signatures, same error-string style).
5. `codenotch/src/main.rs`: grep `autostart::` for the call sites (the `autostart on|off` CLI
   subcommand and wherever Settings' toggle invokes it) — confirm the contract doesn't need to
   change, only the Linux-side implementation.
6. `codenotch/src/config.rs`'s `config_path()` / `dirs::config_dir()` usage, for consistency —
   this ticket does **not** touch config.rs, just matches its `dirs`-crate style.

## Scope (exactly this)

1. Add a `#[cfg(not(windows))]` implementation of `is_enabled()`, `enable()`, `disable()` in
   `autostart.rs`, gating the existing Windows code behind `#[cfg(windows)]` (matching every
   other file's pattern in this tree — never `#[cfg(linux)]`).
2. `enable()`: write `~/.config/autostart/codenotch.desktop` (via `dirs::config_dir()`) with:
   ```
   [Desktop Entry]
   Type=Application
   Name=Codenotch
   Exec=<absolute path to current_exe()> --silent
   X-GNOME-Autostart-enabled=true
   NoDisplay=true
   ```
   Use the atomic-write pattern already established in this tree (see `agy_cli.rs`'s
   `atomic_write`, or `usage.rs`'s persist path) rather than a plain `std::fs::write`, so a
   crash mid-write can't corrupt or truncate the file.
3. `is_enabled()`: read the `.desktop` file back from disk and check it actually exists **and**
   still points at the current `current_exe()` path (a stale entry from a different install
   location should read as disabled, not silently launch a moved/deleted binary) — reading the
   filesystem fresh every call, never a cached value.
4. `disable()`: remove the `.desktop` file; if it doesn't exist, return the same
   "was not enabled" success message the Windows branch returns for the equivalent case
   (`reg.exe`'s "unable to find" branch) rather than an error.

## Dependencies allowed

None — plain `std::fs`, `dirs` (already a dependency).

## Must NOT

- Touch `windows/` or the `#[cfg(windows)]` branch.
- Touch `main.rs`, `config.rs`, or `settings.html` — the call sites and UI already work through
  the existing `autostart::is_enabled/enable/disable` contract; this ticket only fills in the
  Linux implementation behind it.
- Invent a systemd user-service unit instead of a `.desktop` file — the plan explicitly chose
  XDG autostart, matching every desktop environment without extra setup.

## Definition of Done

- [ ] `cargo check --workspace` clean; `cargo test --workspace` green (add unit tests for the
      `.desktop` file content generation — a pure string-building function is easy to test
      without touching the filesystem).
- [ ] Live probe: run `./target/debug/codenotch autostart on`, confirm
      `~/.config/autostart/codenotch.desktop` exists with correct `Exec=` path; run
      `./target/debug/codenotch autostart off` — wait, check the actual CLI subcommand name in
      `main.rs` first — and confirm the file is gone.
- [ ] Live probe via the Settings UI: toggle "Start with Windows" (or whatever it's labelled —
      check if a Linux-specific label like "Start automatically" makes more sense; if you
      rename it, do it only in `settings.html`'s Linux-relevant copy, and say so in the report),
      confirm the toggle reflects the real file state on reopen.
- [ ] Report states exactly what was verified live vs compile-only.

## Orchestrator acceptance

Re-run `cargo check`/`cargo test`; diff review (should be `autostart.rs` only, plus
`settings.html` only if a label was deliberately changed and justified); confirm `windows/`
untouched; manually confirm the `.desktop` file's `Exec=` line is properly shell-quoted if the
install path could ever contain a space.
