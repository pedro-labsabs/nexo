use crate::annotations::{UserAnnotations, is_valid_date};
use crate::model::Snapshot;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

/// Detalhe estruturado de erro de validação com caminho exato do campo e mensagem descritiva.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationErrorDetail {
    pub path: String,
    pub message: String,
}

impl std::fmt::Display for ValidationErrorDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

fn build_path(prefix: &str, field: &str) -> String {
    if prefix.is_empty() {
        field.to_string()
    } else {
        format!("{}.{}", prefix, field)
    }
}

/// Valida um Snapshot 1.0 a partir de uma referência a `serde_json::Value`,
/// garantindo integridade de schema, datas ISO, limites de escores e ausência de duplicatas.
pub fn validate_snapshot(
    value: &Value,
) -> std::result::Result<Snapshot, Vec<ValidationErrorDetail>> {
    let mut errors = Vec::new();

    let root_obj = match value.as_object() {
        Some(obj) => obj,
        None => {
            errors.push(ValidationErrorDetail {
                path: "".to_string(),
                message: "esperado objeto JSON na raiz".to_string(),
            });
            return Err(errors);
        }
    };

    // 1. Validação de Metadata
    let mut declared_total_repos: Option<u64> = None;
    match root_obj.get("metadata") {
        None | Some(Value::Null) => {
            errors.push(ValidationErrorDetail {
                path: "metadata".to_string(),
                message: "campo obrigatório ausente".to_string(),
            });
        }
        Some(Value::Object(m)) => {
            // version
            match m.get("version") {
                None | Some(Value::Null) => {
                    errors.push(ValidationErrorDetail {
                        path: "metadata.version".to_string(),
                        message: "campo obrigatório ausente".to_string(),
                    });
                }
                Some(Value::String(v)) => {
                    if v != "1.0" {
                        errors.push(ValidationErrorDetail {
                            path: "metadata.version".to_string(),
                            message: format!(
                                "versão incompatível do schema: esperado 1.0, encontrado {}",
                                v
                            ),
                        });
                    }
                }
                Some(_) => {
                    errors.push(ValidationErrorDetail {
                        path: "metadata.version".to_string(),
                        message: "versão deve ser uma string".to_string(),
                    });
                }
            }

            // createdAt
            match m.get("createdAt") {
                None | Some(Value::Null) => {
                    errors.push(ValidationErrorDetail {
                        path: "metadata.createdAt".to_string(),
                        message: "campo obrigatório ausente".to_string(),
                    });
                }
                Some(Value::String(s)) => {
                    if !is_valid_date(s) {
                        errors.push(ValidationErrorDetail {
                            path: "metadata.createdAt".to_string(),
                            message: "data inválida (esperado formato ISO 8601 / RFC 3339)"
                                .to_string(),
                        });
                    }
                }
                Some(_) => {
                    errors.push(ValidationErrorDetail {
                        path: "metadata.createdAt".to_string(),
                        message: "esperada string de data".to_string(),
                    });
                }
            }

            // referenceDate
            match m.get("referenceDate") {
                None | Some(Value::Null) => {
                    errors.push(ValidationErrorDetail {
                        path: "metadata.referenceDate".to_string(),
                        message: "campo obrigatório ausente".to_string(),
                    });
                }
                Some(Value::String(s)) => {
                    if !is_valid_date(s) {
                        errors.push(ValidationErrorDetail {
                            path: "metadata.referenceDate".to_string(),
                            message: "data inválida (esperado formato ISO 8601 / RFC 3339)"
                                .to_string(),
                        });
                    }
                }
                Some(_) => {
                    errors.push(ValidationErrorDetail {
                        path: "metadata.referenceDate".to_string(),
                        message: "esperada string de data".to_string(),
                    });
                }
            }

            // authenticatedUser
            match m.get("authenticatedUser") {
                None | Some(Value::Null) => {
                    errors.push(ValidationErrorDetail {
                        path: "metadata.authenticatedUser".to_string(),
                        message: "campo obrigatório ausente".to_string(),
                    });
                }
                Some(Value::String(s)) => {
                    if s.trim().is_empty() {
                        errors.push(ValidationErrorDetail {
                            path: "metadata.authenticatedUser".to_string(),
                            message: "campo obrigatório ausente".to_string(),
                        });
                    }
                }
                Some(_) => {
                    errors.push(ValidationErrorDetail {
                        path: "metadata.authenticatedUser".to_string(),
                        message: "esperada string de usuário".to_string(),
                    });
                }
            }

            // totalRepositories
            match m.get("totalRepositories") {
                None | Some(Value::Null) => {
                    errors.push(ValidationErrorDetail {
                        path: "metadata.totalRepositories".to_string(),
                        message: "campo obrigatório ausente".to_string(),
                    });
                }
                Some(v) => {
                    if let Some(total) = v.as_u64() {
                        declared_total_repos = Some(total);
                    } else {
                        errors.push(ValidationErrorDetail {
                            path: "metadata.totalRepositories".to_string(),
                            message: "número inteiro esperado".to_string(),
                        });
                    }
                }
            }
        }
        Some(_) => {
            errors.push(ValidationErrorDetail {
                path: "metadata".to_string(),
                message: "esperado objeto JSON".to_string(),
            });
        }
    }

    // 2. Validação de Repositórios
    let mut actual_repos_count = 0;
    match root_obj.get("repositories") {
        None | Some(Value::Null) => {
            errors.push(ValidationErrorDetail {
                path: "repositories".to_string(),
                message: "campo obrigatório ausente".to_string(),
            });
        }
        Some(Value::Array(repos)) => {
            actual_repos_count = repos.len();
            let mut seen_repo_ids = HashSet::new();

            for (i, repo_val) in repos.iter().enumerate() {
                validate_repository_entry(i, repo_val, &mut seen_repo_ids, &mut errors);
            }
        }
        Some(_) => {
            errors.push(ValidationErrorDetail {
                path: "repositories".to_string(),
                message: "esperado array de repositórios".to_string(),
            });
        }
    }

    // Verificação de divergência em totalRepositories
    if let Some(declared) = declared_total_repos.filter(|&d| d != actual_repos_count as u64) {
        errors.push(ValidationErrorDetail {
            path: "metadata.totalRepositories".to_string(),
            message: format!(
                "total de repositórios diverge do tamanho da lista de repositórios (declarado: {}, encontrado: {})",
                declared, actual_repos_count
            ),
        });
    }

    // 3. Validação de Organizações
    match root_obj.get("organizations") {
        None | Some(Value::Null) => {
            errors.push(ValidationErrorDetail {
                path: "organizations".to_string(),
                message: "campo obrigatório ausente".to_string(),
            });
        }
        Some(Value::Array(orgs)) => {
            let mut seen_org_ids = HashSet::new();
            for (i, org_val) in orgs.iter().enumerate() {
                validate_organization_entry(i, org_val, &mut seen_org_ids, &mut errors);
            }
        }
        Some(_) => {
            errors.push(ValidationErrorDetail {
                path: "organizations".to_string(),
                message: "esperado array de organizações".to_string(),
            });
        }
    }

    // 4. Validação de Relacionamentos
    match root_obj.get("relationships") {
        None | Some(Value::Null) => {
            errors.push(ValidationErrorDetail {
                path: "relationships".to_string(),
                message: "campo obrigatório ausente".to_string(),
            });
        }
        Some(Value::Array(rels)) => {
            for (i, rel_val) in rels.iter().enumerate() {
                validate_relationship_entry("relationships", i, rel_val, &mut errors);
            }
        }
        Some(_) => {
            errors.push(ValidationErrorDetail {
                path: "relationships".to_string(),
                message: "esperado array de relacionamentos".to_string(),
            });
        }
    }

    // 5. Validação opcional de UserAnnotations aninhadas
    if let Some(user_ann_val) = root_obj.get("userAnnotations").filter(|v| !v.is_null()) {
        validate_annotations_structure("userAnnotations", user_ann_val, &mut errors);
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    // Desserialização segura para modelo Snapshot após validação completa
    serde_json::from_value::<Snapshot>(value.clone()).map_err(|e| {
        vec![ValidationErrorDetail {
            path: "".to_string(),
            message: format!("falha na desserialização do snapshot: {}", e),
        }]
    })
}

fn validate_repository_entry(
    i: usize,
    repo_val: &Value,
    seen_repo_ids: &mut HashSet<u64>,
    errors: &mut Vec<ValidationErrorDetail>,
) {
    let repo_obj = match repo_val.as_object() {
        Some(obj) => obj,
        None => {
            errors.push(ValidationErrorDetail {
                path: format!("repositories[{}]", i),
                message: "esperado objeto JSON".to_string(),
            });
            return;
        }
    };

    // id
    match repo_obj.get("id") {
        Some(v) => {
            if let Some(id) = v.as_u64() {
                if id == 0 {
                    errors.push(ValidationErrorDetail {
                        path: format!("repositories[{}].id", i),
                        message: "ID inválido".to_string(),
                    });
                } else if !seen_repo_ids.insert(id) {
                    errors.push(ValidationErrorDetail {
                        path: format!("repositories[{}].id", i),
                        message: format!("ID de repositório duplicado: {}", id),
                    });
                }
            } else {
                errors.push(ValidationErrorDetail {
                    path: format!("repositories[{}].id", i),
                    message: "ID inválido".to_string(),
                });
            }
        }
        None => {
            errors.push(ValidationErrorDetail {
                path: format!("repositories[{}].id", i),
                message: "campo obrigatório ausente".to_string(),
            });
        }
    }

    // name
    match repo_obj.get("name") {
        Some(Value::String(s)) if !s.trim().is_empty() => {}
        _ => {
            errors.push(ValidationErrorDetail {
                path: format!("repositories[{}].name", i),
                message: "campo obrigatório ausente".to_string(),
            });
        }
    }

    // fullName
    match repo_obj.get("fullName") {
        Some(Value::String(s)) if !s.trim().is_empty() => {}
        _ => {
            errors.push(ValidationErrorDetail {
                path: format!("repositories[{}].fullName", i),
                message: "campo obrigatório ausente".to_string(),
            });
        }
    }

    // owner
    match repo_obj.get("owner") {
        Some(Value::Object(owner)) => {
            match owner.get("login") {
                Some(Value::String(s)) if !s.trim().is_empty() => {}
                _ => {
                    errors.push(ValidationErrorDetail {
                        path: format!("repositories[{}].owner.login", i),
                        message: "campo obrigatório ausente".to_string(),
                    });
                }
            }
            match owner.get("id").and_then(|v| v.as_u64()) {
                Some(oid) if oid > 0 => {}
                _ => {
                    errors.push(ValidationErrorDetail {
                        path: format!("repositories[{}].owner.id", i),
                        message: "ID inválido".to_string(),
                    });
                }
            }
            match owner.get("type").and_then(|v| v.as_str()) {
                Some("User") | Some("Organization") => {}
                _ => {
                    errors.push(ValidationErrorDetail {
                        path: format!("repositories[{}].owner.type", i),
                        message: "tipo de proprietário inválido".to_string(),
                    });
                }
            }
        }
        _ => {
            errors.push(ValidationErrorDetail {
                path: format!("repositories[{}].owner", i),
                message: "campo obrigatório ausente".to_string(),
            });
        }
    }

    // Datas: createdAt, updatedAt, pushedAt
    let date_fields = ["createdAt", "updatedAt", "pushedAt"];
    for field in date_fields {
        match repo_obj.get(field) {
            Some(Value::String(s)) => {
                if !is_valid_date(s) {
                    errors.push(ValidationErrorDetail {
                        path: format!("repositories[{}].{}", i, field),
                        message: "data inválida (esperado formato ISO 8601 / RFC 3339)".to_string(),
                    });
                }
            }
            _ => {
                errors.push(ValidationErrorDetail {
                    path: format!("repositories[{}].{}", i, field),
                    message: "campo de data obrigatório ausente ou inválido".to_string(),
                });
            }
        }
    }

    // affiliation
    match repo_obj.get("affiliation").and_then(|v| v.as_str()) {
        Some("owner") | Some("collaborator") | Some("organization_member") => {}
        _ => {
            errors.push(ValidationErrorDetail {
                path: format!("repositories[{}].affiliation", i),
                message: "afiliação inválida".to_string(),
            });
        }
    }

    // indicators (opcionais)
    if let Some(Value::Object(ind_obj)) = repo_obj.get("indicators") {
        // overallQualityScore
        match ind_obj.get("overallQualityScore") {
            Some(v) => {
                if let Some(score) = v.as_i64() {
                    if !(0..=100).contains(&score) {
                        errors.push(ValidationErrorDetail {
                            path: format!("repositories[{}].indicators.overallQualityScore", i),
                            message: "escore fora dos limites permitidos (0-100)".to_string(),
                        });
                    }
                } else {
                    errors.push(ValidationErrorDetail {
                        path: format!("repositories[{}].indicators.overallQualityScore", i),
                        message: "escore deve ser numérico".to_string(),
                    });
                }
            }
            None => {
                errors.push(ValidationErrorDetail {
                    path: format!("repositories[{}].indicators.overallQualityScore", i),
                    message: "campo obrigatório ausente".to_string(),
                });
            }
        }

        let indicator_fields = [
            "testInfrastructure",
            "ciConfiguration",
            "documentation",
            "activityRecency",
            "maintenanceSignals",
        ];

        for ind_name in indicator_fields {
            if let Some(Value::Object(detail)) = ind_obj.get(ind_name) {
                // score
                match detail.get("score") {
                    Some(v) => {
                        if let Some(score) = v.as_i64() {
                            if !(0..=100).contains(&score) {
                                errors.push(ValidationErrorDetail {
                                    path: format!(
                                        "repositories[{}].indicators.{}.score",
                                        i, ind_name
                                    ),
                                    message: "escore fora dos limites permitidos (0-100)"
                                        .to_string(),
                                });
                            }
                        } else {
                            errors.push(ValidationErrorDetail {
                                path: format!("repositories[{}].indicators.{}.score", i, ind_name),
                                message: "escore deve ser numérico".to_string(),
                            });
                        }
                    }
                    None => {
                        errors.push(ValidationErrorDetail {
                            path: format!("repositories[{}].indicators.{}.score", i, ind_name),
                            message: "campo obrigatório ausente".to_string(),
                        });
                    }
                }

                // status
                match detail.get("status").and_then(|v| v.as_str()) {
                    Some("passed") | Some("failed") | Some("unknown") | Some("partial") => {}
                    _ => {
                        errors.push(ValidationErrorDetail {
                            path: format!("repositories[{}].indicators.{}.status", i, ind_name),
                            message: "status de indicador inválido".to_string(),
                        });
                    }
                }
            }
        }
    }
}

fn validate_organization_entry(
    i: usize,
    org_val: &Value,
    seen_org_ids: &mut HashSet<u64>,
    errors: &mut Vec<ValidationErrorDetail>,
) {
    let org_obj = match org_val.as_object() {
        Some(obj) => obj,
        None => {
            errors.push(ValidationErrorDetail {
                path: format!("organizations[{}]", i),
                message: "esperado objeto JSON".to_string(),
            });
            return;
        }
    };

    match org_obj.get("id") {
        Some(v) => {
            if let Some(id) = v.as_u64() {
                if id == 0 {
                    errors.push(ValidationErrorDetail {
                        path: format!("organizations[{}].id", i),
                        message: "ID inválido".to_string(),
                    });
                } else if !seen_org_ids.insert(id) {
                    errors.push(ValidationErrorDetail {
                        path: format!("organizations[{}].id", i),
                        message: format!("ID de organização duplicado: {}", id),
                    });
                }
            } else {
                errors.push(ValidationErrorDetail {
                    path: format!("organizations[{}].id", i),
                    message: "ID inválido".to_string(),
                });
            }
        }
        None => {
            errors.push(ValidationErrorDetail {
                path: format!("organizations[{}].id", i),
                message: "campo obrigatório ausente".to_string(),
            });
        }
    }

    match org_obj.get("login") {
        Some(Value::String(s)) if !s.trim().is_empty() => {}
        _ => {
            errors.push(ValidationErrorDetail {
                path: format!("organizations[{}].login", i),
                message: "campo obrigatório ausente".to_string(),
            });
        }
    }
}

fn validate_relationship_entry(
    prefix: &str,
    i: usize,
    rel_val: &Value,
    errors: &mut Vec<ValidationErrorDetail>,
) {
    let rel_obj = match rel_val.as_object() {
        Some(obj) => obj,
        None => {
            errors.push(ValidationErrorDetail {
                path: format!("{}[{}]", prefix, i),
                message: "esperado objeto JSON".to_string(),
            });
            return;
        }
    };

    // id
    match rel_obj.get("id") {
        Some(Value::String(s)) if !s.trim().is_empty() => {}
        _ => {
            errors.push(ValidationErrorDetail {
                path: format!("{}[{}].id", prefix, i),
                message: "campo obrigatório ausente".to_string(),
            });
        }
    }

    // source
    match rel_obj.get("source") {
        Some(Value::String(s)) if !s.trim().is_empty() => {}
        _ => {
            errors.push(ValidationErrorDetail {
                path: format!("{}[{}].source", prefix, i),
                message: "campo obrigatório ausente".to_string(),
            });
        }
    }

    // target
    match rel_obj.get("target") {
        Some(Value::String(s)) if !s.trim().is_empty() => {}
        _ => {
            errors.push(ValidationErrorDetail {
                path: format!("{}[{}].target", prefix, i),
                message: "campo obrigatório ausente".to_string(),
            });
        }
    }

    // sourceType
    match rel_obj.get("sourceType").and_then(|v| v.as_str()) {
        Some("repository") | Some("organization") => {}
        _ => {
            errors.push(ValidationErrorDetail {
                path: format!("{}[{}].sourceType", prefix, i),
                message: "tipo de nó de origem inválido".to_string(),
            });
        }
    }

    // targetType
    match rel_obj.get("targetType").and_then(|v| v.as_str()) {
        Some("repository") | Some("organization") => {}
        _ => {
            errors.push(ValidationErrorDetail {
                path: format!("{}[{}].targetType", prefix, i),
                message: "tipo de nó de destino inválido".to_string(),
            });
        }
    }

    // type
    match rel_obj.get("type").and_then(|v| v.as_str()) {
        Some("belongs_to") | Some("references") | Some("depends_on") | Some("fork_of")
        | Some("custom") => {}
        _ => {
            errors.push(ValidationErrorDetail {
                path: format!("{}[{}].type", prefix, i),
                message: "tipo de relacionamento inválido".to_string(),
            });
        }
    }

    // provenance
    match rel_obj.get("provenance").and_then(|v| v.as_str()) {
        Some("automatic") | Some("manual") => {}
        _ => {
            errors.push(ValidationErrorDetail {
                path: format!("{}[{}].provenance", prefix, i),
                message: "proveniência de relacionamento inválida".to_string(),
            });
        }
    }

    // createdAt
    match rel_obj.get("createdAt") {
        Some(Value::String(s)) => {
            if !is_valid_date(s) {
                errors.push(ValidationErrorDetail {
                    path: format!("{}[{}].createdAt", prefix, i),
                    message: "data inválida (esperado formato ISO 8601 / RFC 3339)".to_string(),
                });
            }
        }
        _ => {
            errors.push(ValidationErrorDetail {
                path: format!("{}[{}].createdAt", prefix, i),
                message: "campo obrigatório ausente".to_string(),
            });
        }
    }

    // updatedAt (opcional)
    if rel_obj
        .get("updatedAt")
        .and_then(|v| v.as_str())
        .is_some_and(|s| !s.is_empty() && !is_valid_date(s))
    {
        errors.push(ValidationErrorDetail {
            path: format!("{}[{}].updatedAt", prefix, i),
            message: "data inválida (esperado formato ISO 8601 / RFC 3339)".to_string(),
        });
    }
}

fn validate_tombstone_entry(
    prefix: &str,
    i: usize,
    t_val: &Value,
    errors: &mut Vec<ValidationErrorDetail>,
) {
    let t_obj = match t_val.as_object() {
        Some(obj) => obj,
        None => {
            errors.push(ValidationErrorDetail {
                path: format!("{}[{}]", prefix, i),
                message: "esperado objeto JSON".to_string(),
            });
            return;
        }
    };

    // id, source, target
    for field in ["id", "source", "target"] {
        match t_obj.get(field) {
            Some(Value::String(s)) if !s.trim().is_empty() => {}
            _ => {
                errors.push(ValidationErrorDetail {
                    path: format!("{}[{}].{}", prefix, i, field),
                    message: "campo obrigatório ausente".to_string(),
                });
            }
        }
    }

    // type
    match t_obj.get("type").and_then(|v| v.as_str()) {
        Some("belongs_to") | Some("references") | Some("depends_on") | Some("fork_of")
        | Some("custom") => {}
        _ => {
            errors.push(ValidationErrorDetail {
                path: format!("{}[{}].type", prefix, i),
                message: "tipo de relacionamento inválido".to_string(),
            });
        }
    }

    // deletedAt
    match t_obj.get("deletedAt") {
        Some(Value::String(s)) => {
            if !is_valid_date(s) {
                errors.push(ValidationErrorDetail {
                    path: format!("{}[{}].deletedAt", prefix, i),
                    message: "data inválida (esperado formato ISO 8601 / RFC 3339)".to_string(),
                });
            }
        }
        _ => {
            errors.push(ValidationErrorDetail {
                path: format!("{}[{}].deletedAt", prefix, i),
                message: "campo obrigatório ausente".to_string(),
            });
        }
    }
}

fn validate_potential_assessment_entry(
    prefix: &str,
    key: &str,
    assess_val: &Value,
    errors: &mut Vec<ValidationErrorDetail>,
) {
    let assess_obj = match assess_val.as_object() {
        Some(obj) => obj,
        None => {
            errors.push(ValidationErrorDetail {
                path: format!("{}.{}", prefix, key),
                message: "esperado objeto de avaliação".to_string(),
            });
            return;
        }
    };

    // repositoryId
    match assess_obj.get("repositoryId").and_then(|v| v.as_u64()) {
        Some(id) if id > 0 => {}
        _ => {
            errors.push(ValidationErrorDetail {
                path: format!("{}.{}.repositoryId", prefix, key),
                message: "ID de repositório inválido".to_string(),
            });
        }
    }

    // strategicValue
    match assess_obj.get("strategicValue").and_then(|v| v.as_str()) {
        Some("low") | Some("medium") | Some("high") | Some("critical") => {}
        _ => {
            errors.push(ValidationErrorDetail {
                path: format!("{}.{}.strategicValue", prefix, key),
                message: "valor estratégico inválido".to_string(),
            });
        }
    }

    // commercialPotential
    match assess_obj
        .get("commercialPotential")
        .and_then(|v| v.as_str())
    {
        Some("none") | Some("experimental") | Some("viable") | Some("production_ready") => {}
        _ => {
            errors.push(ValidationErrorDetail {
                path: format!("{}.{}.commercialPotential", prefix, key),
                message: "potencial comercial inválido".to_string(),
            });
        }
    }

    // updatedAt
    match assess_obj.get("updatedAt") {
        Some(Value::String(s)) => {
            if !is_valid_date(s) {
                errors.push(ValidationErrorDetail {
                    path: format!("{}.{}.updatedAt", prefix, key),
                    message: "data inválida (esperado formato ISO 8601 / RFC 3339)".to_string(),
                });
            }
        }
        _ => {
            errors.push(ValidationErrorDetail {
                path: format!("{}.{}.updatedAt", prefix, key),
                message: "campo obrigatório ausente".to_string(),
            });
        }
    }
}

fn validate_annotations_structure(
    prefix: &str,
    val: &Value,
    errors: &mut Vec<ValidationErrorDetail>,
) {
    let obj = match val.as_object() {
        Some(o) => o,
        None => {
            errors.push(ValidationErrorDetail {
                path: prefix.to_string(),
                message: "esperado objeto JSON".to_string(),
            });
            return;
        }
    };

    // schemaVersion
    match obj.get("schemaVersion") {
        None | Some(Value::Null) => {
            errors.push(ValidationErrorDetail {
                path: build_path(prefix, "schemaVersion"),
                message: "campo obrigatório ausente".to_string(),
            });
        }
        Some(Value::String(s)) => {
            if s != "1.0" {
                errors.push(ValidationErrorDetail {
                    path: build_path(prefix, "schemaVersion"),
                    message: format!(
                        "versão incompatível do schema: esperado 1.0, encontrado {}",
                        s
                    ),
                });
            }
        }
        Some(_) => {
            errors.push(ValidationErrorDetail {
                path: build_path(prefix, "schemaVersion"),
                message: "esperada string".to_string(),
            });
        }
    }

    // exportedAt
    match obj.get("exportedAt") {
        None | Some(Value::Null) => {
            errors.push(ValidationErrorDetail {
                path: build_path(prefix, "exportedAt"),
                message: "campo obrigatório ausente".to_string(),
            });
        }
        Some(Value::String(s)) => {
            if !is_valid_date(s) {
                errors.push(ValidationErrorDetail {
                    path: build_path(prefix, "exportedAt"),
                    message: "data inválida (esperado formato ISO 8601 / RFC 3339)".to_string(),
                });
            }
        }
        Some(_) => {
            errors.push(ValidationErrorDetail {
                path: build_path(prefix, "exportedAt"),
                message: "esperada string de data".to_string(),
            });
        }
    }

    // manualRelationships: default to [] if missing/null
    if let Some(mr_val) = obj.get("manualRelationships").filter(|v| !v.is_null()) {
        if let Some(arr) = mr_val.as_array() {
            let mr_path = build_path(prefix, "manualRelationships");
            for (i, rel_val) in arr.iter().enumerate() {
                validate_relationship_entry(&mr_path, i, rel_val, errors);
            }
        } else {
            errors.push(ValidationErrorDetail {
                path: build_path(prefix, "manualRelationships"),
                message: "esperado array".to_string(),
            });
        }
    }

    // deletedRelationships: default to [] if missing/null
    if let Some(dr_val) = obj.get("deletedRelationships").filter(|v| !v.is_null()) {
        if let Some(arr) = dr_val.as_array() {
            let dr_path = build_path(prefix, "deletedRelationships");
            for (i, t_val) in arr.iter().enumerate() {
                validate_tombstone_entry(&dr_path, i, t_val, errors);
            }
        } else {
            errors.push(ValidationErrorDetail {
                path: build_path(prefix, "deletedRelationships"),
                message: "esperado array".to_string(),
            });
        }
    }

    // potentialAssessments: default to {} if missing/null
    if let Some(pa_val) = obj.get("potentialAssessments").filter(|v| !v.is_null()) {
        if let Some(pa_obj) = pa_val.as_object() {
            let pa_path = build_path(prefix, "potentialAssessments");
            for (k, assess_val) in pa_obj {
                validate_potential_assessment_entry(&pa_path, k, assess_val, errors);
            }
        } else {
            errors.push(ValidationErrorDetail {
                path: build_path(prefix, "potentialAssessments"),
                message: "esperado objeto".to_string(),
            });
        }
    }
}

/// Valida anotações do usuário (`UserAnnotations 1.0`) a partir de uma referência a `serde_json::Value`,
/// normalizando automaticamente coleções ausentes para seus padrões vazios (`[]` ou `{}`).
pub fn validate_annotations(
    value: &Value,
) -> std::result::Result<UserAnnotations, Vec<ValidationErrorDetail>> {
    let mut errors = Vec::new();

    let obj = match value.as_object() {
        Some(o) => o,
        None => {
            errors.push(ValidationErrorDetail {
                path: "".to_string(),
                message: "esperado objeto JSON na raiz".to_string(),
            });
            return Err(errors);
        }
    };

    validate_annotations_structure("", value, &mut errors);

    if !errors.is_empty() {
        return Err(errors);
    }

    // Normaliza coleções ausentes para permitir deserialização padrão
    let mut normalized = obj.clone();
    if !normalized.contains_key("manualRelationships")
        || normalized.get("manualRelationships") == Some(&Value::Null)
    {
        normalized.insert("manualRelationships".to_string(), Value::Array(Vec::new()));
    }
    if !normalized.contains_key("deletedRelationships")
        || normalized.get("deletedRelationships") == Some(&Value::Null)
    {
        normalized.insert("deletedRelationships".to_string(), Value::Array(Vec::new()));
    }
    if !normalized.contains_key("potentialAssessments")
        || normalized.get("potentialAssessments") == Some(&Value::Null)
    {
        normalized.insert(
            "potentialAssessments".to_string(),
            Value::Object(serde_json::Map::new()),
        );
    }

    serde_json::from_value::<UserAnnotations>(Value::Object(normalized)).map_err(|e| {
        vec![ValidationErrorDetail {
            path: "".to_string(),
            message: format!("falha na desserialização de anotações: {}", e),
        }]
    })
}
