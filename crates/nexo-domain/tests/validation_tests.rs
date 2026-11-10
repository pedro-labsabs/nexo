use nexo_domain::validation::{ValidationErrorDetail, validate_annotations, validate_snapshot};
use serde_json::json;

#[test]
fn test_validation_error_detail_display() {
    let err = ValidationErrorDetail {
        path: "metadata.authenticatedUser".to_string(),
        message: "campo obrigatório ausente".to_string(),
    };
    assert_eq!(
        err.to_string(),
        "metadata.authenticatedUser: campo obrigatório ausente"
    );
}

#[test]
fn test_valid_snapshot_demo_succeeds() {
    let demo_raw = std::fs::read_to_string("../../src/demo/demo-snapshot.json")
        .or_else(|_| std::fs::read_to_string("src/demo/demo-snapshot.json"))
        .expect("failed to load demo-snapshot.json");

    let demo_val: serde_json::Value =
        serde_json::from_str(&demo_raw).expect("failed to parse demo-snapshot.json as JSON");

    let result = validate_snapshot(&demo_val);
    assert!(
        result.is_ok(),
        "Expected valid snapshot, got errors: {:?}",
        result.err()
    );

    let snapshot = result.unwrap();
    assert_eq!(snapshot.metadata.version, "1.0");
    assert_eq!(snapshot.metadata.authenticated_user, "alexdev");
    assert_eq!(snapshot.repositories.len(), 8);
    assert_eq!(snapshot.organizations.len(), 2);
    assert_eq!(snapshot.relationships.len(), 8);
    assert!(snapshot.user_annotations.is_some());
}

#[test]
fn test_snapshot_missing_mandatory_metadata_fields() {
    let demo_raw = std::fs::read_to_string("../../src/demo/demo-snapshot.json")
        .or_else(|_| std::fs::read_to_string("src/demo/demo-snapshot.json"))
        .expect("failed to load demo-snapshot.json");

    let base_val: serde_json::Value =
        serde_json::from_str(&demo_raw).expect("failed to parse demo-snapshot.json");

    // Missing metadata object entirely
    let mut val = base_val.clone();
    val.as_object_mut().unwrap().remove("metadata");
    let errs = validate_snapshot(&val).unwrap_err();
    assert!(
        errs.iter()
            .any(|e| e.path == "metadata" && e.message.contains("campo obrigatório ausente")),
        "Expected error for missing metadata, got: {:?}",
        errs
    );

    // Missing metadata.authenticatedUser
    let mut val = base_val.clone();
    val.get_mut("metadata")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("authenticatedUser");
    let errs = validate_snapshot(&val).unwrap_err();
    assert!(
        errs.iter()
            .any(|e| e.to_string() == "metadata.authenticatedUser: campo obrigatório ausente"),
        "Expected 'metadata.authenticatedUser: campo obrigatório ausente', got: {:?}",
        errs
    );

    // Empty metadata.authenticatedUser
    let mut val = base_val.clone();
    val.get_mut("metadata")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("authenticatedUser".to_string(), json!("   "));
    let errs = validate_snapshot(&val).unwrap_err();
    assert!(
        errs.iter().any(|e| e.path == "metadata.authenticatedUser"),
        "Expected error for empty authenticatedUser, got: {:?}",
        errs
    );

    // Missing metadata.version
    let mut val = base_val.clone();
    val.get_mut("metadata")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("version");
    let errs = validate_snapshot(&val).unwrap_err();
    assert!(
        errs.iter().any(
            |e| e.path == "metadata.version" && e.message.contains("campo obrigatório ausente")
        ),
        "Expected error for missing metadata.version, got: {:?}",
        errs
    );

    // Missing metadata.createdAt
    let mut val = base_val.clone();
    val.get_mut("metadata")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("createdAt");
    let errs = validate_snapshot(&val).unwrap_err();
    assert!(
        errs.iter()
            .any(|e| e.path == "metadata.createdAt"
                && e.message.contains("campo obrigatório ausente")),
        "Expected error for missing metadata.createdAt, got: {:?}",
        errs
    );

    // Missing metadata.referenceDate
    let mut val = base_val.clone();
    val.get_mut("metadata")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("referenceDate");
    let errs = validate_snapshot(&val).unwrap_err();
    assert!(
        errs.iter().any(|e| e.path == "metadata.referenceDate"
            && e.message.contains("campo obrigatório ausente")),
        "Expected error for missing metadata.referenceDate, got: {:?}",
        errs
    );

    // Missing metadata.totalRepositories
    let mut val = base_val.clone();
    val.get_mut("metadata")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("totalRepositories");
    let errs = validate_snapshot(&val).unwrap_err();
    assert!(
        errs.iter().any(|e| e.path == "metadata.totalRepositories"
            && e.message.contains("campo obrigatório ausente")),
        "Expected error for missing metadata.totalRepositories, got: {:?}",
        errs
    );
}

#[test]
fn test_snapshot_schema_version_mismatch() {
    let demo_raw = std::fs::read_to_string("../../src/demo/demo-snapshot.json")
        .or_else(|_| std::fs::read_to_string("src/demo/demo-snapshot.json"))
        .expect("failed to load demo-snapshot.json");

    let base_val: serde_json::Value =
        serde_json::from_str(&demo_raw).expect("failed to parse demo-snapshot.json");

    for bad_version in &["0.9", "2.0", "1.1"] {
        let mut val = base_val.clone();
        val.get_mut("metadata")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("version".to_string(), json!(bad_version));

        let errs = validate_snapshot(&val).unwrap_err();
        assert!(
            errs.iter()
                .any(|e| e.path == "metadata.version" && e.message.contains("1.0")),
            "Expected version mismatch error mentioning 1.0 for version {}, got: {:?}",
            bad_version,
            errs
        );
    }
}

#[test]
fn test_snapshot_invalid_dates() {
    let demo_raw = std::fs::read_to_string("../../src/demo/demo-snapshot.json")
        .or_else(|_| std::fs::read_to_string("src/demo/demo-snapshot.json"))
        .expect("failed to load demo-snapshot.json");

    let base_val: serde_json::Value =
        serde_json::from_str(&demo_raw).expect("failed to parse demo-snapshot.json");

    // Invalid metadata.createdAt
    let mut val = base_val.clone();
    val.get_mut("metadata")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("createdAt".to_string(), json!("data-invalida"));
    let errs = validate_snapshot(&val).unwrap_err();
    assert!(
        errs.iter()
            .any(|e| e.path == "metadata.createdAt" && e.message.contains("data")),
        "Expected date error on metadata.createdAt, got: {:?}",
        errs
    );

    // Invalid metadata.referenceDate
    let mut val = base_val.clone();
    val.get_mut("metadata")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("referenceDate".to_string(), json!("2026-99-99"));
    let errs = validate_snapshot(&val).unwrap_err();
    assert!(
        errs.iter().any(|e| e.path == "metadata.referenceDate"),
        "Expected date error on metadata.referenceDate, got: {:?}",
        errs
    );

    // Invalid repositories[0].createdAt
    let mut val = base_val.clone();
    val.get_mut("repositories")
        .unwrap()
        .get_mut(0)
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("createdAt".to_string(), json!("not-a-date"));
    let errs = validate_snapshot(&val).unwrap_err();
    assert!(
        errs.iter().any(|e| e.path == "repositories[0].createdAt"),
        "Expected date error on repositories[0].createdAt, got: {:?}",
        errs
    );

    // Invalid repositories[0].updatedAt
    let mut val = base_val.clone();
    val.get_mut("repositories")
        .unwrap()
        .get_mut(0)
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("updatedAt".to_string(), json!("not-a-date"));
    let errs = validate_snapshot(&val).unwrap_err();
    assert!(
        errs.iter().any(|e| e.path == "repositories[0].updatedAt"),
        "Expected date error on repositories[0].updatedAt, got: {:?}",
        errs
    );

    // Invalid repositories[0].pushedAt
    let mut val = base_val.clone();
    val.get_mut("repositories")
        .unwrap()
        .get_mut(0)
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("pushedAt".to_string(), json!("not-a-date"));
    let errs = validate_snapshot(&val).unwrap_err();
    assert!(
        errs.iter().any(|e| e.path == "repositories[0].pushedAt"),
        "Expected date error on repositories[0].pushedAt, got: {:?}",
        errs
    );

    // Invalid relationships[0].createdAt
    let mut val = base_val.clone();
    val.get_mut("relationships")
        .unwrap()
        .get_mut(0)
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("createdAt".to_string(), json!("not-a-date"));
    let errs = validate_snapshot(&val).unwrap_err();
    assert!(
        errs.iter().any(|e| e.path == "relationships[0].createdAt"),
        "Expected date error on relationships[0].createdAt, got: {:?}",
        errs
    );
}

#[test]
fn test_snapshot_out_of_bounds_indicator_scores() {
    let demo_raw = std::fs::read_to_string("../../src/demo/demo-snapshot.json")
        .or_else(|_| std::fs::read_to_string("src/demo/demo-snapshot.json"))
        .expect("failed to load demo-snapshot.json");

    let base_val: serde_json::Value =
        serde_json::from_str(&demo_raw).expect("failed to parse demo-snapshot.json");

    // overallQualityScore > 100
    let mut val = base_val.clone();
    val.get_mut("repositories")
        .unwrap()
        .get_mut(0)
        .unwrap()
        .get_mut("indicators")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("overallQualityScore".to_string(), json!(101));
    let errs = validate_snapshot(&val).unwrap_err();
    assert!(
        errs.iter()
            .any(|e| e.path == "repositories[0].indicators.overallQualityScore"),
        "Expected score out of bounds error, got: {:?}",
        errs
    );

    // testInfrastructure score < 0
    let mut val = base_val.clone();
    val.get_mut("repositories")
        .unwrap()
        .get_mut(0)
        .unwrap()
        .get_mut("indicators")
        .unwrap()
        .get_mut("testInfrastructure")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("score".to_string(), json!(-5));
    let errs = validate_snapshot(&val).unwrap_err();
    assert!(
        errs.iter()
            .any(|e| e.path == "repositories[0].indicators.testInfrastructure.score"),
        "Expected score out of bounds error, got: {:?}",
        errs
    );

    // testInfrastructure score > 100
    let mut val = base_val.clone();
    val.get_mut("repositories")
        .unwrap()
        .get_mut(0)
        .unwrap()
        .get_mut("indicators")
        .unwrap()
        .get_mut("testInfrastructure")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("score".to_string(), json!(150));
    let errs = validate_snapshot(&val).unwrap_err();
    assert!(
        errs.iter()
            .any(|e| e.path == "repositories[0].indicators.testInfrastructure.score"),
        "Expected score out of bounds error, got: {:?}",
        errs
    );
}

#[test]
fn test_snapshot_duplicate_repo_ids_and_org_ids() {
    let demo_raw = std::fs::read_to_string("../../src/demo/demo-snapshot.json")
        .or_else(|_| std::fs::read_to_string("src/demo/demo-snapshot.json"))
        .expect("failed to load demo-snapshot.json");

    let base_val: serde_json::Value =
        serde_json::from_str(&demo_raw).expect("failed to parse demo-snapshot.json");

    // Duplicate repository ID
    let mut val = base_val.clone();
    let first_id = val["repositories"][0]["id"].as_u64().unwrap();
    val.get_mut("repositories")
        .unwrap()
        .get_mut(1)
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("id".to_string(), json!(first_id));
    let errs = validate_snapshot(&val).unwrap_err();
    assert!(
        errs.iter()
            .any(|e| e.path == "repositories[1].id" && e.message.contains("duplicado")),
        "Expected duplicate repo ID error on repositories[1].id, got: {:?}",
        errs
    );

    // Duplicate organization ID
    let mut val = base_val.clone();
    let first_org_id = val["organizations"][0]["id"].as_u64().unwrap();
    val.get_mut("organizations")
        .unwrap()
        .get_mut(1)
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("id".to_string(), json!(first_org_id));
    let errs = validate_snapshot(&val).unwrap_err();
    assert!(
        errs.iter()
            .any(|e| e.path == "organizations[1].id" && e.message.contains("duplicado")),
        "Expected duplicate org ID error on organizations[1].id, got: {:?}",
        errs
    );

    // Invalid repository ID (0 or negative)
    let mut val = base_val.clone();
    val.get_mut("repositories")
        .unwrap()
        .get_mut(0)
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("id".to_string(), json!(0));
    let errs = validate_snapshot(&val).unwrap_err();
    assert!(
        errs.iter()
            .any(|e| e.path == "repositories[0].id" && e.message.contains("ID inválido")),
        "Expected 'repositories[0].id: ID inválido', got: {:?}",
        errs
    );
}

#[test]
fn test_snapshot_total_repositories_mismatch() {
    let demo_raw = std::fs::read_to_string("../../src/demo/demo-snapshot.json")
        .or_else(|_| std::fs::read_to_string("src/demo/demo-snapshot.json"))
        .expect("failed to load demo-snapshot.json");

    let base_val: serde_json::Value =
        serde_json::from_str(&demo_raw).expect("failed to parse demo-snapshot.json");

    let mut val = base_val.clone();
    val.get_mut("metadata")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("totalRepositories".to_string(), json!(999));

    let errs = validate_snapshot(&val).unwrap_err();
    assert!(
        errs.iter()
            .any(|e| e.path == "metadata.totalRepositories" && e.message.contains("diverge")),
        "Expected total repositories count mismatch error, got: {:?}",
        errs
    );
}

#[test]
fn test_valid_annotations_and_normalization_of_missing_collections() {
    let minimal_annotations = json!({
        "schemaVersion": "1.0",
        "exportedAt": "2026-11-10T12:00:00Z"
    });

    let result = validate_annotations(&minimal_annotations);
    assert!(
        result.is_ok(),
        "Expected minimal annotations to validate, got: {:?}",
        result.err()
    );

    let annotations = result.unwrap();
    assert_eq!(annotations.schema_version, "1.0");
    assert_eq!(annotations.exported_at, "2026-11-10T12:00:00Z");
    assert_eq!(annotations.manual_relationships.len(), 0);
    assert_eq!(annotations.deleted_relationships.len(), 0);
    assert_eq!(annotations.potential_assessments.len(), 0);
}

#[test]
fn test_annotations_invalid_schema_version_and_date() {
    // Bad schemaVersion
    let bad_version = json!({
        "schemaVersion": "2.0",
        "exportedAt": "2026-11-10T12:00:00Z"
    });
    let errs = validate_annotations(&bad_version).unwrap_err();
    assert!(
        errs.iter()
            .any(|e| e.path == "schemaVersion" && e.message.contains("1.0")),
        "Expected schemaVersion mismatch error, got: {:?}",
        errs
    );

    // Bad exportedAt
    let bad_date = json!({
        "schemaVersion": "1.0",
        "exportedAt": "data-invalida"
    });
    let errs = validate_annotations(&bad_date).unwrap_err();
    assert!(
        errs.iter()
            .any(|e| e.path == "exportedAt" && e.message.contains("data")),
        "Expected invalid exportedAt error, got: {:?}",
        errs
    );
}

#[test]
fn test_validation_does_not_mutate_input() {
    let demo_raw = std::fs::read_to_string("../../src/demo/demo-snapshot.json")
        .or_else(|_| std::fs::read_to_string("src/demo/demo-snapshot.json"))
        .expect("failed to load demo-snapshot.json");

    let val: serde_json::Value =
        serde_json::from_str(&demo_raw).expect("failed to parse demo-snapshot.json");

    let cloned_before = val.clone();
    let result = validate_snapshot(&val);
    assert!(result.is_ok());
    assert_eq!(val, cloned_before, "validate_snapshot mutated input Value");

    let annotations_val = json!({
        "schemaVersion": "1.0",
        "exportedAt": "2026-11-10T12:00:00Z"
    });
    let annotations_cloned = annotations_val.clone();
    let result_ann = validate_annotations(&annotations_val);
    assert!(result_ann.is_ok());
    assert_eq!(
        annotations_val, annotations_cloned,
        "validate_annotations mutated input Value"
    );
}
