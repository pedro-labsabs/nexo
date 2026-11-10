# NEXO — Remediação R1 da PR #10 (Issue #2) — Plano de Implementação

## Metadados do Plano
- **Data:** 2026-11-09
- **PR:** [#10](https://github.com/pedro-labsabs/nexo/pull/10)
- **Issue:** [#2 — Infraestrutura completa Rust/Rune Linux](https://github.com/pedro-labsabs/nexo/issues/2)
- **Branch:** `feat/m1-rune-linux-foundation`
- **HEAD auditado:** `0fbbc8c81cfb9ba65bc0b9fef4186634b6c209bb`
- **Base:** `recovery/r1-from-nex0`
- **Rune SHA:** `42f2d3543f5e076a67736f0cb7dcb2182e8eb4b9`
- **Rust Toolchain:** `1.99.1`

> [!IMPORTANT]
> A issue #2 permanece aberta até o cumprimento de todos os critérios de aceite. O escopo é integral e a implementação existente será preservada e corrigida sem reduções.

---

## Análise Diagnóstica das Deficiências Apontadas na Auditoria

| Deficiência | Causa Raiz Identificada | Solução Planejada |
|---|---|---|
| **Layout: Falta de separação no rodapé e no banner de erro** | O Rune não possui o método `.between()`. Os itens de `row()` ficaram contíguos sem espaçamento flex. | Usar o padrão canônico do Rune de espaçador flex: `.child(el().grow(1.0))` entre os itens à esquerda e à direita em `header`, `footer` e `error_banner`. |
| **Interação por teclado não coberta na suíte automatizada** | Os testes anteriores usavam apenas `pointer_down` / `pointer_up`. Não havia testes cobrindo `rt.key(KeyEvent { key, mods })`. | Implementar testes com `Key::Tab` para navegação de foco (`rt.elements().find(|e| e.focused)`), `Key::Space` para acionamento de botões/tabs, `Key::Escape` para dismiss de erros e navegação por teclado. |
| **Validação de ciclo de vida e encerramento nativo** | A suíte cobriu apenas frames no `Runtime` headless, sem documentar nem testar o encerramento limpo da janela e do processo em Linux. | Implementar testes de ciclo de vida, verificação de encerramento sem panic/leak de recursos, e documentar o procedimento de smoke test interativo nativo em X11/Wayland. |
| **Erro intermitente do wgpu (`UnsupportedQueueFamily`)** | No `wgpu 30.0.1`, `Surface::configure` falha se `surface.get_capabilities(adapter)` falhar na validação da família de filas do dispositivo Vulkan (comum em sessões X11 com multi-GPU/PRIME/offloading quando o prewarm escolhe adaptador sem surface compatível). | Investigar e documentar o fluxo `prewarm_gpu` vs `for_surface` no Rune, as variáveis de controle (`WGPU_ADAPTER_NAME`, `WGPU_POWER_PREF=low|high`, `RUNE_GPU_LOG=1`), a diferença entre ambiente headless/virtual e sessão gráfica real, e registrar o diagnóstico detalhado na ADR 0002. |
| **Comportamento responsivo em diferentes dimensões de janela** | Layout só foi testado em 900x600. | Testar e validar a renderização em dimensões variadas (ex.: 640x480, 1280x800, 1920x1080) garantindo que o layout flex preserva a usabilidade. |

---

## Arquivos Envolvidos

- `crates/nexo-desktop/src/app.rs`:
  - Correção do layout (spacers `el().grow(1.0)` no header, barra de abas, footer e banner de erro).
  - Suporte a atalho global de teclado (`Key::Escape` para limpar erros e atalhos de navegação).
  - Novos testes automatizados de teclado (`Tab`, `Space`, `Escape`, alternância de abas e foco).
  - Novos testes de responsividade em múltiplas dimensões (640x480, 1280x800, 1920x1080).
- `crates/nexo-desktop/src/main.rs`:
  - Adição de scenes adicionais para testes de layout (`light`, `dark`, `error`, `minimal_size`, `wide_size`).
- `crates/nexo-desktop/tests/lifecycle_tests.rs`:
  - Teste de inicialização e encerramento seguro do runtime sem panics ou recursos retidos.
- `docs/architecture/0002-rune-pin-and-linux-deps.md`:
  - Análise aprofundada da investigação do erro intermitente `wgpu::Surface::configure` (`UnsupportedQueueFamily`).
  - Procedimento de smoke test interativo em ambiente gráfico real Linux (Wayland e X11).
- `README.md`:
  - Atualização dos comandos de smoke test nativo e instruções de diagnóstico para GPU.

---

## Tarefas de Execução Passo a Passo

### Tarefa 1: Correção do Layout com Espaçadores Reais do Rune
**Objetivo:** Garantir separação visual perfeita no cabeçalho, rodapé e banner de erro em qualquer resolução.

1. Em `crates/nexo-desktop/src/app.rs`:
   - No `header`: Inserir `.child(el().grow(1.0))` entre `header_left` e `theme_btn`.
   - No `footer` (`status_bar`): Inserir `.child(el().grow(1.0))` entre `status_label` e `interactions_label`.
   - No `error_banner`: Inserir `.child(el().grow(1.0))` entre `err_label` e `clear_btn`.
   - Adicionar margem/gap no banner de erro para espaçamento adequado em relação ao conteúdo.
   - Adicionar suporte a `on_key` na raiz para atalhos (`Escape` limpa erro se houver).
2. Verificar que compila com `cargo check -p nexo-desktop`.

---

### Tarefa 2: Testes Automatizados de Interação por Teclado
**Objetivo:** Implementar testes automatizados cobrindo a semântica real de teclado do Rune.

1. Implementar helper de foco e teclado nos testes de `crates/nexo-desktop/src/app.rs`:
   ```rust
   fn press_key(rt: &mut Runtime<AppState>, key: rune::Key) {
       rt.key(rune::KeyEvent { key, mods: rune::Mods::default() });
       rt.frame(0.0);
   }

   fn get_focused_key(rt: &Runtime<AppState>) -> Option<String> {
       rt.elements().into_iter().find(|e| e.focused).and_then(|e| e.key)
   }
   ```
2. Adicionar os seguintes testes específicos:
   - `test_keyboard_tab_focus_navigation`: `Tab` percorre sequencialmente os controles focáveis (`theme_switch`, `tab_dashboard`, `tab_graph`, `tab_table`, `action_button`, `trigger_error_btn`).
   - `test_keyboard_space_activates_theme_switch`: Focar no `theme_switch` com `Tab` e pressionar `Space` alterna o tema dark/light.
   - `test_keyboard_space_activates_tabs`: Focar nas abas via `Tab` e pressionar `Space` altera `domain.view_mode` para `Graph` e `Table`.
   - `test_keyboard_error_banner_dismiss`: Disparar erro, navegar com `Tab` até o botão `clear_error_btn`, pressionar `Space` e validar que o erro foi limpo.
   - `test_keyboard_escape_clears_error`: Pressionar `Escape` limpa o erro diretamente.
3. Executar `cargo test -p nexo-desktop` e validar que todos passam.

---

### Tarefa 3: Testes de Responsividade em Múltiplas Dimensões
**Objetivo:** Comprovar comportamento adequado e sem estouro de layout em diferentes resoluções.

1. Adicionar testes:
   - `test_layout_responsive_compact_640x480`: Renderiza em janela compacta, valida que elementos mantêm posições coerentes.
   - `test_layout_responsive_standard_900x600`: Resolução padrão.
   - `test_layout_responsive_wide_1920x1080`: Resolução widescreen, valida que `el().grow(1.0)` afasta o status e contador para os extremos.
2. Executar `cargo test -p nexo-desktop`.

---

### Tarefa 4: Verificação de Inicialização e Encerramento Nativo
**Objetivo:** Validar que o aplicativo inicializa e encerra de forma limpa, sem reter processos ou vazar recursos.

1. Adicionar teste de integração de ciclo de vida (`crates/nexo-desktop/tests/lifecycle_tests.rs`):
   - Criar e descartar múltiplos `Runtime` sequencialmente para verificar liberação de memória/GPU.
   - Testar chamadas rápidas de renderização e drop sem panics.
2. Documentar o procedimento de teste interativo com X11 / Wayland no Linux.

---

### Tarefa 5: Investigação Aprofundada do Erro Intermitente do wgpu
**Objetivo:** Investigar a causa de `wgpu error: Validation Error: In Surface::configure: Surface does not support the adapter's queue family` e documentar na ADR 0002.

1. Análise técnica detalhada:
   - Arquitetura de inicialização do Rune: `prewarm_gpu` inicia em thread concorrente chamando `Gpu::try_for_display(instance, wanted)` com `surface: None`.
   - Em Linux X11 com multi-GPU (ex.: Intel integrada + NVIDIA dedicada ou PRIME render offload), a seleção antecipada pode escolher um adaptador Vulkan que não suporta a família de filas de apresentação para o `Window` X11 criado posteriormente.
   - Quando `Surface::configure` é chamado em `crates/rune/src/window/create.rs:145`, `wgpu-core` executa `surface.get_capabilities(&self.adapter)`, que em caso de incompatibilidade dispara `UnsupportedQueueFamily`.
   - Variáveis de ambiente fornecidas pelo Rune/wgpu para contornar o problema (`WGPU_ADAPTER_NAME`, `WGPU_POWER_PREF=low|high`, `RUNE_GPU_LOG=1`).
2. Atualizar [docs/architecture/0002-rune-pin-and-linux-deps.md](file:///home/pedro/Projetos/miscellaneous/Nex0/docs/architecture/0002-rune-pin-and-linux-deps.md) com seção dedicada a essa investigação, distinguindo causas comprovadas no código de hipóteses de runtime.

---

### Tarefa 6: Regeneração de Snapshots e Verificação Visual
**Objetivo:** Gerar snapshots visuais das cenas atualizadas com o novo layout corrigido.

1. Adicionar scenes em `main.rs`: `default`, `light`, `dark`, `error`.
2. Gerar snapshots PNG:
   - `/tmp/nexo-smoke-v2-default.png`
   - `/tmp/nexo-smoke-v2-light.png`
   - `/tmp/nexo-smoke-v2-error.png`
3. Copiar os novos snapshots para o diretório de artifacts do brain.

---

### Tarefa 7: Validação Completa, Commits e Atualização da PR #10
**Objetivo:** Confirmar que todos os testes passam, fazer commits atômicos de remediação, push para o branch da PR #10 e acompanhar a CI.

1. Executar bateria de verificação completa:
   - `cargo fmt --all -- --check`
   - `cargo clippy --workspace --all-targets --locked -- -D warnings`
   - `cargo test --workspace --locked`
   - `cargo build --workspace --locked`
   - `npm test`
   - `npm run lint`
   - `npm run typecheck`
   - `npm run build:all`
2. Criar commits semânticos de remediação no branch `feat/m1-rune-linux-foundation`.
3. Fazer `git push origin feat/m1-rune-linux-foundation`.
4. Acompanhar a CI na PR #10 até 100% verde.
5. Elaborar relatório final de remediação em português brasileiro.
