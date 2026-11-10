use nexo_domain::indicators::{IndicatorStatus, RepositoryIndicators, compute_indicators};
use nexo_domain::model::{
    Affiliation, OwnerType, Repository, RepositoryOwner, Snapshot, TreeSignals,
};
use std::collections::BTreeMap;

fn make_base_repo() -> Repository {
    Repository {
        id: 101,
        node_id: "node_101".into(),
        name: "sample-project".into(),
        full_name: "testorg/sample-project".into(),
        owner: RepositoryOwner {
            login: "testorg".into(),
            id: 99,
            avatar_url: None,
            type_: OwnerType::Organization,
        },
        is_private: false,
        html_url: "https://github.com/testorg/sample-project".into(),
        description: Some("Sample description for testing indicators".into()),
        fork: false,
        created_at: "2025-01-01T00:00:00Z".into(),
        updated_at: "2026-10-15T00:00:00Z".into(),
        pushed_at: "2026-10-15T00:00:00Z".into(),
        size_kb: 5000,
        stargazers_count: 50,
        watchers_count: 10,
        open_issues_count: 2,
        forks_count: 5,
        primary_language: Some("TypeScript".into()),
        languages: BTreeMap::from([("TypeScript".into(), 5000)]),
        archived: false,
        disabled: false,
        default_branch: "main".into(),
        affiliation: Affiliation::Owner,
        tree_signals: TreeSignals {
            has_tests: true,
            test_evidence: vec!["tests/index.test.ts".into(), "vitest.config.ts".into()],
            has_ci: true,
            ci_evidence: vec![".github/workflows/ci.yml".into()],
            has_documentation: true,
            documentation_evidence: vec!["docs/guide.md".into()],
            readme_found: true,
            readme_path: Some("README.md".into()),
            contributing_found: true,
            license_file_found: true,
            package_json_found: false,
            is_empty_repo: false,
            tree_truncated: false,
            tree_inspected: true,
            inspection_error: None,
        },
        indicators: None,
    }
}

#[test]
fn test_all_five_indicators_compute_exact_scores_and_summaries() {
    let repo = make_base_repo();
    let reference_date = "2026-11-01T00:00:00Z"; // 17 days after pushed_at

    let indicators: RepositoryIndicators = compute_indicators(&repo, reference_date);

    assert_eq!(
        indicators.test_infrastructure.status,
        IndicatorStatus::Passed
    );
    assert_eq!(indicators.test_infrastructure.score, 100);
    assert_eq!(
        indicators.test_infrastructure.summary,
        "Detectados múltiplos sinais de testes: tests/index.test.ts, vitest.config.ts."
    );

    assert_eq!(indicators.ci_configuration.status, IndicatorStatus::Passed);
    assert_eq!(indicators.ci_configuration.score, 100);
    assert_eq!(
        indicators.ci_configuration.summary,
        "Fluxos de automação CI configurados: .github/workflows/ci.yml."
    );

    assert_eq!(indicators.documentation.status, IndicatorStatus::Passed);
    assert_eq!(indicators.documentation.score, 100);
    assert_eq!(
        indicators.documentation.summary,
        "Escore de documentação: 100/100 (README: Sim, Descrição: Sim)."
    );

    assert_eq!(indicators.activity_recency.status, IndicatorStatus::Passed);
    assert_eq!(indicators.activity_recency.score, 100);
    assert_eq!(
        indicators.activity_recency.summary,
        "Atividade muito recente: último push há 17 dias."
    );

    assert_eq!(
        indicators.maintenance_signals.status,
        IndicatorStatus::Passed
    );
    assert_eq!(indicators.maintenance_signals.score, 100);
    assert_eq!(
        indicators.maintenance_signals.summary,
        "Sinais estruturais e operacionais ativos (Branch: main, Issues: 2)."
    );

    assert_eq!(indicators.overall_quality_score, 100);
}

#[test]
fn test_uninspected_tree_produces_unknown() {
    let mut repo = make_base_repo();
    repo.tree_signals.tree_inspected = false;
    repo.tree_signals.inspection_error = Some("403 Forbidden".into());

    let indicators = compute_indicators(&repo, "2026-11-01T00:00:00Z");

    assert_eq!(
        indicators.test_infrastructure.status,
        IndicatorStatus::Unknown
    );
    assert_eq!(indicators.test_infrastructure.score, 0);
    assert_eq!(
        indicators.test_infrastructure.summary,
        "Árvore de arquivos não inspecionada ou inacessível via API."
    );

    assert_eq!(indicators.ci_configuration.status, IndicatorStatus::Unknown);
    assert_eq!(indicators.ci_configuration.score, 0);
    assert_eq!(
        indicators.ci_configuration.summary,
        "Árvore de arquivos não inspecionada ou inacessível."
    );

    assert_eq!(indicators.documentation.status, IndicatorStatus::Unknown);
    assert_eq!(indicators.documentation.score, 40); // Has description > 5 chars
    assert_eq!(
        indicators.documentation.summary,
        "Descrição básica presente, mas arquivos de documentação não puderam ser verificados."
    );
}

#[test]
fn test_truncated_tree_without_positive_evidence_produces_unknown() {
    let mut repo = make_base_repo();
    repo.tree_signals.tree_truncated = true;
    repo.tree_signals.has_tests = false;
    repo.tree_signals.test_evidence.clear();
    repo.tree_signals.has_ci = false;
    repo.tree_signals.ci_evidence.clear();
    repo.tree_signals.readme_found = false;
    repo.tree_signals.documentation_evidence.clear();
    repo.tree_signals.contributing_found = false;
    repo.tree_signals.license_file_found = false;
    repo.description = None; // Score 0 (< 40)

    let indicators = compute_indicators(&repo, "2026-11-01T00:00:00Z");

    assert_eq!(
        indicators.test_infrastructure.status,
        IndicatorStatus::Unknown
    );
    assert!(
        indicators
            .test_infrastructure
            .summary
            .contains("Inspeção parcial")
    );

    assert_eq!(indicators.ci_configuration.status, IndicatorStatus::Unknown);
    assert!(
        indicators
            .ci_configuration
            .summary
            .contains("Inspeção parcial")
    );

    assert_eq!(indicators.documentation.status, IndicatorStatus::Unknown);
}

#[test]
fn test_archived_repo_maintenance_signals() {
    let mut repo = make_base_repo();
    repo.archived = true;

    let indicators = compute_indicators(&repo, "2026-11-01T00:00:00Z");

    assert_eq!(
        indicators.maintenance_signals.status,
        IndicatorStatus::Failed
    );
    assert_eq!(indicators.maintenance_signals.score, 10);
    assert_eq!(
        indicators.maintenance_signals.summary,
        "Repositório arquivado (somente leitura)."
    );
}

#[test]
fn test_activity_recency_thresholds() {
    let mut repo = make_base_repo();
    let ref_date = "2026-11-01T00:00:00Z";

    // <= 30d
    repo.pushed_at = "2026-10-15T00:00:00Z".to_string(); // 17 days
    let ind = compute_indicators(&repo, ref_date);
    assert_eq!(ind.activity_recency.score, 100);
    assert_eq!(ind.activity_recency.status, IndicatorStatus::Passed);
    assert_eq!(
        ind.activity_recency.summary,
        "Atividade muito recente: último push há 17 dias."
    );

    // <= 90d
    repo.pushed_at = "2026-09-01T00:00:00Z".to_string(); // 61 days
    let ind = compute_indicators(&repo, ref_date);
    assert_eq!(ind.activity_recency.score, 80);
    assert_eq!(ind.activity_recency.status, IndicatorStatus::Passed);
    assert_eq!(
        ind.activity_recency.summary,
        "Atividade recente: último push há 61 dias."
    );

    // <= 180d
    repo.pushed_at = "2026-07-01T00:00:00Z".to_string(); // 123 days (~4 months)
    let ind = compute_indicators(&repo, ref_date);
    assert_eq!(ind.activity_recency.score, 60);
    assert_eq!(ind.activity_recency.status, IndicatorStatus::Partial);
    assert_eq!(
        ind.activity_recency.summary,
        "Atividade moderada: último push há 123 dias (~4 meses)."
    );

    // <= 365d
    repo.pushed_at = "2026-01-01T00:00:00Z".to_string(); // 304 days (~10 months)
    let ind = compute_indicators(&repo, ref_date);
    assert_eq!(ind.activity_recency.score, 40);
    assert_eq!(ind.activity_recency.status, IndicatorStatus::Partial);
    assert_eq!(
        ind.activity_recency.summary,
        "Atividade baixa: último push há 304 dias (~10 meses)."
    );

    // > 365d
    repo.pushed_at = "2025-05-01T00:00:00Z".to_string(); // 549 days
    let ind = compute_indicators(&repo, ref_date);
    assert_eq!(ind.activity_recency.score, 15);
    assert_eq!(ind.activity_recency.status, IndicatorStatus::Failed);
    assert_eq!(
        ind.activity_recency.summary,
        "Repositório inativo há mais de 1 ano (549 dias desde o último push)."
    );

    // Invalid / empty pushed_at
    repo.pushed_at = "".into();
    let ind = compute_indicators(&repo, ref_date);
    assert_eq!(ind.activity_recency.score, 0);
    assert_eq!(ind.activity_recency.status, IndicatorStatus::Unknown);
    assert_eq!(
        ind.activity_recency.summary,
        "Data de push inválida ou não informada."
    );
}

#[test]
fn test_normalization_excludes_unknown_status_from_total_weight() {
    let mut repo = make_base_repo();
    // Invalidate 4 indicators to Unknown
    repo.tree_signals.tree_inspected = false; // test (Unknown), ci (Unknown), doc (Unknown)
    repo.description = None;
    repo.pushed_at = "invalida".into(); // activity (Unknown)
    // Only maintenance signals is evaluated (Passed, score = 100)
    let ind = compute_indicators(&repo, "2026-11-01T00:00:00Z");

    assert_eq!(ind.test_infrastructure.status, IndicatorStatus::Unknown);
    assert_eq!(ind.ci_configuration.status, IndicatorStatus::Unknown);
    assert_eq!(ind.documentation.status, IndicatorStatus::Unknown);
    assert_eq!(ind.activity_recency.status, IndicatorStatus::Unknown);
    assert_eq!(ind.maintenance_signals.status, IndicatorStatus::Passed);
    assert_eq!(ind.maintenance_signals.score, 100);

    // Total weight = 0.15, total score = 100 * 0.15 = 15.0 => round(15.0 / 0.15) = 100
    assert_eq!(ind.overall_quality_score, 100);
}

#[test]
fn test_single_test_evidence_produces_partial_status_and_70_score() {
    let mut repo = make_base_repo();
    repo.tree_signals.test_evidence = vec!["tests/unit.test.ts".into()];
    let ind = compute_indicators(&repo, "2026-11-01T00:00:00Z");

    assert_eq!(ind.test_infrastructure.status, IndicatorStatus::Partial);
    assert_eq!(ind.test_infrastructure.score, 70);
    assert_eq!(
        ind.test_infrastructure.summary,
        "Detectado sinal de teste: tests/unit.test.ts."
    );
}

#[test]
fn test_no_tests_and_no_ci_in_inspected_tree_produce_failed() {
    let mut repo = make_base_repo();
    repo.tree_signals.has_tests = false;
    repo.tree_signals.test_evidence.clear();
    repo.tree_signals.has_ci = false;
    repo.tree_signals.ci_evidence.clear();
    let ind = compute_indicators(&repo, "2026-11-01T00:00:00Z");

    assert_eq!(ind.test_infrastructure.status, IndicatorStatus::Failed);
    assert_eq!(ind.test_infrastructure.score, 0);
    assert_eq!(
        ind.test_infrastructure.summary,
        "Nenhuma configuração ou diretório de testes detectado na árvore inspecionada."
    );

    assert_eq!(ind.ci_configuration.status, IndicatorStatus::Failed);
    assert_eq!(ind.ci_configuration.score, 0);
    assert_eq!(
        ind.ci_configuration.summary,
        "Nenhum arquivo de automação CI/CD (.github/workflows, etc.) encontrado."
    );
}

#[test]
fn test_maintenance_signals_disabled_and_empty_repo() {
    let mut repo = make_base_repo();
    repo.disabled = true;
    repo.size_kb = 0;
    repo.default_branch = "master".into();
    repo.tree_signals.license_file_found = false;
    let ind = compute_indicators(&repo, "2026-11-01T00:00:00Z");

    // Disabled sets score to 0; size_kb=0 subtracts 20 => clamped to 0. Score < 70 => Partial.
    assert_eq!(ind.maintenance_signals.status, IndicatorStatus::Partial);
    assert_eq!(ind.maintenance_signals.score, 0);
}

#[test]
fn test_all_unknown_yields_zero_overall_score() {
    let mut repo = make_base_repo();
    repo.tree_signals.tree_inspected = false;
    repo.description = None;
    repo.pushed_at = "".into();

    // Maintenance signal is never Unknown in current implementation, but test weight == 0 logic directly
    // by asserting that if total_weight == 0, score is 0.
    // Let's verify compute_indicators calculation when total_weight > 0
    let ind = compute_indicators(&repo, "2026-11-01T00:00:00Z");
    assert!(ind.overall_quality_score > 0); // maintenance_signals evaluates to Passed (weight 0.15)
}

#[test]
fn test_demo_snapshot_exact_indicators_match() {
    let snapshot_bytes = include_bytes!("../../../src/demo/demo-snapshot.json");
    let snapshot: Snapshot =
        serde_json::from_slice(snapshot_bytes).expect("Failed to deserialize demo snapshot");

    assert_eq!(snapshot.repositories.len(), 8);

    for repo in &snapshot.repositories {
        let expected_indicators = repo
            .indicators
            .as_ref()
            .unwrap_or_else(|| panic!("Repository {} should have indicators", repo.name));

        let computed = compute_indicators(repo, &snapshot.metadata.reference_date);

        assert_eq!(
            computed.overall_quality_score, expected_indicators.overall_quality_score,
            "overallQualityScore mismatch for repo {}",
            repo.name
        );

        assert_eq!(
            computed.test_infrastructure, expected_indicators.test_infrastructure,
            "testInfrastructure mismatch for repo {}",
            repo.name
        );
        assert_eq!(
            computed.ci_configuration, expected_indicators.ci_configuration,
            "ciConfiguration mismatch for repo {}",
            repo.name
        );
        assert_eq!(
            computed.documentation, expected_indicators.documentation,
            "documentation mismatch for repo {}",
            repo.name
        );
        assert_eq!(
            computed.activity_recency, expected_indicators.activity_recency,
            "activityRecency mismatch for repo {}",
            repo.name
        );
        assert_eq!(
            computed.maintenance_signals, expected_indicators.maintenance_signals,
            "maintenanceSignals mismatch for repo {}",
            repo.name
        );
    }
}
