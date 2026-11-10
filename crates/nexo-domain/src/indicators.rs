use crate::model::{Repository, TreeSignals};
use serde::{Deserialize, Serialize};

/// Status determinístico de um indicador técnico.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndicatorStatus {
    Passed,
    Failed,
    Unknown,
    Partial,
}

/// Detalhes estruturados de um indicador técnico individual.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndicatorDetail {
    pub id: String,
    pub name: String,
    pub status: IndicatorStatus,
    pub score: u32,
    pub summary: String,
    pub inputs: serde_json::Value,
    pub calculation: String,
    pub limitations: Vec<String>,
}

/// Conjunto completo de indicadores técnicos de um repositório e escore agregado.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryIndicators {
    pub test_infrastructure: IndicatorDetail,
    pub ci_configuration: IndicatorDetail,
    pub documentation: IndicatorDetail,
    pub activity_recency: IndicatorDetail,
    pub maintenance_signals: IndicatorDetail,
    pub overall_quality_score: u32,
}

fn parse_date_to_millis(date_str: &str) -> Option<i64> {
    if date_str.trim().is_empty() {
        return None;
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
        return Some(dt.timestamp_millis());
    }
    if let Ok(ndt) = chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S") {
        return Some(ndt.and_utc().timestamp_millis());
    }
    if let Ok(nd) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return Some(nd.and_hms_opt(0, 0, 0)?.and_utc().timestamp_millis());
    }
    None
}

fn compute_test_infrastructure(tree: &TreeSignals) -> IndicatorDetail {
    if !tree.tree_inspected {
        return IndicatorDetail {
            id: "test_infrastructure".into(),
            name: "Infraestrutura de Testes".into(),
            status: IndicatorStatus::Unknown,
            score: 0,
            summary: "Árvore de arquivos não inspecionada ou inacessível via API.".into(),
            inputs: serde_json::json!({
                "treeInspected": false,
                "inspectionError": tree.inspection_error,
            }),
            calculation: "Nenhum dado de arquivo disponível para inspeção.".into(),
            limitations: vec![
                "A ausência de acesso à árvore impede checar arquivos de testes.".into(),
                "Não foi presumido resultado negativo.".into(),
            ],
        };
    }

    if tree.tree_truncated && !tree.has_tests {
        return IndicatorDetail {
            id: "test_infrastructure".into(),
            name: "Infraestrutura de Testes".into(),
            status: IndicatorStatus::Unknown,
            score: 0,
            summary: "Inspeção parcial: a árvore de arquivos foi truncada pelo GitHub e arquivos de teste podem ter sido omitidos.".into(),
            inputs: serde_json::json!({
                "hasTests": false,
                "treeTruncated": true,
                "evidenceFound": [],
            }),
            calculation: "Informação inconclusiva/desconhecida devido a truncamento de árvore.".into(),
            limitations: vec![
                "A árvore de arquivos ultrapassou o limite do GitHub e foi truncada.".into(),
                "A ausência de testes não é comprovada; arquivos em diretórios profundos podem existir.".into(),
            ],
        };
    }

    let count = tree.test_evidence.len();
    let mut status = IndicatorStatus::Failed;
    let mut score = 0;
    let mut summary =
        "Nenhuma configuração ou diretório de testes detectado na árvore inspecionada.".to_string();

    if tree.has_tests && count >= 2 {
        status = IndicatorStatus::Passed;
        score = 100;
        let joined = tree
            .test_evidence
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        summary = format!("Detectados múltiplos sinais de testes: {joined}.");
    } else if tree.has_tests && count == 1 {
        status = IndicatorStatus::Partial;
        score = 70;
        summary = format!("Detectado sinal de teste: {}.", tree.test_evidence[0]);
    }

    let mut limitations = vec![
        "A presença de arquivos ou pastas de teste NÃO garante que os testes foram executados ou que estejam passando.".into(),
    ];
    if tree.tree_truncated {
        limitations.push("Árvore de arquivos truncada: evidência observada parcialmente.".into());
    }

    IndicatorDetail {
        id: "test_infrastructure".into(),
        name: "Infraestrutura de Testes".into(),
        status,
        score,
        summary,
        inputs: serde_json::json!({
            "hasTests": tree.has_tests,
            "evidenceFound": tree.test_evidence,
            "treeTruncated": tree.tree_truncated,
        }),
        calculation: "Escore de 100 para >=2 evidências, 70 para 1 evidência, 0 para nenhuma."
            .into(),
        limitations,
    }
}

fn compute_ci_configuration(tree: &TreeSignals) -> IndicatorDetail {
    if !tree.tree_inspected {
        return IndicatorDetail {
            id: "ci_configuration".into(),
            name: "Configuração de CI/CD".into(),
            status: IndicatorStatus::Unknown,
            score: 0,
            summary: "Árvore de arquivos não inspecionada ou inacessível.".into(),
            inputs: serde_json::json!({
                "treeInspected": false,
            }),
            calculation: "Nenhum dado de arquivo disponível.".into(),
            limitations: vec![
                "Sem acesso aos arquivos do repositório para verificar diretórios de workflows."
                    .into(),
            ],
        };
    }

    if tree.tree_truncated && !tree.has_ci {
        return IndicatorDetail {
            id: "ci_configuration".into(),
            name: "Configuração de CI/CD".into(),
            status: IndicatorStatus::Unknown,
            score: 0,
            summary: "Inspeção parcial: árvore truncada e diretórios de CI podem ter sido omitidos da resposta.".into(),
            inputs: serde_json::json!({
                "hasCI": false,
                "treeTruncated": true,
            }),
            calculation: "Informação inconclusiva devido ao truncamento da árvore.".into(),
            limitations: vec![
                "A ausência de fluxos não é comprovada em árvores truncadas pelo GitHub.".into(),
                "Sistemas externos de CI também podem não ter sido verificados.".into(),
            ],
        };
    }

    let mut status = IndicatorStatus::Failed;
    let mut score = 0;
    let mut summary =
        "Nenhum arquivo de automação CI/CD (.github/workflows, etc.) encontrado.".to_string();

    if tree.has_ci {
        status = IndicatorStatus::Passed;
        score = 100;
        let joined = tree.ci_evidence.join(", ");
        summary = format!("Fluxos de automação CI configurados: {joined}.");
    }

    let mut limitations = vec![
        "A existência de workflows não assegura que as execuções de CI estejam com status verde no GitHub.".into(),
        "Sistemas externos de CI (ex: CircleCI externo ou Jenkins sem manifesto no repo) podem não ser identificados.".into(),
    ];
    if tree.tree_truncated {
        limitations
            .push("Árvore de arquivos truncada: evidência observada nos arquivos parciais.".into());
    }

    IndicatorDetail {
        id: "ci_configuration".into(),
        name: "Configuração de CI/CD".into(),
        status,
        score,
        summary,
        inputs: serde_json::json!({
            "hasCI": tree.has_ci,
            "ciEvidence": tree.ci_evidence,
            "treeTruncated": tree.tree_truncated,
        }),
        calculation:
            "Escore de 100 para presença de arquivos de workflow CI detectados, 0 caso contrário."
                .into(),
        limitations,
    }
}

fn compute_documentation(tree: &TreeSignals, repo: &Repository) -> IndicatorDetail {
    let has_desc = repo
        .description
        .as_ref()
        .map(|d| d.trim().len() > 5)
        .unwrap_or(false);

    let inputs = serde_json::json!({
        "hasDescription": has_desc,
        "readmeFound": tree.readme_found,
        "contributingFound": tree.contributing_found,
        "licenseFileFound": tree.license_file_found,
        "docEvidence": tree.documentation_evidence,
        "treeTruncated": tree.tree_truncated,
    });

    if !tree.tree_inspected {
        let (score, summary) = if has_desc {
            (
                40,
                "Descrição básica presente, mas arquivos de documentação não puderam ser verificados.".to_string(),
            )
        } else {
            (0, "Metadados de documentação inacessíveis.".to_string())
        };

        return IndicatorDetail {
            id: "documentation".into(),
            name: "Documentação do Projeto".into(),
            status: IndicatorStatus::Unknown,
            score,
            summary,
            inputs,
            calculation: "Baseado apenas na descrição do repositório devido a árvore inacessível.".into(),
            limitations: vec![
                "Árvore de arquivos inacessível; checagem de README e guias não pôde ser executada.".into(),
            ],
        };
    }

    let mut points: u32 = 0;
    if has_desc {
        points += 20;
    }
    if tree.readme_found {
        points += 40;
    }
    if !tree.documentation_evidence.is_empty() {
        points += 20;
    }
    if tree.contributing_found {
        points += 10;
    }
    if tree.license_file_found {
        points += 10;
    }

    let score = points.min(100);

    let status = if score >= 70 {
        IndicatorStatus::Passed
    } else if score >= 40 {
        IndicatorStatus::Partial
    } else if tree.tree_truncated {
        IndicatorStatus::Unknown
    } else {
        IndicatorStatus::Failed
    };

    let readme_str = if tree.readme_found { "Sim" } else { "Não" };
    let desc_str = if has_desc { "Sim" } else { "Não" };
    let summary = format!(
        "Escore de documentação: {score}/100 (README: {readme_str}, Descrição: {desc_str})."
    );

    let mut limitations = vec![
        "A presença de README.md não garante completude, clareza ou qualidade da documentação técnica.".into(),
        "Não avalia compreensão semântica do conteúdo textual.".into(),
    ];
    if tree.tree_truncated {
        limitations.push(
            "Árvore de arquivos truncada: documentação em subpastas pode ter sido omitida.".into(),
        );
    }

    IndicatorDetail {
        id: "documentation".into(),
        name: "Documentação do Projeto".into(),
        status,
        score,
        summary,
        inputs,
        calculation: "README (+40), Descrição (+20), Docs adicionais (+20), CONTRIBUTING (+10), LICENSE (+10).".into(),
        limitations,
    }
}

fn compute_activity_recency(repo: &Repository, reference_date_str: &str) -> IndicatorDetail {
    let last_push = parse_date_to_millis(&repo.pushed_at).unwrap_or(0);
    if last_push == 0 {
        return IndicatorDetail {
            id: "activity_recency".into(),
            name: "Recência de Atividade".into(),
            status: IndicatorStatus::Unknown,
            score: 0,
            summary: "Data de push inválida ou não informada.".into(),
            inputs: serde_json::json!({ "pushedAt": repo.pushed_at }),
            calculation: "Sem data de referência disponível.".into(),
            limitations: vec![
                "Impossível calcular a recência sem carimbo de data confiável.".into(),
            ],
        };
    }

    let ref_ts = parse_date_to_millis(reference_date_str).unwrap_or(0);
    let diff_ms = (ref_ts - last_push).max(0);
    let diff_days = diff_ms / (1000 * 60 * 60 * 24);

    let (score, status, summary) = if diff_days <= 30 {
        (
            100,
            IndicatorStatus::Passed,
            format!("Atividade muito recente: último push há {diff_days} dias."),
        )
    } else if diff_days <= 90 {
        (
            80,
            IndicatorStatus::Passed,
            format!("Atividade recente: último push há {diff_days} dias."),
        )
    } else if diff_days <= 180 {
        let months = (diff_days as f64 / 30.0).round() as i64;
        (
            60,
            IndicatorStatus::Partial,
            format!("Atividade moderada: último push há {diff_days} dias (~{months} meses)."),
        )
    } else if diff_days <= 365 {
        let months = (diff_days as f64 / 30.0).round() as i64;
        (
            40,
            IndicatorStatus::Partial,
            format!("Atividade baixa: último push há {diff_days} dias (~{months} meses)."),
        )
    } else {
        (
            15,
            IndicatorStatus::Failed,
            format!("Repositório inativo há mais de 1 ano ({diff_days} dias desde o último push)."),
        )
    };

    let ref_iso = chrono::DateTime::from_timestamp_millis(ref_ts)
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
        .unwrap_or_else(|| reference_date_str.to_string());

    IndicatorDetail {
        id: "activity_recency".into(),
        name: "Recência de Atividade".into(),
        status,
        score,
        summary,
        inputs: serde_json::json!({
            "pushedAt": repo.pushed_at,
            "referenceDate": ref_iso,
            "daysSinceLastPush": diff_days,
        }),
        calculation: "<=30d: 100, <=90d: 80, <=180d: 60, <=365d: 40, >365d: 15.".into(),
        limitations: vec![
            "A recência de push mede frequência temporal, não produtividade, estabilidade ou valor do código.".into(),
            "Projetos maduros e estáveis podem ter baixa atividade recente propositalmente.".into(),
        ],
    }
}

fn compute_maintenance_signals(repo: &Repository, tree: &TreeSignals) -> IndicatorDetail {
    let inputs = serde_json::json!({
        "archived": repo.archived,
        "disabled": repo.disabled,
        "openIssuesCount": repo.open_issues_count,
        "sizeKb": repo.size_kb,
        "defaultBranch": repo.default_branch,
    });

    if repo.archived {
        return IndicatorDetail {
            id: "maintenance_signals".into(),
            name: "Sinais de Manutenção".into(),
            status: IndicatorStatus::Failed,
            score: 10,
            summary: "Repositório arquivado (somente leitura).".into(),
            inputs,
            calculation: "Repositórios arquivados recebem pontuação 10.".into(),
            limitations: vec![
                "O arquivamento sinaliza encerramento formal de manutenção no GitHub.".into(),
            ],
        };
    }

    let mut score: i32 = 80;
    if repo.disabled {
        score = 0;
    }
    if repo.size_kb == 0 {
        score -= 20;
    }
    if repo.default_branch == "main" {
        score += 10;
    }
    if tree.license_file_found {
        score += 10;
    }

    let score = score.clamp(0, 100) as u32;
    let status = if score >= 70 {
        IndicatorStatus::Passed
    } else {
        IndicatorStatus::Partial
    };

    let summary = format!(
        "Sinais estruturais e operacionais ativos (Branch: {}, Issues: {}).",
        repo.default_branch, repo.open_issues_count
    );

    IndicatorDetail {
        id: "maintenance_signals".into(),
        name: "Sinais de Manutenção".into(),
        status,
        score,
        summary,
        inputs,
        calculation: "Base 80, bônus para branch main (+10) e licença (+10), penalidades para repo vazio ou desativado.".into(),
        limitations: vec![
            "A quantidade de issues abertas NÃO é prova isolada de baixa qualidade (projetos populares recebem muitas issues).".into(),
            "Tamanho em KB não reflete a complexidade ou elegância da arquitetura.".into(),
        ],
    }
}

/// Computa os 5 indicadores técnicos determinísticos e o escore ponderado de qualidade.
pub fn compute_indicators(repo: &Repository, reference_date_str: &str) -> RepositoryIndicators {
    let tree = &repo.tree_signals;

    let test_infra = compute_test_infrastructure(tree);
    let ci_config = compute_ci_configuration(tree);
    let doc = compute_documentation(tree, repo);
    let activity = compute_activity_recency(repo, reference_date_str);
    let maintenance = compute_maintenance_signals(repo, tree);

    let mut total_score = 0.0;
    let mut total_weight = 0.0;

    if test_infra.status != IndicatorStatus::Unknown {
        total_score += (test_infra.score as f64) * 0.25;
        total_weight += 0.25;
    }
    if ci_config.status != IndicatorStatus::Unknown {
        total_score += (ci_config.score as f64) * 0.20;
        total_weight += 0.20;
    }
    if doc.status != IndicatorStatus::Unknown {
        total_score += (doc.score as f64) * 0.25;
        total_weight += 0.25;
    }
    if activity.status != IndicatorStatus::Unknown {
        total_score += (activity.score as f64) * 0.15;
        total_weight += 0.15;
    }
    if maintenance.status != IndicatorStatus::Unknown {
        total_score += (maintenance.score as f64) * 0.15;
        total_weight += 0.15;
    }

    let overall_quality_score = if total_weight > 0.0 {
        (total_score / total_weight).round() as u32
    } else {
        0
    };

    RepositoryIndicators {
        test_infrastructure: test_infra,
        ci_configuration: ci_config,
        documentation: doc,
        activity_recency: activity,
        maintenance_signals: maintenance,
        overall_quality_score,
    }
}
