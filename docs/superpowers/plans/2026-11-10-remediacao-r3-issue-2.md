# NEXO — PR #10 | Remediação R3 da Issue #2 — Plano de Implementação

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Corrigir o defeito P1 de identificação e encerramento de janelas no teste nativo de ciclo de vida (`crates/nexo-desktop/tests/lifecycle_tests.rs`), garantindo associação inequívoca entre o processo filho e sua janela via PID (`wmctrl -l -p`), encerramento estrito por Window ID (`wmctrl -i -c <window_id>`), suíte completa de testes de regressão para cenários de concorrência e falha, e correção documental rigorosa no ADR 0002.

**Architecture:** Refatorar a infraestrutura de testes nativos de ciclo de vida em módulos/funções de domínio bem delimitadas:
1. `WindowLocator`: parser determinístico e validador de associação processo/janela baseado em `wmctrl -l -p` e propriedades EWMH (`_NET_WM_PID`), imune a colisões de título textual;
2. `SafeWindowCloser`: despachante de encerramento gracioso EWMH exclusivamente por ID numérico/hexadecimal (`wmctrl -i -c <id>`);
3. Bateria de testes de regressão: isolamento contra janelas preexistentes, detecção de término prematuro, captura de exit code diferente de zero, acionamento de timeout com limpeza via RAII `ChildGuard`, e detecção precisa de ausência de suporte EWMH/X11 (incluindo Wayland puro sem ponte);
4. Atualização documental do ADR 0002 registrando a correção do histórico de panic da stack gráfica e as novas garantias de ciclo de vida.

**Tech Stack:** Rust 1.99.1, Rune (`dev` @ `42f2d354`), wgpu 30.0.1, X11/EWMH (`wmctrl -l -p`, `wmctrl -i -c`), cargo test/clippy/fmt, vitest/eslint/tsc.

**Spec:** Prompt de remediação R3 da Issue #2 e PR #10 (`feat/m1-rune-linux-foundation`).

---

## Global Constraints

- **Repositório:** `https://github.com/pedro-labsabs/nexo`
- **PR:** `https://github.com/pedro-labsabs/nexo/pull/10`
- **Branch:** `feat/m1-rune-linux-foundation` (base: `origin/recovery/r1-from-nex0`)
- **HEAD reprovado:** `f626441510594bdf1bda2016ff80de7d923db40b`
- Preservar integralmente as 18 suítes Rust e 30 TypeScript já aprovadas.
- Não abrir nova PR, não redefinir escopo da Issue #2, não fazer merge e não fechar a issue antes da auditoria do mantenedor.
- Nunca fechar janelas por correspondência de título (`wmctrl -c <title>` é estritamente proibido).
- Sempre associar a janela ao PID do processo filho e fechar pelo ID da janela (`wmctrl -i -c <window_id>`).
- Se a associação processo/janela for insegura ou não suportada, registrar a limitação e abortar sem agir sobre janelas de terceiros.
- Corrigir o ADR 0002 para registrar que o erro original do wgpu foi um panic no processo NEXO disparado pela stack gráfica.

---

### Task 1: Parser e Validador Determinístico de Janelas por PID

**Files:**
- Modify: `crates/nexo-desktop/tests/lifecycle_tests.rs`

**Interfaces:**
- Consumes: Linhas brutas de saída de `wmctrl -l -p`, PID do processo filho (`u32`).
- Produces:
  - `struct WindowInfo { id: String, desktop: i32, pid: u32, host: String, title: String }`
  - `fn parse_wmctrl_l_p(output: &str) -> Vec<WindowInfo>`
  - `fn find_window_by_pid(windows: &[WindowInfo], target_pid: u32) -> Result<WindowInfo, WindowBindingError>`
  - `enum WindowBindingError { PidNotFound(u32), PidNotSupported, AmbiguousWindows(Vec<String>) }`

- [ ] **Step 1: Escrever testes unitários em `lifecycle_tests.rs` para o parser e associação de janela por PID**

Criar casos de teste cobrindo:
1. `parse_wmctrl_l_p_valid_lines`: linhas padrão com ID hex, desktop, PID, hostname e título com múltiplos espaços.
2. `find_window_by_pid_ignores_preexisting_window_with_same_title`: lista contendo uma janela preexistente com título "NEXO" pertencente ao PID 1111 e outra janela pertencente ao PID 2222 do teste. A busca por PID 2222 deve retornar exclusivamente a janela do PID 2222, ignorando a preexistente.
3. `find_window_by_pid_returns_not_found_when_no_match`: quando a janela do PID do teste ainda não abriu, mesmo que haja uma janela preexistente intitulada "NEXO".
4. `find_window_by_pid_rejects_zero_pid`: quando o window manager reporta PID 0 (propriedade `_NET_WM_PID` ausente).

- [ ] **Step 2: Executar os testes para confirmar que falham (TDD)**

Executar: `cargo test -p nexo-desktop --test lifecycle_tests parse_wmctrl`
Esperado: FAIL (funções ainda não implementadas).

- [ ] **Step 3: Implementar `parse_wmctrl_l_p` e `find_window_by_pid`**

Implementar a estrutura `WindowInfo`, os erros tipados `WindowBindingError` e a lógica de parsing estrito das colunas de `wmctrl -l -p`.

- [ ] **Step 4: Executar os testes unitários e verificar aprovação**

Executar: `cargo test -p nexo-desktop --test lifecycle_tests parse_wmctrl`
Esperado: PASS em todos os 4 cenários.

- [ ] **Step 5: Commit do parser e associação determinística de janelas**

```bash
git add crates/nexo-desktop/tests/lifecycle_tests.rs
git commit -m "test(desktop): implementar parser e associação determinística de janelas por PID"
```

---

### Task 2: Refatorar o Teste Nativo de Ciclo de Vida para Encerramento Seguro por Window ID

**Files:**
- Modify: `crates/nexo-desktop/tests/lifecycle_tests.rs`

**Interfaces:**
- Consumes: `find_window_by_pid`, `WindowInfo`, `ChildGuard`, `Command::new("wmctrl")`.
- Produces: Teste `native_window_process_lifecycle_clean_exit` seguro:
  - Polling de até 5s executando `wmctrl -l -p`.
  - Associação comprovada: `window.pid == child.id()`.
  - Encerramento: `wmctrl -i -c <window.id>`.
  - Tratamento de limitação ambiental: aborta e documenta se EWMH/_NET_WM_PID não estiver disponível, sem fechar janelas alheias.
  - Verificação de logs contra panics e validação de `status.code() == Some(0)`.

- [ ] **Step 1: Implementar o encerramento seguro por Window ID**

Substituir em `native_window_process_lifecycle_clean_exit`:
1. Polling agora chama `wmctrl -l -p` a cada 100ms.
2. Faz o parse da saída e busca a janela vinculada ao `child.id()`.
3. Garante que `window.pid == child_pid`.
4. Ao encontrar, despacha `Command::new("wmctrl").args(["-i", "-c", &window.id])`.
5. Se o timeout de polling for atingido sem encontrar janela vinculada ao PID do processo:
   - Se nenhuma janela do sistema tem PID válido (PID 0 em todas), reporta limitação de ambiente EWMH e encerra o child com segurança sem falso sucesso.
   - Se o processo filho saiu prematuramente, falha com pânico explícito informando o status de saída antecipada e os logs.
   - Se o child continua rodando mas não gerou janela mapeada para seu PID, falha e limpa o child via `ChildGuard`.

- [ ] **Step 2: Executar o teste nativo localmente**

Executar: `cargo test -p nexo-desktop --test lifecycle_tests native_window_process_lifecycle_clean_exit -- --nocapture`
Esperado: PASS com log demonstrando:
`[lifecycle-test] Child process spawned with PID <PID>`
`[lifecycle-test] Associated window found: ID <0x...>, PID <PID>, Title: NEXO`
`[lifecycle-test] Requesting graceful close via wmctrl -i -c <0x...>`
`[lifecycle-test] Child process exited cleanly with code 0`

- [ ] **Step 3: Commit da refatoração do teste de ciclo de vida nativo**

```bash
git add crates/nexo-desktop/tests/lifecycle_tests.rs
git commit -m "fix(desktop): vincular janela ao PID do processo filho e encerrar por window ID"
```

---

### Task 3: Implementar Bateria de Testes de Regressão Obrigatórios

**Files:**
- Modify: `crates/nexo-desktop/tests/lifecycle_tests.rs`

**Interfaces:**
- Produces: Testes de regressão automatizados para todos os cenários exigidos pela auditoria:
  1. `regression_preexisting_nexo_window_is_not_closed`: simula ou valida que uma janela preexistente não é alvo do encerramento.
  2. `regression_premature_child_exit_triggers_failure`: valida que se o processo filho terminar antes de criar janela, o teste detecta a saída prematura imediatamente.
  3. `regression_nonzero_exit_code_triggers_failure`: valida que se o processo terminar com código diferente de zero, o ciclo de vida detecta e falha.
  4. `regression_child_hang_triggers_timeout_and_clean_kill`: valida que um processo que não responde é eliminado pelo `ChildGuard` após o timeout.
  5. `regression_missing_ewmh_or_display_handled_cleanly`: valida que ambientes sem X11/EWMH ou Wayland puro sem ponte reportam skip informativo e nunca marcam teste nativo como executado com sucesso.

- [ ] **Step 1: Escrever as funções de teste de regressão em `lifecycle_tests.rs`**

Implementar os cenários de regressão usando helpers e processos sintéticos (como `sh -c 'exit 1'` ou processos mock para validar as rotinas de espera, timeout e detecção de código de saída).

- [ ] **Step 2: Executar a suíte completa de `lifecycle_tests`**

Executar: `cargo test -p nexo-desktop --test lifecycle_tests -- --nocapture`
Esperado: PASS em todos os testes novos e existentes.

- [ ] **Step 3: Commit dos testes de regressão**

```bash
git add crates/nexo-desktop/tests/lifecycle_tests.rs
git commit -m "test(desktop): adicionar testes de regressão para janelas concorrentes, timeout e exit codes"
```

---

### Task 4: Atualização Documental do ADR 0002

**Files:**
- Modify: `docs/architecture/0002-rune-pin-and-linux-deps.md`

**Interfaces:**
- Consumes: Requisitos de auditoria R3.
- Produces:
  1. Correção da seção 4: Registrar explicitamente que o erro anterior do `wgpu` (`UnsupportedQueueFamily`) manifestou-se como um **panic** dentro do processo NEXO, disparado pela stack gráfica durante `Surface::configure`.
  2. Rigor conceitual mantido: separar origem do erro (validação no `wgpu-core`), comportamento observado (panic no processo do `nexo-desktop`) e causa ambiental ainda não comprovada (hipótese multi-GPU/PRIME).
  3. Atualização da seção 4.6 e seção 5: Descrever a vinculação estrita de janela por PID via `wmctrl -l -p`, fechamento por Window ID via `wmctrl -i -c <window_id>`, e regras de mitigação para janelas preexistentes e ambientes sem EWMH.

- [ ] **Step 1: Aplicar as correções no texto do ADR 0002**

Editar `docs/architecture/0002-rune-pin-and-linux-deps.md` substituindo as menções que sugeriam execução "sem pânico" pela descrição factual exata do panic gerado na stack gráfica do wgpu, e detalhar a nova política de associação de janela e PID.

- [ ] **Step 2: Revisar a coerência do documento com a especificação R3**

Garantir que não há contradições entre a documentação de arquitetura e o código implementado em `lifecycle_tests.rs`.

- [ ] **Step 3: Commit do ADR 0002 atualizado**

```bash
git add docs/architecture/0002-rune-pin-and-linux-deps.md
git commit -m "docs(adr): corrigir registro de panic da stack wgpu e documentar identificação segura de janelas"
```

---

### Task 5: Validação Integral, Smoke Test e Atualização da PR #10

**Files:**
- Workspace Rust completo e suíte TypeScript legada.

- [ ] **Step 1: Executar matriz completa de validação Rust**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --locked
```

- [ ] **Step 2: Executar matriz completa de validação TypeScript**

```bash
npm test
npm run lint
npm run typecheck
npm run build:all
```

- [ ] **Step 3: Executar smoke test nativo de processo com log detalhado**

```bash
cargo test -p nexo-desktop --test lifecycle_tests native_window_process_lifecycle_clean_exit -- --nocapture
```
Confirmar a associação unívoca entre o PID do processo filho e a janela fechada via `wmctrl -i -c`.

- [ ] **Step 4: Push dos novos commits para a PR #10 existente**

```bash
git push origin feat/m1-rune-linux-foundation
```

- [ ] **Step 5: Elaborar o relatório conclusivo de auditoria R3**

Apresentar o relatório técnico final em português do Brasil contendo:
1. Diagnóstico do defeito P1 da R2;
2. Descrição da correção implementada;
3. Evidência técnica de identidade processo/janela;
4. Resultados dos testes de regressão;
5. Matriz de conformidade integral da Issue #2;
6. Limitações ambientais identificadas e documentadas;
7. Registro explícito de não merge e não fechamento da issue antes da aprovação do mantenedor.
