use nexo_domain::annotations::{UserAnnotations, merge_user_annotations};
use nexo_domain::indicators::compute_indicators;
use nexo_domain::model::Snapshot;
use nexo_domain::relations::extract_explicit_relationships;
use nexo_domain::validation::validate_snapshot;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn get_repo_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("parent of crates")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

const ALL_SCENARIOS: &[&str] = &[
    "valid_complete_snapshot",
    "valid_with_defaults_and_optionals_missing",
    "empty_snapshot",
    "archived_repository",
    "disabled_repository",
    "uninspected_tree",
    "truncated_tree",
    "inspection_error",
    "different_affiliations",
    "large_ids_and_renamed_entities",
    "precomputed_indicators",
    "unknown_and_partial_indicators",
    "belongs_to_relations",
    "explicit_readme_references",
    "explicit_manifest_dependencies",
    "manual_and_automatic_relations",
    "fork_with_proven_origin",
    "fork_without_proven_origin",
    "annotations_and_tombstones",
    "unknown_version",
    "missing_mandatory_fields",
    "invalid_types_and_malformed_dates",
    "inconsistent_identities",
];

#[test]
fn test_all_23_scenarios_exist() {
    let root = get_repo_root();
    let fixtures_dir = root.join("fixtures").join("domain");
    let expected_dir = fixtures_dir.join("expected");

    for scenario in ALL_SCENARIOS {
        let fixture_path = fixtures_dir.join(format!("{scenario}.json"));
        let expected_path = expected_dir.join(format!("{scenario}.expected.json"));
        assert!(
            fixture_path.exists(),
            "Missing fixture file: {}",
            fixture_path.display()
        );
        assert!(
            expected_path.exists(),
            "Missing expected file: {}",
            expected_path.display()
        );
    }
}

#[test]
fn test_cross_language_domain_equivalence_matrix() {
    let root = get_repo_root();
    let fixtures_dir = root.join("fixtures").join("domain");
    let expected_dir = fixtures_dir.join("expected");

    for scenario in ALL_SCENARIOS {
        let fixture_path = fixtures_dir.join(format!("{scenario}.json"));
        let expected_path = expected_dir.join(format!("{scenario}.expected.json"));

        let fixture_str = std::fs::read_to_string(&fixture_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", fixture_path.display(), e));
        let expected_str = std::fs::read_to_string(&expected_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", expected_path.display(), e));

        let fixture_val: Value = serde_json::from_str(&fixture_str)
            .unwrap_or_else(|e| panic!("Invalid JSON in {}: {}", fixture_path.display(), e));
        let expected_val: Value = serde_json::from_str(&expected_str)
            .unwrap_or_else(|e| panic!("Invalid JSON in {}: {}", expected_path.display(), e));

        let expected_valid = expected_val["valid"]
            .as_bool()
            .expect("expected.valid must be boolean");

        // 1. Validation check
        let validation_res = validate_snapshot(&fixture_val);
        assert_eq!(
            validation_res.is_ok(),
            expected_valid,
            "Validation parity mismatch in scenario '{}'. Expected valid: {}, got: {:?}",
            scenario,
            expected_valid,
            validation_res.err()
        );

        if !expected_valid {
            // Negative scenario verified, proceed to next
            continue;
        }

        let snapshot: Snapshot = validation_res.expect("Snapshot should be valid");
        let ref_date = &snapshot.metadata.reference_date;

        // 2. Indicators Equivalence Check
        let expected_indicators_arr = expected_val["indicators"]
            .as_array()
            .expect("expected.indicators must be array");
        assert_eq!(
            snapshot.repositories.len(),
            expected_indicators_arr.len(),
            "Repository count mismatch in scenario '{}'",
            scenario
        );

        for exp_repo_entry in expected_indicators_arr {
            let repo_id = exp_repo_entry["repositoryId"]
                .as_u64()
                .expect("repositoryId u64");
            let repo = snapshot
                .repositories
                .iter()
                .find(|r| r.id == repo_id)
                .unwrap_or_else(|| {
                    panic!(
                        "Scenario '{}': repository with id {} not found in snapshot",
                        scenario, repo_id
                    )
                });

            let rust_indicators = compute_indicators(repo, ref_date);
            let expected_score = exp_repo_entry["overallQualityScore"]
                .as_u64()
                .expect("overallQualityScore u64") as u32;

            assert_eq!(
                rust_indicators.overall_quality_score, expected_score,
                "Overall quality score mismatch for repo '{}' in scenario '{}'",
                repo.name, scenario
            );

            // Compare each of the 5 indicators
            let exp_ind_obj = &exp_repo_entry["indicators"];

            let ind_pairs = [
                (
                    &rust_indicators.test_infrastructure,
                    &exp_ind_obj["testInfrastructure"],
                ),
                (
                    &rust_indicators.ci_configuration,
                    &exp_ind_obj["ciConfiguration"],
                ),
                (
                    &rust_indicators.documentation,
                    &exp_ind_obj["documentation"],
                ),
                (
                    &rust_indicators.activity_recency,
                    &exp_ind_obj["activityRecency"],
                ),
                (
                    &rust_indicators.maintenance_signals,
                    &exp_ind_obj["maintenanceSignals"],
                ),
            ];

            for (rust_ind, exp_ind) in ind_pairs {
                let exp_score = exp_ind["score"].as_u64().expect("score u64") as u32;
                let exp_status = exp_ind["status"].as_str().expect("status string");
                let exp_summary = exp_ind["summary"].as_str().expect("summary string");
                let exp_calc = exp_ind["calculation"].as_str().expect("calculation string");

                assert_eq!(
                    rust_ind.score, exp_score,
                    "Scenario '{}', repo '{}', indicator '{}' score mismatch",
                    scenario, repo.name, rust_ind.id
                );
                let rust_status_str = match rust_ind.status {
                    nexo_domain::indicators::IndicatorStatus::Passed => "passed",
                    nexo_domain::indicators::IndicatorStatus::Failed => "failed",
                    nexo_domain::indicators::IndicatorStatus::Unknown => "unknown",
                    nexo_domain::indicators::IndicatorStatus::Partial => "partial",
                };
                assert_eq!(
                    rust_status_str, exp_status,
                    "Scenario '{}', repo '{}', indicator '{}' status mismatch",
                    scenario, repo.name, rust_ind.id
                );
                assert_eq!(
                    rust_ind.summary, exp_summary,
                    "Scenario '{}', repo '{}', indicator '{}' summary mismatch",
                    scenario, repo.name, rust_ind.id
                );
                assert_eq!(
                    rust_ind.calculation, exp_calc,
                    "Scenario '{}', repo '{}', indicator '{}' calculation mismatch",
                    scenario, repo.name, rust_ind.id
                );

                // Limitations
                let exp_limitations: Vec<String> = exp_ind["limitations"]
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|v| v.as_str().unwrap().to_string())
                    .collect();
                assert_eq!(
                    rust_ind.limitations, exp_limitations,
                    "Scenario '{}', repo '{}', indicator '{}' limitations mismatch",
                    scenario, repo.name, rust_ind.id
                );
            }
        }

        // 3. Relationships Extraction Equivalence Check
        let mut readme_map = HashMap::new();
        if let Some(readmes) = fixture_val
            .get("readmeContents")
            .and_then(|v| v.as_object())
        {
            for (k, v) in readmes {
                if let (Ok(id), Some(s)) = (k.parse::<u64>(), v.as_str()) {
                    readme_map.insert(id, s.to_string());
                }
            }
        }

        let mut manifest_map = HashMap::new();
        if let Some(manifests) = fixture_val
            .get("manifestContents")
            .and_then(|v| v.as_object())
        {
            for (k, v) in manifests {
                if let (Ok(id), Some(s)) = (k.parse::<u64>(), v.as_str()) {
                    manifest_map.insert(id, s.to_string());
                }
            }
        }

        let rust_extracted = extract_explicit_relationships(
            &snapshot.repositories,
            &snapshot.organizations,
            &readme_map,
            &manifest_map,
        );

        let exp_extracted = expected_val["extractedRelationships"]
            .as_array()
            .expect("extractedRelationships array");

        assert_eq!(
            rust_extracted.len(),
            exp_extracted.len(),
            "Scenario '{}': extracted relationships count mismatch (rust: {}, exp: {})",
            scenario,
            rust_extracted.len(),
            exp_extracted.len()
        );

        for (i, exp_rel) in exp_extracted.iter().enumerate() {
            let r = &rust_extracted[i];
            assert_eq!(r.id, exp_rel["id"].as_str().unwrap());
            assert_eq!(r.source, exp_rel["source"].as_str().unwrap());
            assert_eq!(r.target, exp_rel["target"].as_str().unwrap());
            assert_eq!(
                r.source_type.as_str(),
                exp_rel["sourceType"].as_str().unwrap()
            );
            assert_eq!(
                r.target_type.as_str(),
                exp_rel["targetType"].as_str().unwrap()
            );
            assert_eq!(r.type_.as_str(), exp_rel["type"].as_str().unwrap());
            assert_eq!(
                r.provenance.as_str(),
                exp_rel["provenance"].as_str().unwrap()
            );
            assert_eq!(r.verified, exp_rel["verified"].as_bool().unwrap());
        }

        // 4. Annotations Merge Equivalence Check
        if let Some(incoming_val) = fixture_val.get("incomingAnnotations") {
            let incoming_annotations: UserAnnotations =
                serde_json::from_value(incoming_val.clone())
                    .expect("incomingAnnotations should deserialize");

            let default_local = UserAnnotations {
                schema_version: "1.0".to_string(),
                exported_at: ref_date.clone(),
                authenticated_user: Some(snapshot.metadata.authenticated_user.clone()),
                manual_relationships: vec![],
                deleted_relationships: vec![],
                potential_assessments: std::collections::BTreeMap::new(),
            };

            let local_annotations = snapshot.user_annotations.as_ref().unwrap_or(&default_local);
            let rust_merged =
                merge_user_annotations(local_annotations, &incoming_annotations, Some(ref_date));

            let exp_merged = &expected_val["mergedAnnotations"];
            assert_eq!(
                rust_merged.schema_version,
                exp_merged["schemaVersion"].as_str().unwrap()
            );
            assert_eq!(
                rust_merged.exported_at,
                exp_merged["exportedAt"].as_str().unwrap()
            );

            // Manual relationships length and details
            let exp_mr = exp_merged["manualRelationships"].as_array().unwrap();
            assert_eq!(rust_merged.manual_relationships.len(), exp_mr.len());
            for (i, exp_rel) in exp_mr.iter().enumerate() {
                let r = &rust_merged.manual_relationships[i];
                assert_eq!(r.id, exp_rel["id"].as_str().unwrap());
                assert_eq!(r.source, exp_rel["source"].as_str().unwrap());
                assert_eq!(r.target, exp_rel["target"].as_str().unwrap());
                assert_eq!(r.type_.as_str(), exp_rel["type"].as_str().unwrap());
            }

            // Deleted relationships length and details
            let exp_dr = exp_merged["deletedRelationships"].as_array().unwrap();
            assert_eq!(rust_merged.deleted_relationships.len(), exp_dr.len());
            for (i, exp_tomb) in exp_dr.iter().enumerate() {
                let t = &rust_merged.deleted_relationships[i];
                assert_eq!(t.id, exp_tomb["id"].as_str().unwrap());
                assert_eq!(t.source, exp_tomb["source"].as_str().unwrap());
                assert_eq!(t.target, exp_tomb["target"].as_str().unwrap());
                assert_eq!(t.type_.as_str(), exp_tomb["type"].as_str().unwrap());
                assert_eq!(t.deleted_at, exp_tomb["deletedAt"].as_str().unwrap());
            }

            // Potential assessments
            let exp_pa = exp_merged["potentialAssessments"].as_object().unwrap();
            assert_eq!(rust_merged.potential_assessments.len(), exp_pa.len());
            for (k, exp_assess) in exp_pa {
                let rust_assess = rust_merged
                    .potential_assessments
                    .get(k)
                    .unwrap_or_else(|| panic!("Missing assessment for repo id {}", k));
                assert_eq!(
                    rust_assess.repository_id,
                    exp_assess["repositoryId"].as_u64().unwrap()
                );
                assert_eq!(
                    rust_assess.updated_at,
                    exp_assess["updatedAt"].as_str().unwrap()
                );
            }
        }
    }
}

trait AsStrExt {
    fn as_str(&self) -> &'static str;
}

impl AsStrExt for nexo_domain::relations::NodeType {
    fn as_str(&self) -> &'static str {
        match self {
            nexo_domain::relations::NodeType::Repository => "repository",
            nexo_domain::relations::NodeType::Organization => "organization",
        }
    }
}

impl AsStrExt for nexo_domain::relations::RelationshipProvenance {
    fn as_str(&self) -> &'static str {
        match self {
            nexo_domain::relations::RelationshipProvenance::Automatic => "automatic",
            nexo_domain::relations::RelationshipProvenance::Manual => "manual",
        }
    }
}

#[test]
fn test_archived_and_disabled_indicators_invariants() {
    let root = get_repo_root();
    let fixtures_dir = root.join("fixtures").join("domain");

    // Archived repo
    let raw = std::fs::read_to_string(fixtures_dir.join("archived_repository.json")).unwrap();
    let val: Value = serde_json::from_str(&raw).unwrap();
    let snapshot = validate_snapshot(&val).expect("Valid archived snapshot");
    let ind = compute_indicators(&snapshot.repositories[0], &snapshot.metadata.reference_date);
    assert_eq!(
        ind.maintenance_signals.status,
        nexo_domain::indicators::IndicatorStatus::Failed
    );
    assert_eq!(ind.maintenance_signals.score, 10);
    assert_eq!(
        ind.maintenance_signals.summary,
        "Repositório arquivado (somente leitura)."
    );

    // Disabled repo
    let raw = std::fs::read_to_string(fixtures_dir.join("disabled_repository.json")).unwrap();
    let val: Value = serde_json::from_str(&raw).unwrap();
    let snapshot = validate_snapshot(&val).expect("Valid disabled snapshot");
    let ind = compute_indicators(&snapshot.repositories[0], &snapshot.metadata.reference_date);
    assert_eq!(
        ind.maintenance_signals.status,
        nexo_domain::indicators::IndicatorStatus::Partial
    );
    assert_eq!(ind.maintenance_signals.score, 10);
}

#[test]
fn test_uninspected_and_truncated_trees_unknown_signals() {
    let root = get_repo_root();
    let fixtures_dir = root.join("fixtures").join("domain");

    // Uninspected
    let raw = std::fs::read_to_string(fixtures_dir.join("uninspected_tree.json")).unwrap();
    let val: Value = serde_json::from_str(&raw).unwrap();
    let snapshot = validate_snapshot(&val).expect("Valid uninspected snapshot");
    let ind = compute_indicators(&snapshot.repositories[0], &snapshot.metadata.reference_date);
    assert_eq!(
        ind.test_infrastructure.status,
        nexo_domain::indicators::IndicatorStatus::Unknown
    );
    assert_eq!(
        ind.ci_configuration.status,
        nexo_domain::indicators::IndicatorStatus::Unknown
    );
    assert_eq!(
        ind.documentation.status,
        nexo_domain::indicators::IndicatorStatus::Unknown
    );

    // Truncated tree without tests/CI
    let raw = std::fs::read_to_string(fixtures_dir.join("truncated_tree.json")).unwrap();
    let val: Value = serde_json::from_str(&raw).unwrap();
    let snapshot = validate_snapshot(&val).expect("Valid truncated snapshot");
    let ind = compute_indicators(&snapshot.repositories[0], &snapshot.metadata.reference_date);
    assert_eq!(
        ind.test_infrastructure.status,
        nexo_domain::indicators::IndicatorStatus::Unknown
    );
    assert_eq!(
        ind.ci_configuration.status,
        nexo_domain::indicators::IndicatorStatus::Unknown
    );
    assert_eq!(
        ind.documentation.status,
        nexo_domain::indicators::IndicatorStatus::Unknown
    );
}

#[test]
fn test_fork_provenance_isolation_invariants() {
    let root = get_repo_root();
    let fixtures_dir = root.join("fixtures").join("domain");

    let raw =
        std::fs::read_to_string(fixtures_dir.join("fork_without_proven_origin.json")).unwrap();
    let val: Value = serde_json::from_str(&raw).unwrap();
    let snapshot = validate_snapshot(&val).expect("Valid fork without proven origin");

    let extracted = extract_explicit_relationships(
        &snapshot.repositories,
        &snapshot.organizations,
        &HashMap::new(),
        &HashMap::new(),
    );
    // Verified: No fork_of relations fabricated for fork without provenance
    assert!(
        !extracted
            .iter()
            .any(|r| r.type_ == nexo_domain::relations::RelationType::ForkOf),
        "Fork without proven origin must never generate fabricated fork_of relations"
    );
}

#[test]
fn test_negative_validation_error_paths() {
    let root = get_repo_root();
    let fixtures_dir = root.join("fixtures").join("domain");

    // unknown version
    let raw = std::fs::read_to_string(fixtures_dir.join("unknown_version.json")).unwrap();
    let val: Value = serde_json::from_str(&raw).unwrap();
    let errs = validate_snapshot(&val).unwrap_err();
    assert!(errs.iter().any(|e| e.path == "metadata.version"));

    // missing mandatory fields
    let raw = std::fs::read_to_string(fixtures_dir.join("missing_mandatory_fields.json")).unwrap();
    let val: Value = serde_json::from_str(&raw).unwrap();
    let errs = validate_snapshot(&val).unwrap_err();
    assert!(errs.iter().any(|e| e.path == "metadata.authenticatedUser"));
    assert!(errs.iter().any(|e| e.path == "repositories[0].name"));

    // invalid types and malformed dates
    let raw = std::fs::read_to_string(fixtures_dir.join("invalid_types_and_malformed_dates.json"))
        .unwrap();
    let val: Value = serde_json::from_str(&raw).unwrap();
    let errs = validate_snapshot(&val).unwrap_err();
    assert!(errs.iter().any(|e| e.path == "metadata.createdAt"));
    assert!(errs.iter().any(|e| e.path == "repositories[0].id"));

    // inconsistent identities
    let raw = std::fs::read_to_string(fixtures_dir.join("inconsistent_identities.json")).unwrap();
    let val: Value = serde_json::from_str(&raw).unwrap();
    let errs = validate_snapshot(&val).unwrap_err();
    assert!(
        errs.iter()
            .any(|e| e.path == "repositories[1].id" && e.message.contains("duplicado"))
    );
    assert!(
        errs.iter()
            .any(|e| e.path == "metadata.totalRepositories" && e.message.contains("diverge"))
    );
}
