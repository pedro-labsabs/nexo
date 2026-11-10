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

Durante a auditoria de execução em sessão gráfica Linux X11, foi identificado o seguinte erro intermitente de validação:

```text
wgpu error: Validation Error

In Surface::configure
Surface does not support the adapter's queue family
```

### Análise de Causa-Raiz no Código Upstream

A inspeção do código-fonte do `wgpu` (v30.0.1) e do `rune` (SHA `42f2d354`) revelou:

1. **Localização no wgpu:** Em `wgpu-core/src/device/resource.rs:5277`, durante `Surface::configure`, o wgpu executa:
   ```rust
   let caps = surface.get_capabilities(&self.adapter)
       .map_err(|_| E::UnsupportedQueueFamily)?;
   ```
   Se a consulta de capacidades do adaptador falhar para aquela superfície específica, a validação é abortada com `UnsupportedQueueFamily` ("Surface does not support the adapter's queue family").

2. **Localização no Rune:** Em `crates/rune/src/window/create.rs:145`:
   ```rust
   surface.configure(&gpu.device, &config);
   ```
   O Rune possui uma rotina assíncrona de pré-aquecimento de GPU (`prewarm_gpu()`), que invoca `Gpu::try_for_display(&event_loop)` em background antes mesmo da janela nativa ser instanciada. Posteriormente, ao criar a janela, invoca `Gpu::for_surface(..., &window)`.

### Fatos Comprovados pelo Código vs. Hipótese de Runtime

- **Fatos Comprovados pelo Código:**
  - O wgpu exige estritamente que a fila de comandos (queue family) do adaptador físico selecionado suporte a apresentação direta para a superfície da janela na conexão de display ativa.
  - O Rune tenta selecionar e pré-aquecer o adaptador preferencial de GPU antes da criação da superfície.
  - A falha não decorre de corrupção de memória ou panic na UI do NEXO, mas sim da recusa estrita da API Vulkan/wgpu durante `Surface::configure`.

- **Hipótese de Runtime (Causa Real no Linux X11):**
  - Em ambientes Linux com múltiplos adaptadores gráficos (ex.: laptop com GPU integrada Intel/AMD e GPU discreta NVIDIA via PRIME offload, ou estações com drivers Mesa e proprietários concorrentes), o pré-aquecimento do Rune ou a enumeração de adaptadores do wgpu pode selecionar um adaptador diferente do que gerencia o visual e a tela X11 corrente.
  - Em servidores X11 com aceleração indireta ou extensões DRI3 inconsistentes entre adaptadores, a consulta de capacidades Vulkan (`vkGetPhysicalDeviceSurfaceSupportKHR`) retorna `false` para o adaptador selecionado, impedindo a configuração da superfície.

### Mitigações Determinísticas e Controles de Diagnóstico

O Rune e o wgpu disponibilizam controles de ambiente para forçar o adaptador e depurar a seleção:

1. **Forçar Adaptador Específico:**
   ```bash
   # Selecionar adaptador integrado ou discreto pelo nome
   WGPU_ADAPTER_NAME="Intel" cargo run -p nexo-desktop
   # ou
   WGPU_ADAPTER_NAME="NVIDIA" cargo run -p nexo-desktop
   ```
2. **Forçar Preferência de Consumo:**
   ```bash
   WGPU_POWER_PREF=low cargo run -p nexo-desktop   # Favorece GPU integrada
   WGPU_POWER_PREF=high cargo run -p nexo-desktop  # Favorece GPU discreta
   ```
3. **Diagnóstico Completo de GPU no Rune:**
   ```bash
   RUNE_GPU_LOG=1 cargo run -p nexo-desktop
   ```
   Exibe os adaptadores encontrados, driver Vulkan/GL ativo e a cadeia de configuração do pipeline gráfico.
4. **Fallback Seguro em Ambientes Virtuais ou Sem Aceleração de Hardware:**
   ```bash
   LIBGL_ALWAYS_SOFTWARE=1 cargo run -p nexo-desktop
   ```

---

## 5. Procedimento de Smoke Test Nativo em Linux (X11 e Wayland)

Para verificar o ciclo de vida da janela nativa e a resposta a eventos fora do ambiente de CI:

### Pré-requisitos
- Sessão gráfica ativa (X11 ou Wayland).
- Driver Vulkan instalado (`mesa-vulkan-drivers` ou driver proprietário).

### Execução Interativa
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
