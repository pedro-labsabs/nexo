# Remediação R2 da Issue #2 — Plano de Implementação

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Sanar integralmente as duas lacunas de verificação da R1 apontadas pela auditoria de mantenedor (o falso teste de encerramento nativo em `lifecycle_tests.rs` e o teste fraco de responsividade geométrica em `app.rs`), refinar a documentação do ADR 0002 sobre wgpu e validar 100% dos requisitos da Issue #2 na PR #10.

**Architecture:** Fortalecer a camada de testes e observabilidade gráfica com asserções geométricas determinísticas (verificação de caixas delimitadoras e não sobreposição) em múltiplas resoluções, introduzir um teste de ciclo de vida nativo genuíno com processo filho real via `CARGO_BIN_EXE_nexo-desktop` e `ChildGuard` (RAII) com fechamento gracioso via `_NET_CLOSE_WINDOW` (`wmctrl -c`), e auditar a precisão das hipóteses no ADR 0002.

**Tech Stack:** Rust 1.99.1, Rune (`dev` @ `42f2d354`), wgpu 30.0.1, X11/EWMH, cargo test, clippy, vitest/tsc/vite.

**Spec:** Prompt do usuário "NEXO — PR #10 | Remediação R2 | Issue #2" e corpo da Issue #2.

## Global Constraints

- Branch de trabalho: `feat/m1-rune-linux-foundation` (base: `recovery/r1-from-nex0`).
- Manter a PR #10 existente (não abrir nova PR, não realizar merge, não fechar a issue).
- Não reduzir o escopo da Issue #2 nem adotar atalhos de MVP.
- Preservar integralmente o histórico da recuperação R1 e a suíte TypeScript legada.
- Não usar `SIGKILL` nem asserções triviais como substituto de testes reais de encerramento.
- Não usar ausência de panic como substituto de asserções geométricas de layout.

---

### Task 1: Fortalecer Testes Geométricos de Responsividade e Separação de Elementos

**Files:**
- Modify: `crates/nexo-desktop/src/app.rs`
- Test: `crates/nexo-desktop/src/app.rs:tests`

**Interfaces:**
- Consumes: `rune::Runtime`, `rune::El`, `rune::tok`, `AppState`, `ViewMode`.
- Produces: Identificadores estáveis (`key`) nos elementos visuais do layout e asserções geométricas de não-sobreposição (`rect[0] + rect[2] <= target.rect[0]`) testadas em 640x480, 900x600 e 1920x1080.

- [ ] **Step 1: Adicionar chaves estáveis (`key`) aos elementos do layout**

Em `crates/nexo-desktop/src/app.rs`:
- No cabeçalho: adicionar `.key("header_left")` no container `header_left` (que agrupa título e badge de versão).
- Na barra de status: adicionar `.key("status_message_label")` em `status_label` e `.key("interactions_counter_label")` em `interactions_label`.
- No banner de erro: adicionar `.key("error_banner_text")` em `err_label`.

- [ ] **Step 2: Escrever testes geométricos determinísticos de não-sobreposição**

Em `crates/nexo-desktop/src/app.rs::tests`:
Substituir o teste raso `layout_responsive_separates_elements_across_resolutions` por uma validação geométrica rigorosa:
1. Para cada resolução `[(640.0, 480.0), (900.0, 600.0), (1920.0, 1080.0)]`:
   - Verificar dimensões positivas da janela.
   - **Cabeçalho:** `header_left.rect[0] + header_left.rect[2] <= theme_switch.rect[0]`. Ambos com `rect[2] > 0.0` e contidos na largura total do cabeçalho.
   - **Ações Principais:** `action_btn.rect[0] + action_btn.rect[2] <= trigger_error_btn.rect[0]`.
   - **Barra de Status:** `status_label.rect[0] + status_label.rect[2] <= interactions_label.rect[0]`. Ambos contidos na largura da `status_bar`.
2. Para o estado com erro (`error_message.is_some()`):
   - **Banner de Erro:** `err_label.rect[0] + err_label.rect[2] <= clear_btn.rect[0]`. Ambos com largura positiva e contidos no banner.
3. Usabilidade: botões e abas devem ter largura e altura utilizáveis (`width >= 20.0`, `height >= 20.0`).

- [ ] **Step 3: Executar teste e validar passagem**

Executar: `cargo test -p nexo-desktop --lib app::tests::layout_responsive_separates_elements_across_resolutions`
Esperado: PASS com todas as validações geométricas satisfeitas.

- [ ] **Step 4: Realizar teste de mutação / invariante violada**

Alterar temporariamente a margem ou remover o `.grow(1.0)` para confirmar que a asserção geométrica falha quando há sobreposição ou colisão, restaurando em seguida o código correto.

- [ ] **Step 5: Commit das melhorias de layout e testes geométricos**

```bash
git add crates/nexo-desktop/src/app.rs
git commit -m "test(desktop): fortalecer testes de layout com asserções geométricas de não-sobreposição"
```

---

### Task 2: Implementar Teste Genuíno de Ciclo de Vida Nativo com Processo Filho

**Files:**
- Modify: `crates/nexo-desktop/tests/lifecycle_tests.rs`

**Interfaces:**
- Consumes: Binário nativo `nexo-desktop` via `env!("CARGO_BIN_EXE_nexo-desktop")`, comando do sistema `wmctrl`, utilitários de processo `std::process::Command`.
- Produces: Teste de integração `native_window_process_lifecycle_clean_exit` que testa a abertura da janela real, captura da janela pelo window manager, despacho gracioso de encerramento (`_NET_CLOSE_WINDOW`), aguarda término com timeout e valida exit code 0 sem panic.

- [ ] **Step 1: Implementar o guard RAII `ChildGuard` para segurança contra processos órfãos**

Em `crates/nexo-desktop/tests/lifecycle_tests.rs`:
Criar struct `ChildGuard` que implementa `Drop`:
```rust
struct ChildGuard(Option<std::process::Child>);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
```

- [ ] **Step 2: Substituir o teste raso pelo teste real de processo**

Substituir `process_clean_exit_without_hanging_threads` por `native_window_process_lifecycle_clean_exit`:
1. Detectar se há display gráfico (`DISPLAY` ou `WAYLAND_DISPLAY`) E se `wmctrl` está instalado.
2. Se presente:
   - Disparar `Command::new(env!("CARGO_BIN_EXE_nexo-desktop"))` com logs redirecionados para arquivo de log temporário.
   - Envolver o `child` em `ChildGuard`.
   - Polling de até 5 segundos buscando janela "NEXO" via `wmctrl -l`.
   - Assegurar que a janela foi encontrada.
   - Enviar requisição nativa de encerramento: `Command::new("wmctrl").args(["-c", "NEXO"]).status()`.
   - Aguardar a saída do processo filho (com timeout).
   - Validar que o código de saída é exatamente `0` (`status.success()`).
   - Ler o log e verificar que não ocorreram pânicos nem mensagens de falha.
3. Se não houver display ou `wmctrl` (ambiente de CI headless puro):
   - Imprimir mensagem explicativa no log do teste documentando explicitamente que o teste nativo requer sessão gráfica com window manager, sem emitir falsas alegações de verificação de processo.
4. Manter o teste `native_runtime_lifecycle_repeated_init_render_and_clean_drop` clarificando seu escopo (teste headless de runtime Rune e liberação de estruturas de memória).

- [ ] **Step 3: Executar os testes de integração do nexo-desktop**

Executar: `cargo test -p nexo-desktop --test lifecycle_tests`
Esperado: PASS (ambos os testes passam com o processo real abrindo, sendo fechado via `wmctrl -c` e saindo com status 0).

- [ ] **Step 4: Commit do teste genuíno de ciclo de vida nativo**

```bash
git add crates/nexo-desktop/tests/lifecycle_tests.rs
git commit -m "test(desktop): implementar teste genuíno de ciclo de vida nativo e encerramento limpo"
```

---

### Task 3: Refinar o ADR 0002 para Rigor Técnico na Investigação do wgpu

**Files:**
- Modify: `docs/architecture/0002-rune-pin-and-linux-deps.md`

**Interfaces:**
- Consumes: Evidências da auditoria R1 e R2, código do `wgpu-core` 30.0.1, código do `rune`.
- Produces: Documentação técnica precisa que separa estritamente fatos comprovados, ocorrência do Linux, hipóteses de runtime, alternativas diagnósticas e comportamentos não reproduzidos.

- [ ] **Step 1: Revisar a seção 4 do ADR 0002**

Garantir que a seção:
- Distingue claramente o que é comprovado pelo código-fonte de wgpu e Rune (consulta de capacidades da superfície com a fila do adaptador selecionado).
- Documenta a ocorrência observada no Linux durante a auditoria (sessão X11 específica).
- Categoriza expressamente como "Hipótese de Runtime Não Comprovada" a tese de seleção de adaptador secundário em sistemas multi-GPU/PRIME offload.
- Aponta os controles diagnósticos reais (`WGPU_ADAPTER_NAME`, `WGPU_POWER_PREF`, `RUNE_GPU_LOG`).
- Registra explicitamente quais comportamentos ainda não foram reproduzidos sinteticamente em ambiente controlado.
- Descreve a validação nativa de encerramento via `wmctrl -c` (`_NET_CLOSE_WINDOW`) e o teste com `ChildGuard`.

- [ ] **Step 2: Commit do ADR 0002 revisado**

```bash
git add docs/architecture/0002-rune-pin-and-linux-deps.md
git commit -m "docs(adr): refinar rigor técnico da análise de wgpu e ciclo de vida nativo"
```

---

### Task 4: Validação Abrangente e Atualização da PR #10

**Files:**
- Verify: Workspace Rust completo, suíte legada TypeScript.

- [ ] **Step 1: Executar suite de validação Rust local**

Executar:
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked`
- `cargo build --workspace --locked`
- `cargo run -p nexo-desktop -- --snap /tmp/nexo-smoke-v2.png --scene default`

- [ ] **Step 2: Executar suite de validação TypeScript legada**

Executar:
- `npm test && npm run lint && npm run typecheck && npm run build:all`

- [ ] **Step 3: Push para `feat/m1-rune-linux-foundation` e monitoramento da CI**

Executar:
- `git push origin feat/m1-rune-linux-foundation`
- Monitorar `gh pr checks 10` até conclusão com sucesso.

- [ ] **Step 4: Gerar relatório final de remediação R2**

Apresentar o relatório conclusivo em português do Brasil contendo os 7 tópicos exigidos na especificação.
