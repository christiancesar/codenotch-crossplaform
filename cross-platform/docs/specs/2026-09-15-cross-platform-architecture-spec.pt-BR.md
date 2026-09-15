# Spec de Arquitetura: Reconstruindo o Codenotch sobre o Scaffold do create-tauri-app

(rascunho em português só para revisão, a versão que vai pro repositório é a em
inglês, `cross-platform/docs/specs/2026-09-15-cross-platform-architecture-spec.md`)

Complementa `docs/specs/2026-09-11-linux-port-spec.md`. Aquele documento cobre o que o
port para Linux mantém e muda no nível de produto. Este cobre como o código é organizado.
O `cross-platform/codenotch` cresceu sem estrutura dos dois lados da fronteira de IPC, e
esta spec propõe reconstruí-lo sobre o scaffold padrão do `create-tauri-app`, com um
layout explícito pro backend, pro frontend e pro contrato entre os dois.

Os exemplos de código são ilustrativos. Mostram o formato da solução, não uma
implementação pronta, e não são uma revisão de cada ponto de chamada.

## Declaração do problema

Estado do `cross-platform/codenotch` na v0.3.0.

**Backend**

- `src/` são 20 módulos num único diretório plano. Só o `main.rs` tem 1527 linhas e
  concentra os subcomandos de CLI, o builder do Tauri, o posicionamento do notch, o
  hit-testing, o drag, o watchdog do ponteiro, a renderização e atualização do tray, as
  threads de limpeza em segundo plano e os 37 `#[tauri::command]`.
- Cada fonte de uso (`usage.rs` pro Claude, `codex.rs`, `cursor.rs`, `antigravity.rs`
  mais `agy_cli.rs`) reimplementa as mesmas peças: leitura de credencial, fetch HTTP,
  parsing, persistência em arquivo JSON, loop de polling e `app.emit(...)`. A detecção
  de atividade desses mesmos provedores fica em `activity.rs`, a busca de ícone em
  `glyphs.rs`, o diagnóstico em `doctor.rs`.
- Helpers duplicados: `now_ms` existe em 8 módulos, `sleep_interruptible` em 3, código
  de base64 em 3, `parse_iso`, `cap` e `open_ro` em 2 cada.
- Código específico de SO está espalhado em 11 módulos como pares inline de
  `#[cfg(windows)]` / `#[cfg(not(windows))]`, 27 deles no `main.rs`. A mesma função tem
  dois corpos no mesmo arquivo, e nada mostra quais preocupações de SO um módulo toca sem
  ler ele inteiro.

**Frontend**

- A UI são dois arquivos HTML estáticos, `ui/notch.html` (538 linhas) e
  `ui/settings.html` (1273 linhas), com blocos `<script>` inline. Sem componentes, sem
  estado compartilhado, sem build, Tauri acessado via `window.__TAURI__`
  (`withGlobalTauri: true`).
- Renderização por template string atribuída a `innerHTML`. O `settings.html` escapa os
  labels dos provedores porque eles vêm de servidores remotos; o `notch.html` interpola
  `${w.label}` sem escape (`ui/notch.html:286`). Dois arquivos, duas convenções.
- Toda chamada passa pela API core sem tipagem, nome do comando como string mais um
  objeto de argumentos solto:

  ```js
  invoke('set_scale', { scale: scalePct / 100 })
  invoke('get_usage').then(u => { usage = u || usage; renderRing(); })
  ```

  O notch também escuta 13 eventos (`usage`, `state`, `codex`, `notch_slots`, ...) com a
  mesma falta de tipagem.

- Nada liga `'set_scale'` a `fn set_scale(app: AppHandle, scale: f64)`. O formato do
  payload, o tipo de retorno e se a chamada pode falhar são fatos que um contribuidor
  precisa recuperar do código Rust, um comando de cada vez, 37 vezes. Essa carga
  cognitiva não some quando quem lê é uma LLM em vez de uma pessoa; o modelo ainda
  precisa carregar e cruzar os mesmos arquivos pra não chutar um formato.

Isso é normal pra um projeto em estágio inicial e não é uma crítica ao código atual. A
proposta é reconstruir os dois lados numa estrutura deliberada, não mover arquivos de
lugar.

## Escopo

Dentro do escopo:

- Um projeto novo gerado com `create-tauri-app` (React + TypeScript + Vite). O código da
  árvore atual é portado pra ele e reorganizado durante a mudança. Não é uma conversão
  1:1: cada módulo é dividido por responsabilidade ao entrar.
- O layout de módulos do backend: provedores, isolamento de plataforma, sessões, notch,
  tray, armazenamento, diagnóstico.
- A reescrita das janelas `notch` e `settings` como árvores React num único projeto Vite.
- Um contrato de IPC tipado e gerado, snake_case de ponta a ponta.
- Compatibilidade com instalações existentes: arquivos persistidos, o hook, autostart,
  nomes de binário.

Fora do escopo:

- Mudanças de produto. A régua de paridade é a v0.3.0: mesmo visual, mesmo
  comportamento, no Windows e no Linux.
- Adotar o contrato de provedor da spec do port Linux (`fidelity`, `usedFraction`, ...).
  Isso muda o formato dos dados salvos, então segue o caminho de migração em
  [Compatibilidade com instalações existentes](#compatibilidade-com-instalações-existentes),
  depois da reconstrução.
- `codenotch-hook`. Continua um crate separado e pequeno; aqui só aparecem os contratos
  de que ele depende.

## Ponto de partida: create-tauri-app

A base é o scaffold padrão. Qualquer pessoa reproduz:

```
cargo create-tauri-app
# ou: npm create tauri-app@latest
```

Respostas pra este projeto:

```
Project name        codenotch
Identifier          com.immidi.codenotch   (tem que ser igual ao atual, ver Compatibilidade)
Frontend language   TypeScript / JavaScript
Package manager     npm
UI template         React
UI flavor           TypeScript
```

O scaffold gerado:

```
codenotch/
├── src/                          # frontend (React, roda na webview)
│   ├── main.tsx                  # ponto de entrada React, monta <App />
│   ├── App.tsx
│   ├── App.css
│   ├── vite-env.d.ts
│   └── assets/
├── index.html                    # HTML de entrada do Vite
├── vite.config.ts
├── tsconfig.json
├── package.json
└── src-tauri/                    # Tauri (Rust, roda do lado do SO)
    ├── src/
    │   ├── main.rs                # ponto de entrada do binário, chama o run() do lib.rs
    │   └── lib.rs                 # Builder, plugins, invoke_handler![...]
    ├── capabilities/
    │   └── default.json           # permissões de IPC por janela
    ├── Cargo.toml
    ├── build.rs
    ├── icons/
    └── tauri.conf.json
```

O único comando dele já mostra o problema em escala 1: nada confere `"greet"` e
`{ name }` contra `fn greet(name: &str) -> String`.

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

A divisão existe por causa do process model do Tauri: o processo core é dono do SO, das
janelas e do acesso ao sistema; a webview renderiza e chama de volta via IPC.
https://v2.tauri.app/concept/process-model/

O que é adaptado depois de gerar:

1. **Layout.** `src-tauri/` vira `backend/`. `src/`, `index.html`, `vite.config.ts` e
   `tsconfig*.json` vão pra `frontend/`. O `package.json` fica na raiz do projeto, então
   o CLI do Tauri rodado da raiz continua encontrando `backend/tauri.conf.json`. A
   ligação exata (root do Vite, `devUrl`, `frontendDist`, `beforeDevCommand`,
   `beforeBuildCommand`) é definida e verificada no PR do scaffold.
2. **`tauri.conf.json`.** `productName`, `version`, `identifier`, as duas janelas com as
   propriedades atuais (`transparent`, `decorations`, `alwaysOnTop`, `skipTaskbar`,
   tamanhos, `visible: false`), configurações de bundle e ícones vêm do arquivo atual.
   `withGlobalTauri` vira `false` porque o frontend importa `@tauri-apps/api`. A CSP, hoje
   `null`, ganha uma política de verdade agora que todo asset vem do bundle.
3. **`Cargo.toml`.** Dependências atuais, incluindo as específicas de target, as features
   `tray-icon` e `image-png` e o `tauri-plugin-single-instance`. O padrão do scaffold de
   `[lib]` mais `main.rs` fino é mantido. O binário continua se chamando `codenotch`.
4. **Capabilities.** `default.json` continua concedendo `core:default` pra `notch` e
   `settings`.
5. **CI.** `actions/setup-node` e `npm ci` rodam antes do `tauri-action`. Os caminhos
   mudam no corte final (ver [Migração](#migração)).

Uma nota de nomenclatura: "janela" sempre significa uma janela do SO no nível do Tauri
(`notch` e `settings` em `app.windows`), nunca uma página no sentido de roteamento de
SPA. Uma árvore React por janela, em `frontend/src/app/notch` e
`frontend/src/app/settings`.

## Layout do projeto

```
cross-platform/codenotch/
├── package.json                  # vite, react, @tauri-apps/api, @tauri-apps/cli
├── backend/                      # Rust / Tauri (processo core)
└── frontend/                     # React + Vite (webview)
```

O nível raiz divide por processo, não por linguagem. O nome `ui/` foi abandonado porque
costuma significar biblioteca de componentes, que é uma coisa dentro de `frontend/`, não
o frontend inteiro.

## Estrutura do backend

### Por que não entities / repositories / use_cases

O primeiro rascunho desta spec propunha camadas de Clean Architecture. Foram
descartadas. A maior parte deste backend é código de integração: APIs do SO, endpoints
HTTP de fornecedores, arquivos locais, SQLite. Existe pouca regra de negócio entre
entrada e saída. Dividir por papel técnico espalharia cada provedor em quatro diretórios
e criaria um use case por comando pra funções de uma linha como `get_lang` ou
`nub_capable`.

O layout agrupa por funcionalidade e obtém responsabilidade única com três regras:

1. **Código específico de SO mora só em `platform/`.** Nenhum `#[cfg(windows)]` fora
   dele.
2. **Provedores, sessões e diagnóstico não conhecem o Tauri.** Não recebem `AppHandle` e
   não emitem nada. `app/` liga eles ao runtime.
3. **Comandos são adaptadores.** Um `#[tauri::command]` lê ou atualiza o `AppState`,
   chama uma função de módulo e retorna. Sem lógica própria.

### Árvore

```
backend/
├── Cargo.toml
├── build.rs
├── tauri.conf.json
├── capabilities/default.json
├── icons/
├── assets/glyphs/                     # marcas SVG embutidas (hoje glyphs/)
├── tests/fixtures/
│   ├── vendors/                       # respostas de API, dumps SQLite (fixtures de hoje)
│   └── persisted/v0.3.0/              # arquivos de uma instalação real v0.3.0, ver Compatibilidade
└── src/
    ├── main.rs                        # argv: subcomando de CLI ou lib::run()
    ├── lib.rs                         # árvore de módulos, run()
    ├── cli.rs                         # install-hooks, uninstall-hooks, autostart, doctor
    ├── app/                           # único lugar que segura AppHandle
    │   ├── mod.rs                     # Builder, plugins, setup()
    │   ├── state.rs                   # AppState
    │   ├── events.rs                  # eventos tipados enviados pra webview
    │   └── workers.rs                 # inicia scheduler, watcher, servidor do hook, limpezas
    ├── commands/                      # adaptadores #[tauri::command] finos
    │   ├── usage.rs                   # get_usage, get_codex, get_cursor, get_antigravity, refresh_usage
    │   ├── sessions.rs                # get_state, get_activity, focus_session, dismiss_session
    │   ├── notch.rs                   # drag_begin, set_hot, report_dpr, escala, slots do notch
    │   ├── tray.rs                    # opções, config e preview do tray
    │   ├── settings.rs                # idioma, ui flags, autostart, hooks, reset_notch_position
    │   └── system.rs                  # open_data_dir, open_provider_page, open_usage_page, log_js
    ├── providers/                     # um diretório por fonte de uso
    │   ├── mod.rs                     # trait UsageProvider, registro
    │   ├── model.rs                   # UsageSnapshot, LimitWindow, ProviderId, FetchError
    │   ├── scheduler.rs               # polling, backoff, persistência, aviso de mudança
    │   ├── claude/                    # hoje: usage.rs, parte claude do activity.rs
    │   │   ├── mod.rs                 # impl UsageProvider
    │   │   ├── credentials.rs
    │   │   ├── api.rs
    │   │   ├── parse.rs
    │   │   └── activity.rs
    │   ├── codex/                     # hoje: codex.rs, parte codex do activity.rs
    │   │   ├── mod.rs
    │   │   ├── credentials.rs
    │   │   ├── api.rs
    │   │   ├── parse.rs
    │   │   ├── rollout.rs             # fallback pelos arquivos de rollout locais
    │   │   └── activity.rs
    │   ├── cursor/                    # hoje: cursor.rs, parte cursor do activity.rs
    │   │   ├── mod.rs
    │   │   ├── credentials.rs         # state.vscdb
    │   │   ├── api.rs
    │   │   ├── parse.rs
    │   │   └── activity.rs
    │   └── antigravity/               # hoje: antigravity.rs, agy_cli.rs, parte do activity.rs
    │       ├── mod.rs
    │       ├── discovery.rs           # endpoint do language server local
    │       ├── bridge.rs
    │       ├── direct.rs
    │       ├── cli.rs                 # execução do CLI agy e parsing de cota
    │       ├── transcripts.rs         # requisições contadas pelos transcripts
    │       └── activity.rs
    ├── sessions/                      # rastreio de sessões do Claude Code
    │   ├── store.rs                   # hoje: state.rs
    │   ├── watcher.rs                 # watcher de transcripts
    │   ├── hook_server.rs             # hoje: server.rs, POST /event do codenotch-hook
    │   ├── hooks_install.rs           # merge em ~/.claude/settings.json
    │   └── sweep.rs                   # limpeza de sessões antigas, varredura de "visto"
    ├── notch/
    │   ├── placement.rs               # place_notch_sized, tamanho alvo, expand/collapse
    │   ├── hit_test.rs                # cursor_in_hot e seus testes (puro)
    │   ├── drag.rs
    │   └── pointer_watchdog.rs
    ├── tray/
    │   ├── menu.rs                    # hoje: tray.rs
    │   ├── render.rs                  # hoje: trayicon.rs (desenho de pixels, puro)
    │   └── updater.rs                 # start_tray_updater, paint_tray, reading_for_slot
    ├── glyphs/                        # sanitização e cache; candidatos vêm de cada provedor
    ├── config/
    │   ├── model.rs                   # Config, TraySlot
    │   └── migrate.rs                 # os upgrades que o config::load() faz inline hoje
    ├── storage/                       # todo arquivo que o Codenotch grava
    │   ├── paths.rs                   # config_dir()/codenotch/*
    │   ├── atomic.rs                  # arquivo temporário + rename
    │   └── versioned.rs               # versão de schema, migrações, quarentena
    ├── diagnostics/                   # hoje: doctor.rs + diag.rs, ver abaixo
    ├── i18n/
    ├── platform/                      # ver Isolamento de plataforma
    └── support/                       # now_ms, sleep_interruptible, base64, sqlite open_ro
```

### Provedores

Todo provedor faz o mesmo trabalho com fontes diferentes, então a parte comum vira uma
trait e o loop repetido vira um scheduler só.

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

- `FetchError` unifica os enums por módulo que existem hoje (`FetchErr` em `usage.rs` e
  `cursor.rs`, `LiveErr` em `codex.rs`): precisa de autenticação, rate limit com
  retry-after, rede, parsing.
- `scheduler.rs` assume o que cada módulo repete hoje: o intervalo, o backoff (hoje só o
  Claude tem, em `usage.rs::backoff_secs`), a persistência via `storage/` e um callback
  de mudança que `app/` transforma em evento. O scheduler também nunca vê `AppHandle`.
- O Antigravity mantém seus dois runtimes (CLI e language server legado) como estratégia
  interna do próprio provedor.
- As funções de parsing se mudam sem alteração junto com seus testes de fixture
  (`parse_response`, `windows_from_usage`, `parse_summary`, `parse_quota`, o fixture do
  bridge). São os testes mais valiosos da árvore e a prova de que o port não mudou
  comportamento.

### Isolamento de plataforma

O que é específico de SO hoje:

| Preocupação | Funções hoje | Onde |
| --- | --- | --- |
| Comportamento de janela | `noactivate`, `expand_notch`, `collapse_notch`, `target_logical_size`, `attach_console` | `main.rs` |
| Entrada do ponteiro | `left_button_down` | `main.rs` |
| Foco | `focus_terminal`, `focus_claude_desktop`, `fg_pid`, `ack_scan` | `focus.rs`, `main.rs` |
| Processos | `proc_maps`, `claude_net_pid`, `claude_io_bytes`, `lower_thread_priority`, `listening_ports`, `run_hidden` | `focus.rs`, `activity.rs`, `antigravity.rs` |
| Pseudo-terminal | `run_cmd_conpty` | `agy_cli.rs` |
| Credenciais | `read_credential_raw` | `antigravity.rs` |
| Abrir coisas | `open_data_dir`, `open_provider_page`, `open_usage_page` | `main.rs` |
| Autostart | só chave Run via `reg.exe`; no Linux as chamadas falham | `autostart.rs` |
| Localizar executáveis | `find_agy`, `find_executable`, nome do binário do hook | `agy_cli.rs`, `codex.rs`, `hooks_install.rs` |
| Ícones | `from_exe` | `glyphs.rs` |
| Idioma | idioma do sistema | `i18n.rs` |

Escolher entre duas implementações conforme o SO é normal. O problema é onde a escolha
acontece. Depois da reconstrução ela acontece uma vez só, em `platform/mod.rs`:

```
platform/
├── mod.rs              # traits pequenas + a implementação do SO alvo
├── windows/
│   ├── mod.rs          # pub struct Platform; impl de todas as traits
│   ├── window.rs
│   ├── input.rs
│   ├── focus.rs
│   ├── process.rs
│   ├── pty.rs
│   ├── credentials.rs
│   ├── open.rs
│   ├── autostart.rs    # chave Run do HKCU
│   ├── icon.rs
│   ├── locale.rs
│   └── console.rs
└── linux/              # mesmos arquivos: x11rb, GTK, /proc, XDG
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

- As traits são pequenas de propósito. Quem consome depende só do que usa
  (`fn focus_session(focus: &impl Focus, ...)`), e os testes passam um fake.
- Cada módulo de SO tem uma struct `Platform` que implementa todas as traits. Um método
  faltando num SO vira erro de compilação no job de CI daquele SO, não surpresa em
  runtime.
- O `self::` no `pub use` importa: `windows::` sozinho fica ambíguo com o crate
  `windows`.
- A árvore cross-platform mira Windows e Linux, então o código diz
  `target_os = "linux"` em vez do `not(windows)` de hoje.
- O autostart ganha implementação no Linux (entrada de autostart XDG) como parte disso,
  já que a trait exige uma.
- O CI falha se `cfg(windows)` ou `cfg(not(windows))` aparecer fora de `platform/`.

### Diagnóstico

Hoje o `doctor.rs` é uma função que monta uma `String` importando `usage`, `codex`,
`cursor`, `antigravity`, `glyphs`, `activity` e `watcher` direto. O `diag.rs` é a
variante `doctor deep`. Os dois viram um diretório:

```
diagnostics/
├── mod.rs              # run(), run_deep(), grava doctor.log via storage/
├── report.rs           # Report feito de seções, renderizado em texto no final
├── checks/
│   ├── config.rs       # caminho e valores da config
│   ├── port.rs         # porta do servidor do hook livre ou em uso
│   ├── sessions.rs     # raízes observadas, transcripts mais recentes, parse do final
│   ├── providers.rs    # percorre o registro e chama probe()
│   ├── glyphs.rs
│   └── watch_log.rs
└── deep/               # hoje: diag.rs
    ├── recent_files.rs
    ├── sqlite_dump.rs
    └── json_dump.rs
```

Cada verificação é independente e testável. `providers.rs` percorre o registro, então um
provedor novo aparece no `doctor` sem mexer no diagnóstico.

## A camada de IPC tipada

### snake_case de ponta a ponta

Nomes de comando, chaves de argumento, campos de resposta e nomes de evento são todos
snake_case.

- O serde já serializa campos Rust em snake_case, os arquivos JSON persistidos são
  snake_case e o JS atual já lê `fetched_at` e `resets_at`. Nenhuma camada de conversão
  em lugar nenhum.
- O identificador é o mesmo em Rust e TypeScript, então uma busca encontra os dois lados.
- O Tauri converte os **argumentos** de comando pra camelCase por padrão: o
  `settings.html` manda `{ notchVisible, trayVisible }` pra
  `set_ui_flags(notch_visible, tray_visible)`. Todo comando é declarado com
  `#[tauri::command(rename_all = "snake_case")]` pra desligar isso. Valores de retorno não
  são afetados; seguem o serde.

### Gerado a partir do Rust, não escrito à mão

O primeiro rascunho propunha tipos escritos à mão primeiro e geração de código depois. O
próprio rascunho é o argumento contra: o `UsageSnapshot` dele (`providerID`,
`fetchedAt: string`, `usedFraction`, `fidelity`) não batia com o que o `get_usage`
retorna (`fetched_at: u64`, `used`, `count`, `derived`). `invoke<T>` é só um cast, então
teria compilado e produzido `undefined` em runtime.

Os tipos são gerados com `tauri-specta`. Os tipos do modelo derivam `specta::Type`, os
comandos ganham `#[specta::specta]` e o builder exporta comandos e eventos pra
`frontend/src/libs/ipc/bindings.ts`. O arquivo é commitado; o CI gera de novo e falha se
houver diferença.

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

Como o arquivo gerado fica pros tipos de hoje (ilustrativo):

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

Pontos de chamada:

```tsx
const usage = await commands.get_usage();
await commands.set_scale(scalePct / 100);
```

em vez de:

```js
invoke('get_usage').then(u => { usage = u || usage; renderRing(); })
```

Um spike na fase 1 confirma, antes de os tipos do modelo começarem a derivar
`specta::Type`:

- **Versão.** O `tauri-specta` ainda é release candidate (`2.0.0-rc.25` quando isto foi
  escrito). Fixar `tauri-specta` e `specta` com `=`.
- **Casing dos argumentos.** `rename_all = "snake_case"` junto com `#[specta::specta]`
  gera chaves snake_case nos wrappers.
- **Inteiros de 64 bits.** `fetched_at`, `resets_at`, `backoff_until` (`u64`) e `count`
  (`i64`) precisam de `dangerously_cast_bigints_to_number`. São milissegundos de epoch e
  contadores, muito abaixo de 2^53, então `number` é exato pra eles.
- **Nomes de evento.** Os nomes gerados a partir dos tipos de evento batem com os nomes
  snake_case atuais, ou são renomeados uma vez nos dois lados.

Se o spike falhar, o plano B é gerar só os tipos (`specta` ou `ts-rs`) e manter um
`commands.ts` fino escrito à mão que liga nomes a tipos gerados, revisado no mesmo PR do
comando Rust.

### Validação em runtime (Zod): avaliada, não adotada como contrato

- O Zod valida em runtime contra um schema que alguém escreve. Troca um `undefined`
  silencioso por um erro visível, mas só no caminho de código que executa, e o schema
  continua sendo uma segunda cópia do tipo Rust que pode divergir.
- Frontend e backend saem no mesmo binário. Não existe diferença de versão entre eles em
  produção. Um descompasso é erro de build, e a geração de código pega isso em tempo de
  compilação.
- Dados que vêm de fora de verdade (APIs de fornecedores, arquivos em disco) são
  parseados pelo serde no Rust antes de chegar na webview.

Rever se a webview passar a consumir dados que o backend não produziu. Se quiserem
checagem em runtime nos builds de dev, gerar os schemas a partir do `bindings.ts` (por
exemplo com `ts-to-zod`) em vez de escrever à mão.

## Estrutura do frontend

```
frontend/
├── index.html                      # a única entrada do Vite
├── src/
│   ├── main.tsx                    # escolhe a árvore pelo label da janela
│   ├── app/
│   │   ├── notch/                  # substitui o notch.html
│   │   │   ├── Notch.tsx
│   │   │   ├── Ring.tsx
│   │   │   ├── HoverCard.tsx
│   │   │   └── notch.css
│   │   └── settings/               # substitui o settings.html
│   │       ├── Settings.tsx
│   │       ├── panes/              # um componente por seção (tray, notch, comportamento, ...)
│   │       └── settings.css
│   ├── components/                 # peças de UI compartilhadas
│   ├── contexts/                   # providers de contexto React compartilhados
│   ├── libs/
│   │   ├── ipc/
│   │   │   ├── bindings.ts         # gerado pelo tauri-specta
│   │   │   └── index.ts            # re-exports + tratamento de falha compartilhado
│   │   └── ...                     # formatação, tempo, outros helpers puros
│   └── vite-env.d.ts
├── vite.config.ts
└── tsconfig.json
```

As duas janelas do SO carregam o mesmo `index.html`. Uma janela do Tauri é uma webview
com um label, e nada impede duas delas de carregarem o mesmo bundle. O `main.tsx` escolhe
a árvore:

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

- Uma entrada só mantém o setup padrão do Vite. Um build multi-entrada
  (`rollupOptions.input`) também funciona, mas não traz ganho aqui, já que as duas janelas
  compartilham a camada de IPC e boa parte dos componentes.
- O `lazy` mantém o código de settings fora da janela do notch, que fica sempre na tela.
  Cada árvore exporta seu componente como `default`.
- Cada árvore importa sua própria folha de estilo. A janela do notch é transparente, então
  estilos de página (fundo do `body`, margens) pertencem a cada árvore, nunca a um CSS
  global compartilhado.
- O React escapa texto interpolado por padrão, o que elimina o `innerHTML` sem escape do
  `notch.html`. Os glyphs SVG dos provedores são o único lugar que precisa de markup cru,
  e já chegam sanitizados pelo Rust (`sanitize_svg`).
- `libs/ipc/index.ts` mantém o comportamento que o `settings.html` tem hoje em `call()`:
  um comando que falha pinta a faixa de erro e resolve com um fallback em vez de deixar a
  janela renderizada pela metade.

Os HTML originais servem de referência durante a reescrita. Não são mantidos nem servidos
junto com o frontend novo.

## Compatibilidade com instalações existentes

**Esta seção é um bloqueio.** Nenhum PR da reconstrução entra enquanto as verificações
abaixo falharem.

### O que existe na máquina do usuário

Em `dirs::config_dir()/codenotch/` (`%APPDATA%\codenotch` no Windows,
`~/.config/codenotch` no Linux):

| Arquivo | Gravado por | Conteúdo |
| --- | --- | --- |
| `config.json` | `config.rs` | porta, idioma, posição e escala do notch, slots do tray e do notch, visibilidade |
| `usage.json` | `usage.rs` | snapshot do Claude, incluindo `backoff_until` |
| `codex.json`, `cursor.json`, `antigravity.json` | módulos de provedor | snapshot por provedor |
| `glyphs/` | `glyphs.rs` | ícones de fornecedor em cache |
| `quota-work/` | `agy_cli.rs` | diretório de trabalho do CLI do Antigravity |
| `run.log`, `install.log`, `doctor.log`, `watch.log` | vários | logs |

Fora desse diretório: entradas de hook em `~/.claude/settings.json` (`hooks_install.rs`)
e, no Windows, o valor `Codenotch` em `HKCU\...\Run` (`autostart.rs`).

O arquivo de snapshot do Claude é `usage.json`, não `claude.json`. Um store indexado por
id de provedor precisa mapear o nome legado.

### Por que isso é perigoso hoje: as falhas são silenciosas e destrutivas

Todo loader segue o mesmo padrão:

```rust
std::fs::read_to_string(path)
    .ok()
    .and_then(|t| serde_json::from_str(&t).ok())
    .unwrap_or_default()
```

Se um campo for renomeado ou mudar de tipo durante a reconstrução, o arquivo inteiro
falha no parse e o app sobe com valores padrão sem registrar nada.

Com o `config.json` é pior. O `setup()` salva a config logo depois de iniciar (o bloco
"Persist the config (codenotch-hook reads the port from it)" no `main.rs`), então na
primeira execução da versão nova os valores padrão **sobrescrevem** o arquivo do usuário.
Posição do notch, escala, slots do tray e do notch, idioma e visibilidade somem de vez,
sem erro em lugar nenhum.

Nos snapshots de provedor a perda é menor, mas real: perder `backoff_until` faz o polling
do Claude ignorar um backoff de rate limit em andamento.

As gravações também não são atômicas (`std::fs::write` direto, só o `agy_cli.rs` grava via
arquivo temporário). Um crash no meio da gravação deixa um arquivo truncado, que cai no
mesmo reset silencioso.

### Solução

1. **Formato gravado separado do modelo.** Cada store tem seu próprio DTO com os nomes de
   campo exatos de hoje (`StoredConfig`, `StoredSnapshot`) e conversões `From` pros tipos
   do modelo. Tipos de modelo e de IPC podem ser renomeados à vontade; o formato em disco
   só muda pelo passo 2.
2. **Arquivos versionados com migrações.** Os arquivos carregam `"schema": N`. Sem a
   chave significa v1, o formato da v0.3.0. O loader lê um `serde_json::Value`, roda as
   migrações em ordem e só então desserializa. Os upgrades que o `config::load()` faz
   inline hoje (`tray_providers` pra `tray_slots`, `notch_providers` pra `notch_slots`,
   fixar `tray_mode` em configs anteriores a essa opção) viram as primeiras migrações.
   Adotar o `fidelity` da spec do port Linux depois é uma migração v1 pra v2
   (`derived: true` vira `fidelity: "derived"`).
3. **Nunca sobrescrever um arquivo que não pôde ser lido.** Um arquivo que existe mas
   falha no parse é renomeado pra `<nome>.unreadable-<timestamp>` e registrado no
   `run.log`. O app roda com padrões e o original continua no disco. O save incondicional
   no `setup()` sai; a config é gravada quando muda ou quando o arquivo não existe.
4. **Gravação atômica em todo lugar.** `storage/atomic.rs` (arquivo temporário mais
   rename, vindo do `agy_cli.rs::atomic_write`) é o único jeito de gravar um arquivo
   persistido.
5. **Fixtures de referência da v0.3.0.** Antes do primeiro PR de backend, arquivos reais
   de uma instalação v0.3.0 (anonimizados) entram em
   `backend/tests/fixtures/persisted/v0.3.0/`, junto com os formatos legados que o
   `config::load()` ainda trata (sem `tray_mode`, só com `tray_providers`). Os testes
   carregam cada um e conferem que todo valor sobrevive. Esses testes ficam depois da
   reconstrução.

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

### Contratos que não mudam

- **Nomes dos binários** `codenotch` e `codenotch-hook`. O hook inicia o app principal
  pelo nome a partir do próprio diretório (`codenotch-hook/src/main.rs`, `spawn_main`), e
  as entradas em `~/.claude/settings.json` apontam pro `codenotch-hook` e são reconhecidas
  por esse nome (`hooks_install.rs`, `is_ours`).
- **A chave `"port"` no nível raiz do `config.json`.** O hook encontra ela por varredura
  de texto, não por serde (`read_port`).
- **A rota do hook** `POST /event?e=...&ppid=...` em `127.0.0.1`.
- **`identifier` e `productName`.** Os instaladores e o plugin de instância única derivam
  a identidade do app deles.
- **Autostart**: o nome do valor Run `Codenotch` e a flag `--silent`.
- **Subcomandos de CLI**: `install-hooks`, `uninstall-hooks`, `autostart on|off`,
  `doctor [deep]`.

## Migração

O projeto novo é construído ao lado do atual, em `cross-platform/codenotch-next/`, até o
corte final. A árvore atual continua recebendo correções de bug durante a reconstrução;
toda correção feita lá é portada pra árvore nova num PR seguinte. A árvore nova fica fora
do workspace do Cargo (`exclude` em `cross-platform/Cargo.toml`) porque os dois pacotes se
chamam `codenotch`.

0. **Bloqueio de compatibilidade.** Fixtures de referência dos arquivos persistidos, a
   lista de contratos congelados e um checklist de paridade do comportamento da v0.3.0
   no Windows e no Linux (notch, hover card, drag, escala, modos do tray, cada painel de
   settings, hooks, autostart, doctor).
1. **Scaffold.** Rodar o `create-tauri-app`, adaptar o layout pra `backend/` e
   `frontend/`, trazer `tauri.conf.json`, `Cargo.toml`, capabilities e ícones. O CI
   compila no Windows e no Linux com as duas janelas abrindo vazias. O spike do
   `tauri-specta` roda aqui, com um comando e um evento.
2. **Fundações.** `support/`, `storage/` (load versionado, gravação atômica, quarentena),
   `config/` com migrações passando nas fixtures de referência, traits de `platform/` com
   as duas implementações.
3. **Provedores.** Trait, registro e scheduler, depois um provedor por PR, começando pelo
   Claude como referência. Código de parsing e testes de fixture se mudam sem alteração.
4. **Sessões, notch, tray, glyphs, diagnóstico, CLI.**
5. **Comandos e bindings.** Adaptadores finos, `rename_all = "snake_case"`, `bindings.ts`
   gerado e commitado, checagem de diferença no CI.
6. **Frontend.** `notch` e `settings` reescritos em React sobre o `bindings.ts`. Pode
   começar em paralelo às fases 3 e 4 assim que os primeiros comandos existirem.
7. **Corte final.** O checklist de paridade passa nos dois SOs. Instalar por cima de uma
   v0.3.0 preserva a config e os snapshots do usuário. A árvore antiga é removida,
   `codenotch-next/` é renomeado pra `codenotch/`, os membros do workspace viram
   `codenotch/backend` e `codenotch-hook`, e `projectPath` e `working-directory` dos
   workflows são atualizados.
