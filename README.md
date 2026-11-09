# NEXO — Observatório Local de Portfólio GitHub

NEXO é uma aplicação local-first desenvolvida em Node.js 22, TypeScript estrito, React, Vite e Cytoscape.js para descobrir, analisar e visualizar repositórios acessíveis a um usuário no GitHub, mapear suas relações estruturais e técnicas e calcular indicadores de qualidade determinísticos e explicáveis.

---

> **Direção futura — reconstrução em Rune/Rust (decisão de 09/11/2026).** Este README descreve a implementação TypeScript/React **atual**. Foi aprovada a reconstrução integral do NEXO como app Linux nativo usando [Rune da Anomaly](https://github.com/anomalyco/rune) e Rust, com o sincronizador Node temporariamente mantido. Leia o [plano e decisão arquitetural](docs/architecture/0001-rune-native-rebuild.md). **A migração ainda não foi implementada.**

## 🌟 Recursos Principais

1. **Sincronização Segura via CLI**:
   - Autenticação por variável de ambiente (`GH_TOKEN` ou `GITHUB_TOKEN`).
   - Paginação completa em todos os repositórios acessíveis (proprietário, colaborador e membro de organizações).
   - Desduplicação determinística pelo ID imutável do GitHub.
   - Inspeção de árvore de arquivos em busca de infraestrutura de testes, CI e documentação com tratamento gracioso de rate limits, árvores truncadas e repositórios vazios.
   - Preservação de snapshots em JSON versionado sem armazenar credenciais ou tokens.
   - Operações estritamente de **leitura** (nunca grava nada no GitHub).

2. **Dashboard Offline e Responsivo**:
   - Interface completa em Português do Brasil com temas Claro e Escuro.
   - Métricas agregadas de portfólio (repositórios, organizações, relações, testes, CI, estrelas).
   - Filtros dinâmicos por texto (nome/descrição), organização, visibilidade (público/privado) e linguagem primária.
   - Ordenação customizada por recência de atividade, escore de qualidade, estrelas, tamanho e nome.
   - Não requer conexão com o GitHub ou internet em tempo de visualização.

3. **Grafo Interativo do Portfólio (Cytoscape.js)**:
   - Visualização visual de nós de organizações e repositórios.
   - Controles de zoom, pan e centralização automática (*fit*).
   - Distinção visual explícita entre relações comprovadas (pertencimento e links documentais ou dependências) e relações manuais anotações pelo usuário.
   - Modal para adição de novas conexões manuais com persistência local.

4. **Indicadores Técnicos Determinísticos e Explicáveis**:
   - **Infraestrutura de Testes**: verificação de configs e arquivos de teste (com disclaimer explícito: *a presença de arquivos não prova execução ou sucesso dos testes*).
   - **Configuração de CI**: identificação de fluxos automatizados (`.github/workflows`, etc.).
   - **Documentação**: avaliação ponderada de README, CONTRIBUTING, LICENSE e manuais técnicos.
   - **Recência de Atividade**: cálculo offline determinístico com base em data de referência fixa no snapshot.
   - **Sinais de Manutenção**: análise de branch padrão, arquivamento e operabilidade.
   - Exibição de inputs inspecionados, fórmula de cálculo e limitações para cada indicador.
   - Dados ausentes ou inacessíveis produzem o estado `unknown` em vez de presumir falha.

5. **Avaliação Subjetiva / Potencial Mantida pelo Usuário**:
   - Camada separada e editável para anotação de valor estratégico e potencial comercial.
   - Declaração explícita de transparência (*metadados do GitHub não medem objetivamente o potencial comercial ou estratégico*).

6. **Relatórios e Portabilidade**:
   - Exportação de relatório completo e auditável em Markdown em Português.
   - Aviso de privacidade e segurança automático antes de exportar dados de repositórios privados.
   - Exportação e importação de snapshots JSON versionados com validação estrita via Zod.
   - Preservação e exportação de anotações e relações manuais que sobrevivem a re-sincronizações.

---

## 🛠️ Instalação e Execução

### Pré-requisitos
- Node.js versão 22.x
- npm versão 10 ou superior

### 1. Clonar e Instalar Dependências
```bash
git clone <url-do-repositorio>
cd nexo
npm install
```

### 2. Iniciar o Dashboard Offline (Modo Demonstração)
Para explorar a aplicação imediatamente sem necessidade de credenciais ou tokens:
```bash
npm run dev &
```
Abra http://localhost:5173 no seu navegador. Você poderá explorar a base de demonstração sintética realista, alternar filtros, interagir com o grafo Cytoscape e exportar relatórios Markdown.

### 3. Sincronizar Repositórios do GitHub via CLI
Para gerar um snapshot local dos seus repositórios reais:
```bash
export GH_TOKEN="seu_token_pessoal_aqui"

# Executar sincronização direta com TypeScript:
npm run sync

# Ou com opções personalizadas:
npx tsx src/cli/index.ts --out snapshots/meu-portfolio.json --annotations annotations.json
```
O snapshot gerado poderá ser carregado no dashboard via botão **Importar** na barra superior.

---

## 🧪 Verificação e Qualidade

O projeto conta com uma suíte abrangente de testes automatizados e checagens estritas:

| Comando | Descrição |
| :--- | :--- |
| `npm run lint` | Validação de estilo e boas práticas com ESLint |
| `npm run typecheck` | Checagem estrita de tipos TypeScript (projeto web e CLI) |
| `npm run test` | Execução dos testes automatizados com Vitest |
| `npm run build:all` | Compilação de produção da aplicação web Vite e binário do CLI |

---

## 📂 Estrutura do Repositório

```text
nexo/
├── .github/
│   └── workflows/
│       └── ci.yml                 # Pipeline GitHub Actions (lint, typecheck, test, build)
├── public/
│   └── demo-snapshot.json         # Base sintética estática para demonstração
├── src/
│   ├── cli/
│   │   ├── index.ts               # Ponto de entrada do CLI executável
│   │   └── sync.ts                # Motor de paginação, inspeção de árvore e sincronização
│   ├── components/
│   │   ├── FiltersBar.tsx         # Barra de busca, filtros e ordenação
│   │   ├── Header.tsx             # Barra superior com ações e alternador de tema
│   │   ├── OverviewMetrics.tsx    # Cards com métricas consolidadas
│   │   ├── ProjectGraph.tsx       # Grafo interativo Cytoscape.js e modal de relações manuais
│   │   ├── RepositoryDetailDrawer.tsx # Gaveta lateral com indicadores e notas de potencial
│   │   └── RepositoryTable.tsx    # Tabela responsiva de projetos
│   ├── demo/
│   │   ├── demo-snapshot.json     # Fixture sintética realista versionada
│   │   └── generateDemoData.ts    # Gerador determinístico dos dados de demonstração
│   ├── shared/
│   │   ├── indicators.ts          # Motor de cálculo determinístico dos indicadores
│   │   ├── relations.ts           # Extrator de relações explícitas (doc e estruturais)
│   │   ├── report.ts              # Gerador de relatório em Markdown com avisos de privacidade
│   │   ├── types.ts               # Modelos e esquemas de dados Zod
│   │   └── validation.ts          # Validadores de integridade de snapshots e anotações
│   ├── test/
│   │   ├── app.test.tsx           # Testes de integração de componentes React
│   │   ├── indicators.test.ts     # Testes unitários do cálculo determinístico
│   │   ├── relations.test.ts      # Testes de extração estrita de relacionamentos
│   │   ├── report.test.ts         # Testes de geração do relatório Markdown
│   │   ├── setup.ts               # Configuração do ambiente de testes (jest-dom)
│   │   ├── sync.test.ts           # Testes de inspeção de árvores e tratamento de falhas
│   │   └── validation.test.ts     # Testes de validação Zod
│   ├── App.tsx                    # Componente raiz do Dashboard
│   ├── index.css                  # Folha de estilos base com TailwindCSS
│   └── main.tsx                   # Ponto de montagem React DOM
├── .gitignore                     # Proteção de snapshots locais privados e credenciais
├── eslint.config.js               # Configuração ESLint 9
├── index.html                     # Template HTML
├── package.json                   # Dependências e scripts
├── tailwind.config.js             # Configuração Tailwind
├── tsconfig.json                  # TypeScript para Frontend
├── tsconfig.cli.json              # TypeScript para CLI Node
└── vite.config.ts                 # Configuração de build Vite e Vitest
```

---

## 🔒 Princípios de Segurança e Privacidade

- **Local-First**: Todos os dados e snapshots residem exclusivamente na máquina do usuário.
- **Leitura Estrita**: O token informado é utilizado unicamente para chamadas de leitura (REST GET) no GitHub.
- **Proteção de Segredos**: Arquivos `snapshots/*.json` e extensões `*.snapshot.json` estão protegidos no `.gitignore` para evitar commits acidentais de metadados privados.
- **Avisos de Publicação**: O relatório Markdown gerado alerta explicitamente sobre a presença de repositórios privados antes de qualquer compartilhamento externo.

---

## 🚀 Manutenção Independente

Este projeto é autocontido e livre de dependências de modelos de IA, servidores externos permanentes ou bibliotecas proprietárias. Ele foi concebido para ser mantido de forma perene e independente:

1. **Adicionar novos indicadores**: Edite `src/shared/indicators.ts` e declare a nova regra determinística e suas limitações no schema `src/shared/types.ts`.
2. **Novas regras de relacionamentos**: Edite `src/shared/relations.ts` mantendo o princípio de que nenhuma relação deve ser inventada por similaridade de nome.
3. **Atualização de dependências**: O projeto utiliza módulos padrão do ecossistema TypeScript/Vite/React e suporta `npm update` sem dependências obsoletas ou microserviços acoplados.
