use crate::model::{Organization, Repository};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tipos de relação suportados no grafo de portfólio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    BelongsTo,
    References,
    DependsOn,
    ForkOf,
    Custom,
}

/// Proveniência da relação: automática (inferida por evidência explícita) ou manual (anotação do usuário).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipProvenance {
    Automatic,
    Manual,
}

/// Tipo do nó no relacionamento (repositório ou organização).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Repository,
    Organization,
}

/// Modelo completo de Relacionamento no grafo com proveniência estrita.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Relationship {
    pub id: String,
    pub source: String,
    pub target: String,
    pub source_type: NodeType,
    pub target_type: NodeType,
    #[serde(rename = "type")]
    pub type_: RelationType,
    pub provenance: RelationshipProvenance,
    pub verified: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// Registro de exclusão lógica (tombstone) de relacionamento excluído manualmente.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RelationshipTombstone {
    pub id: String,
    pub source: String,
    pub target: String,
    #[serde(rename = "type")]
    pub type_: RelationType,
    pub deleted_at: String,
}

/// Extrai relacionamentos explícitos e verificáveis a partir do conjunto de repositórios,
/// organizações, conteúdos de README e manifestos (ex: package.json).
///
/// Regra estrita de proveniência:
/// - Apenas evidências inequívocas geram relacionamentos automáticos.
/// - Forks sem pai explicitamente comprovado NUNCA geram relações fabricadas.
/// - Nomes de pacotes compartilhados sem URLs git/github NUNCA geram depends_on.
pub fn extract_explicit_relationships(
    repos: &[Repository],
    orgs: &[Organization],
    readme_contents_map: &HashMap<u64, String>,
    manifest_contents_map: &HashMap<u64, String>,
) -> Vec<Relationship> {
    let mut relationships = Vec::new();

    // 1. Relações de Pertencimento (belongs_to): Repo -> Org/Owner
    for repo in repos {
        let owner_login = &repo.owner.login;
        if let Some(org_match) = orgs
            .iter()
            .find(|o| o.login.eq_ignore_ascii_case(owner_login))
        {
            relationships.push(Relationship {
                id: format!("rel_belongs_{}_{}", repo.id, org_match.id),
                source: repo.id.to_string(),
                target: org_match.id.to_string(),
                source_type: NodeType::Repository,
                target_type: NodeType::Organization,
                type_: RelationType::BelongsTo,
                provenance: RelationshipProvenance::Automatic,
                verified: true,
                evidence: Some(format!(
                    "Repositório {} pertence à organização {}",
                    repo.full_name, org_match.login
                )),
                description: Some(format!("Membro da organização {}", org_match.login)),
                created_at: repo.created_at.clone(),
                updated_at: None,
            });
        }
    }

    // 2. Relações Verificadas entre Repositórios a partir de documentação (README)
    for repo in repos {
        let readme = match readme_contents_map.get(&repo.id) {
            Some(content) if !content.is_empty() => content,
            _ => continue,
        };

        for target_repo in repos {
            if target_repo.id == repo.id {
                continue;
            }

            let escaped_full_name = regex::escape(&target_repo.full_name);
            let url_regex_str = format!(r"(?i)github\.com/{escaped_full_name}(\b|/|#)");
            let md_regex_str = format!(r"(?i)\[[^\]]+\]\([^)]*{escaped_full_name}[^)]*\)");

            let matches_url = Regex::new(&url_regex_str)
                .map(|re| re.is_match(readme))
                .unwrap_or(false);
            let matches_md = Regex::new(&md_regex_str)
                .map(|re| re.is_match(readme))
                .unwrap_or(false);

            if matches_url || matches_md {
                relationships.push(Relationship {
                    id: format!("rel_ref_{}_{}", repo.id, target_repo.id),
                    source: repo.id.to_string(),
                    target: target_repo.id.to_string(),
                    source_type: NodeType::Repository,
                    target_type: NodeType::Repository,
                    type_: RelationType::References,
                    provenance: RelationshipProvenance::Automatic,
                    verified: true,
                    evidence: Some(format!(
                        "Citação explícita encontrada no README de {} apontando para {}",
                        repo.full_name, target_repo.full_name
                    )),
                    description: Some(format!("Referenciado em documentação de {}", repo.name)),
                    created_at: repo.created_at.clone(),
                    updated_at: None,
                });
            }
        }
    }

    // 3. Relações Verificadas entre Repositórios a partir de manifestos (ex: package.json)
    // REGRA ESTRITA: apenas referências explícitas verificáveis (URLs github/git),
    // sem inferir por mero nome do pacote no registro npm.
    for repo in repos {
        let manifest_str = match manifest_contents_map.get(&repo.id) {
            Some(s) if !s.is_empty() => s,
            _ => continue,
        };

        let manifest: serde_json::Value = match serde_json::from_str(manifest_str) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let dep_section_names = [
            "dependencies",
            "devDependencies",
            "peerDependencies",
            "optionalDependencies",
        ];

        for section_name in dep_section_names {
            let section_obj = match manifest.get(section_name).and_then(|v| v.as_object()) {
                Some(obj) => obj,
                None => continue,
            };

            for (dep_pkg_name, dep_target_val) in section_obj {
                let dep_target = match dep_target_val.as_str() {
                    Some(s) => s,
                    None => continue,
                };

                for target_repo in repos {
                    if target_repo.id == repo.id {
                        continue;
                    }

                    let escaped_full_name = regex::escape(&target_repo.full_name);
                    let dep_regex_str = format!(
                        r"(?i)(github\.com[/:#]|git\+https?://github\.com/|github:){escaped_full_name}(\.git)?(#|/|$)"
                    );

                    let matches_dep = Regex::new(&dep_regex_str)
                        .map(|re| re.is_match(dep_target))
                        .unwrap_or(false);

                    if matches_dep {
                        let rel_id = format!("rel_dep_{}_{}", repo.id, target_repo.id);

                        // Evita duplicar se declarado em mais de uma seção de dependências
                        if !relationships.iter().any(|r| r.id == rel_id) {
                            relationships.push(Relationship {
                                id: rel_id,
                                source: repo.id.to_string(),
                                target: target_repo.id.to_string(),
                                source_type: NodeType::Repository,
                                target_type: NodeType::Repository,
                                type_: RelationType::DependsOn,
                                provenance: RelationshipProvenance::Automatic,
                                verified: true,
                                evidence: Some(format!(
                                    "package.json: dependência \"{}\": \"{}\" aponta inequivocamente para {}",
                                    dep_pkg_name, dep_target, target_repo.full_name
                                )),
                                description: Some(format!(
                                    "Dependência declarada em manifesto ({})",
                                    dep_pkg_name
                                )),
                                created_at: repo.created_at.clone(),
                                updated_at: None,
                            });
                        }
                    }
                }
            }
        }
    }

    relationships
}
