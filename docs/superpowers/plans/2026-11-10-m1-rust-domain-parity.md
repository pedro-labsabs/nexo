# NEXO M1 Issue #3 — Rust Domain Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace TypeScript models, validations, indicators, and relationships with a complete, deterministic, and verifiable Rust domain library in `nexo-domain`, providing 100% semantic equivalence and JSON 1.0 interoperability.

**Architecture:** A pure Rust domain layer partitioned into focused modules (`model`, `indicators`, `relations`, `annotations`, `validation`, `state`, `error`) without UI or Rune dependencies. High-precision Serde serialization guaranteeing exact 1.0 JSON compatibility, deterministic reference-date calculations, strict relationship provenance, LWW annotation conflict resolution with tombstones, and automated cross-language comparison tests against TypeScript.

**Tech Stack:** Rust 1.99.1 (edition 2021), `serde`, `serde_json`, `chrono`, `thiserror`, TypeScript, Vitest, Node.js 20+.

**Spec:** Issue #3 description, `AGENTS.md`, `ROADMAP.md`, `docs/architecture/0001-rune-native-rebuild.md`, and `docs/architecture/0002-rune-pin-and-linux-deps.md`.

## Global Constraints

- Branch-base: `recovery/r1-from-nex0` at commit `91d4437513e3eec0d29afd9bd9d0af5a59b7a58d`.
- Implementation branch: `feat/m1-rust-domain-parity`.
- No placeholders, no TODOs, no minimal slice: 100% complete issue scope.
- Maintain existing Rune foundation, tests, and headless lifecycles approved in Issue #2.
- Snapshot `metadata.version = "1.0"` and annotations `schemaVersion = "1.0"` must be preserved.
- Never infer relationships from naming similarity; require verifiable evidence.
- Deterministic indicator calculations strictly parameterized by `referenceDate`.
- Preserve untracked workspace files (`pnpm-lock.yaml`, `pnpm-workspace.yaml`).

---

### Task 1: Core Domain Models & Serde Interoperability 1.0

**Files:**
- Create: `crates/nexo-domain/src/model.rs`
- Modify: `crates/nexo-domain/src/lib.rs:1-10`
- Test: `crates/nexo-domain/tests/model_tests.rs`

**Interfaces:**
- Consumes: `chrono`, `serde`, `serde_json`
- Produces:
  - `pub enum Affiliation { Owner, Collaborator, OrganizationMember }`
  - `pub enum OwnerType { User, Organization }`
  - `pub struct RepositoryOwner { pub login: String, pub id: u64, pub avatar_url: Option<String>, pub type_: OwnerType }`
  - `pub struct TreeSignals { pub has_tests: bool, pub test_evidence: Vec<String>, pub has_ci: bool, pub ci_evidence: Vec<String>, pub has_documentation: bool, pub documentation_evidence: Vec<String>, pub readme_found: bool, pub readme_path: Option<String>, pub contributing_found: bool, pub license_file_found: bool, pub package_json_found: bool, pub is_empty_repo: bool, pub tree_truncated: bool, pub tree_inspected: bool, pub inspection_error: Option<String> }`
  - `pub struct Repository { pub id: u64, pub node_id: String, pub name: String, pub full_name: String, pub owner: RepositoryOwner, pub is_private: bool, pub html_url: String, pub description: Option<String>, pub fork: bool, pub created_at: String, pub updated_at: String, pub pushed_at: String, pub size_kb: u64, pub stargazers_count: u64, pub watchers_count: u64, pub open_issues_count: u64, pub forks_count: u64, pub primary_language: Option<String>, pub languages: std::collections::BTreeMap<String, u64>, pub archived: bool, pub disabled: bool, pub default_branch: String, pub affiliation: Affiliation, pub tree_signals: TreeSignals, pub indicators: Option<RepositoryIndicators> }`
  - `pub struct Organization { pub id: u64, pub login: String, pub name: Option<String>, pub avatar_url: Option<String>, pub description: Option<String>, pub repository_count: u64 }`
  - `pub struct SnapshotMetadata { pub version: String, pub created_at: String, pub reference_date: String, pub authenticated_user: String, pub total_repositories: u64, pub incomplete_sync: bool, pub sync_warnings: Vec<String> }`
  - `pub struct Snapshot { pub metadata: SnapshotMetadata, pub organizations: Vec<Organization>, pub repositories: Vec<Repository>, pub relationships: Vec<Relationship>, pub user_annotations: Option<UserAnnotations> }`

- [ ] **Step 1: Write the failing test**

Create `crates/nexo-domain/tests/model_tests.rs`:
```rust
use nexo_domain::model::{Affiliation, Organization, OwnerType, Repository, RepositoryOwner, Snapshot, SnapshotMetadata, TreeSignals};

#[test]
fn test_repository_serde_matches_snapshot_1_0_contract() {
    let json_input = r#"{
        "id": 1001,
        "nodeId": "R_kgDOA1",
        "name": "nexo-core",
        "fullName": "inova-labs/nexo-core",
        "owner": {
            "login": "inova-labs",
            "id": 101,
            "type": "Organization"
        },
        "isPrivate": false,
        "htmlUrl": "https://github.com/inova-labs/nexo-core",
        "description": "Motor de teste",
        "fork": false,
        "createdAt": "2025-02-10T10:00:00Z",
        "updatedAt": "2026-10-28T14:30:00Z",
        "pushedAt": "2026-10-28T14:30:00Z",
        "sizeKb": 14200,
        "stargazersCount": 342,
        "watchersCount": 28,
        "openIssuesCount": 4,
        "forksCount": 19,
        "primaryLanguage": "TypeScript",
        "languages": { "TypeScript": 92000 },
        "archived": false,
        "disabled": false,
        "defaultBranch": "main",
        "affiliation": "organization_member",
        "treeSignals": {
            "hasTests": true,
            "testEvidence": ["tests/core.test.ts"],
            "hasCI": true,
            "ciEvidence": [".github/workflows/ci.yml"],
            "hasDocumentation": true,
            "documentationEvidence": ["docs/architecture.md"],
            "readmeFound": true,
            "readmePath": "README.md",
            "contributingFound": true,
            "licenseFileFound": true,
            "packageJsonFound": false,
            "isEmptyRepo": false,
            "treeTruncated": false,
            "treeInspected": true
        }
    }"#;

    let repo: Repository = serde_json::from_str(json_input).expect("deserialization failed");
    assert_eq!(repo.id, 1001);
    assert_eq!(repo.affiliation, Affiliation::OrganizationMember);
    assert_eq!(repo.owner.type_, OwnerType::Organization);
    assert!(repo.tree_signals.has_tests);

    let serialized = serde_json::to_string(&repo).expect("serialization failed");
    assert!(serialized.contains(r#""nodeId":"R_kgDOA1""#));
    assert!(serialized.contains(r#""affiliation":"organization_member""#));
}

#[test]
fn test_large_repo_id_greater_than_i32() {
    let large_id: u64 = 8_589_934_592; // 2^33
    let json = format!(r#"{{"id":{large_id},"nodeId":"x","name":"x","fullName":"o/x","owner":{{"login":"o","id":1,"type":"User"}},"isPrivate":false,"htmlUrl":"u","description":null,"fork":false,"createdAt":"t","updatedAt":"t","pushedAt":"t","sizeKb":0,"stargazersCount":0,"watchersCount":0,"openIssuesCount":0,"forksCount":0,"primaryLanguage":null,"languages":{{}},"archived":false,"disabled":false,"defaultBranch":"main","affiliation":"owner","treeSignals":{{"hasTests":false,"testEvidence":[],"hasCI":false,"ciEvidence":[],"hasDocumentation":false,"documentationEvidence":[],"readmeFound":false,"contributingFound":false,"licenseFileFound":false,"packageJsonFound":false,"isEmptyRepo":false,"treeTruncated":false,"treeInspected":false}}}}"#);
    let repo: Repository = serde_json::from_str(&json).expect("large id deserialization failed");
    assert_eq!(repo.id, large_id);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p nexo-domain --test model_tests`
Expected: FAIL with "cannot find module or type"

- [ ] **Step 3: Write minimal implementation**

Create `crates/nexo-domain/src/model.rs` and update `crates/nexo-domain/src/lib.rs` exporting models with serde `rename_all = "camelCase"` and appropriate enum serde attributes (`snake_case` for `Affiliation`).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p nexo-domain --test model_tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/nexo-domain/src/model.rs crates/nexo-domain/src/lib.rs crates/nexo-domain/tests/model_tests.rs
git commit -m "feat(domain): port core models and serde contracts for snapshot 1.0"
```

---

### Task 2: Indicators Domain Models & Deterministic Calculation Engine

**Files:**
- Create: `crates/nexo-domain/src/indicators.rs`
- Modify: `crates/nexo-domain/src/lib.rs`
- Test: `crates/nexo-domain/tests/indicator_tests.rs`

**Interfaces:**
- Consumes: `model::{Repository, TreeSignals}`
- Produces:
  - `pub enum IndicatorStatus { Passed, Failed, Unknown, Partial }`
  - `pub struct IndicatorDetail { pub id: String, pub name: String, pub status: IndicatorStatus, pub score: u32, pub summary: String, pub inputs: serde_json::Value, pub calculation: String, pub limitations: Vec<String> }`
  - `pub struct RepositoryIndicators { pub test_infrastructure: IndicatorDetail, pub ci_configuration: IndicatorDetail, pub documentation: IndicatorDetail, pub activity_recency: IndicatorDetail, pub maintenance_signals: IndicatorDetail, pub overall_quality_score: u32 }`
  - `pub fn compute_indicators(repo: &Repository, reference_date_str: &str) -> RepositoryIndicators`

- [ ] **Step 1: Write the failing test**

Create `crates/nexo-domain/tests/indicator_tests.rs` covering:
- Passing scores for comprehensive signals (all 5 indicators)
- Weights: Tests 25%, CI 20%, Docs 25%, Activity 15%, Maintenance 15%
- `unknown` exclusion from denominator normalization
- Truncated tree without observed signals producing `unknown` status
- Uninspected tree producing `unknown`
- Archived repos receiving 10 points and `failed` status in maintenance
- Activity recency thresholds (<=30d: 100, <=90d: 80, <=180d: 60, <=365d: 40, >365d: 15)
- Determinism with fixed `reference_date_str`

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p nexo-domain --test indicator_tests`
Expected: FAIL with "cannot find compute_indicators"

- [ ] **Step 3: Write minimal implementation**

Implement `crates/nexo-domain/src/indicators.rs` replicating `src/shared/indicators.ts` logic verbatim (same Portuguese strings, calculation formulas, and limitations).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p nexo-domain --test indicator_tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/nexo-domain/src/indicators.rs crates/nexo-domain/tests/indicator_tests.rs crates/nexo-domain/src/lib.rs
git commit -m "feat(domain): implement deterministic technical indicators engine in Rust"
```

---

### Task 3: Relations Domain Models & Provenance Extraction Engine

**Files:**
- Create: `crates/nexo-domain/src/relations.rs`
- Modify: `crates/nexo-domain/src/lib.rs`
- Test: `crates/nexo-domain/tests/relations_tests.rs`

**Interfaces:**
- Consumes: `model::{Repository, Organization}`
- Produces:
  - `pub enum RelationType { BelongsTo, References, DependsOn, ForkOf, Custom }`
  - `pub enum RelationshipProvenance { Automatic, Manual }`
  - `pub enum NodeType { Repository, Organization }`
  - `pub struct Relationship { pub id: String, pub source: String, pub target: String, pub source_type: NodeType, pub target_type: NodeType, pub type_: RelationType, pub provenance: RelationshipProvenance, pub verified: bool, pub evidence: Option<String>, pub description: Option<String>, pub created_at: String, pub updated_at: Option<String> }`
  - `pub struct RelationshipTombstone { pub id: String, pub source: String, pub target: String, pub type_: RelationType, pub deleted_at: String }`
  - `pub fn extract_explicit_relationships(repos: &[Repository], orgs: &[Organization], readme_contents_map: &std::collections::HashMap<u64, String>, manifest_contents_map: &std::collections::HashMap<u64, String>) -> Vec<Relationship>`

- [ ] **Step 1: Write the failing test**

Create `crates/nexo-domain/tests/relations_tests.rs` covering:
- `belongs_to` creation only when repo.owner matches org.login
- `references` extraction from README with explicit GitHub URLs or markdown links
- `depends_on` extraction from package.json manifest sections with explicit git/github URLs
- Rejection of false relationships based solely on package name similarity
- No `fork_of` generation when fork parent identity is absent
- Strict ID stability `rel_belongs_{repo.id}_{org.id}`, `rel_ref_{repo.id}_{target.id}`, `rel_dep_{repo.id}_{target.id}`

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p nexo-domain --test relations_tests`
Expected: FAIL

- [ ] **Step 3: Write minimal implementation**

Implement `crates/nexo-domain/src/relations.rs` porting `src/shared/relations.ts` with strict provenance.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p nexo-domain --test relations_tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/nexo-domain/src/relations.rs crates/nexo-domain/tests/relations_tests.rs crates/nexo-domain/src/lib.rs
git commit -m "feat(domain): port relationship models and explicit provenance extraction"
```

---

### Task 4: Annotations, Assessments, LWW Merging & Tombstones

**Files:**
- Create: `crates/nexo-domain/src/annotations.rs`
- Modify: `crates/nexo-domain/src/lib.rs`
- Test: `crates/nexo-domain/tests/annotations_tests.rs`

**Interfaces:**
- Consumes: `relations::{Relationship, RelationshipTombstone, RelationType}`
- Produces:
  - `pub enum StrategicValue { Low, Medium, High, Critical }`
  - `pub enum CommercialPotential { None, Experimental, Viable, ProductionReady }`
  - `pub struct PotentialAssessment { pub repository_id: u64, pub strategic_value: StrategicValue, pub commercial_potential: CommercialPotential, pub target_audience: Option<String>, pub notes: Option<String>, pub tags: Vec<String>, pub updated_at: String }`
  - `pub struct UserAnnotations { pub schema_version: String, pub exported_at: String, pub authenticated_user: Option<String>, pub manual_relationships: Vec<Relationship>, pub deleted_relationships: Vec<RelationshipTombstone>, pub potential_assessments: std::collections::BTreeMap<String, PotentialAssessment> }`
  - `pub fn get_relationship_logical_key(source: &str, target: &str, type_: &RelationType) -> String`
  - `pub fn merge_potential_assessments(local: &std::collections::BTreeMap<String, PotentialAssessment>, incoming: &std::collections::BTreeMap<String, PotentialAssessment>) -> std::collections::BTreeMap<String, PotentialAssessment>`
  - `pub fn merge_manual_relationships(local_rels: &[Relationship], incoming_rels: &[Relationship], local_tombstones: &[RelationshipTombstone], incoming_tombstones: &[RelationshipTombstone]) -> (Vec<Relationship>, Vec<RelationshipTombstone>)`
  - `pub fn merge_user_annotations(local: &UserAnnotations, incoming: &UserAnnotations) -> UserAnnotations`

- [ ] **Step 1: Write the failing test**

Create `crates/nexo-domain/tests/annotations_tests.rs` covering:
- Logical key directed structure `source::target::type`
- LWW for assessments: incoming wins strictly newer, tie favors local
- Invalid timestamp handling (defaults to 0, local wins)
- Tombstone prevention of resurrection on import
- Re-creation allowed strictly after tombstone `deletedAt`
- Deduplication by stable relation ID
- Full merge of `UserAnnotations`

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p nexo-domain --test annotations_tests`
Expected: FAIL

- [ ] **Step 3: Write minimal implementation**

Implement `crates/nexo-domain/src/annotations.rs` porting pure conflict resolution rules from `src/shared/annotations.ts`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p nexo-domain --test annotations_tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/nexo-domain/src/annotations.rs crates/nexo-domain/tests/annotations_tests.rs crates/nexo-domain/src/lib.rs
git commit -m "feat(domain): port annotations, potential assessments, LWW and tombstones"
```

---

### Task 5: Domain Validation Engine with Typed Path Errors

**Files:**
- Create: `crates/nexo-domain/src/validation.rs`
- Modify: `crates/nexo-domain/src/error.rs`
- Modify: `crates/nexo-domain/src/lib.rs`
- Test: `crates/nexo-domain/tests/validation_tests.rs`

**Interfaces:**
- Consumes: `model::Snapshot`, `annotations::UserAnnotations`, `error::NexoError`
- Produces:
  - `pub struct ValidationErrorDetail { pub path: String, pub message: String }`
  - `pub fn validate_snapshot(value: &serde_json::Value) -> Result<Snapshot, Vec<ValidationErrorDetail>>`
  - `pub fn validate_annotations(value: &serde_json::Value) -> Result<UserAnnotations, Vec<ValidationErrorDetail>>`
  - Structural and semantic checks: mandatory metadata, version "1.0", valid ISO dates, indicator scores 0..=100, valid enums, duplicate ID checks.

- [ ] **Step 1: Write the failing test**

Create `crates/nexo-domain/tests/validation_tests.rs` covering:
- Valid full snapshot (e.g. `demo-snapshot.json`)
- Missing metadata fields produces path errors (e.g. `metadata.authenticatedUser: campo obrigatório ausente`)
- Unsupported version (e.g. version = "0.9" or "2.0") rejected explicitly
- Malformed dates rejected
- Out-of-bounds indicator scores rejected
- Normalization of missing collections in annotations
- Validation does not mutate input data

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p nexo-domain --test validation_tests`
Expected: FAIL

- [ ] **Step 3: Write minimal implementation**

Implement `crates/nexo-domain/src/validation.rs` and enrich `NexoError` with detailed field paths.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p nexo-domain --test validation_tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/nexo-domain/src/validation.rs crates/nexo-domain/src/error.rs crates/nexo-domain/tests/validation_tests.rs crates/nexo-domain/src/lib.rs
git commit -m "feat(domain): implement complete validation engine with path error diagnostics"
```

---

### Task 6: Downstream Crates Adaptation (`nexo-storage`, `nexo-github`, `nexo-desktop`)

**Files:**
- Modify: `crates/nexo-domain/src/state.rs` (preserve `NexoState` and `ViewMode`)
- Modify: `crates/nexo-storage/src/lib.rs`
- Modify: `crates/nexo-github/src/lib.rs`
- Modify: `crates/nexo-desktop/src/app.rs`
- Test: `cargo test --workspace --locked`

**Interfaces:**
- Consumes: Updated `nexo-domain` types
- Produces: Full workspace compilation and passing tests for all crates without regression.

- [ ] **Step 1: Write the failing test**

Add test to `crates/nexo-storage/src/lib.rs` verifying that `save_repository`, `get_repository`, and `list_repositories` operate smoothly with the 1.0 `Repository` model (with `id: u64` or string conversion helper).

- [ ] **Step 2: Run test to verify compile/runtime issues**

Run: `cargo check --workspace`
Observe any signature discrepancies with `Repository` constructor or fields.

- [ ] **Step 3: Adapt consumers cleanly**

Update `nexo-storage`, `nexo-github`, and `nexo-desktop` to use the new domain types while keeping `NexoState` and `ViewMode` intact for Rune desktop UI.

- [ ] **Step 4: Run workspace tests to verify all pass**

Run: `cargo test --workspace --locked`
Expected: All tests pass across all crates (including desktop lifecycle tests).

- [ ] **Step 5: Commit**

```bash
git add crates/nexo-domain/ crates/nexo-storage/ crates/nexo-github/ crates/nexo-desktop/
git commit -m "refactor(workspace): integrate updated domain contracts across all crates"
```

---

### Task 7: Cross-Language Equivalence Test Matrix, Fixtures & Executable Comparator

**Files:**
- Create: `fixtures/domain/` (synthetic test matrices: valid, empty, archived, uninspected, truncated, large id, forks, tombstones)
- Create: `crates/nexo-domain/tests/equivalence_runner.rs`
- Create: `scripts/run-parity-check.ts`
- Modify: `package.json` (add npm parity script)

**Interfaces:**
- Consumes: TypeScript reference scripts + Rust domain library
- Produces:
  - Automated comparison verifying that calculating indicators and relations on the same fixture produces identical scores, statuses, counts, and structures.
  - Zero tolerance for silent normalization of discrepancies.

- [ ] **Step 1: Write the failing test**

Create `crates/nexo-domain/tests/equivalence_runner.rs` loading `demo-snapshot.json` and synthetic edge cases, evaluating indicators and relations in Rust and asserting exact match against pre-computed or TS-computed JSON outputs.

- [ ] **Step 2: Run test to verify failure on any difference**

Run: `cargo test -p nexo-domain --test equivalence_runner`
Expected: Highlights any discrepancy between TS output and Rust calculations.

- [ ] **Step 3: Align and refine implementation to reach 100% parity**

Resolve any subtle differences in rounding, limitations text, or empty array defaults until parity is verified for all 23 scenarios.

- [ ] **Step 4: Run equivalence suite to verify it passes**

Run: `cargo test -p nexo-domain --test equivalence_runner`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add fixtures/ crates/nexo-domain/tests/equivalence_runner.rs scripts/run-parity-check.ts package.json
git commit -m "test(domain): establish TS-Rust cross-language equivalence test matrix"
```

---

### Task 8: CI Automation & Technical Architecture Documentation

**Files:**
- Modify: `.github/workflows/ci.yml`
- Create: `docs/architecture/0003-domain-contracts-and-parity.md`
- Modify: `ROADMAP.md` (record progress on M1 Issue #3)

**Interfaces:**
- Consumes: Complete test suite and domain specifications
- Produces:
  - Comprehensive ADR/documentation covering: TypeScript to Rust inventory, field mappings, JSON 1.0 compatibility, indicator formulas and weights, unknown evidence handling, relation provenance, fork handling, annotations LWW, and equivalence matrix results.
  - GitHub Actions workflow executing Rust + TypeScript checks and equivalence verification.

- [ ] **Step 1: Write documentation and workflow updates**

Draft `docs/architecture/0003-domain-contracts-and-parity.md` documenting all items from section 11 of Issue #3. Update `.github/workflows/ci.yml` with the equivalence check.

- [ ] **Step 2: Verify all checks locally**

Run:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --locked
npm run lint
npm run typecheck
npm test
npm run build:all
```
Expected: All commands exit with code 0.

- [ ] **Step 3: Commit**

```bash
git add docs/architecture/0003-domain-contracts-and-parity.md .github/workflows/ci.yml ROADMAP.md
git commit -m "docs(domain): document architecture, contracts inventory and parity matrix for issue #3"
```

---

## Self-Review Checklist

1. **Spec coverage:**
   - Section 3 (Models & Serde): Covered in Task 1.
   - Section 3.2 (Repository identity & large IDs): Covered in Task 1.
   - Section 3.3 (Version compatibility 1.0): Covered in Tasks 1 & 5.
   - Section 4 (Validation & path errors): Covered in Task 5.
   - Section 5 (Indicators & weights): Covered in Task 2.
   - Section 5.2 (Temporal determinism): Covered in Task 2.
   - Section 5.3 (Insufficient evidence & unknown): Covered in Task 2.
   - Section 6 (Relations & provenance): Covered in Task 3.
   - Section 6.4 (Forks without fabricated targets): Covered in Task 3.
   - Section 7 (Annotations, LWW & tombstones): Covered in Task 4.
   - Section 8 (Fixtures & TS/Rust equivalence): Covered in Task 7.
   - Section 9 (Workspace integration): Covered in Task 6.
   - Section 10 (Reproducibility & CI): Covered in Task 8.
   - Section 11 (Documentation): Covered in Task 8.

2. **Placeholder scan:** No "TODO", "TBD", or vague placeholders.
3. **Type consistency:** Field names, types (`u64`, `Affiliation`, `IndicatorStatus`, `RelationType`, `RelationshipProvenance`, `PotentialAssessment`), and module structure are consistent across tasks.
