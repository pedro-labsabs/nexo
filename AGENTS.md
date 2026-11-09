# Instruções para agentes — NEXO

> **Escopo exclusivo deste repositório.** As regras abaixo não se aplicam a outros projetos.

## Contexto obrigatório

O NEXO é uma prova de conceito local-first para explorar a reconstrução **integral** em Rust e [Rune da Anomaly](https://github.com/anomalyco/rune). Ler antes de atuar:

- [Decisão arquitetural Rune/Rust](docs/architecture/0001-rune-native-rebuild.md)
- [ROADMAP.md](ROADMAP.md)
- O código e os testes existentes, que documentam contratos de comportamento, não uma arquitetura intocável.

A prioridade é explorar Rune em um aplicativo completo para Linux; o sincronizador Node/TypeScript é **transitório**. Não confundir Rune da Anomaly com a IDE de rune.build.

## Política de execução integral por issue (obrigatória)

1. **Uma issue = uma entrega funcional completa.** Assumir todo o escopo descrito na issue, inclusive fluxos de erro, integrações, testes, documentação e critérios de aceite. Não implementar apenas um *MVP*, *vertical slice*, protótipo incompleto, happy path ou subconjunto conveniente como substituto do trabalho solicitado.
2. O agente executor deve trabalhar até concluir **todos** os critérios da issue, inclusive os que exijam correção e reteste. Um ponto difícil não autoriza retirar requisitos nem declarar a issue encerrada.
3. Refatorações amplas, remoções de tecnologia antiga e redesenho são permitidos **quando fizerem parte da issue**. Não preservar React, Vite ou Cytoscape.js por inércia; preferir Rust/Rune. Exceções só com impedimento comprovado, justificativa e decisão explícita do mantenedor.
4. **Não reduzir o escopo durante a auditoria.** O mantenedor deve avaliar a implementação contra a issue inteira; defeitos originam correções sobre a entrega integral (na mesma linha de trabalho), não uma redefinição retroativa de "mínimo viável".
5. Se um bloqueio real no Rune impedir requisito obrigatório, registrar o erro reproduzível, commit/versão upstream, alternativas experimentadas, impacto e proposta de correção. **Não mascarar a lacuna como concluída**; solicitar decisão antes de alterar o contrato.
6. "Pronto" exige: código integrado; aceites demonstráveis; testes e verificações realmente executados com resultado; relatórios breves de evidência; documentação atualizada; nenhuma regressão crítica conhecida. Se parte não foi testada, declarar explicitamente.
7. Dividir a **implementação interna** em commits e etapas é permitido; a entrega/aceite da issue permanece indivisível. Subtarefas internas não viram novas issues sem necessidade de escopo independente.
8. Não gerar, publicar ou commitar tokens, dados privados nem snapshots reais. GitHub synchronization deve continuar somente-leitura. Preservar identidades, dados e versões; não destruir arquivos locais em nome do experimento.

## Forma da entrega de um agente

No relatório final de cada issue: apontar SHA/branch/PR, lista de requisitos cumpridos **um a um**, comandos e resultados reais, evidências de UI e casos de borda, regressões, desvios justificáveis e problemas restantes. Se qualquer aceite obrigatório estiver pendente, informar **não concluída** e continuar a remediação.

## Planejamento

O roadmap utiliza [milestones **nativos** M1–M4](ROADMAP.md) e issues amplas, com associação verificada das issues #2–#9 no GitHub. A existência de um milestone não dispensa nenhum critério de aceite. A PR de documentação não é evidência de execução da migração nem autoriza merge de código incompleto.
