use nexo_domain::model::{
    Affiliation, Organization, OwnerType, Repository, RepositoryOwner, Snapshot, SnapshotMetadata,
    TreeSignals,
};

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
    assert_eq!(repo.node_id, "R_kgDOA1");
    assert_eq!(repo.name, "nexo-core");
    assert_eq!(repo.full_name, "inova-labs/nexo-core");
    assert_eq!(repo.owner.login, "inova-labs");
    assert_eq!(repo.owner.id, 101);
    assert_eq!(repo.owner.type_, OwnerType::Organization);
    assert_eq!(repo.affiliation, Affiliation::OrganizationMember);
    assert!(repo.tree_signals.has_tests);
    assert_eq!(repo.tree_signals.test_evidence, vec!["tests/core.test.ts"]);
    assert!(repo.tree_signals.has_ci);
    assert_eq!(
        repo.tree_signals.ci_evidence,
        vec![".github/workflows/ci.yml"]
    );
    assert_eq!(repo.languages.get("TypeScript"), Some(&92000));

    let serialized = serde_json::to_string(&repo).expect("serialization failed");
    assert!(serialized.contains(r#""nodeId":"R_kgDOA1""#));
    assert!(serialized.contains(r#""affiliation":"organization_member""#));
    assert!(serialized.contains(r#""hasTests":true"#));
    assert!(serialized.contains(r#""type":"Organization""#));
}

#[test]
fn test_large_repo_id_greater_than_i32() {
    let large_id: u64 = 8_589_934_592; // 2^33
    let json = format!(
        r#"{{"id":{large_id},"nodeId":"x","name":"x","fullName":"o/x","owner":{{"login":"o","id":1,"type":"User"}},"isPrivate":false,"htmlUrl":"u","description":null,"fork":false,"createdAt":"t","updatedAt":"t","pushedAt":"t","sizeKb":0,"stargazersCount":0,"watchersCount":0,"openIssuesCount":0,"forksCount":0,"primaryLanguage":null,"languages":{{}},"archived":false,"disabled":false,"defaultBranch":"main","affiliation":"owner","treeSignals":{{"hasTests":false,"testEvidence":[],"hasCI":false,"ciEvidence":[],"hasDocumentation":false,"documentationEvidence":[],"readmeFound":false,"contributingFound":false,"licenseFileFound":false,"packageJsonFound":false,"isEmptyRepo":false,"treeTruncated":false,"treeInspected":false}}}}"#
    );
    let repo: Repository = serde_json::from_str(&json).expect("large id deserialization failed");
    assert_eq!(repo.id, large_id);
}

#[test]
fn test_affiliation_and_owner_type_serde() {
    // Affiliation snake_case
    assert_eq!(
        serde_json::to_string(&Affiliation::Owner).unwrap(),
        "\"owner\""
    );
    assert_eq!(
        serde_json::to_string(&Affiliation::Collaborator).unwrap(),
        "\"collaborator\""
    );
    assert_eq!(
        serde_json::to_string(&Affiliation::OrganizationMember).unwrap(),
        "\"organization_member\""
    );

    assert_eq!(
        serde_json::from_str::<Affiliation>("\"owner\"").unwrap(),
        Affiliation::Owner
    );
    assert_eq!(
        serde_json::from_str::<Affiliation>("\"collaborator\"").unwrap(),
        Affiliation::Collaborator
    );
    assert_eq!(
        serde_json::from_str::<Affiliation>("\"organization_member\"").unwrap(),
        Affiliation::OrganizationMember
    );

    // OwnerType PascalCase
    assert_eq!(serde_json::to_string(&OwnerType::User).unwrap(), "\"User\"");
    assert_eq!(
        serde_json::to_string(&OwnerType::Organization).unwrap(),
        "\"Organization\""
    );

    assert_eq!(
        serde_json::from_str::<OwnerType>("\"User\"").unwrap(),
        OwnerType::User
    );
    assert_eq!(
        serde_json::from_str::<OwnerType>("\"Organization\"").unwrap(),
        OwnerType::Organization
    );
}

#[test]
fn test_tree_signals_default_and_serde() {
    let default_signals = TreeSignals::default();
    assert!(!default_signals.has_tests);
    assert!(!default_signals.has_ci);
    assert!(default_signals.test_evidence.is_empty());
    assert!(!default_signals.package_json_found);

    let json = r#"{
        "hasTests": false,
        "hasCI": true,
        "ciEvidence": [".github/workflows/ci.yml"],
        "hasDocumentation": false,
        "readmeFound": false,
        "contributingFound": false,
        "licenseFileFound": false,
        "treeTruncated": false,
        "treeInspected": true
    }"#;
    let signals: TreeSignals =
        serde_json::from_str(json).expect("failed to deserialize tree signals");
    assert!(signals.has_ci);
    assert_eq!(signals.ci_evidence, vec![".github/workflows/ci.yml"]);
    assert_eq!(signals.readme_path, None);
    assert!(!signals.package_json_found);

    let serialized = serde_json::to_string(&signals).expect("failed to serialize tree signals");
    assert!(serialized.contains(r#""hasCI":true"#));
}

#[test]
fn test_organization_and_owner_models() {
    let org = Organization {
        id: 42,
        login: "my-org".into(),
        name: Some("My Org Inc.".into()),
        avatar_url: None,
        description: Some("Observability stack".into()),
        repository_count: 10,
    };
    let json = serde_json::to_string(&org).expect("serialization failed");
    let decoded: Organization = serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(org, decoded);

    let owner = RepositoryOwner {
        login: "owner-login".into(),
        id: 999,
        avatar_url: Some("https://example.com/avatar.png".into()),
        type_: OwnerType::User,
    };
    let owner_json = serde_json::to_string(&owner).expect("owner serialization failed");
    assert!(owner_json.contains(r#""type":"User""#));
    assert!(owner_json.contains(r#""avatarUrl":"https://example.com/avatar.png""#));
    let decoded_owner: RepositoryOwner =
        serde_json::from_str(&owner_json).expect("owner deserialization failed");
    assert_eq!(owner, decoded_owner);
}

#[test]
fn test_snapshot_metadata_serde() {
    let meta = SnapshotMetadata {
        version: "1.0".into(),
        created_at: "2026-11-01T12:00:00Z".into(),
        reference_date: "2026-11-01T12:00:00Z".into(),
        authenticated_user: "alexdev".into(),
        total_repositories: 42,
        incomplete_sync: false,
        sync_warnings: vec!["Warning 1".into()],
    };
    let json = serde_json::to_string(&meta).expect("metadata serialization failed");
    assert!(json.contains(r#""referenceDate":"2026-11-01T12:00:00Z""#));
    assert!(json.contains(r#""authenticatedUser":"alexdev""#));
    assert!(json.contains(r#""totalRepositories":42"#));

    let decoded: SnapshotMetadata =
        serde_json::from_str(&json).expect("metadata deserialization failed");
    assert_eq!(meta, decoded);
}

#[test]
fn test_demo_snapshot_roundtrip() {
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

    assert_eq!(snapshot.metadata.version, "1.0");
    assert_eq!(snapshot.metadata.authenticated_user, "alexdev");
    assert_eq!(snapshot.metadata.total_repositories, 8);
    assert!(!snapshot.metadata.incomplete_sync);
    assert_eq!(snapshot.metadata.sync_warnings.len(), 1);

    assert_eq!(snapshot.organizations.len(), 2);
    assert_eq!(snapshot.organizations[0].login, "inova-labs");
    assert_eq!(snapshot.organizations[0].id, 101);
    assert_eq!(snapshot.organizations[1].login, "rede-aberta");
    assert_eq!(snapshot.organizations[1].id, 102);

    assert_eq!(snapshot.repositories.len(), 8);
    let r1 = &snapshot.repositories[0];
    assert_eq!(r1.id, 1001);
    assert_eq!(r1.name, "nexo-core");
    assert_eq!(r1.full_name, "inova-labs/nexo-core");
    assert_eq!(r1.owner.type_, OwnerType::Organization);
    assert_eq!(r1.affiliation, Affiliation::OrganizationMember);
    assert!(r1.tree_signals.has_tests);
    assert!(r1.tree_signals.tree_inspected);
    assert!(r1.indicators.is_some());

    let r_restricted = &snapshot.repositories[7];
    assert_eq!(r_restricted.id, 3003);
    assert_eq!(r_restricted.name, "pesquisa-inacessivel");
    assert_eq!(r_restricted.primary_language, None);
    assert_eq!(
        r_restricted.tree_signals.inspection_error.as_deref(),
        Some("403 Forbidden: Tree access denied or repository empty")
    );

    // Serialization roundtrip
    let reserialized = serde_json::to_string(&snapshot).expect("Reserialization failed");
    let roundtrip: Snapshot =
        serde_json::from_str(&reserialized).expect("Deserialization of reserialized failed");
    assert_eq!(snapshot, roundtrip);
}
