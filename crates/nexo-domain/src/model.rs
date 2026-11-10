use crate::relations::Relationship;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Tipo de afiliação do usuário com o repositório no GitHub.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Affiliation {
    Owner,
    Collaborator,
    OrganizationMember,
}

/// Tipo de proprietário do repositório (usuário ou organização).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum OwnerType {
    User,
    Organization,
}

/// Proprietário de um repositório no GitHub.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryOwner {
    pub login: String,
    pub id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(rename = "type")]
    pub type_: OwnerType,
}

/// Sinais extraídos da inspeção da árvore de arquivos de um repositório.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TreeSignals {
    pub has_tests: bool,
    #[serde(default)]
    pub test_evidence: Vec<String>,
    #[serde(rename = "hasCI", alias = "hasCi")]
    pub has_ci: bool,
    #[serde(default)]
    pub ci_evidence: Vec<String>,
    pub has_documentation: bool,
    #[serde(default)]
    pub documentation_evidence: Vec<String>,
    pub readme_found: bool,
    #[serde(default)]
    pub readme_path: Option<String>,
    pub contributing_found: bool,
    pub license_file_found: bool,
    #[serde(default)]
    pub package_json_found: bool,
    #[serde(default)]
    pub is_empty_repo: bool,
    pub tree_truncated: bool,
    pub tree_inspected: bool,
    #[serde(default)]
    pub inspection_error: Option<String>,
}

/// Resumo de uma organização do GitHub no snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub id: u64,
    pub login: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub repository_count: u64,
}

/// Metadados do Snapshot do portfólio (Schema 1.0).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotMetadata {
    pub version: String,
    pub created_at: String,
    pub reference_date: String,
    pub authenticated_user: String,
    pub total_repositories: u64,
    #[serde(default)]
    pub incomplete_sync: bool,
    #[serde(default)]
    pub sync_warnings: Vec<String>,
}

/// Modelo completo de Repositório conforme especificação Snapshot 1.0.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    pub id: u64,
    pub node_id: String,
    pub name: String,
    pub full_name: String,
    pub owner: RepositoryOwner,
    pub is_private: bool,
    pub html_url: String,
    #[serde(default)]
    pub description: Option<String>,
    pub fork: bool,
    pub created_at: String,
    pub updated_at: String,
    pub pushed_at: String,
    pub size_kb: u64,
    pub stargazers_count: u64,
    pub watchers_count: u64,
    pub open_issues_count: u64,
    pub forks_count: u64,
    #[serde(default)]
    pub primary_language: Option<String>,
    #[serde(default)]
    pub languages: BTreeMap<String, u64>,
    pub archived: bool,
    pub disabled: bool,
    pub default_branch: String,
    pub affiliation: Affiliation,
    pub tree_signals: TreeSignals,
    #[serde(default)]
    pub indicators: Option<crate::indicators::RepositoryIndicators>,
}

/// Snapshot completo contendo metadados, organizações, repositórios, relacionamentos e anotações.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub metadata: SnapshotMetadata,
    #[serde(default)]
    pub organizations: Vec<Organization>,
    #[serde(default)]
    pub repositories: Vec<Repository>,
    #[serde(default)]
    pub relationships: Vec<Relationship>,
    #[serde(default)]
    pub user_annotations: Option<serde_json::Value>,
}
