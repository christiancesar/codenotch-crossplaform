# Ticket A.1 — X11 edge reservation (`_NET_WM_STRUT_PARTIAL`)

- Epic: **A** (edge pinning) · Depends on: Epic 0 (done) · Lane: A — first dispatch
- Verifies on: real X11. Host has `DISPLAY=:1` with a WM (`_NET_SUPPORTING_WM_CHECK` set),
  `xprop`, `xwininfo`, `import` available.

## Goal

On an X11 session the notch reserves its screen edge exactly like a panel: maximized and
tiled client windows cannot cover it, and the pill sits flush on that edge (matching
`place_notch` position, `codenotch/src/main.rs:72`). Today the window floats; this ticket
adds the strut so the edge is *owned*.

## Read first (in order)

1. `AGENTS.md` and `.opencode/agent/executor.md` (already respected — re-read the hard rules).
2. `.opencode/skills/edge-pinning/SKILL.md` (the X11 section) and
   `.opencode/skills/rust-tauri/SKILL.md` (gating idiom, verify commands).
3. `cross-platform/docs/notes/window-managers.md` §1 (X11 struts).
4. `cross-platform/docs/plans/2026-09-11-cross-platform-plan.md` — "Epic A".
5. `codenotch/src/main.rs`:
   - `place_notch` (≈72), `place_bar` (200), `toggle_drag` (203), `apply_visibility` (808),
     the window builder in `main()` (≈1019) and the `noactivate` pair (223/243).
   - Note how `place_notch` picks monitor, edge and size before you decide where the strut
     math lives.
6. `codenotch/Cargo.toml` — the `[target.'cfg(windows)'.dependencies]` pattern to mirror.

## Scope (exactly this)

1. New module `codenotch/src/window_layer/` with `x11_struts.rs`, registered from `main.rs`
   behind `#[cfg(not(windows))]` (never `#[cfg(linux)]`). Windows path of `place_notch` stays
   byte-for-byte.
2. When the session is X11 and the notch window is **visible**, set `_NET_WM_STRUT_PARTIAL`
   on the notch toplevel so the right edge is reserved. The reserved width/height follows
   what `place_notch` computed (the pill is 70 logical pt on the right edge; the strut
   segment should span the notch's exact vertical footprint — no more, no less).
3. Obtaining the XID: use webkit2gtk/gtk's native window on the tauri `WebviewWindow` (the
   gdk X11 path is already linked — `gdkx11` appears in the build). Prefer reaching it with
   **no new dependency**; `gtk`/`gdk` (0.18, matching tauri 2's) are allowed only if tauri
   does not expose the XID through its own handle.
4. Strut lifecycle: set when the notch becomes visible, **cleared when it hides** (inside
   `apply_visibility`), re-set when the window is placed/resized, and it must not be re-set
   over a dead connection. Window death clears the property by itself — don't fight that.
5. Unit test for the pure strut-array builder (monitor bounds, edge, notch span →
   `[l,b,r,t, …]` 12-int array) — no X connection needed, runs on CI headless.

## Dependencies allowed

`x11rb` (default features) and, only if unavoidable for the XID, `gtk` + `gdk` at 0.18.
Document which route you took and why (a one-line `// why` comment if it was `gtk`).

## Must NOT

- Touch `windows/`; rewrite the Windows `place_notch`; change polling/model code.
- Add deps beyond the allowed list; refactor unrelated code; gold-plate.
- Use `#[cfg(linux)]`; invent numbers; log anything secret.

## Definition of Done

- [ ] `cargo check --workspace` clean; the strut-array unit test passes
- [ ] Strut present while the notch is visible, gone when hidden/exited.
- [ ] Live probe on `:1`: launch the app, `xwininfo -root -tree | grep -i codenotch` → XID,
      `xprop -id <XID> _NET_WM_STRUT_PARTIAL` non-empty; a maximized client (any window)
      tiles to the left of the notch instead of under it.
- [ ] Your report states exactly what was verified live vs compiled-only (honesty rule).

## Orchestrator acceptance

Re-run `cargo check`/scoped tests; diff review against this scope; confirm
`windows/` untouched. If the strut could not be proven live (WM quirk on `:1`), the report
must say so and the manual probe gets handed to the operator.