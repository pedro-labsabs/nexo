//! Fronteira de sincronização com o GitHub para o NEXO.
//!
//! Esta crate define as abstrações necessárias para futura sincronização
//! somente-leitura em Rust (Issue #7).
//!
//! Durante a fase de transição (M1/M2), a ingestão de snapshots é alimentada
//! pelo CLI Node.js preservado em `src/cli/`.

use nexo_domain::{Repository, Result};

/// Trait que define a fronteira do cliente GitHub (estritamente somente-leitura).
pub trait GitHubClient: Send + Sync {
    /// Obtém os dados de um repositório pelo proprietário e nome.
    fn fetch_repository(
        &self,
        owner: &str,
        repo: &str,
    ) -> impl std::future::Future<Output = Result<Repository>> + Send;
}

/// Estado da conexão com o GitHub.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SyncStatus {
    #[default]
    Disconnected,
    Connecting,
    Idle,
    Syncing,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_sync_status_is_disconnected() {
        assert_eq!(SyncStatus::default(), SyncStatus::Disconnected);
    }
}
