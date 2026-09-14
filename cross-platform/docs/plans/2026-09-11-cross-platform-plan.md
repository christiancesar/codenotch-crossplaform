# Codenotch Cross-Platform (Windows + Linux) — Implementation Plan

- Supersedes: `2026-09-11-linux-port-plan.md` (folded in below)
- Spec: `docs/specs/2026-09-11-linux-port-spec.md`
- Platform notes: `docs/notes/window-managers.md`
- Started: 2026-09-11 · Status: Epic 0 done (gate re-verified 2026-09-11 on the
  Linux host), Epics A + C ticketized in `docs/plans/tickets/`, dispatch underway.
- Working mode: orchestrated — the orchestrator dispatches tickets to executor
  agents (the oikos-style executor pool) and validates their output.

## Goal / product

Turn the Windows Tauri 2 port into **one cross-platform codebase** that compiles
and runs on Windows **and** Linux, living in this `cross-platform/` directory
(= the former `linux/` + the Windows port copied in). The upstream `windows/`
tree stays untouched as the shipping/reference port until the maintainer reworks.

Requirements agreed with the maintainer-equivalent (this repo's operator):

1. **Compiler parity first** — the tree must build and unit-test on Linux, and
   must keep building on Windows (verified later on a Windows host/CI).
2. **Providers**: exactly the current 4 (Claude, Codex, Cursor, Antigravity).
   No Linux-only providers (Copilot/Ollama/LM Studio/DeepSeek) in this pass.
   *(Exception added 2026-09-14, operator-authorized: OpenCode as an
   exploratory 5th provider — see Epic H at the end of this plan.)*
3. **Single Tauri config.** No `tauri.windows.conf.json`/`tauri.linux.conf.json`
   split: `tauri.conf.json` covers both. The only needed touch is adding
   `icons/icon.png` (256×256) to `bundle.icon` (Windows uses the `.ico`, Linux
   the `.png`; the list accepts both). Packaging target per OS is decided in
   Epic E (`--bundles` flags vs platform merge files).
4. The app deliberately runs no OAuth of its own and never invents a number:
   failures degrade to a visible status (`stale`/`needsAuth`/…), never a guess.
5. Docker-true parity with the design frames; layout changed only per
   `Design.px` and the macOS port's lessons (whole-pixel rounding, real-size
   readback).

## Structure

```
cross-platform/
├── Cargo.toml            workspace: codenotch, codenotch-hook
├── codenotch/            Tauri 2 app (copied from windows/, adapted)
│   ├── src/              main, window-layer (a1_*), tray, config, usage/store,
│   │                     providers/*, session engine, glyphs, doctor
│   ├── ui/notch.html     the pill + hover card (single file, no framework)
│   ├── glyphs/           provider marks (+ NOTICE.md)
│   └── icons/            icon.ico + icon.png
├── codenotch-hook/       <5 ms hook messenger (binary name per platform)
├── docs/                 spec, notes, plans, tickets
└── README.md             adapted per-OS pre-reqs
```

## Epics and tickets

### Epic 0 — Foundation: tree + compile parity

| Ticket | Delivery | Acceptance |
|---|---|---|
| T0.1 Toolchain & deps | Install rustup + apt per official Tauri v2 pre-reqs: `libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev` (no `libgtk-3-dev` needed — pulled transitively; `patchelf` deferred to Epic E / .deb bundling) | `pkg-config` finds all; `cargo`/`rustc` present |
| T0.2 Restructure | `cross-platform/` = this dir + Windows workspace copied in (Cargo.toml, Cargo.lock, codenotch/, codenotch-hook/, LICENSE, README adapted); merge `.gitignore`/`.gitattributes`; plan + tickets in `docs/plans/` | tree mirrors `windows/` + docs; `windows/` untouched |
| T0.3 Linux runtime behaviour | `main.rs` → `xdg-open` for `open_data_dir`/`open_provider_page`/`open_usage_page` (keep `explorer`/`cmd /C start` + `CREATE_NO_WINDOW` on Windows); `hooks_install.rs` → platform hook name; `codenotch-hook` → `read_port()` on `~/.config/codenotch/config.json` and `spawn_main()` without `.exe`; `i18n::resolve_auto()` from `$LANG`/`LC_ALL`; `agy_cli::find_agy()` and `codex::find_executable()` Linux candidates | `cargo check` clean; no Windows branch touched |
| T0.4 Icons | Add `icons/icon.png`; extend `bundle.icon` | `generate_context!` runs on Linux |
| T0.5 Verify | `cargo build` + `cargo test` headless on the Linux host | green; Windows build verified on host/CI as follow-up |

**Done.** Gate re-run 2026-09-11: `cargo check --workspace` + `cargo test --workspace`
green on the Linux host (`rustc 1.98.1`, X11 `:1` with a WM). Notes: the tray feature
compiles without the `appindicator3-0.1` pkg-config name (`libappindicator v0.9.0` was
pulled); `cargo tauri` CLI not installed — only needed for packaging (Epic E).

**Done:** `cross-platform/` compiles and tests on Linux; nothing Windows changed.

### Epic A — Edge pinning (was M1)

- A.1 X11 `_NET_WM_STRUT_PARTIAL` (x11rb) — reserve the right edge; flush
  placement; re-place on monitor change.
- A.2 Wayland `zwlr_layer_shell` — dock on wlroots (sway/hyprland/river) and
  KWin 6.2+.
- A.3 Runtime probe + floating fallback (GNOME Mutter) — always-on-top, drag to
  edge, persist position (reuse `place_notch`/`drag_begin`/`notch_y`).
- A.4 Click-through / hit regions on Linux — validate `set_ignore_cursor_events`
  on X11 vs Wayland.
- A.5 Whole-pixel rounding + real-size readback (macOS lesson).

**Done:** X11 tiles windows around the notch; sway/hyprland dock; GNOME floats
and still works.

### Epic B — Tray, autostart, settings, persistence (was M5)

- B.1 TrayIcon = StatusNotifierItem (tauri `tray-icon`) with the same menu; the
  app stays fully usable without a tray on stock GNOME (settings reachable from
  the notch, mirroring the macOS orb).
- B.2 XDG autostart: `~/.config/autostart/codenotch.desktop`
  (`Exec=<exe> --silent`, `X-GNOME-Autostart-enabled=true`); `is_enabled()`
  reads the filesystem, never a remembered preference.
- B.3 Settings window parity: provider drag-to-reorder by id (unknown ids
  appended), slot pickers, real-icon preview.
- B.4 Config/data layout via `dirs`: config `~/.config/codenotch`, data
  `~/.local/share/codenotch`.

**Done:** provider added/removed/reordered without rebuild; launch-at-login
honest; settings survive relaunch.

### Epic C — Providers & engine parity (was M4 core; all file-based already)

Adapt-and-verify tickets, each with a pinned recorded-body test:
- C.1 Claude: `~/.claude/.credentials.json` → `/api/oauth/usage`; one ring per
  `~/.claude-<slug>`; sessions from `~/.claude/sessions/*.json`.
- C.2 Codex: `~/.codex/auth.json` → `wham/usage`; rollout-*.jsonl fallback;
  stale >5 min.
- C.3 Cursor: confirm `~/.config/Cursor/User/globalStorage/state.vscdb` (WAL ro)
  → `usage-summary`.
- C.4 Antigravity: `agy` in `~/.local/share/agy/bin` / PATH + local bridge +
  transcript fallback; Credential Manager stays Windows-only.
- C.5 Session engine: `watcher.rs` (`~/.claude/projects` + desktop-app
  `local-agent-mode-sessions`), `state.rs` (4-state machine, unchanged
  constants), hook events via `server.rs` (port 48666).
- C.6 Archive/honesty: archived readings come back `.stale` + dimmed; 429
  backoff persisted (60 s → double → 15 min cap); pinned suite runs on CI.

**Done:** the Claude cell moves on its own; offline/expired shows dimmed stale,
never blank or guessed.

### Epic D — Notifications, sessions, activity (was M6)

- D.1 Threshold alerts 80%/100% via `notify-rust` (org.freedesktop.Notifications),
  once per crossing, per-provider mute.
- D.2 Session-end peek/chime: notch opens 5 s + best-effort sound (test per
  shell; sound optional).
- D.3 Activity indicator parity (busy spin / waiting pulse).
- D.4 Liveness via `/proc/<pid>/stat` field 22; priority/activity polling per
  platform.

**Done:** ending a Claude turn opens the notch and warns once; a crossed
threshold notifies exactly once until rollover.

### Epic E — Polish, packaging, updates (was M7)

- E.1 Multi-monitor (X11 randr; Wayland per-compositor best effort). Partially done: the
  strut leak onto a second monitor was found and fixed live (`44a3a0a`) — the notch itself
  already behaves correctly with 2 monitors; `randr`-based "follow the active screen" is
  still open.
- E.2 Reduced-motion; app icon; README screenshots.
- **E.3 Packaging — mostly done (`2dbc0ba`), no bundle-config split needed.** The blocker
  wasn't the Rust code, it was `tauri.conf.json`'s `bundle.targets` being hardcoded to
  `["nsis"]` (Windows-only). Changed to `"all"` (Tauri resolves the right targets per host
  OS from one config, no `--bundles` flag or platform merge file needed) and installed
  `cargo-tauri` (wasn't present). First `cargo tauri build --bundles deb,appimage` attempt
  succeeded outright. Live-verified: the AppImage runs standalone with no install and no
  root (`--appimage-extract-and-run`, real window opened correctly titled "Codenotch"); the
  `.deb`'s contents were inspected and are correct (binary, icon, `.desktop` file, resolved
  runtime `Depends`). **Not yet verified: an actual `sudo dpkg -i` install** — this sandbox
  has no non-interactive root, so that step needs a real run on a host with a password
  prompt. `.rpm` was not attempted (no Fedora/RHEL host available).
- E.4 **Updates**: signed-feed self-update vs distro-managed vs link-out;
  visible disclosure.

**Done:** `make`-free installs on Debian/Ubuntu, Fedora, AppImage; update story
written into README.

### Epic F — Governance

- F.1 CI Linux job (headless `cargo check`/`cargo test`) — connects to open
  upstream issues #147/#151.
- F.2 Draft upstream issue describing the cross-platform tree — **not submitted
  before local validation**.
- F.3 Documented sync rules `windows/` ↔ `cross-platform/`.

## Sequencing & dependencies

```
T0.1 → T0.2 → T0.3 → T0.4 → T0.5
                ├──→ Epic A ─────→ Epic B
                └──→ Epic C ─────→ Epic D ─→ Epic E
F.1 after Epic 0 · F.2 after Epic 0 validated · F.3 continuous
```

Each epic is cuttable without breaking the ones before it (a notch is visible
on an X11 edge by the end of Epic A). Nothing in Epic 0 depends on a graphical
session; CI stays headless for provider tests.

## Orchestration model and agent pool

This port uses the same orchestrated pool as the `mobile` and `api` projects:

- **Model routing**: primary lane is **OpenRouter**; fallback to **local opencode
  (big-pickle)** when routing, policy or cost requires it. For the cross-platform
  Rust/Tauri surface, executor work is currently handled by the local opencode lane.
- **Orchestrator**: **opencode (Go binary)** with a high-context model such as
  **Kimi 2** or **GLM**. The orchestrator owns this plan, the ticket list, diff
  review, and merge decisions.
- **Executor lane**: opencode executor subagents receive one ticket at a time,
  read the listed files first (AGENTS.md, CONTRIBUTING.md, the matched Windows
  sources in `windows/`, plus the ticket's plan/notes), self-check via compile +
  pinned tests + real-desktop probes, and report back.
- **Reviewer lane**: optional opencode reviewer subagent gates scope, platform
  gating, tests, and the AGENTS.md invariants before the orchestrator merges.
- **Merge**: the orchestrator runs compile gates and commits only validated
  output.

Workflow:

1. Orchestrator dispatches one ticket to a fresh executor.
2. Executor edits + self-checks + returns a report (per
   `.opencode/agent/executor.md`).
3. Orchestrator (or a reviewer) validates: `windows/` untouched, compile gates
   green, diff review, real-desktop claims verified or honestly reported.
4. Orchestrator merges and updates the ticket index.

*Note:* In this session the orchestrator is running on the local opencode
(big-pickle) lane while the OpenRouter/Kimi/GLM routing is recorded as the
target operating model. A concrete local-agent registry example (same pool
pattern as mobile/api, but scoped to this repo) lives in
`agent-pool-example.md` and in the updated `opencode.json` +
`.opencode/prompt/` files.

Work items land in `docs/plans/tickets/` (one file per ticket; see
`docs/plans/tickets/README.md` for the index, lanes and ticket anatomy).
Epics A and C are ticketized; B/D/E/F stay at epic level here until their
dependency lands.

## Open decisions

1. **Wayland depth** — runtime probe vs compile-time default (recommendation:
   probe + float fallback).
2. **Stock GNOME** — settings reachable without the AppIndicator extension
   (recommendation: notch-attached affordance, mirroring macOS).
3. **Cursor on Linux** — confirm `~/.config/Cursor/User/globalStorage/state.vscdb`
   on a real install.
4. **DeepSeek web login in WebKitGTK** — deferred/cut; stays out (4 providers
   only).
5. **Updates (E.4)** — the one product decision not dictated by upstream.
6. **Identifier** — keep `com.immidi.codenotch`.
7. **Windows verification** of `cross-platform/` — Windows host or CI.

## Risks

- **Data fidelity is the whole product** — every adapter pinned by recorded-body
  tests; the provider-path table in `docs/notes/window-managers.md` §6 is the
  checklist.
- **Wayland fragmentation** — biggest scope risk; mitigated by X11-first →
  layer-shell → float ordering.
- **WebKitGTK vs WebView2** (DPI, zoom, transparency) — validate early (T0.5
  shows the real window).
- **No keyring / no tray** in odd environments — file-based stores and a
  notch-only UX; never require either.

---

## Checkpoint — 2026-09-11

- **Epic 0 concluído e re-verificado**: `cargo check --workspace` + `cargo test
  --workspace` verdes no host Linux (`rustc 1.98.1`).
- **Tickets criados e commitados**: Epic A (A.1–A.5) + Epic C (C.1–C.6) + README de
  índice em `cross-platform/docs/plans/tickets/`.
- **Agent pool local criado e commitado**: registry em `opencode.json` com
  OpenRouter → big-pickle → opencode-go/Kimi/GLM, prompts em
  `.opencode/prompt/`, e documentação em `cross-platform/docs/plans/agent-pool-example.md`.
- **Regra de execução ajustada**: sem worktree, na branch `cross-platform-port`, uma
  tarefa por vez, executor commita progresso frequentemente (proteção contra
  cancelamento/`git reset`), sem push.
- **A.1 (X11 struts) implementado**: o executor produziu o módulo
  `window_layer::x11_struts` com `_NET_WM_STRUT_PARTIAL`, lifecycle atrelado à
  visibilidade e testes unitários. Também fez adaptações cross-platform em
  praticamente todos os módulos (activity, agy_cli, antigravity, autostart, codex,
  config, cursor, diag, doctor, glyphs, hooks_install, main, server, trayicon,
  usage, watcher). Tudo foi commitado em `d946b91`.
- **Estado de compilação**: `cargo check --workspace` e `cargo test --workspace`
  verdes (28 passed, 1 ignored).
- **Pendências**: revisar o diff misturado, validar o comportamento de desktop real
  do A.1 (strut reservando a borda no `:1`), e decidir se separamos o commit
  `d946b91` por ticket ou seguimos com ele.
- **Próxima ação**: revisão do `d946b91` e validação em desktop real do A.1 antes
  de prosseguir para A.5/A.2/etc.

---

## Reassessment — 2026-09-14

Revisão do `d946b91` feita comparando byte a byte (via `rustfmt`) cada arquivo
contra `windows/`. Achados que mudam a priorização:

1. **12 dos 16 módulos listados no commit (`activity`, `antigravity`, `autostart`,
   `config`, `cursor`, `diag`, `doctor`, `glyphs`, `server`, `trayicon`, `usage`,
   `watcher`) não tinham nenhuma mudança semântica — só reformatação.** Revertidos
   para o texto exato de `windows/` em `46a7d33`. Código real de port ficou só em
   `agy_cli.rs`, `codex.rs`, `hooks_install.rs`, `i18n.rs`, `main.rs` e
   `window_layer/` (novo).
2. **O app já roda no host Linux.** `codenotch doctor` confirma: credencial Claude
   válida, sessões lidas de `~/.claude/projects` sem erro, Cursor com leitura real
   (`Free · via Cursor`). O motor de estado (`state.rs`/`server.rs`, running →
   attention → done → idle) já é 100% cross-platform via os hook events — Epic C é,
   na prática, maioria confirmação de caminho, não escrita de adaptador novo.
3. **O strut X11 (A.1) funciona no GNOME/Mutter padrão desta máquina** —
   testado ao vivo (`xprop _NET_WORKAREA` reflete a largura reduzida, 1850 em vez
   de 1920, no desktop onde o notch está). A nota em `window-managers.md` de que
   "GNOME não tem docking de edge" está certa para a API de layer, mas o
   `_NET_WM_STRUT`/`_NET_WM_STRUT_PARTIAL` legado ainda é respeitado para
   workarea. Corrigido só o bug real: a primeira chamada rodava com
   `outer_size()` ainda 0x0 (antes do GTK realizar a janela) — corrigido em
   `5c8d4e7`; mantido ligado por padrão.
4. **O que falta pra usar o app de verdade no Linux não estava ticketizado**:
   vários caminhos `#[cfg(not(windows))]` são stubs que sempre retornam
   `false`/`None`/vazio — `left_button_down` e `noactivate` (`main.rs`),
   `ack_scan` (`main.rs`), `focus_terminal`/`focus_claude_desktop` (`focus.rs`).
   Sem eles: arrastar o notch não funciona, ele pode roubar foco, e clicar numa
   sessão não traz o terminal/app pra frente. `claude_io_bytes` (`activity.rs`)
   também é stub, mas é só um heurístico supletivo de atividade via IO de rede —
   o sinal primário (hook events → `state.rs`) já é cross-platform, então fica
   deliberadamente adiado.
5. **Sem CI de Rust.** `ci.yml`/`package.yml` só cobrem o app macOS. Nada garante
   que uma mudança em `cross-platform/` quebra o build Windows. Adicionado
   `.github/workflows/cross-platform-ci.yml` (matrix ubuntu-latest +
   windows-latest, `cargo check` + `cargo test`).
6. **A orquestração multi-agente (opencode/Kimi/GLM, executor/reviewer
   separados) gerou mais overhead documental que código real** — dos 8 commits
   antes desta revisão, 6 eram plano/config de agente. Para o tamanho real do
   diff (dezenas a poucas centenas de linhas por ticket), a via direta —
   Claude Code executando e testando no próprio desktop, sem round-trip por
   outro CLI/modelo — chega no mesmo resultado com menos etapas. O modelo de
   ticket (read first / scope / must not / definition of done / verify) continua
   valendo como formato de trabalho, só não passa mais pelo roteamento externo.

### Prioridades revisadas

**Fase 1 — app usável no X11 (esta é a validação que importa, não `cargo check`)**
- [x] Reverter reformatação sem função (`46a7d33`)
- [x] Corrigir corrida de tamanho zero no strut (`5c8d4e7`)
- [x] CI Linux+Windows (`cross-platform-ci.yml`)
- [x] P1.1 — `left_button_down` + `noactivate` reais no Linux (`39b8cf8`, drag e
  no-activate verificados ao vivo)
- [x] P1.2 — `focus_terminal`/`focus_claude_desktop` via X11 (`d1635d9`, EWMH
  confirmado num gnome-terminal real; reconstrução documentada no commit após
  um `git checkout` concorrente ter apagado o trabalho não commitado)
- [x] P1.3 — ligar `ack_scan` à detecção de foco de P1.2 (`fc633e9`, evento de
  hook sintético limpou a sessão `done` ao focar o terminal certo)
- [x] Achados extras corrigidos no mesmo desktop real: CI do Windows quebrada
  desde o `d946b91` (`fc633e9`), strut vazando pra segunda tela em
  multi-monitor (`44a3a0a`), `icon.png` sumido do disco + título de janela
  genérico (`44a3a0a`), ícones Windows/Linux separados em subpastas (`b0ad84c`)
- [x] Checklist manual no desktop real (2026-09-14, via X11 XTEST sintético +
  screenshots reais): hover abre o card com dado real (`Codex Usage / 100%
  Used`); menu do botão direito (Settings/Refresh/Quit) funciona; Settings
  abre e navega entre abas. click-through e drag já verificados no P1.1;
  ack_scan com evento real de hook já verificado no P1.3. Achado nessa
  passada: `<select>` ilegível no WebKitGTK (fundo branco nativo ignorando o
  CSS, texto cinza-claro quase invisível) — corrigido em `133a643`.
  Não verificado: um turno real do Claude via API abrindo o notch sozinho
  (o mecanismo já foi provado pelo evento sintético do P1.3; falta só a
  ponta-a-ponta com uma sessão real, item de baixo risco).

**Fase 2 — o resto do Epic C (confirmação, não escrita) + autostart/notificações
básicas (B.2, D.1).**

**Fase 3 — opcional, cortável sem quebrar nada acima**: Wayland (testar
`GDK_BACKEND=x11` sob XWayland antes de layer-shell nativo), multi-monitor,
empacotamento `.deb`/AppImage, self-update, issue upstream.

Ticket README (`docs/plans/tickets/README.md`) e os arquivos individuais
foram atualizados para refletir isso; A.2–A.5 e a maior parte de B/D/E ficam
formalmente adiados até a Fase 1 fechar.

**Fase 4 — pedida em 2026-09-14, no fim da fila** (não bloqueia nem depende de
nada acima): Epic G (i18n pt-BR) e Epic H (OpenCode como 5º provedor,
exploratório). Ver os épicos abaixo.

---

## Epic G — i18n: adicionar português do Brasil

Hoje `i18n.rs` traduz só o menu de botão direito da bandeja (`settings.html`
é explícito sobre isso: "the only part of Codenotch that is translated").
Locales existentes: `zh`, `ja`, `ko` (mais `en` como default/fallback e
`auto` que resolve pelo `$LANG`/`$LC_ALL` no Linux, registro do Windows no
Windows). 16 chaves ao todo: `autostart`, `hooks_missing`, `install`,
`lang_auto`, `language`, `open_data`, `quit`, `refresh`, `reset_pos`,
`settings`, `tray_bars`, `tray_icon`, `tray_numbers`, `tray_off`,
`tray_which`, `uninstall`.

- G.1 — adicionar `pt` a `resolve_auto()` (prefixo `pt` em `$LANG`/`$LC_ALL`
  no Linux; ramo `#[cfg(windows)]` correspondente também, já que `i18n.rs` é
  cross-platform e o Windows tem seu próprio bloco de detecção) e as 16
  entradas `("pt", chave) => "…"` em `t()`. Decisão a confirmar: `pt` genérico
  (Portugal + Brasil) ou `pt-BR` explícito — o pedido foi por Brasil
  especificamente, então o texto deve usar o vocabulário/ortografia do
  Brasil mesmo se a chave de detecção ficar `pt` sem sufixo de região (o
  `resolve_auto` de `ja`/`ko`/`zh` já usa prefixo solto, sem variante
  regional, então seguir o mesmo padrão é o caminho de menor atrito).

**Escopo**: só `i18n.rs`. Nenhum outro arquivo toca nisso. Ticket:
`docs/plans/tickets/G1-ptbr-i18n.md`.

## Epic H — OpenCode como 5º provedor (exploratório)

Pedido em 2026-09-14: rastrear o OpenCode CLI (já instalado nesta máquina)
do mesmo jeito que os outros — lendo o que a própria ferramenta já grava em
disco, nunca um login/OAuth próprio do Codenotch, nunca cópia de sessão web.
Isso expande a lista fechada do requisito 2 (só 4 provedores); autorizado
pelo operador como exceção explícita.

**O que já foi confirmado neste host** (sem nenhuma escrita, só leitura):

- `~/.local/share/opencode/auth.json` e `account.json` — credenciais por
  serviço (`google`, `github-copilot`, `deepseek`, `sakana-fugu`,
  `opencode-go`), tipo `api` (chave) ou `oauth` (access/refresh/expires).
  Isso são as chaves que o OpenCode usa para *chamar* os modelos, não uma
  cota do OpenCode em si — o OpenCode é uma ferramenta BYOK/roteadora, não
  uma assinatura com limite mensal como Claude/Codex/Cursor. Ou seja, **não
  existe um "% usado" natural pra desenhar no anel** do jeito que os outros
  4 têm.
- `~/.local/share/opencode/opencode.db` — SQLite real (`sqlite3`, ~1 GB
  neste host), com tabelas `session`, `message`, `project`, `event`,
  `workspace`, `credential`, `todo`, entre outras. `session` tem
  `time_created`/`time_updated`/`cost`/`tokens_input`/`tokens_output`/
  `title`/`directory` por linha — dado rico o bastante pra alimentar uma
  visão de **atividade de sessão** (equivalente ao que `watcher.rs` deriva
  dos `.jsonl` do Claude Code), não uma cota. `event` grava
  `session.created`/`session.updated`/`message.updated`/`message.part.updated`
  em sequência — dá pra tail-ear por `seq` como um log de eventos.
- Mesmo padrão técnico que o Cursor já usa no port: SQLite lido read-only
  (`rusqlite`, `SQLITE_OPEN_READ_ONLY`) — nenhuma dependência nova.

**Decisão em aberto (resolver no início do H.1, não adivinhar agora)**: o
valor certo é (a) uma leitura tipo Cursor — uma cota/contagem no anel sem
denominador (o modelo já suporta isso, `usedCount` sem `usedFraction`,
renderiza sem porcentagem); (b) uma entrada no motor de sessão
(`state.rs`/`watcher.rs`) mostrando sessões do OpenCode na lista, do jeito
que sessões do Claude aparecem; ou (c) as duas. O pedido original ("do jeito
que é feito no Claude") sugere que sessão (b) é o núcleo do pedido, e uma
leitura de custo/tokens acumulados como (a) é o extra opcional.

- H.1 — pesquisa + decisão: abrir `opencode.db` read-only, mapear o schema
  completo das tabelas relevantes (`session`, `event`, `project`), decidir
  (a)/(b)/(c) acima, escrever o ticket de implementação de verdade só depois
  disso (este H.1 é deliberadamente um ticket de investigação, não de
  código).
- H.2 — implementação, escopo definido pelo resultado do H.1.

**Escopo**: novo módulo `opencode.rs` (mirror de `cursor.rs`); nenhuma
mudança em `windows/`; sem dependência nova. Ticket:
`docs/plans/tickets/H1-opencode-research.md`.

---

## Pivô de prioridade — 2026-09-14

**Objetivo agora: só lançar uma versão instalável e estável para Linux.** Epic H
(OpenCode) fica pausado, sem dispatch novo (a tentativa de H.1 rodou duas vezes sem
produzir nada, aparentemente instabilidade do backend opencode-go/Kimi, não do escopo
do ticket em si — fica pra retomar depois, ticket já escrito e válido). Tradução
completa da Settings (além do menu da bandeja, já feito no G.1) também fica de fora
por enquanto, a pedido explícito.

**Isso resultou direto em Epic E.3 (empacotamento) andar rápido** — ver a entrada
atualizada acima: `.deb` e `AppImage` já buildam limpo, AppImage já rodou de verdade
no desktop. Falta só confirmar `sudo dpkg -i` num host com root disponível.

**B.2 (autostart XDG)** — ticket já escrito (`docs/plans/tickets/B2-xdg-autostart.md`),
mas não disparado ainda: não bloqueia "instalável", fica pra depois do E.3 fechar de
vez (a instalação real do `.deb` confirmada).

**C.5 (motor de sessão) e C.6 (arquivo/429)** — dispatchados, mas ambos travaram no
banner inicial do opencode sem produzir nada (mesmo sintoma do H.1), não é falha do
prompt. Não são bloqueio pra "instalável" (são confirmação/robustez de dados, não
empacotamento), então ficam de baixo prioridade por ora — não recomendo insistir
disparando de novo até esse padrão de trava do backend se resolver sozinho, ou o
usuário testar o backend fora deste fluxo.

**Próximo passo real**: pedir pro usuário rodar `sudo dpkg -i
cross-platform/target/release/bundle/deb/Codenotch_0.3.0_amd64.deb` (ou usar o
`.AppImage` direto, que já não precisa de root) num terminal com senha disponível,
confirmar que abre normal depois de instalado, e então Epic E.3 fecha por completo.

---

## Estrutura de branches e release — 2026-09-14

Adotado GitFlow: `development` e `release` criadas a partir do `main` (ainda só a
versão Windows, sem o port). `cross-platform-port` segue como branch de trabalho;
[PR #1](https://github.com/christiancesar/codenotch-crossplaform/pull/1) aberto pra
`development` com todo o histórico deste plano. Fluxo acordado:

```
cross-platform-port → (PR, merge manual) → development → release (estabiliza)
                                                              │
                                              merge em main + development
                                                              │
                                                    tag v* em main
                                                              │
                                            .github/workflows/release-publish.yml
                                              builda Windows+Linux, publica
                                              GitHub Release como DRAFT
```

Dois workflows novos, propósitos diferentes:
- `cross-platform-package.yml` — builda `.deb`/`AppImage`/`.msi` a cada push em
  `cross-platform-port`/`development`, sobe como artifact da própria run (exige
  login GitHub, expira em 90 dias). É o "disponibilizar no repositório" pedido:
  já roda sem esperar a cadeia até `main`.
- `release-publish.yml` — dispara só por tag `v*` em `main` (nunca por push direto
  em `release`, é assim que o GitFlow separa "preparar" de "publicar"). Builda os
  dois SOs via `tauri-apps/tauri-action` e publica como **rascunho** (draft) — não
  fica público sozinho, alguém revisa e clica Publish no GitHub antes de expor pra
  download. Decisão deliberada: publicar release pública é ação visível e
  semi-irreversível, não deveria disparar sozinha sem revisão.

Pendente: `.rpm` não foi tentado (sem host Fedora disponível nesta sessão);
`sudo dpkg -i` real ainda não confirmado (sem root não-interativo neste sandbox).

**Cadeia executada 2026-09-14**: PR #1 mergeado em `development` (fast-forward),
`development` → `release` → `main` (todos fast-forward, sem conflito), tag `v0.3.0`
criada em `main` e enviada — disparou `release-publish.yml`
([run](https://github.com/christiancesar/codenotch-crossplaform/actions)). Fica como
rascunho (draft) na página de Releases do repositório até alguém clicar Publish.