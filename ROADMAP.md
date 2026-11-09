# NEXO — Roadmap de reconstrução integral em Rust + Rune

**Atualizado em 2026-11-09 · Escopo: somente NEXO · Status: planejamento aberto**  
**Decisão:** [ADR 0001](docs/architecture/0001-rune-native-rebuild.md) · **Política dos executores:** [AGENTS.md](AGENTS.md) · **PR desta documentação:** [#1](https://github.com/pedro-labsabs/nexo/pull/1)

## Objetivo e estratégia

Reconstruir o NEXO — observatório local-first de portfólios GitHub — **integralmente com Rust e o framework Rune da Anomaly**, priorizando aplicativo **desktop Linux**. A aplicação atual em React/Vite/TypeScript/Cytoscape é **referência funcional transitória**, não arquitetura a preservar. O CLI de sincronização Node poderá alimentar snapshots JSON versionados até que exista substituto Rust completo e testado.

O NEXO **não é um produto de produção**: refatorações amplas, experimentos e substituições radicais são desejados. A liberdade arquitetural não autoriza perder dados locais nem eliminar critérios obrigatórios das entregas.

### Regra especial de execução neste projeto

**Cada issue é uma entrega completa e indivisível para o agente executor.** Não reduzir uma issue a *MVP*, *vertical slice*, tela demonstrativa ou subconjunto de requisitos. O agente é responsável por implementar, integrar, testar e documentar **todos os critérios**. Se a auditoria encontrar problemas, executar remediação **sobre o escopo integral daquela mesma issue**, não redefinir sucesso como “mínimo viável”. Divisões internas em commits são permitidas; microissues para requisitos já previstos não são a abordagem padrão.

**Dono do aceite:** mantenedor. **Dono da implementação:** agente executor designado. Uma PR não é evidência de conclusão por si só. Ver [AGENTS.md](AGENTS.md).

## Milestones (marcos lógicos de planejamento)

> **Situação do GitHub:** estes são **marcos documentados**, não objetos nativos de *GitHub Milestones*. A conexão atual não oferece criação de milestones nativos. Os códigos M1–M4 vinculam as issues ao roadmap; associar nativamente quando houver ferramenta/permissão compatível. Não confundir esta tabela com milestones já cadastrados no GitHub.

| Marco | Entrega consolidada | Issues abrangidas | Critério de encerramento |
| --- | --- | --- | --- |
| **M1 — Fundação Rune e domínio equivalente** | Infraestrutura nativa reproduzível e núcleo Rust capaz de validar/modelar/calcular os dados do NEXO | [#2](https://github.com/pedro-labsabs/nexo/issues/2), [#3](https://github.com/pedro-labsabs/nexo/issues/3) | Compilação/smoke Linux + equivalência dos contratos e indicadores com testes |
| **M2 — Experiência offline integral** | Persistência e anotações robustas, dashboard completo, grafo Rune com CRUD de relações manuais | [#4](https://github.com/pedro-labsabs/nexo/issues/4), [#5](https://github.com/pedro-labsabs/nexo/issues/5), [#6](https://github.com/pedro-labsabs/nexo/issues/6) | Exploração offline de ponta a ponta, edição persistida, importação/exportação e grafo interativo |
| **M3 — Autonomia Rust e corte definitivo** | Sincronização Github somente-leitura em Rust, integração total, remoção da stack web legada e validação | [#7](https://github.com/pedro-labsabs/nexo/issues/7), [#8](https://github.com/pedro-labsabs/nexo/issues/8) | Aplicação Linux autônoma, testes/CI/regressões e documentação completos |
| **M4 — Web e portabilidade (opcional)** | Experimento WASM/WebGPU e matriz de suporte, com decisão fundamentada | [#9](https://github.com/pedro-labsabs/nexo/issues/9) | Experimento reproduzível e relatório final; **não bloqueia M3** |

Sem datas artificiais: progredir conforme evidência técnica, conclusão integral e maturidade do Rune.

## Fila de implementação: issues grandes e fechadas

| Ordem sugerida | Issue / pacote de trabalho | Depende de | Resultado obrigatório |
| --- | --- | --- | --- |
| 1 | [#2 — Infraestrutura completa Rust/Rune Linux](https://github.com/pedro-labsabs/nexo/issues/2) | ADR 0001 | Workspace, pin upstream, app real, smoke, toolchain e CI |
| 2 | [#3 — Modelos, indicadores e relações Rust](https://github.com/pedro-labsabs/nexo/issues/3) | #2 | Contratos 1.0, validação, cálculos determinísticos, evidências, fixtures e equivalência |
| 3 | [#4 — Persistência/anotações/import-export/relatórios Rust](https://github.com/pedro-labsabs/nexo/issues/4) | #3 | Persistência segura, isolamento, CRUD, LWW, tombstones e relatórios |
| 4 | [#5 — Dashboard Rune offline completo](https://github.com/pedro-labsabs/nexo/issues/5) | #2–#4 | Dashboard, filtros, detalhes, temas, edição, UX e falhas |
| 5 | [#6 — Grafo Rune interativo completo](https://github.com/pedro-labsabs/nexo/issues/6) | #3–#5 | Render/layout, pan/zoom/fit, proveniência e CRUD de relações persistidas |
| 6, paralelizável depois de #5 | [#7 — Sincronização GitHub Rust completa](https://github.com/pedro-labsabs/nexo/issues/7) | #3–#5 | Auth, paginação, deduplicação, rate limits, árvore e CLI sem operações de escrita |
| 7 | [#8 — Corte integral, E2E, CI e benchmark](https://github.com/pedro-labsabs/nexo/issues/8) | #2–#7 | Eliminação de runtime web legado, suíte regressiva, empacotamento Linux e docs |
| Após o desktop, opcional | [#9 — Web/WASM/WebGPU e portabilidade](https://github.com/pedro-labsabs/nexo/issues/9) | #8 | Spike completo, evidências e decisão sobre manter/adiar/abandonar |

**Dependências não impõem desenvolvimento minúsculo:** cada issue recebe a totalidade do seu pacote funcional. #6 e #7 podem se sobrepor **após** as bases necessárias, se as alterações forem coordenadas sem conflitos. Não iniciar #8 como substituto de trabalho inacabado de #2–#7.

## Contratos que atravessam todas as fases

1. **Local-first:** snapshot visível offline; sincronização GitHub somente-leitura; nenhum token persistido em JSON, relatórios ou commits.
2. **Dados versionados e interoperáveis:** snapshot `metadata.version = 1.0`, anotações `schemaVersion = 1.0` e migrações explícitas em caso de mudança.
3. **Correção e rastreabilidade:** indicadores com fórmula, entradas, evidências e limitações; `unknown` quando o dado é insuficiente.
4. **Relações:** nunca deduzir relação por semelhança de nomes; diferenciar automática de manual e guardar proveniência.
5. **Anotações:** preservar LWW (empate local), tombstones, identidade por usuário, não ressurreição e persistência segura inclusive sob erro de IO.
6. **Experiência de uso:** português brasileiro, estados de erro/vazio, navegação e testes de interações (mouse e teclado).
7. **Rune primeiro:** usar Rust/Rune em interface, domínio, armazenamento e sincronização; outra tecnologia apenas com bloqueio comprovado e decisão registrada.
8. **Aceite honesto:** se o Rune fixado não suportar uma função, registrar tentativa, limitação, alternativas e próximo passo; nunca mascarar uma entrega parcial.

## Protocolo de mantenedor ↔ agente executor

1. Selecionar uma issue quando as dependências estiverem atendidas. Passar **o corpo inteiro**, [AGENTS.md](AGENTS.md), ADR, contratos e testes legados como requisitos de trabalho.
2. Exigir execução de **todas** as capacidades agrupadas da issue; evitar instruções como “faça primeiro o slice mínimo e pare”. O agente pode planejar etapas internas, mas deve terminar o pacote.
3. Auditoria do mantenedor compara implementação e critérios da issue **item a item**; verifica testes reais, regressões, logs e demonstrações. Conclusões parciais continuam abertas.
4. Em reprovação, fornecer remediação com falhas observadas, preservar critérios e trabalhar na implementação existente até resolver; não reduzir escopo nem encerrar a issue prematuramente.
5. Encerrar a issue somente após evidência consolidada e confirmação do aceite. Encerrar o milestone lógico somente quando todas as issues obrigatórias estiverem concluídas.

## Fora do compromisso obrigatório

- Publicação comercial ou SLA de produção.
- Paridade web/multiplataforma já na fase Linux.
- Comparações de desempenho sem metodologia controlada.
- Preservação de arquitetura, UX ou dependências React/Vite/Cytoscape por mera compatibilidade histórica.

## Registro de progresso

- **09/11/2026:** ADR 0001 e PR #1 abertas; roadmap e política NEXO-only definidos; issues **#2 a #9** criadas como pacotes integrais.
- **Estado de implementação:** nenhum trabalho de código Rust/Rune foi comprovado neste planejamento; todas as issues de implementação exigem execução e aceite futuros.
- **Milestones nativos GitHub:** **não criados** pela conexão atual; M1–M4 são exclusivamente marcos lógicos no documento e nos títulos/corpos das issues.
