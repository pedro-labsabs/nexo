# ADR 0002 — Fixação do Rune upstream e dependências Linux para o NEXO

- **Data:** 2026-11-09
- **Status:** Aprovada e implementada (Issue #2 / Milestone M1)
- **Contexto:** Fundação nativa Rust/Rune para o observatório NEXO no Linux.

## 1. Decisão

Fixar o framework [Rune da Anomaly](https://github.com/anomalyco/rune) via dependência Git com SHA imutável e definir a toolchain Rust oficial necessária para compilação reproduzível.

### Pin Upstream
- **Repositório:** `https://github.com/anomalyco/rune`
- **Branch:** `dev`
- **SHA fixado:** `42f2d3543f5e076a67736f0cb7dcb2182e8eb4b9` (Commit de 2026-11-10T01:21:43Z)
- **Rust Toolchain:** `1.99.1` (definido no `rust-toolchain.toml` do upstream Rune)

## 2. Requisitos de Sistema e Dependências Linux

Para compilar e executar o `nexo-desktop` em uma distribuição Linux baseada em Debian/Ubuntu, os seguintes pacotes de desenvolvimento são obrigatórios:

```bash
sudo apt-get update
sudo apt-get install -y \
  libxkbcommon-dev \
  libwayland-dev \
  libxcb1-dev \
  libx11-dev \
  libxi-dev \
  libxrandr-dev \
  libxcursor-dev \
  libvulkan-dev \
  mesa-vulkan-drivers \
  libegl1-mesa-dev \
  libgles2-mesa-dev
```

### Arquitetura Gráfica
O Rune utiliza GPU rendering baseado no `wgpu` v30 (Vulkan em Linux nativo, com fallback para EGL/OpenGL). Em ambientes sem display gráfico físico (servidores de CI), o Rune suporta execução headless integral via `Runtime::new()` e CLI `--snap <path>.png`, sem necessidade de X11/Wayland rodando.

## 3. APIs Reais Verificadas vs. Limitações

### APIs Verificadas e Funcionais no SHA `42f2d354`
- `rune::run(AppSpec)` — Ciclo de vida da janela nativa e runloop.
- `rune::Runtime::new(state, view, size, env)` — Runtime headless para testes determinísticos.
- `rt.frame(dt)` — Avanço do relógio de animação e renderização.
- `rt.find(key)` / `rt.pointer_down(x, y)` / `rt.pointer_up(x, y)` — Simulação precisa de interações do usuário.
- Elementos fundamentais: `el()`, `col()`, `row()`, `text()`.
- Layout: `.padding(p)`, `.gap(g)`, `.w_full()`, `.h_full()`, `.grow(f)`, `.radius(r)`.
- Tokens e Temas: `tok::theme(dark)`, `tok::accent(index, dark)`, `tok::type_::TITLE`, `tok::type_::BODY`, `tok::type_::LABEL`.
- Eventos e Interatividade: `.on_press(|s: &mut State| ...)`.
- Animação: `.motion(motion::SNAPPY)`.

### Limitações Comprovadas do Rune
1. **API de Cores:** `rune::Color` não possui construtor `Color::rgb(r, g, b)`; cores devem ser construídas via OKLab (`Color::from_oklab`), hex ou tokens semânticos de `tok::theme()`.
2. **Padding Uni-argumento:** `.padding()` recebe um único valor escalar `f32` (não dois argumentos horizontal/vertical). Para paddings diferenciados, usam-se insets específicos.
3. **Alinhamento Flex:** Métodos como `.between()` não existem diretamente em `El`; a distribuição de espaço entre filhos é feita combinando `.grow()` e containers aninhados.
4. **Dependência de Rust 1.99.1:** Versões anteriores do compilador Rust (ex.: 1.88) falham devido a dependências recentes da árvore do Rune.

## 4. Investigação Técnica: Erro Intermitente no wgpu 30.0.1 (`UnsupportedQueueFamily`)

### 4.1. Ocorrência Observada no Linux (X11)
Durante a auditoria de execução em sessão gráfica Linux X11 em hardware específico, observou-se a seguinte mensagem intermitente de validação emitida pela stack do wgpu:

```text
wgpu error: Validation Error

In Surface::configure
Surface does not support the adapter's queue family
```

A ocorrência manifestou-se de forma esporádica durante a etapa de inicialização da janela nativa do Rune: o processo NEXO sofreu um **pânico (panic) em tempo de execução disparado a partir da stack gráfica (`wgpu-core`)** durante a chamada interna de `Surface::configure`, impedindo a apresentação do primeiro frame da janela naquela sessão específica e encerrando o processo de forma anômala.

### 4.2. Comportamento Demonstrado pelo Código Upstream (`wgpu-core` 30.0.1 e `rune` `42f2d35`)
A inspeção detalhada do código-fonte upstream evidencia os seguintes comportamentos e invariantes formais:

1. **No `wgpu-core` (v30.0.1) — `wgpu-core/src/device/resource.rs:5277`:**
   Durante a execução de `Surface::configure`:
   ```rust
   let caps = surface.get_capabilities(&self.adapter)
       .map_err(|_| E::UnsupportedQueueFamily)?;
   ```
   O `wgpu-core` consulta as capacidades de apresentação da superfície (`surface.get_capabilities`) passando a referência da fila/adaptador selecionado (`&self.adapter`). Se a chamada subjacente ao driver gráfico nativo (como `vkGetPhysicalDeviceSurfaceSupportKHR` no backend Vulkan) falhar ou retornar que a fila de comandos não suporta apresentação direta para aquela superfície X11/Wayland, o `wgpu-core` mapeia o erro exclusivamente como `UnsupportedQueueFamily` ("Surface does not support the adapter's queue family").

2. **No `rune` (SHA `42f2d3543f5e076a67736f0cb7dcb2182e8eb4b9`) — `crates/rune/src/window/create.rs:145`:**
   ```rust
   surface.configure(&gpu.device, &config);
   ```
   O Rune inclui uma rotina assíncrona de pré-aquecimento de GPU (`prewarm_gpu()`), que invoca `Gpu::try_for_display(&event_loop)` em background antes mesmo da criação da janela nativa. Posteriormente, ao instanciar a janela, chama `Gpu::for_surface(..., &window)`.

3. **Distinção Rigorosa dos Fatos Comprovados:**
   - **Origem do erro:** Recusa do driver gráfico nativo / validação formal no `wgpu-core`. A camada Vulkan nativa (via `vkGetPhysicalDeviceSurfaceSupportKHR`) retorna incompatibilidade ou falha na fila de apresentação para a superfície X11/Wayland fornecida, fazendo com que `surface.get_capabilities(&self.adapter)` mapeie estritamente para o erro formal `UnsupportedQueueFamily` no `wgpu-core`. Não decorre de corrupção de memória ou referências inválidas no código do NEXO.
   - **Comportamento observado no processo NEXO:** Pânico em tempo de execução (*runtime panic*) dentro do processo da aplicação NEXO, originado na stack gráfica do `wgpu-core` durante a chamada de `surface.configure` na inicialização da janela nativa (`window/create.rs:145`), abortando o processo antes da exibição do frame inicial.
   - **Causa ambiental (delimitação):** O Rune delega a enumeração e a seleção de adaptadores à implementação padrão do wgpu (`Instance::request_adapter`), sem filtros customizados no código da aplicação. Conforme detalhado na seção 4.3 a seguir, a hipótese de divergência na seleção de adaptadores em ambientes híbridos/PRIME permanece estritamente como uma **hipótese de runtime ainda não comprovada por telemetria ambiental direta**, devendo ser tratada com o devido isolamento conceitual.

### 4.3. Hipótese de Runtime Não Comprovada (Sistemas Multi-GPU / PRIME Offload)
- **Status:** *Hipótese de runtime não comprovada por telemetria ambiental direta.*
- **Descrição da Hipótese:**
  - Foi levantada a tese de que, em ambientes Linux com múltiplos adaptadores gráficos (ex.: notebooks com GPU integrada Intel/AMD e GPU discreta NVIDIA via PRIME offload, ou estações com drivers Mesa e proprietários concorrentes), o pré-aquecimento do Rune ou a enumeração de adaptadores do wgpu poderia selecionar um adaptador secundário diferente daquele que gerencia a tela e o visual X11 ativo.
  - Como consequência, a consulta de capacidades Vulkan (`vkGetPhysicalDeviceSurfaceSupportKHR`) retornaria `false` para o adaptador selecionado em relação à superfície da janela.
- **Rigor Técnico e Delimitação:**
  - Esta hipótese **não possui evidência ambiental comprovada** no ambiente de desenvolvimento local nem nos logs de execução do projeto.
  - Não foram constatados múltiplos adaptadores ativos divergentes nem logs do driver confirmando rejeição por PRIME no hardware de teste.
  - Portanto, a documentação delimita formalmente essa teoria como hipótese não confirmada, sendo inadequado declarar causa-raiz comprovada sem evidência correspondente, ou modificar arbitrariamente o upstream Rune nem alterar seu SHA fixado para mitigar uma suposição sem comprovação causal.

### 4.4. Alternativas Diagnósticas e de Contorno Realmente Disponíveis
O Rune e o wgpu disponibilizam variáveis de ambiente oficiais para diagnosticar e controlar a seleção de GPU em tempo de execução:

1. **Forçar Adaptador Específico (`WGPU_ADAPTER_NAME`):**
   Restringe o adaptador gráfico inspecionado por correspondência de nome:
   ```bash
   # Selecionar adaptador integrado ou discreto pelo nome
   WGPU_ADAPTER_NAME="Intel" cargo run -p nexo-desktop
   # ou
   WGPU_ADAPTER_NAME="NVIDIA" cargo run -p nexo-desktop
   # ou
   WGPU_ADAPTER_NAME="AMD" cargo run -p nexo-desktop
   ```
2. **Forçar Preferência de Consumo (`WGPU_POWER_PREF`):**
   Orienta o wgpu na escolha entre eficiência energética e desempenho gráfico:
   ```bash
   WGPU_POWER_PREF=low cargo run -p nexo-desktop   # Favorece GPU integrada
   WGPU_POWER_PREF=high cargo run -p nexo-desktop  # Favorece GPU discreta
   ```
3. **Diagnóstico Completo de GPU no Rune (`RUNE_GPU_LOG`):**
   Ativa o logging interno do Rune para enumeração e seleção de pipeline:
   ```bash
   RUNE_GPU_LOG=1 cargo run -p nexo-desktop
   ```
   Exibe os adaptadores encontrados, driver Vulkan/GL ativo e cada etapa de configuração do pipeline gráfico.
4. **Fallback Seguro para Renderização em Software (`LIBGL_ALWAYS_SOFTWARE`):**
   Permite execução em ambientes virtuais, contêineres ou estações sem aceleração gráfica física por hardware:
   ```bash
   LIBGL_ALWAYS_SOFTWARE=1 cargo run -p nexo-desktop
   ```

### 4.5. Comportamentos Ainda Não Reproduzidos Sinteticamente
Registra-se com total clareza e transparência os comportamentos que **ainda não foram reproduzidos de forma determinística**:
- O erro `UnsupportedQueueFamily` não foi reproduzido sinteticamente em bancada de teste isolada nem em contêineres CI headless rodando Vulkan lavapipe/llvmpipe.
- Não foi possível reproduzir artificialmente a rejeição de `Surface::configure` em ambiente controlado sem emulação de múltiplos adaptadores conflitantes sob X11.
- Por essa razão, a engenharia do NEXO evita correções cegas ou alterações no upstream Rune, mantendo a investigação aberta e documentada e priorizando a coleta de telemetria diagnóstica (`RUNE_GPU_LOG=1`) caso o comportamento volte a ser observado.

### 4.6. Validação Genuína de Ciclo de Vida Nativo e Encerramento
Para garantir a estabilidade do ciclo de vida da aplicação nativa independentemente de particularidades de GPU:

1. **Validação Headless Determinística:**
   - O teste `native_runtime_lifecycle_repeated_init_render_and_clean_drop` (em `crates/nexo-desktop/tests/lifecycle_tests.rs`) instancia sequencialmente o `Runtime` do Rune por 5 iterações completas, renderizando frames, alternando temas e destruindo as instâncias sem vazamentos de memória ou panics de encerramento.

2. **Validação de Processo Nativo Real com Associação Segura e Protocolo EWMH:**
   - O teste de integração `native_window_process_lifecycle_clean_exit` executa uma verificação real de processo de ponta a ponta com garantias estritas de segurança:
     - **Isolamento de Processo via RAII (`ChildGuard`):** Dispara o binário real compilado `nexo-desktop` (`CARGO_BIN_EXE_nexo-desktop`) como processo filho sob custódia do guard RAII `ChildGuard`. Em caso de qualquer falha, asserção, pânico ou timeout, o `drop` garante que `child.kill()` e `child.wait()` sejam executados imediatamente, eliminando o risco de processos órfãos ou zumbis.
     - **Vinculação Estrita de Janela Exclusivamente por PID (`wmctrl -l -p`):** Elimina categoricamente a correspondência frágil por título de janela. Utiliza o comando `wmctrl -l -p` para obter a propriedade oficial `_NET_WM_PID` mantida pelo window manager. A função `parse_wmctrl_l_p` analisa a listagem bruta em estruturas `WindowInfo` (`id`, `desktop`, `pid`, `host`, `title`), preservando inclusive múltiplos espaços internos no título. A resolução via `find_window_by_pid` associa a janela estritamente ao PID do processo filho (`child.id()`).
     - **Imunidade a Janelas Concorrentes ou Preexistentes:** Se houver outras janelas no ambiente com o título "NEXO" (por exemplo, sessões abertas manualmente pelo desenvolvedor ou instâncias anteriores), o teste ignora-as categoricamente pois seus PIDs não coincidem com o processo sob teste.
     - **Encerramento Cirúrgico por Window ID Hexadecimal (`wmctrl -i -c`):** O encerramento gracioso não utiliza títulos (`wmctrl -c NEXO` foi totalmente abolido). A função `close_window_by_id` despacha a mensagem cliente `_NET_CLOSE_WINDOW` diretamente para o Window ID hexadecimal correspondente (`wmctrl -i -c <window_id>`), afetando exclusivamente a janela alvo.
     - **Detecção Imediata de Saídas Prematuras:** Durante o polling da janela (intervalo de 100ms até 5s em `poll_child_window`), verifica periodicamente `child.try_wait()`. Se o processo filho sofrer um encerramento inesperado (seja por erro, pânico ou término com código 0) antes de mapear a janela, a rotina falha imediatamente com `LifecycleError::PrematureExit(code)` e descarrega os logs do processo para diagnóstico imediato.
     - **Verificação Rigorosa de Código de Saída:** A rotina `wait_for_clean_exit` aguarda o término e valida via `verify_clean_exit` que o processo encerrou com código estritamente `0` (`status.success()`). Códigos diferentes de zero geram `LifecycleError::NonZeroExit(code)` e interrupções por sinal geram `LifecycleError::TerminatedBySignal`.
     - **Tratamento Seguro de Limitações Ambientais e Ambientes sem EWMH:** Caso o ambiente não suporte a propriedade `_NET_WM_PID` (todas as janelas reportando PID 0) ou se execute sob Wayland puro sem suporte a PID, a rotina sinaliza `WindowBindingError::PidNotSupported` e aborta a verificação com skip informativo (`SKIPPED`), sem interagir com janelas de terceiros e sem emitir falsas declarações de sucesso.
     - **Bateria de Regressões Automatizadas:** A suíte contém testes unitários e de integração dedicados que garantem cobertura para:
       1. `regression_preexisting_nexo_window_is_not_closed`: Comprova que janelas pré-existentes intituladas "NEXO" não são associadas nem fechadas.
       2. `regression_premature_child_exit_triggers_failure`: Comprova detecção e falha imediata em caso de término precoce com código 1 ou 0.
       3. `regression_nonzero_exit_code_triggers_failure`: Comprova rejeição de códigos de saída não-zero.
       4. `regression_child_hang_triggers_timeout_and_clean_kill`: Comprova disparo de timeout em processos em hang e limpeza garantida pelo `ChildGuard`.
       5. `regression_missing_ewmh_or_display_handled_cleanly`: Comprova skip transparente em ambientes sem display, sem `wmctrl` ou sem suporte a `_NET_WM_PID`.

---

## 5. Procedimento de Smoke Test Nativo em Linux (X11 e Wayland)

Para verificar o ciclo de vida da janela nativa e a resposta a eventos fora do ambiente de CI:

### Pré-requisitos
- Sessão gráfica ativa (X11 ou Wayland).
- Driver Vulkan instalado (`mesa-vulkan-drivers` ou driver proprietário).
- Pacote `wmctrl` (para automação via linha de comando).

### Execução Automatizada do Teste de Ciclo de Vida
```bash
cargo test -p nexo-desktop --test lifecycle_tests -- --nocapture
```
Executa a validação headless (5 iterações completas do runtime), os testes determinísticos do parser de janelas, a bateria completa de regressões de ciclo de vida e, caso detecte sessão gráfica com window manager compatível com EWMH e suporte a `_NET_WM_PID`, abre a janela real, associa unicamente ao PID via `wmctrl -l -p`, despacha encerramento gracioso cirúrgico por Window ID (`wmctrl -i -c <window_id>`) e valida a terminação limpa com código de saída `0`.

### Execução Interativa Manual
```bash
cargo run -p nexo-desktop
```

### Roteiro de Verificação Manual:
1. **Inicialização:** A janela nativa "NEXO" abre centrada com dimensões 900x600 px no tema escuro por padrão.
2. **Navegação por Teclado:**
   - Pressione `Tab`: O foco visual move-se sequencialmente entre:
     1. Alternador de tema (`theme_switch`)
     2. Aba "Dashboard" (`tab_dashboard`)
     3. Aba "Grafo" (`tab_graph`)
     4. Aba "Tabela" (`tab_table`)
     5. Botão "Explorar Grafo de Dependências" (`action_button`)
     6. Botão "Simular Falha de Conexão" (`trigger_error_btn`)
   - Com o foco em `tab_graph` ou `tab_table`, pressione `Espaço`: A visualização e a barra de status alternam instantaneamente.
   - Com o foco em `trigger_error_btn`, pressione `Espaço`: O banner de erro é exibido no topo do conteúdo.
   - Pressione `Tab`: O foco salta para o botão "Fechar" do banner de erro (`clear_error_btn`). Pressione `Espaço`: O erro é dispensado.
   - Alternativamente, pressione `Escape` em qualquer momento com o banner ativo: O erro é fechado globalmente.
3. **Encerramento Limpo:**
   - Feche a janela pelo controle do sistema operacional (botão 'X' da janela) ou feche via sinal SIGINT (`Ctrl+C`).
   - O processo termina imediatamente com código de saída `0`, sem threads zumbis ou vazamentos de recursos.

### Estratégia de CI Headless
Em runners GitHub Actions (`ubuntu-latest`), onde não há servidor X11 ou GPU física disponível:
- A suíte automatizada `cargo test --workspace --locked` executa 23 testes determinísticos (18 testes headless de UI/Runtime e 5 testes de regressão de ciclo de vida e binding de janelas).
- O teste de integração de processo nativo de ponta a ponta avalia as variáveis `DISPLAY`, `WAYLAND_DISPLAY`, a presença do utilitário `wmctrl` e o suporte a `_NET_WM_PID`, efetuando skip transparente e documentado quando os requisitos de desktop estão ausentes, prevenindo falsas alegações de conformidade.
- A geração de snapshots visuais (`cargo run -p nexo-desktop -- --snap <path>`) valida a árvore de layout, renderização de tipografia e estilo sem necessidade de janela nativa.

---

## 6. Estrutura de Crates Implementada

```
crates/
  nexo-desktop/  # Aplicativo nativo Rune, views, temas, ciclo de vida e testes headless
  nexo-domain/   # Entidades, tipos, enums e erros do domínio (puro Rust, zero UI)
  nexo-storage/  # Abstração da camada de persistência local (trait Storage)
  nexo-github/   # Abstração de cliente para sincronização futura (trait GitHubClient)
```

## 7. Diretrizes para a Issue #3

A Issue #3 implementará a modelagem de domínio completa, validação de dados e cálculo de indicadores. A crate `nexo-domain` foi construída totalmente independente de `rune`, permitindo que os cálculos técnicos e contratos de indicadores sejam testados isoladamente sem carregar pipelines gráficos.
