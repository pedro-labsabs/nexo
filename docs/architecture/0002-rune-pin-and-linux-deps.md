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

## 4. Estrutura de Crates Implementada

```
crates/
  nexo-desktop/  # Aplicativo nativo Rune, views, temas e testes headless
  nexo-domain/   # Entidades, tipos, enums e erros do domínio (puro Rust, zero UI)
  nexo-storage/  # Abstração da camada de persistência local (trait Storage)
  nexo-github/   # Abstração de cliente para sincronização futura (trait GitHubClient)
```

## 5. Diretrizes para a Issue #3

A Issue #3 implementará a modelagem de domínio completa, validação de dados e cálculo de indicadores. A crate `nexo-domain` foi construída totalmente independente de `rune`, permitindo que os cálculos técnicos e contratos de indicadores sejam testados isoladamente sem carregar pipelines gráficos.
