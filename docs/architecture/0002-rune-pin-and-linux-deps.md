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

A ocorrência manifestou-se de forma esporádica durante a etapa de inicialização da janela nativa do Rune, sem produzir corrupção de memória ou pânico na aplicação NEXO, mas impedindo a apresentação do primeiro frame da janela naquela sessão específica.

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

3. **Fatos Irrefutáveis Comprovados pelo Código:**
   - O wgpu exige estritamente que a fila de comandos (queue family) do adaptador físico selecionado suporte a apresentação direta para a superfície da janela na conexão de display ativa.
   - A falha não decorre de corrupção de memória, referências inválidas ou pânicos na camada de UI da aplicação NEXO, mas sim da recusa formal da API Vulkan/wgpu durante `Surface::configure`.
   - O Rune delega a enumeração e a seleção de adaptadores à implementação padrão do wgpu (`Instance::request_adapter`), sem filtros customizados no código da aplicação.

### 4.3. Hipótese de Runtime Não Comprovada (Sistemas Multi-GPU / PRIME Offload)
- **Status:** *Hipótese de runtime não comprovada por telemetria ambiental direta.*
- **Descrição da Hipótese:**
  - Foi levantada a tese de que, em ambientes Linux com múltiplos adaptadores gráficos (ex.: notebooks com GPU integrada Intel/AMD e GPU discreta NVIDIA via PRIME offload, ou estações com drivers Mesa e proprietários concorrentes), o pré-aquecimento do Rune ou a enumeração de adaptadores do wgpu poderia selecionar um adaptador secundário diferente daquele que gerencia a tela e o visual X11 ativo.
  - Como consequência, a consulta de capacidades Vulkan (`vkGetPhysicalDeviceSurfaceSupportKHR`) retornaria `false` para o adaptador selecionado em relação à superfície da janela.
- **Rigor Técnico e Delimitação:**
  - Esta hipótese **não possui evidência ambiental comprovada** no ambiente de desenvolvimento local nem nos logs de execução do projeto.
  - Não foram constatados múltiplos adaptadores ativos divergentes nem logs do driver confirmando rejeição por PRIME no hardware de teste.
  - Portanto, a documentação delimita expressamente essa teoria como hipótese não confirmada, sendo inadequado declarar causa-raiz comprovada sem evidência correspondente, ou modificar arbitrariamente o upstream Rune nem alterar seu SHA fixado para mitigar uma suposição.

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

2. **Validação de Processo Nativo Real com Protocolo EWMH (`_NET_CLOSE_WINDOW`):**
   - O teste de integração `native_window_process_lifecycle_clean_exit` executa uma verificação real de processo de ponta a ponta:
     - Dispara o binário real compilado `nexo-desktop` (`CARGO_BIN_EXE_nexo-desktop`) como processo filho.
     - Implementa o guard RAII `ChildGuard`, garantindo que `child.kill()` e `child.wait()` sejam chamados automaticamente em caso de pânico ou término prematuro, prevenindo processos órfãos ou zumbis.
     - Efetua polling de até 5 segundos via `wmctrl -l` para certificar que a janela intitulada "NEXO" foi instanciada com sucesso pelo gerenciador de janelas.
     - Envia requisição oficial de encerramento através do protocolo EWMH (`wmctrl -c NEXO`), despachando a mensagem cliente `_NET_CLOSE_WINDOW`.
     - Aguarda o encerramento do processo e valida que o código de saída é estritamente `0` (`status.success()`).
     - Lê o arquivo de log temporário gerado e assegura que não ocorreram mensagens de erro fatal ou pânico (`panicked at`, `fatal runtime error`).
     - Em ambientes CI puramente headless (sem `DISPLAY`/`WAYLAND_DISPLAY` ou sem `wmctrl`), o teste executa skip explícito com mensagem informativa, evitando falsas alegações de conformidade de processo.

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
Executa a validação headless e, caso detecte sessão gráfica com window manager compatível com EWMH, abre a janela real, envia `wmctrl -c NEXO` e valida o encerramento com código `0`.

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
- A suíte automatizada `cargo test --workspace --locked` executa 18 testes determinísticos via headless `Runtime::new()`.
- O teste de integração de janela nativa faz skip transparente quando `DISPLAY` e `wmctrl` estão ausentes.
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
