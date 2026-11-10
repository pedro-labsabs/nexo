use crate::relations::{RelationType, Relationship, RelationshipTombstone};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Nível de valor estratégico atribuído manualmente a um repositório.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrategicValue {
    Low,
    Medium,
    High,
    Critical,
}

/// Estágio de potencial comercial atribuído manualmente a um repositório.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommercialPotential {
    None,
    Experimental,
    Viable,
    ProductionReady,
}

/// Avaliação de potencial técnico e comercial de um repositório.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PotentialAssessment {
    pub repository_id: u64,
    pub strategic_value: StrategicValue,
    pub commercial_potential: CommercialPotential,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_audience: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub updated_at: String,
}

/// Camada independente de anotações e curadoria manual do usuário.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UserAnnotations {
    pub schema_version: String, // literal "1.0"
    pub exported_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authenticated_user: Option<String>,
    #[serde(default)]
    pub manual_relationships: Vec<Relationship>,
    #[serde(default)]
    pub deleted_relationships: Vec<RelationshipTombstone>,
    #[serde(default)]
    pub potential_assessments: BTreeMap<String, PotentialAssessment>,
}

/// Valida se uma string de data é um timestamp ISO válido (RFC 3339).
pub fn is_valid_date(date_str: &str) -> bool {
    if date_str.is_empty() {
        return false;
    }
    if DateTime::parse_from_rfc3339(date_str).is_ok() {
        return true;
    }
    date_str.parse::<DateTime<Utc>>().is_ok()
}

/// Extrai timestamp numérico comparável em milissegundos a partir de string ISO,
/// retornando 0 se inválido.
pub fn get_timestamp(date_str: &str) -> i64 {
    if date_str.is_empty() {
        return 0;
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        return dt.timestamp_millis();
    }
    if let Ok(dt) = date_str.parse::<DateTime<Utc>>() {
        return dt.timestamp_millis();
    }
    0
}

/// Gera a chave lógica direcionada estável para relacionamentos: `(source, target, type)`: `source::target::type`.
pub fn get_relationship_logical_key(source: &str, target: &str, type_: &RelationType) -> String {
    format!("{}::{}::{}", source, target, type_.as_str())
}

/// Mescla duas coleções de `PotentialAssessment` deterministamente usando Last-Write-Wins (LWW).
/// Em caso de empate de timestamp ou data inválida, a versão local tem precedência.
pub fn merge_potential_assessments(
    local: &BTreeMap<String, PotentialAssessment>,
    incoming: &BTreeMap<String, PotentialAssessment>,
) -> BTreeMap<String, PotentialAssessment> {
    let mut merged = local.clone();

    for (key, incoming_val) in incoming {
        if let Some(local_val) = local.get(key) {
            let local_time = get_timestamp(&local_val.updated_at);
            let incoming_time = get_timestamp(&incoming_val.updated_at);

            // Se incoming for estritamente mais recente, incoming vence.
            // Caso contrário, local vence (empates e datas inválidas permanecem local).
            if incoming_time > local_time {
                merged.insert(key.clone(), incoming_val.clone());
            } else {
                merged.insert(key.clone(), local_val.clone());
            }
        } else {
            merged.insert(key.clone(), incoming_val.clone());
        }
    }

    merged
}

/// Mescla relacionamentos manuais deterministamente usando:
/// 1. Chave lógica direcionada estável `(source, target, type)`
/// 2. Tombstones (`deleted_relationships`) para impedir ressurreição de excluídos
/// 3. Re-criação permitida se timestamp for estritamente posterior a `deleted_at`
/// 4. Desempate por Last-Write-Wins (LWW) em `updated_at` (ou `created_at`), empates favorecendo local
/// 5. Deduplicação por identificador estável `id`
pub fn merge_manual_relationships(
    local_rels: &[Relationship],
    incoming_rels: &[Relationship],
    local_tombstones: &[RelationshipTombstone],
    incoming_tombstones: &[RelationshipTombstone],
) -> (Vec<Relationship>, Vec<RelationshipTombstone>) {
    // 1. Mescla tombstones deterministamente por chave lógica (LWW)
    let mut tombstone_map: BTreeMap<String, RelationshipTombstone> = BTreeMap::new();

    let mut process_tombstone = |t: &RelationshipTombstone| {
        let key = get_relationship_logical_key(&t.source, &t.target, &t.type_);
        match tombstone_map.get(&key) {
            None => {
                tombstone_map.insert(key, t.clone());
            }
            Some(existing) => {
                let existing_time = get_timestamp(&existing.deleted_at);
                let incoming_time = get_timestamp(&t.deleted_at);
                if incoming_time > existing_time {
                    tombstone_map.insert(key, t.clone());
                }
            }
        }
    };

    for t in local_tombstones {
        process_tombstone(t);
    }
    for t in incoming_tombstones {
        process_tombstone(t);
    }

    // Determina tempo da última modificação do relacionamento
    let get_mod_time = |r: &Relationship| -> i64 {
        let date_str = r.updated_at.as_deref().unwrap_or(&r.created_at);
        get_timestamp(date_str)
    };

    // 2. Indexa relacionamentos ativos por chave lógica
    let mut relation_map: BTreeMap<String, Relationship> = BTreeMap::new();

    // Adiciona relacionamentos locais primeiro
    for r in local_rels {
        let key = get_relationship_logical_key(&r.source, &r.target, &r.type_);
        if let Some(tombstone) = tombstone_map.get(&key) {
            let deleted_time = get_timestamp(&tombstone.deleted_at);
            let mod_time = get_mod_time(r);
            if deleted_time >= mod_time {
                // Relacionamento foi excluído pelo tombstone
                continue;
            }
        }
        relation_map.insert(key, r.clone());
    }

    // Mescla relacionamentos incoming
    for r in incoming_rels {
        let key = get_relationship_logical_key(&r.source, &r.target, &r.type_);
        let incoming_mod_time = get_mod_time(r);

        if let Some(tombstone) = tombstone_map.get(&key) {
            let deleted_time = get_timestamp(&tombstone.deleted_at);
            if deleted_time >= incoming_mod_time {
                // Relacionamento foi excluído pelo tombstone
                continue;
            }
        }

        match relation_map.get(&key) {
            None => {
                relation_map.insert(key, r.clone());
            }
            Some(local_rel) => {
                let local_mod_time = get_mod_time(local_rel);
                // Incoming estritamente mais recente vence; empate ou anterior mantém local
                if incoming_mod_time > local_mod_time {
                    relation_map.insert(key, r.clone());
                }
            }
        }
    }

    // 3. Deduplica por ID caso o mesmo ID tenha sido reutilizado em diferentes tuplas
    let mut id_map: BTreeMap<String, Relationship> = BTreeMap::new();
    for rel in relation_map.into_values() {
        match id_map.get(&rel.id) {
            None => {
                id_map.insert(rel.id.clone(), rel);
            }
            Some(existing) => {
                if get_mod_time(&rel) > get_mod_time(existing) {
                    id_map.insert(rel.id.clone(), rel);
                }
            }
        }
    }

    (
        id_map.into_values().collect(),
        tombstone_map.into_values().collect(),
    )
}

/// Mescla deterministamente dois objetos `UserAnnotations`.
pub fn merge_user_annotations(
    local: &UserAnnotations,
    incoming: &UserAnnotations,
    exported_at_now: Option<&str>,
) -> UserAnnotations {
    let merged_assessments =
        merge_potential_assessments(&local.potential_assessments, &incoming.potential_assessments);

    let (merged_relationships, merged_tombstones) = merge_manual_relationships(
        &local.manual_relationships,
        &incoming.manual_relationships,
        &local.deleted_relationships,
        &incoming.deleted_relationships,
    );

    let exported_at = match exported_at_now {
        Some(ts) => ts.to_string(),
        None => Utc::now().to_rfc3339(),
    };

    let authenticated_user = local
        .authenticated_user
        .clone()
        .or_else(|| incoming.authenticated_user.clone());

    UserAnnotations {
        schema_version: "1.0".to_string(),
        exported_at,
        authenticated_user,
        manual_relationships: merged_relationships,
        deleted_relationships: merged_tombstones,
        potential_assessments: merged_assessments,
    }
}
