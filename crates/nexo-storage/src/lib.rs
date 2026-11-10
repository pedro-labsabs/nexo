//! Fronteira de persistência para o NEXO.
//!
//! Esta crate define a abstração para armazenamento local de snapshots,
//! anotações e preferências do usuário.
//!
//! NOTA: A implementação completa com isolamento de identidade, LWW,
//! tombstones e relatórios pertence ao escopo da Issue #4.

use nexo_domain::{NexoError, Repository, Result};
use std::collections::HashMap;

/// Trait que define o contrato de persistência do NEXO.
pub trait Storage: Send + Sync {
    /// Salva um repositório no armazenamento local.
    fn save_repository(&mut self, repo: Repository) -> Result<()>;

    /// Recupera um repositório pelo ID.
    fn get_repository(&self, id: &str) -> Result<Option<Repository>>;

    /// Lista todos os repositórios armazenados.
    fn list_repositories(&self) -> Result<Vec<Repository>>;

    /// Remove um repositório pelo ID.
    fn delete_repository(&mut self, id: &str) -> Result<bool>;
}

/// Implementação em memória para testes e prototipação nesta etapa da M1.
#[derive(Debug, Default, Clone)]
pub struct InMemoryStorage {
    repositories: HashMap<String, Repository>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Storage for InMemoryStorage {
    fn save_repository(&mut self, repo: Repository) -> Result<()> {
        if repo.id.trim().is_empty() {
            return Err(NexoError::ValidationError(
                "ID do repositório não pode ser vazio".into(),
            ));
        }
        self.repositories.insert(repo.id.clone(), repo);
        Ok(())
    }

    fn get_repository(&self, id: &str) -> Result<Option<Repository>> {
        Ok(self.repositories.get(id).cloned())
    }

    fn list_repositories(&self) -> Result<Vec<Repository>> {
        Ok(self.repositories.values().cloned().collect())
    }

    fn delete_repository(&mut self, id: &str) -> Result<bool> {
        Ok(self.repositories.remove(id).is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_storage_crud_lifecycle() {
        let mut storage = InMemoryStorage::new();
        assert_eq!(storage.list_repositories().unwrap().len(), 0);

        let repo = Repository::new("r1", "nexo", "pedro-labsabs/nexo");
        storage.save_repository(repo.clone()).unwrap();

        assert_eq!(storage.list_repositories().unwrap().len(), 1);
        assert_eq!(storage.get_repository("r1").unwrap(), Some(repo));

        let deleted = storage.delete_repository("r1").unwrap();
        assert!(deleted);
        assert_eq!(storage.get_repository("r1").unwrap(), None);
    }

    #[test]
    fn reject_empty_id() {
        let mut storage = InMemoryStorage::new();
        let invalid = Repository::new("", "empty", "empty/empty");
        assert!(storage.save_repository(invalid).is_err());
    }
}
