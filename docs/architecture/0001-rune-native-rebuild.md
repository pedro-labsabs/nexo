# ADR 0001 — Reconstrução integral do NEXO com Rune e Rust

- **Data:** 2026-11-09
- **Estado:** decisão aprovada; implementação ainda não iniciada
- **Escopo:** NEXO (prova de conceito local-first)
- **Framework:** [Rune da Anomaly](https://github.com/anomalyco/rune), **não** a IDE homônima de [rune.build](https://rune.build/)
- **Base histórica de referência:** `recovery/r1-from-nex0` no momento desta decisão

## 1. Decisão

Reconstruir o NEXO como **aplicativo desktop nativo para Linux**, usando **Rust + Rune como stack principal da interface e, progressivamente, da lógica de negócio**. A reconstrução é integral: não se exige compatibilidade de código com React/Vite/TypeScript/Cytoscape.js, nem reprodução pixel a pixel da interface atual. O produto permanece um observatório local de portfólios GitHub, mas seu desenho de interface e sua arquitetura poderão ser refeitos livremente para explorar Rune.

**Regra de escolha de tecnologia:** preferir Rust/Rune para novas funcionalidades. Reter ou introduzir outra tecnologia **somente quando houver impedimento técnico demonstrável ou vantagem substancial documentada**, com fronteira clara, teste e plano de substituição/reavaliação. Um adaptador transitório não deve virar dependência permanente sem nova decisão.

O objetivo principal é **aprender, verificar e explorar o Rune por meio de um aplicativo real**, não preparar o NEXO para publicação ou produção. Instabilidade e mudanças de API são aceitáveis. Ainda assim, deve-se evitar perda desnecessária de dados locais e de rastreabilidade.

## 2. Premissas verificadas e limites

- O NEXO de referência usa React, TypeScript, Vite, Cytoscape.js, Node.js, Octokit e Zod, com CLI para obter snapshots GitHub e UI offline.
- Os contratos existentes estão em `src/shared/types.ts`, `src/shared/annotations.ts`, `src/shared/storage.ts`, `src/shared/indicators.ts`, `src/shared/relations.ts` e `src/shared/validation.ts`.
- O Rune da Anomaly se descreve como framework de interface Rust renderizado por GPU para macOS, Windows, Linux e web; inclui modos de inspeção/renderização headless. **É pré-lançamento**; exemplos e APIs são sujeitos a mudanças.
- **Não presumir** que todos os componentes, acessibilidade, entrada de texto, persistência, desempenho ou recursos web sejam maduros no ambiente-alvo. Verificar contra um commit específico do upstream antes de implementar.
- **Não presumir superioridade de desempenho** sobre Electron ou React: criar medições reproduzíveis no mesmo hardware.
- WebAssembly/WebGPU são um **alvo experimental posterior**, não condição para a primeira entrega Linux.

Fontes upstream para implementação: [README do Rune](https://github.com/anomalyco/rune), [`examples/hello`](https://github.com/anomalyco/rune/tree/dev/examples/hello), [README da crate](https://github.com/anomalyco/rune/blob/dev/crates/rune/README.md), [WEB.md](https://github.com/anomalyco/rune/blob/dev/WEB.md). Seguir o exemplo compilável e o código reais, não tutoriais que usem interfaces antigas.

## 3. Arquitetura-alvo

```text
NEXO desktop (Linux)
  ├── Shell e interface: Rune (Rust)
  │    ├── dashboard, busca, filtros e detalhes
  │    ├── grafo interativo e edição de relações
  │    └── importação/exportação, temas e feedback de erros
  ├── Domínio: Rust, sem dependência da UI
  │    ├── schemas/validação e modelos versionados
  │    ├── indicadores determinísticos e relações com proveniência
  │    ├── mesclagem de anotações, LWW e tombstones
  │    └── relatórios e políticas de privacidade
  ├── Persistência local
  │    └── snapshots, anotações e preferências por identidade
  └── Sincronização GitHub somente-leitura
       ├── transitória: CLI Node/TypeScript já existente
       └── alvo: cliente/CLI Rust, se tecnicamente viável
```

**Fronteira transitória:** enquanto a sincronização GitHub não for migrada, o CLI atual produz **JSON versionado validado**; o novo aplicativo Rune o importa localmente. Não integrar React dentro do Rune nem exigir Vite para executar o desktop. O CLI antigo é um adaptador temporário e isolado; a substituição por Rust vem depois que a interface, o domínio e a importação estiverem comprovados.

**Fonte de verdade dos dados:** manter a semântica dos contratos atuais na transição, inclusive `metadata.version` do snapshot, `schemaVersion` das anotações, `authenticatedUser`, identidade imutável do repositório, proveniência das relações e estado `unknown` de indicadores. Se evoluir o formato, definir **nova versão** e uma migração explícita e testada, sem reescrever silenciosamente os arquivos existentes.

## 4. Grafo: reconstrução nativa, não incorporação de Cytoscape

Criar o grafo como módulo Rust integrado à UI Rune. Um primeiro corte pode usar layout determinístico simples; depois evoluir para posicionamento por forças e outros layouts conforme os testes demonstrarem viabilidade.

Comportamentos a preservar como **referência funcional**, não de implementação: nós de organizações/repositórios; arestas direcionadas; distinção visual de relações automáticas e manuais; proveniência e evidências; zoom, pan, fit e seleção; criação, edição e exclusão de relações manuais com persistência. **Não inferir relações por semelhança de nomes.**

Medir e validar limites reais do Rune para desenho de milhares de primitivas, eventos, clipping, texto e hit testing. Se uma primitiva faltar, preferir implementar componente/renderização auxiliar em Rust (ou outra integração justificada e isolada) **sem recolocar o Cytoscape como dependência central**. Documentar cada lacuna e a solução adotada.

## 5. Contratos comportamentais a reimplementar

1. **Local-first e privacidade:** visualização offline de snapshots, sem telemetria mandatória; credenciais jamais embutidas em JSON, logs, relatórios ou UI persistida. O acesso GitHub permanece estritamente de leitura.
2. **Dados importáveis:** validação antes de ativar snapshot/anotação; mensagens claras para dados inválidos, versões desconhecidas, árvores incompletas e rate limits. Nunca interpretar ausência de evidência como reprovação técnica.
3. **Anotações:** isolar por identidade do usuário/portfólio; preservar avaliações subjetivas e sua separação dos indicadores técnicos; manter mesclagem determinística por timestamp (*last write wins*, empate favorece local) e tombstones de relações excluídas, respeitando os detalhes verificados nos testes atuais.
4. **Escrita segura:** erro de persistência deve ser mostrado como falha, sem simular sucesso; priorizar gravação atômica/substituição segura e backup/exportação explícita. Exclusões de relações e importações não podem ressuscitar dados tombstonados.
5. **Transparência:** preservar fórmulas, entradas, ressalvas e evidências dos indicadores. Não confundir presença de testes/CI com execução bem-sucedida.
6. **Portabilidade:** importação/exportação de snapshots e anotações versionados e relatórios Markdown com avisos para dados de repositórios privados.
7. **Usabilidade mínima:** interface em português brasileiro; modos claro/escuro conforme capacidades; estados vazio, carregando, inválido e erro recuperável; suporte a mouse e teclado nas operações essenciais.

Os testes da implementação antiga servem como **oráculos de comportamento**; não é necessário portar bibliotecas, componentes ou estruturas internas equivalentes.

## 6. Plano de execução (substituição completa, não duas UIs permanentes)

| Etapa | Entrega verificável | Critério para avançar |
| --- | --- | --- |
| 0. Viabilidade | Pin de SHA do Rune em `dev`, toolchain Rust definida, app mínima abre janela Linux, exemplo headless executado onde suportado | Compila, inicia e registra limitações reais com logs |
| 1. Núcleo de dados | Crate Rust para importar/validar snapshot e anotações `1.0`, fixtures sintéticas e testes comparativos | Aceita fixtures válidas, rejeita inválidas, preserva significado dos dados |
| 2. Interface nativa | Dashboard Rune, tabela/lista de repos, filtros, detalhes, temas e mensagens de erro | Fluxo de exploração offline utilizável **sem React/Vite** |
| 3. Grafo nativo | Nós, arestas, zoom/pan/fit, seleção, evidências, relações manuais e testes de interação | Fluxos principais equivalentes aos contratos existentes, inclusive persistência |
| 4. Domínio em Rust | Indicadores, relações, relatórios, mesclagem LWW/tombstones e persistência local substituem o TS | Testes de equivalência para cenários críticos e falhas |
| 5. Sincronização | Portar o CLI/cliente GitHub para Rust e retirar o adaptador Node quando viável | Paginação, rate limits, deduplicação e leitura estrita cobertos por testes |
| 6. Corte | Remover React, Vite, Cytoscape e arquivos TS **apenas quando não forem mais dependências necessárias** | NEXO inicializa, importa, visualiza, edita, exporta e sincroniza conforme escopo |
| 7. Exploração opcional | Spike web com WASM/WebGPU e comparativos de consumo/latência | Relatório distingue medições de hipóteses e limitações da plataforma |

**Ordem preferencial:** Linux desktop primeiro; depois avaliar macOS/Windows e web conforme recursos, tempo e aprendizado. Não prometer paridade multiplataforma por antecipação.

## 7. Organização proposta (não criada nesta decisão)

```text
crates/
  nexo-domain/       # modelos, regras e testes
  nexo-storage/      # persistência/importação/exportação locais
  nexo-github/       # sincronização somente-leitura (etapa posterior)
  nexo-desktop/      # app Rune; dashboard e grafo
fixtures/            # exclusivamente dados sintéticos
docs/architecture/   # ADRs e resultados de experimentos
```

Até completar a migração, a estrutura TypeScript existente pode permanecer como **referência temporária**, sem obrigação de preservar a UI antiga. Scripts e CI devem ser atualizados conforme os módulos realmente criados.

## 8. Protocolo de trabalho e validação

- Isolar alterações de migração em commits/branches identificáveis; o Git serve como histórico recuperável, **não como veto a refatorações radicais**.
- Antes de codificar, registrar SHA upstream do Rune e versões do Rust, dependências e comandos reproduzíveis. Usar dependência Git **fixada em `rev`**, não apontar para um `dev` flutuante; atualizar deliberadamente.
- Iniciar pelas instruções oficiais de Rune (`examples/hello` e README da crate), executando um smoke test de janela/renderização. Registrar incompatibilidades em vez de inventar APIs.
- Manter testes unitários do domínio, testes com dados sintéticos, importação inválida, concorrência/mesclagem e testes de interações do grafo. Usar cenas/snapshots headless do Rune quando disponíveis, com revisão humana adicional.
- Medir tempo de arranque, memória em repouso e após carregar snapshots, fluidez do grafo e estabilidade sob cargas explicitadas. Comparar com baseline React **somente** quando houver ambiente de medição idêntico.
- Nunca incluir tokens, snapshots privados nem dados reais do portfólio em fixtures, CI ou commits.
- **Critérios de sucesso final:** funcionamento local sem stack web legada, funções centrais do NEXO preservadas, limitações registradas e documentação de instalação/reprodução atualizada.
- **Critério para exceções:** só manter Node ou outra dependência não-Rune após registrar a capacidade ausente, a alternativa testada, custos e quando revisar a escolha.

## 9. Fora de escopo desta decisão

Este ADR **não executa a migração**, não altera código, não apaga histórico ou dados e não aprova publicação do aplicativo. Também não promete uma API estável do Rune, paridade de experiência web, vantagens de desempenho ou conclusão em prazo definido.

A decisão é revisável caso o Rune imponha um bloqueio concreto demonstrado por protótipo e testes. Nesse caso, registrar uma nova ADR e preferir a alternativa de menor acoplamento capaz de manter o experimento.

---

**Resumo executivo:** substituir a arquitetura atual do NEXO por Rust + Rune, mantendo **temporariamente** o sincronizador Node/TypeScript como alimentador de snapshots JSON; migrar domínio, grafo e sincronização para Rust; remover dependências antigas quando não forem mais necessárias. Aceita-se experimentação intensa e reconstrução integral, preservando privacidade, identidade de dados e comportamento observável.
