# Exemplo: agentes locais para o porte cross-platform

Este documento mostra como criar agentes no **contexto local do projeto** Codenotch,
seguindo a mesma lógica de pool usada pelos projetos `mobile` e `api` do oikos:

- **T1 primary**: OpenRouter
- **T1 fallback**: local opencode (`big-pickle`, e opcionalmente outros modelos free)
- **T2**: opencode-go com modelos de maior contexto (Kimi 2.7, GLM)
- **Revisor**: modelo barato/free, somente leitura
- **Orquestrador**: opencode-go com Kimi 2.7 para planejamento/review/merge

Os arquivos criados ficam dentro do próprio repo, então qualquer pessoa que abrir o
projeto herda a mesma esteira.

## Arquivos criados

```
codenotch/
├── opencode.json                              ← registry de agentes + references
└── .opencode/
    └── prompt/
        ├── codenotch-exec.md                  ← prompt do executor
        ├── codenotch-review.md                ← prompt do revisor
        └── codenotch-orchestrator.md          ← prompt do orquestrador
```

> Os agentes em `.opencode/agent/executor.md` e `.opencode/agent/reviewer.md` continuam
> existindo como subagentes do modo `task`. Eles são a forma "arquivo markdown" do
> opencode. O `opencode.json` aqui mostra a forma **registry JSON** que mobile/api usam,
> com modelos e permissões declarados explicitamente.

## O registry (`opencode.json`)

A section `agent` define cada agente:

```json
"agent": {
  "codenotch-exec-openrouter": {
    "description": "Executor T1 — OpenRouter primary ...",
    "mode": "primary",
    "model": "openrouter/auto",
    "prompt": "{file:./.opencode/prompt/codenotch-exec.md}",
    "permission": { "edit": "allow", "bash": "allow", "webfetch": "allow" }
  },
  "codenotch-exec-big-pickle": {
    "description": "Executor T1 fallback — modelo local big-pickle ...",
    "mode": "primary",
    "model": "opencode/big-pickle",
    "prompt": "{file:./.opencode/prompt/codenotch-exec.md}",
    "permission": { "edit": "allow", "bash": "allow", "webfetch": "allow" }
  },
  "codenotch-exec-opencode-go-kimi": {
    "description": "Executor T2 — opencode-go com Kimi 2.7 ...",
    "mode": "primary",
    "model": "opencode-go/kimi-k2.7-code",
    "prompt": "{file:./.opencode/prompt/codenotch-exec.md}",
    "permission": { "edit": "allow", "bash": "allow", "webfetch": "allow" }
  },
  "codenotch-review": {
    "description": "Revisor — lê e reporta, nunca edita",
    "mode": "primary",
    "model": "opencode/ling-3.0-tiny-free",
    "prompt": "{file:./.opencode/prompt/codenotch-review.md}",
    "permission": { "edit": "deny", "bash": "allow" }
  }
}
```

### Campos

- `description`: aparece na lista de agentes disponíveis.
- `mode`: `primary` (agente principal da sessão) ou `subagent` (chamado via `task`).
- `model`: provider/modelo. Providers (`openrouter`, `opencode`, `opencode-go`, `ollama`)
  devem estar configurados no `~/.config/opencode/opencode.json` global (com as API keys).
- `prompt`: arquivo de prompt que o agente recebe no contexto; `{file:./...}` carrega do
  repo.
- `permission`: mapa de permissões. `edit: deny` para revisores/orquestradores que não
  devem escrever código.

## Prompts locais

Os prompts são arquivos `.md` em `.opencode/prompt/`. Eles devem ser **específicos do
repo**:

- `codenotch-exec.md`: regras do Codenotch (`#[cfg(not(windows))]`, `AGENTS.md`, gates de
  `cargo`, verificação em desktop real, formato de reporte).
- `codenotch-review.md`: checklist de revisão (escopo, guardrails, multiplataforma,
  convenções, testes).
- `codenotch-orchestrator.md`: responsabilidades de planejamento, despacho, validação e
  merge.

## Como usar

1. O orquestrador (você) inicia uma sessão com o agente `codenotch-orchestrator`.
2. Para cada ticket, despacha para `codenotch-exec-openrouter` (T1).
3. Se o T1 falhar (timeout, erro de parsing, capacidade insuficiente), escala para
   `codenotch-exec-opencode-go-kimi` (T2).
4. Antes de mergear, opcionalmente passe o diff pelo `codenotch-review`.

## Diferença para mobile/api

- mobile/api mantêm os prompts em `~/.config/opencode/prompt/` (config global).
- No Codenotch os prompts foram trazidos para **dentro do repo** (`.opencode/prompt/`),
  tornando o contexto local autocontido e versionável.

## Notas

- `opencode-go/glm-4.5` é um ID exemplo; substitua pelo ID real do modelo GLM disponível na
  sua instalação opencode-go.
- API keys de OpenRouter/Groq/OmniRoute ficam no `~/.config/opencode/opencode.json` global —
  nunca commitadas no repo.
- A `references` section já existente no `opencode.json` aponta para o plano, spec, notas e
  `windows/`, então todo agente carrega esses contextos automaticamente.