# Ticket A.5 — Whole-pixel rounding + real-size readback

- Epic: **A** · Depends on: Epic 0 (A.1/A.2 set the *edge*; A.5 fixes the *pixels*) · Lane: A
  — do before A.4 so click-through test regions sit on the true frame.
- Verifies on: `:1` (X11) here; Wayland per-compositor later.

## Goal

Port the macOS lesson the plan calls out: round the window frame to whole pixels and read
the real size back — no fractional-width content, no pixel-slippage between the strut span
and the drawn pill. On WebKitGTK the DPR/zoom path differs from WebView2 (the code at
`main.rs` `report_dpr` ≈397 and the zoom handler was written for WebView2's devicePixel),
so the driver here is *verify parity*, not rewrite.

## Read first

1. `AGENTS.md` (the design-frame rule: layout only per `Design.px` and the macOS port's
   lessons).
2. `cross-platform/docs/notes/window-managers.md` §9 ("where the port keeps parity").
3. `codenotch/src/main.rs`:
   - `report_dpr` (≈397) and the DPR/report handler in `main()` — the WebView2 math.
   - `place_notch` (72) — where logical→physical conversion happens.
   - `codenotch/ui/notch.html` sizing (the 70 pt pill column).
4. `windows/codenotch/src/main.rs` equivalent DPR section (read-only reference).

## Scope (exactly this)

1. On Linux, after the window is placed, round the outer position and size to integers
   (they already are on the API — the trap is an *internal* fractional offset: verify, then
   fix only if the webview reports one, using size readback from the actual window).
2. Read the real window size back the same way the macOS port does (after realize), and make
   `place_notch`/strut placement use that readback, not the configured size alone.
3. Keep `report_dpr`'s WebView2 math untouched on Windows; add the WebKitGTK path (`#[cfg(not(windows))]`)
   that maps what `webview2` reported onto what WebKitGTK actually reports.
4. A small unit test if you extract any rounding/readback helper.

## Dependencies allowed

None expected — use what tauri already exposes. If something is missing (e.g. a zoom API
differs on Linux), note it in the report; do not add a dep to this ticket.

## Must NOT

- Rewrite the Windows DPR path; change the 70 pt pill layout or `Design.px`-derived
  constants; touch `ui/notch.html` geometry.

## Definition of Done

- [ ] `cargo check --workspace` + `cargo test` green
- [ ] On `:1`, the notch round-trips its position+size without fractional drift: launch,
      log the placed `(x,y)`/size, and confirm the reported readback equals the placement
      (assert via `applog` or a temporary debug line, removed before report).
- [ ] No visual gap between strut span (A.1) and the drawn pill on a screenshot
      (`import -window <XID>`); report the image path or state it could not be captured.

## Orchestrator acceptance

Compile gates + diff review (Windows DPR path byte-identical). A follow-up mark remains if
no Wayland compositor could be exercised.