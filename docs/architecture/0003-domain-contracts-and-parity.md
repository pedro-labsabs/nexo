# ADR 0003 — Contratos de Domínio, Interoperabilidade JSON 1.0 e Matriz de Paridade Rust/TypeScript

- **Data:** 2026-11-10
- **Status:** Aprovada e implementada (Issue #3 / Milestone M1)
- **Contexto:** Fundação nativa Rust do domínio do NEXO (`crates/nexo-domain`), garantindo substituição determinística, pura e formalmente verificada da camada de modelos, validações, indicadores e relações anteriormente implementada em TypeScript.
- **Branch de implementação:** `feat/m1-rust-domain-parity`

---

## 1. Contexto e Decisão

O NEXO foi concebido como um observatório local-first de portfólios GitHub. Conforme definido no [ADR 0001](0001-rune-native-rebuild.md) e planejado no [ROADMAP.md](../../ROADMAP.md), a camada de domínio deve ser integralmente portada para Rust puro, livre de dependências de interface gráfica (Rune), aceleradores de GPU (`wgpu`) ou acoplamentos de rede/I-O.

A **Issue #3** estabeleceu o requisito de construir uma biblioteca Rust (`nexo-domain`) sem concessões ou simplificações ("zero TODOs, zero mocks"), garantindo **100% de paridade semântica**, **interoperabilidade exata com o formato Snapshot 1.0 e UserAnnotations 1.0** e **equivalência matemática comprovável contra a implementação de referência TypeScript**.

### Decisões Arquiteturais Fundamentais:
1. **Crate de Domínio Pura (`nexo-domain`):** Cobre exclusivamente modelos, tipos, validação de integridade, cálculo determinístico de indicadores técnicos, extração de relacionamentos com rastreamento estrito de proveniência e mesclagem de anotações com reconciliação Last-Write-Wins (LWW) e tombstones.
2. **Determinismo Temporal:** Qualquer cálculo dependente de tempo (ex: recência de commits/pushes) recebe explicitamente um carimbo `referenceDate` em formato ISO 8601 / RFC 3339. Relógios de sistema locais (`Utc::now()`) são proibidos nos cálculos de domínio, assegurando que o mesmo snapshot produza idênticos resultados em qualquer máquina e época.
3. **Não-Fabricação e Evidência Rigorosa:** Relações jamais são inferidas por similaridade fonética ou textual de nomes de repositórios. Forks sem repositório pai presente no snapshot nunca geram nós ou arestas artificiais.
4. **Isolamento de Dados Subjetivos:** Avaliações de negócio (`PotentialAssessment`) e curadoria de arestas manuais residem em `UserAnnotations`, estritamente segregadas dos fatos técnicos do snapshot.
5. **Oráculo de Testes Cruzados (Cross-Language Equivalence):** Uma suíte com 23 cenários sintéticos de teste é executada concorrentemente no TypeScript e no Rust, comparando saídas normalizadas e garantindo divergência zero.

---

## 2. Inventário Completo TypeScript ↔ Rust

A correspondência de módulos, estruturas de dados, enums e funções entre a camada TypeScript legado (`src/shared/`) e a biblioteca Rust (`crates/nexo-domain/src/`) é mapeada a seguir:

| Responsabilidade | Módulo TypeScript | Módulo Rust | Tipos e Funções Equivalentes |
|---|---|---|---|
| **Modelos de Entidades** | `src/shared/types.ts` | `crates/nexo-domain/src/model.rs` | `Repository` ↔ `Repository`<br>`RepositoryOwner` ↔ `RepositoryOwner`<br>`OwnerType` ↔ `OwnerType`<br>`Affiliation` ↔ `Affiliation`<br>`TreeSignals` ↔ `TreeSignals`<br>`Organization` ↔ `Organization`<br>`SnapshotMetadata` ↔ `SnapshotMetadata`<br>`Snapshot` ↔ `Snapshot` |
| **Indicadores Técnicos** | `src/shared/indicators.ts` | `crates/nexo-domain/src/indicators.rs` | `IndicatorStatus` ↔ `IndicatorStatus`<br>`IndicatorDetail` ↔ `IndicatorDetail`<br>`RepositoryIndicators` ↔ `RepositoryIndicators`<br>`computeIndicators(repo, refDate)` ↔ `compute_indicators(repo, ref_date)` |
| **Relações de Grafo** | `src/shared/relations.ts` | `crates/nexo-domain/src/relations.rs` | `RelationType` ↔ `RelationType`<br>`RelationshipProvenance` ↔ `RelationshipProvenance`<br>`NodeType` ↔ `NodeType`<br>`Relationship` ↔ `Relationship`<br>`RelationshipTombstone` ↔ `RelationshipTombstone`<br>`extractExplicitRelationships(...)` ↔ `extract_explicit_relationships(...)` |
| **Anotações e Curadoria** | `src/shared/annotations.ts` | `crates/nexo-domain/src/annotations.rs` | `StrategicValue` ↔ `StrategicValue`<br>`CommercialPotential` ↔ `CommercialPotential`<br>`PotentialAssessment` ↔ `PotentialAssessment`<br>`UserAnnotations` ↔ `UserAnnotations`<br>`mergeUserAnnotations(...)` ↔ `merge_user_annotations(...)`<br>`mergeManualRelationships(...)` ↔ `merge_manual_relationships(...)`<br>`mergePotentialAssessments(...)` ↔ `merge_potential_assessments(...)`<br>`getRelationshipLogicalKey(...)` ↔ `get_relationship_logical_key(...)` |
| **Validação e Integridade** | `src/shared/validation.ts` | `crates/nexo-domain/src/validation.rs` | `ValidationResult<T>` ↔ `Result<T, Vec<ValidationErrorDetail>>`<br>`validateSnapshot(value)` ↔ `validate_snapshot(value)`<br>`validateAnnotations(value)` ↔ `validate_annotations(value)`<br>`ValidationErrorDetail` |
| **Erros e Resultados** | N/A (Exceções soltas) | `crates/nexo-domain/src/error.rs` | `NexoError` (`thiserror`), `Result<T>` |
| **Estado e Visão** | `src/shared/types.ts` | `crates/nexo-domain/src/types.rs` | `ViewMode` (Dashboard, Graph, Table)<br>`NexoState` (estado de visualização reativo) |

---

## 3. Mapeamentos de Campos, Tipagem e Estratégia Serde

A serialização e desserialização em Rust utiliza o ecossistema `serde` e `serde_json`, obedecendo rigorosamente à convenção de nomenclatura do JSON 1.0 legado:

### 3.1 Convenção de Cases
- **Structs e Campos JSON:** Regra global `#[serde(rename_all = "camelCase")]`. Campos como `fullName`, `isPrivate`, `sizeKb`, `createdAt`, `openIssuesCount` mantêm compatibilidade com o formato TypeScript.
- **Enums de Domínio:** Regra `#[serde(rename_all = "snake_case")]` para a maioria dos identificadores textuais serializados:
  - `Affiliation`: `"owner"`, `"collaborator"`, `"organization_member"`
  - `IndicatorStatus`: `"passed"`, `"failed"`, `"unknown"`, `"partial"`
  - `RelationType`: `"belongs_to"`, `"references"`, `"depends_on"`, `"fork_of"`, `"custom"`
  - `RelationshipProvenance`: `"automatic"`, `"manual"`
  - `NodeType`: `"repository"`, `"organization"`
  - `StrategicValue`: `"low"`, `"medium"`, `"high"`, `"critical"`
  - `CommercialPotential`: `"none"`, `"experimental"`, `"viable"`, `"production_ready"`
- **Enum de Tipo de Proprietário:** `OwnerType` utiliza `#[serde(rename_all = "PascalCase")]`, gerando `"User"` e `"Organization"`.

### 3.2 Casos Especiais de Serialização
- **Campo `hasCI`:** No schema original TypeScript, o campo foi emitido ora como `hasCI` (acrônimo maiúsculo) ora como `hasCi`. A struct `TreeSignals` em Rust acomoda essa variação via:
  ```rust
  #[serde(rename = "hasCI", alias = "hasCi")]
  pub has_ci: bool,
  ```
- **Campo Reservado `type`:** O campo JSON `"type"` mapeia para `type_` em Rust via `#[serde(rename = "type")]`.
- **Identificadores de 64 bits:** Identificadores do GitHub (como IDs de repositório e organização) são modelados explicitamente como `u64`. Isso previne truncamento e overflow que ocorrem em linguagens ou parsers que utilizam inteiros de 32 bits assinados, mantendo precisão exata para repositórios com IDs elevados (como `8589934592` testado na suíte).
- **Tratamento de Nulos e Omissão:** Campos opcionais como `description`, `primary_language`, `avatar_url`, `readme_path` utilizam `Option<T>` com `#[serde(default, skip_serializing_if = "Option::is_none")]`. Coleções ausentes recebem automaticamente `Vec::new()` ou `BTreeMap::new()` através de `#[serde(default)]`.
- **Determinismo em Mapas:** Dicionários e coleções associativas utilizam `std::collections::BTreeMap` em vez de `HashMap`, assegurando que a serialização JSON ordene as chaves de forma lexicográfica consistente.

---

## 4. Validação Invariante e Relatório com Caminho Tipado

A validação em Rust (`validation.rs`) não depende do runtime do Zod, mas implementa um motor de validação estático e não destrutivo que inspeciona referências `&serde_json::Value` e emite `ValidationErrorDetail`:

```rust
pub struct ValidationErrorDetail {
    pub path: String,
    pub message: String,
}
```

### 4.1 Invariantes Verificados:
1. **Contrato de Versão:**
   - `metadata.version` deve ser estritamente `"1.0"`. Qualquer outro valor (ex: `"2.0"`) é rejeitado com mensagem no caminho `metadata.version`.
   - `userAnnotations.schemaVersion` deve ser estritamente `"1.0"`.
2. **Campos Obrigatórios:** Ausência de campos indispensáveis (`metadata`, `authenticatedUser`, `name`, `fullName`, `owner`, `id`) emite mensagens no formato `<caminho>: campo obrigatório ausente`.
3. **Formatação de Datas:** `createdAt`, `updatedAt`, `pushedAt`, `referenceDate`, `exportedAt` e `deletedAt` são validados contra a gramática estrita de RFC 3339 / ISO 8601. Strings arbitrárias ou timestamps corrompidos são rejeitados com `data inválida (esperado formato ISO 8601 / RFC 3339)`.
4. **Limites de Escores:** Escores de indicadores pré-computados devem residir no intervalo `[0, 100]`.
5. **Unicidade de Identidades:** Repositórios e organizações com identificadores (`id`) duplicados geram erro de validação imediato, prevenindo estados corrompidos no grafo e no armazenamento.
6. **Consistência de Metadados:** Se `metadata.totalRepositories` divergir da contagem de elementos na lista `repositories`, a validação aponta a incongruência no caminho `metadata.totalRepositories`.

---

## 5. Indicadores Técnicos: Fórmulas, Pesos e Exclusão Dinâmica de Unknown

O NEXO avalia a maturidade técnica de cada repositório através de 5 indicadores determinísticos com pesos fixos:

```text
Indicador                    Peso Base
--------------------------------------
Infraestrutura de Testes        25%
Configuração de CI              20%
Documentação do Projeto         25%
Recência de Atividade           15%
Sinais de Manutenção            15%
--------------------------------------
Soma Total                     100%
```

### 5.1 Fórmulas e Regras de Avaliação

#### 1. Infraestrutura de Testes (25%)
- **Evidências:** Diretórios e arquivos de teste detectados (`test_evidence`).
- **Pontuação:**
  - `test_evidence.len() >= 2`: Score 100 (`passed`).
  - `test_evidence.len() == 1`: Score 70 (`partial`).
  - `test_evidence.is_empty()`: Score 0 (`failed`).

#### 2. Configuração de CI (20%)
- **Evidências:** Workflows de CI detectados (`ci_evidence`), tais como `.github/workflows/` ou `.gitlab-ci.yml`.
- **Pontuação:**
  - `ci_evidence.len() >= 2`: Score 100 (`passed`).
  - `ci_evidence.len() == 1`: Score 80 (`passed`).
  - `ci_evidence.is_empty()`: Score 0 (`failed`).

#### 3. Documentação do Projeto (25%)
- **Pontuação Cumulativa (máx 100):**
  - README encontrado: +40 pontos.
  - Descrição do repositório preenchida: +20 pontos.
  - Documentação adicional (`docs/`, etc.): +20 pontos.
  - Guia de contribuição (`CONTRIBUTING.md`): +10 pontos.
  - Arquivo de licença (`LICENSE`): +10 pontos.
- **Status:** `>= 70` → `passed`; `>= 40` → `partial`; `< 40` → `failed`.

#### 4. Recência de Atividade (15%)
Calculada exclusivamente a partir da diferença em dias corridos entre `referenceDate` e `pushedAt`:
- $\le 30$ dias: Score 100 (`passed`)
- $\le 90$ dias: Score 80 (`passed`)
- $\le 180$ dias: Score 60 (`partial`)
- $\le 365$ dias: Score 40 (`partial`)
- $\le 730$ dias (2 anos): Score 20 (`failed`)
- $> 730$ dias: Score 0 (`failed`)

#### 5. Sinais de Manutenção (15%)
- **Repositório Arquivado (`archived == true`):** Score fixo de 10 (`failed`).
- **Repositório Ativo:** Base de 80 pontos:
  - Repositório desativado (`disabled == true`): base 0.
  - Repositório vazio (`sizeKb == 0`): penalidade de -20 pontos.
  - Branch padrão é `"main"`: bônus de +10 pontos.
  - Arquivo de licença presente: bônus de +10 pontos.
  - Clampar para o intervalo `[0, 100]`. Status `>= 70` → `passed`; senão `partial`.

### 5.2 Exclusão Dinâmica de Unknown na Média Ponderada
Quando um indicador possui status `Unknown`, ele é **completamente excluído do numerador e do denominador** no cálculo da média ponderada:

$$\text{OverallQualityScore} = \text{round}\left( \frac{\sum_{i \notin \text{Unknown}} (\text{score}_i \times \text{peso}_i)}{\sum_{i \notin \text{Unknown}} \text{peso}_i} \right)$$

Se **todos** os indicadores resultarem em `Unknown` (ex: repositório com árvore não inspecionada e sem datas válidas), o peso total é 0 e o `OverallQualityScore` resultante é fixado em `0`.

---

## 6. Tratamento de Evidência Insuficiente e Incerteza

O NEXO segue o princípio da **transparência e honestidade epistemológica**: a ausência de prova técnica não autoriza penalizar um repositório como reprovado.

1. **Árvore de Arquivos Não Inspecionada (`tree_inspected == false`):**
   - Os indicadores de Teste e CI retornam `status: IndicatorStatus::Unknown`, `score: 0`.
   - Limitações explicitam: *"A ausência de acesso à árvore impede checar arquivos. Não foi presumido resultado negativo."*
   - O indicador de Documentação computa apenas os pontos de descrição pública do repositório (máx 20) com status `Unknown`.
2. **Árvore Truncada pelo GitHub (`tree_truncated == true`):**
   - Se nenhuma evidência positiva de testes for localizada, o status retornado é `Unknown` (e não `Failed`), pois arquivos de teste podem ter sido omitidos em subdiretórios profundos além do limite da API.
3. **Erros de Inspeção (`inspection_error`):**
   - Erros reportados pela API (como rate limit, timeout ou permissões) são capturados nos inputs estruturados do indicador para auditoria.

---

## 7. Relações do Grafo, Proveniência e Não-Fabricação

O grafo de portfólio suporta 5 tipos de relação (`RelationType`):
- `belongs_to`: Vínculo entre repositório e organização.
- `references`: Citação explícita de outro repositório em documentação (`README.md`).
- `depends_on`: Dependência de manifesto (`package.json`) com especificação de repositório git/github.
- `fork_of`: Derivação de código com repositório de origem comprovado.
- `custom`: Relação inserida manualmente pelo usuário.

### 7.1 Regras de Proveniência Estrita (`extract_explicit_relationships`)
1. **Proibição de Inferência Heurística por Nomes:** Jamais são criadas arestas com base em similaridade fonética, prefixos ou sufixos comuns (ex: `nexo-core` e `nexo-ui` não formam relação a menos que haja citação explícita).
2. **Isolamento de Forks Sem Origem Provada:**
   - Se um repositório estiver marcado com `fork: true`, mas o repositório pai não constar no snapshot e não houver metadado inequívoco de origem, **nenhuma relação `fork_of` é fabricada**.
   - O repositório permanece no grafo com seu badge de fork, sem arestas direcionadas a alvos fantasmas.
3. **Dependências em Manifestos:**
   - Ao inspecionar `package.json`, apenas declarações de dependência que apontam expressamente para URLs ou especificadores de repositório (`git+https://github.com/...`, `github:org/repo`) geram `depends_on`.
   - Pacotes de registros públicos padrão (ex: `react`, `lodash`, `serde`) são ignorados para evitar arestas falsas em repositórios que meramente compartilham o mesmo pacote externo.
4. **Referências em Documentação:**
   - Citações em links Markdown `[texto](...full_name...)` ou URLs explícitas `github.com/org/repo` no `README.md` geram relações do tipo `references` com proveniência `automatic` e `verified: true`.
   - Auto-referências (repositório referenciando a si mesmo) são filtradas.

---

## 8. Anotações do Usuário, LWW e Prevenção de Ressurreição

As anotações manuais representam o conhecimento subjetivo do desenvolvedor sobre seu portfólio. A reconciliação entre estados locais e importados segue regras estritas:

### 8.1 Resolução de Conflitos Last-Write-Wins (LWW)
- Ao mesclar duas anotações para o mesmo repositório ou aresta, os timestamps ISO 8601 (`updated_at` / `deleted_at`) são convertidos para milissegundos UTC.
- **Regra de Empate:** Se `incoming_timestamp > local_timestamp`, o valor de `incoming` é aceito. Se `incoming_timestamp <= local_timestamp` ou se a data de incoming for inválida, **o registro local é preservado** (*tie favors local*).

### 8.2 Chave Lógica Direcionada
A identidade de relacionamentos manuais e tombstones é consolidada através da chave estável:
$$\text{logical\_key} = \text{source} \mathbin{::} \text{target} \mathbin{::} \text{type}$$
Arestas direcionadas em sentidos opostos (`A -> B` e `B -> A`) produzem chaves diferentes e são mantidas isoladamente.

### 8.3 Tombstones e Prevenção de Ressurreição
- Exclusões manuais geram `RelationshipTombstone` contendo a chave lógica e o timestamp `deleted_at`.
- Durante importações, sincronizações ou mesclagens, se uma relação existir mas seu carimbo de modificação for $\le \text{deleted\_at}$, a relação é descartada, impedindo a ressurreição de dados já apagados pelo usuário.
- Uma relação pode ser re-criada legitimamente no futuro se e somente se o seu timestamp de criação/atualização for **estritamente posterior** ao `deleted_at` do tombstone.

---

## 9. Matriz de Equivalência Transversal (23 Cenários)

Para assegurar 100% de paridade entre TypeScript e Rust, foi construída uma matriz executável de 23 cenários sintéticos em `fixtures/domain/`:

| ID | Cenário / Fixture | Tipo | Invariante Comprovado |
|---|---|---|---|
| 01 | `valid_complete_snapshot.json` | Válido | Snapshot canônico com repositórios, orgs, relações e anotações completas. |
| 02 | `valid_with_defaults_and_optionals_missing.json` | Válido | Tolerância a campos opcionais nulos ou coleções omitidas com defaults seguros. |
| 03 | `empty_snapshot.json` | Válido | Snapshot com 0 repositórios, 0 organizações e 0 relações. |
| 04 | `archived_repository.json` | Válido | Repositório arquivado recebe score 10 em manutenção (`failed`). |
| 05 | `disabled_repository.json` | Válido | Repositório desativado recebe score 0 na base de manutenção (`partial`). |
| 06 | `uninspected_tree.json` | Válido | Árvore inacessível resulta em `unknown` nos indicadores sem penalizar falsamente. |
| 07 | `truncated_tree.json` | Válido | Árvore truncada pelo GitHub emite `unknown` para ausência de testes. |
| 08 | `inspection_error.json` | Válido | Erros de inspeção de árvore são registrados estruturadamente nos inputs. |
| 09 | `different_affiliations.json` | Válido | Exercício completo de `owner`, `collaborator` e `organization_member`. |
| 10 | `large_ids_and_renamed_entities.json` | Válido | Identificadores de 64 bits (`> 2^31 - 1` e até `8589934592`) sem perda de precisão. |
| 11 | `precomputed_indicators.json` | Válido | Paridade de serde para snapshots que já contêm indicadores calculados. |
| 12 | `unknown_and_partial_indicators.json` | Válido | Normalização ponderada com múltiplos indicadores `partial` e `unknown`. |
| 13 | `belongs_to_relations.json` | Válido | Extração estrita de relações de afiliação e pertencimento a organizações. |
| 14 | `explicit_readme_references.json` | Válido | Extração de referências verificadas via links Markdown e URLs no README. |
| 15 | `explicit_manifest_dependencies.json` | Válido | Extração de dependências em `package.json` com URLs explícitas de repositórios. |
| 16 | `manual_and_automatic_relations.json` | Válido | Coexistência ordenada de arestas automáticas (verificadas) e manuais (curadoria). |
| 17 | `fork_with_proven_origin.json` | Válido | Relacionamento explícito de proveniência para fork com repositório pai conhecido. |
| 18 | `fork_without_proven_origin.json` | Válido | Garantia de **zero** arestas fabricadas quando o pai do fork não existe no snapshot. |
| 19 | `annotations_and_tombstones.json` | Válido | Mesclagem LWW de avaliações, reconciliação de tombstones e não-ressurreição. |
| 20 | `unknown_version.json` | Inválido | Rejeição imediata de snapshots com versão de schema `"2.0"`. |
| 21 | `missing_mandatory_fields.json` | Inválido | Rejeição por ausência de campos obrigatórios (`name`, `authenticatedUser`). |
| 22 | `invalid_types_and_malformed_dates.json` | Inválido | Rejeição por IDs em formato string e carimbos de data ISO malformados. |
| 23 | `inconsistent_identities.json` | Inválido | Rejeição por IDs duplicados e divergência na contagem de `totalRepositories`. |

### 9.1 Comandos de Execução e Reprodução
- **Referência TypeScript:** `npm run test:parity` (executa `scripts/run-parity-check.ts` e valida as golden outputs em `fixtures/domain/expected/`).
- **Oráculo Rust:** `cargo test -p nexo-domain --test equivalence_runner` (avalia cada cenário em Rust e compara campo a campo com a golden output).
- **Suíte Completa:**
  ```bash
  npm run test:parity && cargo test -p nexo-domain --test equivalence_runner
  ```

---

## 10. Interfaces para as Próximas Issues do Milestone

A crate `nexo-domain` atua como o alicerce estável para todo o roadmap subsequente:

```text
nexo-domain (Crate de Domínio Pura)
  │
  ├──► Issue #4: nexo-storage (Persistência segura, CRUD de anotações, exportação)
  │      └── Consome: Snapshot, UserAnnotations, merge_user_annotations, validate_snapshot
  │
  ├──► Issue #5: nexo-desktop Dashboard (Visualização de métricas e filtros)
  │      └── Consome: Repository, RepositoryIndicators, compute_indicators, IndicatorStatus
  │
  ├──► Issue #6: nexo-desktop Grafo Rune (Layout e renderização nativa de nós/arestas)
  │      └── Consome: Relationship, RelationType, RelationshipTombstone, get_relationship_logical_key
  │
  └──► Issue #7: nexo-github (Cliente assíncrono de sincronização em Rust)
         └── Consome: Repository, TreeSignals, Organization, SnapshotMetadata, extract_explicit_relationships
```

---

## 11. Divergências Justificadas e Evolução de Versões

1. **Validação Rigorosa de Datas e Identidades:** O TypeScript histórico tratava datas apenas superficialmente via Zod `.string()`. Em Rust e na suite alinhada de TypeScript, datas são estritamente validadas quanto ao formato RFC 3339 / ISO 8601, e duplicações de IDs de repositório são expressamente proibidas. Essa rigidez adicional foi estendida a ambas as linguagens e verificada pelos cenários 22 e 23.
2. **Aliasing Permissivo para `hasCI`:** Para garantir que arquivos gravados por versões antigas da CLI Node continuem legíveis pelo motor Rust sem migração manual, `has_ci` aceita tanto `hasCI` quanto `hasCi`.
3. **Evolução de Esquema:** O formato Snapshot 1.0 é imutável. Caso versões futuras do NEXO demandem campos adicionais, será introduzido o schema 1.1 ou 2.0 com conversor/migrador formal e explícito, preservando sempre a retrocompatibilidade com arquivos legados existentes.
