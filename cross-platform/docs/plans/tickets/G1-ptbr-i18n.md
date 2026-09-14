# Ticket G.1 — Add pt-BR to the tray-menu translation

- Epic: **G** (i18n) · Depends on: nothing · Lane: `i18n.rs` only
- Queued 2026-09-14, end of the roadmap — no dependency on Phase 1/2/3 work.

## Goal

Codenotch's only translated surface is the taskbar icon's right-click menu
(`settings.html` says so explicitly: "the only part of Codenotch that is
translated. The notch, its hover card and this settings window are English
only"). Today `i18n.rs` supports `zh`, `ja`, `ko`, plus `en` as the
default/fallback and `auto` which resolves from `$LANG`/`$LC_ALL` on Linux
and the OS locale registry on Windows. Add Brazilian Portuguese.

## Read first

1. `cross-platform/docs/plans/2026-09-11-cross-platform-plan.md` — "Epic G".
2. `codenotch/src/i18n.rs` in full — both `resolve_auto()` branches
   (`#[cfg(windows)]` and the Linux `$LANG`/`$LC_ALL` one) and every
   `("<lang>", "<key>") => "…"` arm in `t()`.
3. `windows/codenotch/src/i18n.rs` — read-only reference; the Windows branch
   of `resolve_auto()` needs the same new locale so behavior stays symmetric
   across platforms (this file is genuinely cross-platform code, unlike most
   of the tree — it is **not** gated to Linux-only, so both `#[cfg]` arms of
   `resolve_auto()` live in the same `cross-platform/codenotch/src/i18n.rs`;
   only `windows/codenotch/src/i18n.rs`, the separate untouched app, must not
   be edited).
4. `codenotch/ui/settings.html` — the "Words used by Codenotch" section, to
   confirm the language picker's option list needs a "Português (Brasil)"
   entry alongside "Follow system" / whatever else is listed.

## Scope (exactly this)

1. `resolve_auto()`: add a `pt` branch (checking for a `pt` prefix,
   case-insensitively, same style as the existing `zh`/`ja`/`ko` checks) in
   **both** the `#[cfg(windows)]` and the `$LANG`/`$LC_ALL` arms.
2. `t()`: add all 16 `("pt", "<key>") => "…"` arms, one per existing key
   (`autostart`, `hooks_missing`, `install`, `lang_auto`, `language`,
   `open_data`, `quit`, `refresh`, `reset_pos`, `settings`, `tray_bars`,
   `tray_icon`, `tray_numbers`, `tray_off`, `tray_which`, `uninstall`).
   Use natural Brazilian Portuguese (not European Portuguese vocabulary or
   orthography) even though the detection key stays the bare `pt` prefix —
   matching how `zh`/`ja`/`ko` already skip regional variants.
3. `settings.html`'s language `<select>`: add the new option, matching
   whatever pattern the existing `zh`/`ja`/`ko` options already follow there
   (read the file to find it — don't guess the markup).
4. If there's a unit test enumerating supported locales (check `i18n.rs`'s
   `#[cfg(test)]` block, if any), extend it for `pt`.

## Dependencies allowed

None.

## Must NOT

- Touch `windows/` (the separate app's copy of `i18n.rs`).
- Touch the notch UI or hover card — they are explicitly English-only by
  design; this ticket is the tray menu only.
- Invent a regional variant (`pt-PT` fallback logic, etc.) beyond the single
  `pt` prefix check already used by every other locale here.

## Definition of Done

- [ ] `cargo check --workspace` clean; `cargo test --workspace` green.
- [ ] Live probe: set `LANG=pt_BR.UTF-8` (or `LC_ALL`), launch the app with
      the config's `lang` at `"auto"`, right-click the tray icon — the menu
      is in Portuguese.
- [ ] Explicitly setting `lang: "pt"` in `config.json` also works regardless
      of the running session's `$LANG`.
- [ ] Report which of the 16 strings were translated and confirms the
      Windows `#[cfg(windows)]` detection branch was updated too (even though
      it can only be live-tested on a Windows host — say so honestly).

## Orchestrator acceptance

Re-run `cargo check`/`cargo test`; diff review (should be `i18n.rs` +
`settings.html` only); confirm `windows/` untouched; spot-check a few
translated strings for tone/naturalness in Brazilian Portuguese.
