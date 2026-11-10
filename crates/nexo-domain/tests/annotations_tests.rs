use nexo_domain::annotations::{
    get_relationship_logical_key, get_timestamp, is_valid_date, merge_manual_relationships,
    merge_potential_assessments, merge_user_annotations, CommercialPotential, PotentialAssessment,
    StrategicValue, UserAnnotations,
};
use nexo_domain::model::Snapshot;
use nexo_domain::relations::{
    NodeType, RelationType, Relationship, RelationshipProvenance, RelationshipTombstone,
};
use std::collections::BTreeMap;

#[test]
fn test_logical_key_generation_and_directed_distinction() {
    let key1 = get_relationship_logical_key("1001", "1002", &RelationType::Custom);
    assert_eq!(key1, "1001::1002::custom");

    let key2 = get_relationship_logical_key("1002", "1001", &RelationType::Custom);
    assert_eq!(key2, "1002::1001::custom");
    assert_ne!(key1, key2, "Directed logical key must distinguish source and target");

    // All RelationType variants
    assert_eq!(
        get_relationship_logical_key("a", "b", &RelationType::BelongsTo),
        "a::b::belongs_to"
    );
    assert_eq!(
        get_relationship_logical_key("a", "b", &RelationType::References),
        "a::b::references"
    );
    assert_eq!(
        get_relationship_logical_key("a", "b", &RelationType::DependsOn),
        "a::b::depends_on"
    );
    assert_eq!(
        get_relationship_logical_key("a", "b", &RelationType::ForkOf),
        "a::b::fork_of"
    );
    assert_eq!(
        get_relationship_logical_key("a", "b", &RelationType::Custom),
        "a::b::custom"
    );
}

#[test]
fn test_is_valid_date_and_get_timestamp() {
    assert!(is_valid_date("2026-01-01T10:00:00Z"));
    assert!(is_valid_date("2026-11-01T12:00:00+00:00"));
    assert!(get_timestamp("2026-01-01T10:00:00Z") > 0);

    assert!(!is_valid_date(""));
    assert!(!is_valid_date("invalid-date"));
    assert!(!is_valid_date("not a date"));
    assert_eq!(get_timestamp("invalid-date"), 0);
    assert_eq!(get_timestamp(""), 0);
}

#[test]
fn test_lww_potential_assessment_incoming_strictly_newer_wins() {
    let mut local = BTreeMap::new();
    local.insert(
        "1001".to_string(),
        PotentialAssessment {
            repository_id: 1001,
            strategic_value: StrategicValue::Medium,
            commercial_potential: CommercialPotential::Experimental,
            target_audience: None,
            notes: None,
            tags: vec!["local".to_string()],
            updated_at: "2026-01-01T10:00:00Z".to_string(),
        },
    );

    let mut incoming = BTreeMap::new();
    incoming.insert(
        "1001".to_string(),
        PotentialAssessment {
            repository_id: 1001,
            strategic_value: StrategicValue::Critical,
            commercial_potential: CommercialPotential::ProductionReady,
            target_audience: Some("Audience".to_string()),
            notes: Some("Notes".to_string()),
            tags: vec!["incoming".to_string()],
            updated_at: "2026-01-02T10:00:00Z".to_string(),
        },
    );

    let merged = merge_potential_assessments(&local, &incoming);
    let item = merged.get("1001").expect("item 1001 must exist");
    assert_eq!(item.strategic_value, StrategicValue::Critical);
    assert_eq!(item.commercial_potential, CommercialPotential::ProductionReady);
    assert_eq!(item.tags, vec!["incoming".to_string()]);
}

#[test]
fn test_lww_potential_assessment_tie_or_older_favors_local() {
    let mut local = BTreeMap::new();
    local.insert(
        "1001".to_string(),
        PotentialAssessment {
            repository_id: 1001,
            strategic_value: StrategicValue::High,
            commercial_potential: CommercialPotential::Viable,
            target_audience: None,
            notes: Some("local version".to_string()),
            tags: vec![],
            updated_at: "2026-01-02T10:00:00Z".to_string(),
        },
    );

    // Tie
    let mut incoming_tie = BTreeMap::new();
    incoming_tie.insert(
        "1001".to_string(),
        PotentialAssessment {
            repository_id: 1001,
            strategic_value: StrategicValue::Low,
            commercial_potential: CommercialPotential::None,
            target_audience: None,
            notes: Some("incoming tie".to_string()),
            tags: vec![],
            updated_at: "2026-01-02T10:00:00Z".to_string(),
        },
    );

    let merged_tie = merge_potential_assessments(&local, &incoming_tie);
    let item_tie = merged_tie.get("1001").unwrap();
    assert_eq!(item_tie.strategic_value, StrategicValue::High);
    assert_eq!(item_tie.notes.as_deref(), Some("local version"));

    // Older incoming
    let mut incoming_older = BTreeMap::new();
    incoming_older.insert(
        "1001".to_string(),
        PotentialAssessment {
            repository_id: 1001,
            strategic_value: StrategicValue::Low,
            commercial_potential: CommercialPotential::None,
            target_audience: None,
            notes: Some("incoming older".to_string()),
            tags: vec![],
            updated_at: "2026-01-01T10:00:00Z".to_string(),
        },
    );

    let merged_older = merge_potential_assessments(&local, &incoming_older);
    let item_older = merged_older.get("1001").unwrap();
    assert_eq!(item_older.strategic_value, StrategicValue::High);
    assert_eq!(item_older.notes.as_deref(), Some("local version"));
}

#[test]
fn test_lww_potential_assessment_invalid_timestamps() {
    let mut local = BTreeMap::new();
    local.insert(
        "1001".to_string(),
        PotentialAssessment {
            repository_id: 1001,
            strategic_value: StrategicValue::High,
            commercial_potential: CommercialPotential::Viable,
            target_audience: None,
            notes: None,
            tags: vec![],
            updated_at: "invalid-date".to_string(),
        },
    );

    let mut incoming = BTreeMap::new();
    incoming.insert(
        "1001".to_string(),
        PotentialAssessment {
            repository_id: 1001,
            strategic_value: StrategicValue::Critical,
            commercial_potential: CommercialPotential::ProductionReady,
            target_audience: None,
            notes: None,
            tags: vec![],
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        },
    );

    // Incoming with valid date wins against invalid local (0 timestamp)
    let merged = merge_potential_assessments(&local, &incoming);
    assert_eq!(merged.get("1001").unwrap().strategic_value, StrategicValue::Critical);

    // If both invalid, both evaluate to 0 -> tie -> local wins
    let mut incoming_invalid = BTreeMap::new();
    incoming_invalid.insert(
        "1001".to_string(),
        PotentialAssessment {
            repository_id: 1001,
            strategic_value: StrategicValue::Low,
            commercial_potential: CommercialPotential::None,
            target_audience: None,
            notes: None,
            tags: vec![],
            updated_at: "garbage-date".to_string(),
        },
    );
    let merged_both_invalid = merge_potential_assessments(&local, &incoming_invalid);
    assert_eq!(merged_both_invalid.get("1001").unwrap().strategic_value, StrategicValue::High);
}

#[test]
fn test_merge_manual_relationships_prevents_duplicate_logical_keys() {
    let rel1 = Relationship {
        id: "rel_1".to_string(),
        source: "1".to_string(),
        target: "2".to_string(),
        source_type: NodeType::Repository,
        target_type: NodeType::Repository,
        type_: RelationType::Custom,
        provenance: RelationshipProvenance::Manual,
        verified: false,
        evidence: None,
        description: Some("First version".to_string()),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: Some("2026-01-01T00:00:00Z".to_string()),
    };

    let rel2 = Relationship {
        id: "rel_1".to_string(),
        source: "1".to_string(),
        target: "2".to_string(),
        source_type: NodeType::Repository,
        target_type: NodeType::Repository,
        type_: RelationType::Custom,
        provenance: RelationshipProvenance::Manual,
        verified: false,
        evidence: None,
        description: Some("Updated description".to_string()),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: Some("2026-01-02T00:00:00Z".to_string()),
    };

    let (merged_rels, _tombstones) = merge_manual_relationships(&[rel1], &[rel2], &[], &[]);
    assert_eq!(merged_rels.len(), 1);
    assert_eq!(
        merged_rels[0].description.as_deref(),
        Some("Updated description")
    );
}

#[test]
fn test_tombstones_prevent_resurrection_of_deleted_relationships() {
    let rel1 = Relationship {
        id: "rel_1".to_string(),
        source: "1".to_string(),
        target: "2".to_string(),
        source_type: NodeType::Repository,
        target_type: NodeType::Repository,
        type_: RelationType::Custom,
        provenance: RelationshipProvenance::Manual,
        verified: false,
        evidence: None,
        description: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: Some("2026-01-01T00:00:00Z".to_string()),
    };

    let tombstone = RelationshipTombstone {
        id: "rel_1".to_string(),
        source: "1".to_string(),
        target: "2".to_string(),
        type_: RelationType::Custom,
        deleted_at: "2026-01-02T00:00:00Z".to_string(),
    };

    // Local has tombstone, incoming imports older rel1
    let (merged_rels, merged_tombstones) =
        merge_manual_relationships(&[], &[rel1], &[tombstone], &[]);

    assert_eq!(merged_rels.len(), 0, "Deleted relationship must not be resurrected");
    assert_eq!(merged_tombstones.len(), 1);
}

#[test]
fn test_recreation_strictly_after_tombstone_allowed() {
    let tombstone = RelationshipTombstone {
        id: "rel_1".to_string(),
        source: "1".to_string(),
        target: "2".to_string(),
        type_: RelationType::Custom,
        deleted_at: "2026-01-02T00:00:00Z".to_string(),
    };

    let re_created_rel = Relationship {
        id: "rel_recreated".to_string(),
        source: "1".to_string(),
        target: "2".to_string(),
        source_type: NodeType::Repository,
        target_type: NodeType::Repository,
        type_: RelationType::Custom,
        provenance: RelationshipProvenance::Manual,
        verified: false,
        evidence: None,
        description: Some("Recreated later".to_string()),
        created_at: "2026-01-03T00:00:00Z".to_string(),
        updated_at: Some("2026-01-03T00:00:00Z".to_string()),
    };

    let (merged_rels, _) =
        merge_manual_relationships(&[re_created_rel], &[], &[tombstone], &[]);

    assert_eq!(merged_rels.len(), 1);
    assert_eq!(
        merged_rels[0].description.as_deref(),
        Some("Recreated later")
    );
}

#[test]
fn test_tombstones_lww_merging() {
    let local_tombstone = RelationshipTombstone {
        id: "rel_1".to_string(),
        source: "1".to_string(),
        target: "2".to_string(),
        type_: RelationType::Custom,
        deleted_at: "2026-01-02T00:00:00Z".to_string(),
    };

    let incoming_newer_tombstone = RelationshipTombstone {
        id: "rel_1".to_string(),
        source: "1".to_string(),
        target: "2".to_string(),
        type_: RelationType::Custom,
        deleted_at: "2026-01-05T00:00:00Z".to_string(),
    };

    let (_, merged_tombstones) = merge_manual_relationships(
        &[],
        &[],
        &[local_tombstone.clone()],
        &[incoming_newer_tombstone],
    );

    assert_eq!(merged_tombstones.len(), 1);
    assert_eq!(merged_tombstones[0].deleted_at, "2026-01-05T00:00:00Z");

    // Tie favors local
    let incoming_tie_tombstone = RelationshipTombstone {
        id: "rel_1_alt".to_string(),
        source: "1".to_string(),
        target: "2".to_string(),
        type_: RelationType::Custom,
        deleted_at: "2026-01-02T00:00:00Z".to_string(),
    };
    let (_, merged_tombstones_tie) = merge_manual_relationships(
        &[],
        &[],
        &[local_tombstone],
        &[incoming_tie_tombstone],
    );
    assert_eq!(merged_tombstones_tie.len(), 1);
    assert_eq!(merged_tombstones_tie[0].id, "rel_1");
}

#[test]
fn test_deduplication_by_stable_relation_id() {
    let rel_old = Relationship {
        id: "rel_dup".to_string(),
        source: "1".to_string(),
        target: "2".to_string(),
        source_type: NodeType::Repository,
        target_type: NodeType::Repository,
        type_: RelationType::Custom,
        provenance: RelationshipProvenance::Manual,
        verified: false,
        evidence: None,
        description: Some("Old tuple".to_string()),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: Some("2026-01-01T00:00:00Z".to_string()),
    };

    let rel_new = Relationship {
        id: "rel_dup".to_string(),
        source: "1".to_string(),
        target: "3".to_string(),
        source_type: NodeType::Repository,
        target_type: NodeType::Repository,
        type_: RelationType::Custom,
        provenance: RelationshipProvenance::Manual,
        verified: false,
        evidence: None,
        description: Some("New tuple".to_string()),
        created_at: "2026-01-02T00:00:00Z".to_string(),
        updated_at: Some("2026-01-02T00:00:00Z".to_string()),
    };

    let (merged, _) = merge_manual_relationships(&[rel_old], &[rel_new], &[], &[]);
    assert_eq!(merged.len(), 1, "Duplicate IDs must be resolved to latest");
    assert_eq!(merged[0].target, "3");
    assert_eq!(merged[0].description.as_deref(), Some("New tuple"));
}

#[test]
fn test_full_user_annotations_merge_deterministic() {
    let mut assessments_a = BTreeMap::new();
    assessments_a.insert(
        "1".to_string(),
        PotentialAssessment {
            repository_id: 1,
            strategic_value: StrategicValue::Medium,
            commercial_potential: CommercialPotential::Experimental,
            target_audience: None,
            notes: None,
            tags: vec![],
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        },
    );

    let anno_a = UserAnnotations {
        schema_version: "1.0".to_string(),
        exported_at: "2026-01-01T00:00:00Z".to_string(),
        authenticated_user: Some("alexdev".to_string()),
        manual_relationships: vec![],
        deleted_relationships: vec![],
        potential_assessments: assessments_a,
    };

    let mut assessments_b = BTreeMap::new();
    assessments_b.insert(
        "1".to_string(),
        PotentialAssessment {
            repository_id: 1,
            strategic_value: StrategicValue::Critical,
            commercial_potential: CommercialPotential::ProductionReady,
            target_audience: None,
            notes: None,
            tags: vec![],
            updated_at: "2026-01-02T00:00:00Z".to_string(),
        },
    );

    let anno_b = UserAnnotations {
        schema_version: "1.0".to_string(),
        exported_at: "2026-01-02T00:00:00Z".to_string(),
        authenticated_user: Some("otheruser".to_string()),
        manual_relationships: vec![],
        deleted_relationships: vec![],
        potential_assessments: assessments_b,
    };

    let merged = merge_user_annotations(&anno_a, &anno_b, Some("2026-01-10T00:00:00Z"));
    assert_eq!(merged.schema_version, "1.0");
    assert_eq!(merged.exported_at, "2026-01-10T00:00:00Z");
    assert_eq!(merged.authenticated_user.as_deref(), Some("alexdev"));
    assert_eq!(
        merged.potential_assessments.get("1").unwrap().strategic_value,
        StrategicValue::Critical
    );

    // Fallback authenticated_user when local is None
    let anno_no_user = UserAnnotations {
        schema_version: "1.0".to_string(),
        exported_at: "2026-01-01T00:00:00Z".to_string(),
        authenticated_user: None,
        manual_relationships: vec![],
        deleted_relationships: vec![],
        potential_assessments: BTreeMap::new(),
    };
    let merged_fallback_user =
        merge_user_annotations(&anno_no_user, &anno_b, Some("2026-01-10T00:00:00Z"));
    assert_eq!(
        merged_fallback_user.authenticated_user.as_deref(),
        Some("otheruser")
    );
}

#[test]
fn test_demo_snapshot_user_annotations_deserialization_and_validation() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let snapshot_path = std::path::Path::new(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("src/demo/demo-snapshot.json");

    let snapshot_content = std::fs::read_to_string(&snapshot_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", snapshot_path.display(), e));

    let snapshot: Snapshot = serde_json::from_str(&snapshot_content)
        .expect("Failed to deserialize demo-snapshot.json into Snapshot");

    let annotations = snapshot
        .user_annotations
        .as_ref()
        .expect("demo-snapshot.json must contain user_annotations");

    assert_eq!(annotations.schema_version, "1.0");
    assert_eq!(annotations.exported_at, "2026-11-01T12:00:00Z");
    assert_eq!(annotations.authenticated_user, None);
    assert_eq!(annotations.deleted_relationships.len(), 0);

    // Manual relationships
    assert_eq!(annotations.manual_relationships.len(), 1);
    let manual_rel = &annotations.manual_relationships[0];
    assert_eq!(manual_rel.id, "rel_manual_telemetria_pesquisa");
    assert_eq!(manual_rel.source, "3002");
    assert_eq!(manual_rel.target, "1002");
    assert_eq!(manual_rel.source_type, NodeType::Repository);
    assert_eq!(manual_rel.target_type, NodeType::Repository);
    assert_eq!(manual_rel.type_, RelationType::Custom);
    assert_eq!(manual_rel.provenance, RelationshipProvenance::Manual);
    assert!(!manual_rel.verified);
    assert_eq!(manual_rel.created_at, "2026-10-30T10:00:00Z");
    assert_eq!(
        manual_rel.description.as_deref(),
        Some("Consumo experimental de métricas da telemetria edge")
    );

    // Potential assessments
    assert_eq!(annotations.potential_assessments.len(), 2);

    let p1001 = annotations
        .potential_assessments
        .get("1001")
        .expect("assessment for 1001 must exist");
    assert_eq!(p1001.repository_id, 1001);
    assert_eq!(p1001.strategic_value, StrategicValue::Critical);
    assert_eq!(p1001.commercial_potential, CommercialPotential::ProductionReady);
    assert_eq!(
        p1001.target_audience.as_deref(),
        Some("Equipes de engenharia de software e observabilidade de código")
    );
    assert_eq!(
        p1001.notes.as_deref(),
        Some("Núcleo central para visibilidade técnica de repositórios.")
    );
    assert_eq!(
        p1001.tags,
        vec!["observabilidade", "local-first", "grafos"]
    );
    assert_eq!(p1001.updated_at, "2026-10-29T11:00:00Z");

    let p1002 = annotations
        .potential_assessments
        .get("1002")
        .expect("assessment for 1002 must exist");
    assert_eq!(p1002.repository_id, 1002);
    assert_eq!(p1002.strategic_value, StrategicValue::High);
    assert_eq!(p1002.commercial_potential, CommercialPotential::Viable);
    assert_eq!(
        p1002.target_audience.as_deref(),
        Some("Operações de infraestrutura e dispositivos remotos")
    );
    assert_eq!(
        p1002.notes.as_deref(),
        Some("Componente importante para telemetria de campo.")
    );
    assert_eq!(p1002.tags, vec!["rust", "iot", "edge"]);
    assert_eq!(p1002.updated_at, "2026-10-29T11:05:00Z");

    // Serialization roundtrip of UserAnnotations
    let serialized_anno = serde_json::to_string(&annotations).expect("Serialization failed");
    let deserialized_anno: UserAnnotations =
        serde_json::from_str(&serialized_anno).expect("Deserialization failed");
    assert_eq!(annotations, &deserialized_anno);
}
