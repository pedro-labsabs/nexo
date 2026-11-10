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

    struct MockGitHubClient;

    impl GitHubClient for MockGitHubClient {
        async fn fetch_repository(&self, owner: &str, repo: &str) -> Result<Repository> {
            Ok(Repository::new(42, repo, format!("{owner}/{repo}")))
        }
    }

    #[test]
    fn default_sync_status_is_disconnected() {
        assert_eq!(SyncStatus::default(), SyncStatus::Disconnected);
    }

    #[test]
    fn mock_client_returns_domain_repository_contract() {
        let client = MockGitHubClient;
        let fut = client.fetch_repository("pedro-labsabs", "nex0");
        let mut pinned = std::pin::pin!(fut);
        let waker = std::task::Waker::noop();
        let mut cx = std::task::Context::from_waker(waker);
        match std::future::Future::poll(pinned.as_mut(), &mut cx) {
            std::task::Poll::Ready(Ok(repo)) => {
                assert_eq!(repo.id, 42);
                assert_eq!(repo.name, "nex0");
                assert_eq!(repo.full_name, "pedro-labsabs/nex0");
                assert_eq!(repo.owner.login, "pedro-labsabs");
            }
            _ => panic!("Esperado que o mock retorne imediatamente Ready(Ok(repo))"),
        }
    }
}
