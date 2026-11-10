//! Fronteira de persistência para o NEXO.
//!
//! Esta crate define a abstração para armazenamento local de snapshots,
//! anotações e preferências do usuário.
//!
//! NOTA: A implementação completa com isolamento de identidade, LWW,
//! tombstones e relatórios pertence ao escopo da Issue #4.

use nexo_domain::{NexoError, Repository, Result};
use std::collections::BTreeMap;

/// Trait que define o contrato de persistência do NEXO.
pub trait Storage: Send + Sync {
    /// Salva um repositório no armazenamento local.
    fn save_repository(&mut self, repo: Repository) -> Result<()>;

    /// Recupera um repositório pelo ID numérico.
    fn get_repository(&self, id: u64) -> Result<Option<Repository>>;

    /// Lista todos os repositórios armazenados.
    fn list_repositories(&self) -> Result<Vec<Repository>>;

    /// Remove um repositório pelo ID numérico.
    fn delete_repository(&mut self, id: u64) -> Result<bool>;

    /// Recupera um repositório pelo ID em formato string (helper que faz parse para u64).
    fn get_repository_by_str(&self, id: &str) -> Result<Option<Repository>> {
        match id.trim().parse::<u64>() {
            Ok(parsed_id) => self.get_repository(parsed_id),
            Err(_) => Ok(None),
        }
    }

    /// Remove um repositório pelo ID em formato string (helper que faz parse para u64).
    fn delete_repository_by_str(&mut self, id: &str) -> Result<bool> {
        match id.trim().parse::<u64>() {
            Ok(parsed_id) => self.delete_repository(parsed_id),
            Err(_) => Ok(false),
        }
    }
}

/// Implementação em memória para testes e prototipação nesta etapa da M1.
#[derive(Debug, Default, Clone)]
pub struct InMemoryStorage {
    repositories: BTreeMap<u64, Repository>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Storage for InMemoryStorage {
    fn save_repository(&mut self, repo: Repository) -> Result<()> {
        if repo.id == 0 {
            return Err(NexoError::ValidationError(
                "ID do repositório não pode ser zero".into(),
            ));
        }
        self.repositories.insert(repo.id, repo);
        Ok(())
    }

    fn get_repository(&self, id: u64) -> Result<Option<Repository>> {
        Ok(self.repositories.get(&id).cloned())
    }

    fn list_repositories(&self) -> Result<Vec<Repository>> {
        Ok(self.repositories.values().cloned().collect())
    }

    fn delete_repository(&mut self, id: u64) -> Result<bool> {
        Ok(self.repositories.remove(&id).is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_storage_crud_lifecycle() {
        let mut storage = InMemoryStorage::new();
        assert_eq!(storage.list_repositories().unwrap().len(), 0);

        let repo = Repository::new(1, "nexo", "pedro-labsabs/nexo");
        storage.save_repository(repo.clone()).unwrap();

        assert_eq!(storage.list_repositories().unwrap().len(), 1);
        assert_eq!(storage.get_repository(1).unwrap(), Some(repo.clone()));
        assert_eq!(storage.get_repository_by_str("1").unwrap(), Some(repo));

        let deleted = storage.delete_repository(1).unwrap();
        assert!(deleted);
        assert_eq!(storage.get_repository(1).unwrap(), None);
        assert_eq!(storage.get_repository_by_str("1").unwrap(), None);

        // Delete via string helper
        let repo2 = Repository::new(2, "nexo-core", "pedro-labsabs/nexo-core");
        storage.save_repository(repo2).unwrap();
        assert_eq!(storage.list_repositories().unwrap().len(), 1);
        assert!(storage.delete_repository_by_str("2").unwrap());
        assert_eq!(storage.get_repository(2).unwrap(), None);
        assert_eq!(storage.list_repositories().unwrap().len(), 0);
    }

    #[test]
    fn reject_empty_id() {
        let mut storage = InMemoryStorage::new();
        let invalid = Repository::new(0, "empty", "empty/empty");
        assert!(storage.save_repository(invalid).is_err());
    }

    #[test]
    fn string_helper_handles_invalid_formats() {
        let storage = InMemoryStorage::new();
        assert_eq!(storage.get_repository_by_str("").unwrap(), None);
        assert_eq!(storage.get_repository_by_str("invalid").unwrap(), None);
        assert_eq!(storage.get_repository_by_str("-1").unwrap(), None);
    }
}
