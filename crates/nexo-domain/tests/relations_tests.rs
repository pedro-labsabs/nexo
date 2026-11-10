use nexo_domain::model::{
    Affiliation, Organization, OwnerType, Repository, RepositoryOwner, Snapshot, TreeSignals,
};
use nexo_domain::relations::{
    NodeType, RelationType, Relationship, RelationshipProvenance, RelationshipTombstone,
    extract_explicit_relationships,
};
use std::collections::{BTreeMap, HashMap};

fn make_test_repo(
    id: u64,
    name: &str,
    full_name: &str,
    owner_login: &str,
    owner_id: u64,
    is_org: bool,
    fork: bool,
) -> Repository {
    Repository {
        id,
        node_id: format!("node_{id}"),
        name: name.to_string(),
        full_name: full_name.to_string(),
        owner: RepositoryOwner {
            login: owner_login.to_string(),
            id: owner_id,
            avatar_url: None,
            type_: if is_org {
                OwnerType::Organization
            } else {
                OwnerType::User
            },
        },
        is_private: false,
        html_url: format!("https://github.com/{full_name}"),
        description: Some(format!("Description for {name}")),
        fork,
        created_at: "2025-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        pushed_at: "2026-01-01T00:00:00Z".to_string(),
        size_kb: 100,
        stargazers_count: 10,
        watchers_count: 5,
        open_issues_count: 2,
        forks_count: 1,
        primary_language: Some("TypeScript".to_string()),
        languages: BTreeMap::new(),
        archived: false,
        disabled: false,
        default_branch: "main".to_string(),
        affiliation: if is_org {
            Affiliation::OrganizationMember
        } else {
            Affiliation::Owner
        },
        tree_signals: TreeSignals::default(),
        indicators: None,
    }
}

fn make_test_org(id: u64, login: &str) -> Organization {
    Organization {
        id,
        login: login.to_string(),
        name: Some(format!("Org {login}")),
        avatar_url: None,
        description: Some(format!("Description of {login}")),
        repository_count: 1,
    }
}

#[test]
fn test_relationship_models_serde() {
    let rel = Relationship {
        id: "rel_belongs_1001_101".to_string(),
        source: "1001".to_string(),
        target: "101".to_string(),
        source_type: NodeType::Repository,
        target_type: NodeType::Organization,
        type_: RelationType::BelongsTo,
        provenance: RelationshipProvenance::Automatic,
        verified: true,
        evidence: Some(
            "Repositório inova-labs/nexo-core pertence à organização inova-labs".to_string(),
        ),
        description: Some("Membro da organização inova-labs".to_string()),
        created_at: "2025-02-10T10:00:00Z".to_string(),
        updated_at: None,
    };

    let json = serde_json::to_string(&rel).expect("Serialization failed");
    assert!(json.contains(r#""sourceType":"repository""#));
    assert!(json.contains(r#""targetType":"organization""#));
    assert!(json.contains(r#""type":"belongs_to""#));
    assert!(json.contains(r#""provenance":"automatic""#));
    assert!(json.contains(r#""verified":true"#));

    let deserialized: Relationship = serde_json::from_str(&json).expect("Deserialization failed");
    assert_eq!(rel, deserialized);

    let tombstone = RelationshipTombstone {
        id: "rel_belongs_1001_101".to_string(),
        source: "1001".to_string(),
        target: "101".to_string(),
        type_: RelationType::BelongsTo,
        deleted_at: "2026-11-10T12:00:00Z".to_string(),
    };

    let tomb_json = serde_json::to_string(&tombstone).expect("Tombstone serialization failed");
    assert!(tomb_json.contains(r#""type":"belongs_to""#));
    assert!(tomb_json.contains(r#""deletedAt":"2026-11-10T12:00:00Z""#));
    let tomb_deser: RelationshipTombstone =
        serde_json::from_str(&tomb_json).expect("Tombstone deserialization failed");
    assert_eq!(tombstone, tomb_deser);
}

#[test]
fn test_belongs_to_extraction() {
    let repos = vec![
        make_test_repo(
            1001,
            "core",
            "inova-labs/core",
            "inova-labs",
            101,
            true,
            false,
        ),
        make_test_repo(
            1002,
            "ext",
            "InOva-Labs/ext",
            "INOVA-LABS",
            101,
            true,
            false,
        ), // case-insensitive check
        make_test_repo(
            1003,
            "personal",
            "alexdev/personal",
            "alexdev",
            999,
            false,
            false,
        ),
    ];
    let orgs = vec![make_test_org(101, "inova-labs")];

    let readme_map = HashMap::new();
    let manifest_map = HashMap::new();

    let rels = extract_explicit_relationships(&repos, &orgs, &readme_map, &manifest_map);
    assert_eq!(rels.len(), 2);

    let rel1 = &rels[0];
    assert_eq!(rel1.id, "rel_belongs_1001_101");
    assert_eq!(rel1.source, "1001");
    assert_eq!(rel1.target, "101");
    assert_eq!(rel1.source_type, NodeType::Repository);
    assert_eq!(rel1.target_type, NodeType::Organization);
    assert_eq!(rel1.type_, RelationType::BelongsTo);
    assert_eq!(rel1.provenance, RelationshipProvenance::Automatic);
    assert!(rel1.verified);
    assert_eq!(
        rel1.evidence.as_deref(),
        Some("Repositório inova-labs/core pertence à organização inova-labs")
    );
    assert_eq!(
        rel1.description.as_deref(),
        Some("Membro da organização inova-labs")
    );
    assert_eq!(rel1.created_at, "2025-01-01T00:00:00Z");

    let rel2 = &rels[1];
    assert_eq!(rel2.id, "rel_belongs_1002_101");
    assert_eq!(rel2.source, "1002");
    assert_eq!(rel2.target, "101");
}

#[test]
fn test_references_extraction_from_readme() {
    let repo_a = make_test_repo(1, "core", "nexo-org/core", "nexo-org", 50, true, false);
    let repo_b = make_test_repo(
        2,
        "extension",
        "nexo-org/extension",
        "nexo-org",
        50,
        true,
        false,
    );
    let repo_c = make_test_repo(3, "docs", "nexo-org/docs", "nexo-org", 50, true, false);

    let repos = vec![repo_a, repo_b, repo_c];
    let orgs = vec![make_test_org(50, "nexo-org")];

    let mut readme_map = HashMap::new();
    // repo 2 references repo 1 via URL
    readme_map.insert(
        2,
        "Consulte o projeto principal em https://github.com/nexo-org/core para maiores detalhes."
            .to_string(),
    );
    // repo 3 references repo 2 via markdown link
    readme_map.insert(
        3,
        "Veja a [extensão](https://github.com/nexo-org/extension/tree/main) oficial.".to_string(),
    );

    let manifest_map = HashMap::new();
    let rels = extract_explicit_relationships(&repos, &orgs, &readme_map, &manifest_map);

    let references: Vec<_> = rels
        .into_iter()
        .filter(|r| r.type_ == RelationType::References)
        .collect();
    assert_eq!(references.len(), 2);

    let ref_2_to_1 = references
        .iter()
        .find(|r| r.source == "2" && r.target == "1")
        .expect("Missing ref 2 -> 1");
    assert_eq!(ref_2_to_1.id, "rel_ref_2_1");
    assert_eq!(ref_2_to_1.source_type, NodeType::Repository);
    assert_eq!(ref_2_to_1.target_type, NodeType::Repository);
    assert!(ref_2_to_1.verified);
    assert_eq!(
        ref_2_to_1.evidence.as_deref(),
        Some(
            "Citação explícita encontrada no README de nexo-org/extension apontando para nexo-org/core"
        )
    );
    assert_eq!(
        ref_2_to_1.description.as_deref(),
        Some("Referenciado em documentação de extension")
    );

    let ref_3_to_2 = references
        .iter()
        .find(|r| r.source == "3" && r.target == "2")
        .expect("Missing ref 3 -> 2");
    assert_eq!(ref_3_to_2.id, "rel_ref_3_2");
}

#[test]
fn test_references_ignores_self_reference() {
    let repo = make_test_repo(1, "core", "nexo-org/core", "nexo-org", 50, true, false);
    let mut readme_map = HashMap::new();
    readme_map.insert(
        1,
        "Nosso repositório é https://github.com/nexo-org/core".to_string(),
    );

    let rels = extract_explicit_relationships(&[repo], &[], &readme_map, &HashMap::new());
    let references: Vec<_> = rels
        .into_iter()
        .filter(|r| r.type_ == RelationType::References)
        .collect();
    assert_eq!(references.len(), 0);
}

#[test]
fn test_depends_on_extraction_from_package_json() {
    let repo_core = make_test_repo(1, "core", "nexo-org/core", "nexo-org", 50, true, false);
    let repo_ext = make_test_repo(
        2,
        "extension",
        "nexo-org/extension",
        "nexo-org",
        50,
        true,
        false,
    );
    let repo_plugin = make_test_repo(3, "plugin", "nexo-org/plugin", "nexo-org", 50, true, false);
    let repo_cli = make_test_repo(4, "cli", "nexo-org/cli", "nexo-org", 50, true, false);

    let repos = vec![repo_core, repo_ext, repo_plugin, repo_cli];
    let mut manifest_map = HashMap::new();

    // extension depends on core via git+https in dependencies
    manifest_map.insert(
        2,
        r#"{
            "name": "@nexo-org/extension",
            "dependencies": {
                "nexo-core": "git+https://github.com/nexo-org/core.git#v1.0.0"
            }
        }"#
        .to_string(),
    );

    // plugin depends on core via github: shortcut in devDependencies
    manifest_map.insert(
        3,
        r#"{
            "name": "@nexo-org/plugin",
            "devDependencies": {
                "@nexo-org/core": "github:nexo-org/core#main"
            }
        }"#
        .to_string(),
    );

    // cli depends on core via peerDependencies with https://github.com/ URL
    manifest_map.insert(
        4,
        r#"{
            "name": "@nexo-org/cli",
            "peerDependencies": {
                "core": "https://github.com/nexo-org/core"
            },
            "optionalDependencies": {
                "core-opt": "https://github.com/nexo-org/core"
            }
        }"#
        .to_string(),
    );

    let rels = extract_explicit_relationships(&repos, &[], &HashMap::new(), &manifest_map);
    let deps: Vec<_> = rels
        .into_iter()
        .filter(|r| r.type_ == RelationType::DependsOn)
        .collect();

    // 2 -> 1, 3 -> 1, and 4 -> 1 (deduplicated between peer and optional)
    assert_eq!(deps.len(), 3);

    let dep_2_1 = deps
        .iter()
        .find(|r| r.source == "2" && r.target == "1")
        .expect("Missing dep 2 -> 1");
    assert_eq!(dep_2_1.id, "rel_dep_2_1");
    assert_eq!(dep_2_1.source_type, NodeType::Repository);
    assert_eq!(dep_2_1.target_type, NodeType::Repository);
    assert!(dep_2_1.verified);
    assert_eq!(
        dep_2_1.evidence.as_deref(),
        Some(
            r#"package.json: dependência "nexo-core": "git+https://github.com/nexo-org/core.git#v1.0.0" aponta inequivocamente para nexo-org/core"#
        )
    );
    assert_eq!(
        dep_2_1.description.as_deref(),
        Some("Dependência declarada em manifesto (nexo-core)")
    );

    let dep_3_1 = deps
        .iter()
        .find(|r| r.source == "3" && r.target == "1")
        .expect("Missing dep 3 -> 1");
    assert_eq!(dep_3_1.id, "rel_dep_3_1");

    // Verify deduplication on repo 4
    let dep_4_1: Vec<_> = deps
        .iter()
        .filter(|r| r.source == "4" && r.target == "1")
        .collect();
    assert_eq!(dep_4_1.len(), 1);
}

#[test]
fn test_rejection_of_false_relationships() {
    let repo_core = make_test_repo(1, "core", "nexo-org/core", "nexo-org", 50, true, false);
    let repo_other = make_test_repo(2, "core-fake", "other/core-fake", "other", 99, false, false);
    let repos = vec![repo_core, repo_other];

    let mut manifest_map = HashMap::new();
    // Merely shares name with package on npm registry, without git/github URL
    manifest_map.insert(
        2,
        r#"{
            "name": "other-app",
            "dependencies": {
                "core": "^1.0.0",
                "core-fake": ">=2.0.0"
            }
        }"#
        .to_string(),
    );

    let mut readme_map = HashMap::new();
    // Mentioning just the word or prefix without github link
    readme_map.insert(
        2,
        "This tool has core features and core-fake extensions.".to_string(),
    );

    let rels = extract_explicit_relationships(&repos, &[], &readme_map, &manifest_map);
    assert_eq!(rels.len(), 0);
}

#[test]
fn test_forks_without_explicit_provenance_do_not_fabricate_relations() {
    let forked_repo = make_test_repo(
        10,
        "forked-repo",
        "user/forked-repo",
        "user",
        1,
        false,
        true,
    );
    assert!(forked_repo.fork);

    let rels =
        extract_explicit_relationships(&[forked_repo], &[], &HashMap::new(), &HashMap::new());
    assert_eq!(rels.len(), 0);

    let fork_relations: Vec<_> = rels
        .into_iter()
        .filter(|r| r.type_ == RelationType::ForkOf)
        .collect();
    assert_eq!(fork_relations.len(), 0);
}

#[test]
fn test_demo_snapshot_relationships_extraction_and_serde() {
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
        .expect("Failed to deserialize demo-snapshot.json into Snapshot with typed relationships");

    // Verify snapshot relationships parsed into typed Relationship
    assert_eq!(snapshot.relationships.len(), 8);
    assert_eq!(snapshot.relationships[0].id, "rel_1001_101");
    assert_eq!(snapshot.relationships[0].type_, RelationType::BelongsTo);
    assert_eq!(snapshot.relationships[0].source, "1001");
    assert_eq!(snapshot.relationships[0].target, "101");
    assert_eq!(snapshot.relationships[0].source_type, NodeType::Repository);
    assert_eq!(
        snapshot.relationships[0].target_type,
        NodeType::Organization
    );

    let custom_rel = snapshot
        .relationships
        .iter()
        .find(|r| r.type_ == RelationType::Custom)
        .expect("Missing custom relation");
    assert_eq!(custom_rel.id, "rel_manual_telemetria_pesquisa");
    assert_eq!(custom_rel.provenance, RelationshipProvenance::Manual);
    assert!(!custom_rel.verified);

    // Now test running extraction on the demo repos and orgs
    let mut readme_map = HashMap::new();
    // nexo-core README contains link to rede-aberta/sdk-javascript (repo 2002)
    readme_map.insert(
        1001,
        r#"Para integrar com redes federadas, veja https://github.com/rede-aberta/sdk-javascript"#
            .to_string(),
    );

    let mut manifest_map = HashMap::new();
    // sdk-javascript (2002) package.json depends on rede-aberta/protocolo-aberto (2001)
    manifest_map.insert(
        2002,
        r#"{
            "name": "@rede-aberta/sdk-javascript",
            "dependencies": {
                "@rede-aberta/protocolo-aberto": "git+https://github.com/rede-aberta/protocolo-aberto.git#v1.2.0"
            }
        }"#.to_string(),
    );

    let extracted = extract_explicit_relationships(
        &snapshot.repositories,
        &snapshot.organizations,
        &readme_map,
        &manifest_map,
    );

    // Belongs to:
    // 1001 -> 101, 1002 -> 101, 1003 -> 101 (inova-labs org)
    // 2001 -> 102, 2002 -> 102 (rede-aberta org)
    let belongs: Vec<_> = extracted
        .iter()
        .filter(|r| r.type_ == RelationType::BelongsTo)
        .collect();
    assert_eq!(belongs.len(), 5);

    // References: 1001 -> 2002
    let refs: Vec<_> = extracted
        .iter()
        .filter(|r| r.type_ == RelationType::References)
        .collect();
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].source, "1001");
    assert_eq!(refs[0].target, "2002");
    assert_eq!(refs[0].id, "rel_ref_1001_2002");

    // Depends on: 2002 -> 2001
    let deps: Vec<_> = extracted
        .iter()
        .filter(|r| r.type_ == RelationType::DependsOn)
        .collect();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].source, "2002");
    assert_eq!(deps[0].target, "2001");
    assert_eq!(deps[0].id, "rel_dep_2002_2001");
}
