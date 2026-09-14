# Codenotch for Linux

A Linux port of [Codenotch](https://github.com/vinzdg/codenotch) — the usage notch that
sits on the edge of your screen and answers two questions at a glance: **how much of my
AI allowance is left**, and **is my coding agent still working**.

Same design language as the macOS original (inverse-rounded pill, colour-graded rings,
hover card with per-window bars), rebuilt in Rust + Tauri 2 for Linux. No code is copied
from the Swift app; providers are reimplemented from their documented behaviour, the same
way the `windows/` tree reimplements them.

## Status

**Planning / exploration only.** This directory holds the spec, the implementation plan
and the Linux platform notes — there is no application code yet. The two questions that
decide whether this port is sound are the window-manager strategy (X11 vs Wayland edge
pinning) and how auto-update works without Sparkle; both are examined in
[`docs/notes/window-managers.md`](docs/notes/window-managers.md).

Read in this order:

1. [`docs/specs/2026-09-11-linux-port-spec.md`](docs/specs/2026-09-11-linux-port-spec.md) — what is being ported, and the rules it must not break.
2. [`docs/notes/window-managers.md`](docs/notes/window-managers.md) — the Linux-specific hard parts (edge pinning, tray, keyring, notifications).
3. [`docs/plans/2026-09-11-linux-port-plan.md`](docs/plans/2026-09-11-linux-port-plan.md) — milestones, in the same ordering as the macOS plan.

## What it shows

| Cell | Source | How it reads it |
|---|---|---|
| **Claude** | Official | `GET https://api.anthropic.com/api/oauth/usage` with the token Claude Code keeps in `~/.claude/.credentials.json` — same store the `windows/` port reads. Session / weekly windows, 429 back-off with a persisted deadline, stale readings dimmed with their age. Activity from `~/.claude/sessions/*.json`. |
| **Codex** | Official | `GET https://chatgpt.com/backend-api/wham/usage` with the session in `~/.codex/auth.json` (read only, never refreshed), falling back to the `rate_limits` snapshot in the newest rollout log. |
| **Cursor** | Official | The editor's own session from `state.vscdb` → `cursor.com/api/usage-summary`. Path on Linux is under `~/.config/Cursor/` — **to confirm**. |
| **GitHub Copilot** | Official | GitHub's Copilot quota endpoint via the `gh` CLI session — `gh` is cross-platform, so this ports as-is. |
| **Ollama / LM Studio** | Local runtime | Same local detection as the other ports; both have Linux builds. |
| + more | — | GLM, Grok, OpenCode, Command Code, Antigravity, DeepSeek (WebView login) — see the spec. Anything with a `~/.config`-style store or a cross-platform CLI ports directly; web-session and editor-SQLite reads need a per-provider path confirmation on Linux. |

Providers that are not installed simply do not get a cell.

## Building (future)

Tauri 2 on Linux builds against WebKitGTK, not WebView2. Once the skeleton lands, a
Debian/Ubuntu build host needs roughly:

```sh
# Debian/Ubuntu packages for a Tauri 2 + tray build
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev patchelf
cargo build --release
```

## Layout

```
.
├── docs/
│   ├── plans/            port plan (milestones)
│   ├── specs/            port spec (what must not change)
│   └── notes/            platform notes (window managers, keyring, autostart)
└── (future) codenotch/   Tauri 2 app — window, tray, providers, session engine
```

## Relationship to upstream

This port follows the upstream design spec (`docs/specs/2026-08-28-usage-notch-design.md`)
and the provider semantics laid out in `CONTRIBUTING.md` at the repo root: fidelity is
declared, failures degrade to visible statuses, and no number is ever invented. The
`windows/` port is the reference for how a port is kept compatible without sharing code.

## License

MIT — see the upstream `LICENSE`. The Codenotch design and name belong to the upstream
author.