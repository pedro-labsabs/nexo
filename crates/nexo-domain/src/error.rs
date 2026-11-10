use crate::validation::ValidationErrorDetail;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum NexoError {
    #[error("Dados de snapshot inválidos: {0}")]
    InvalidSnapshot(String),

    #[error("Versão incompatível do schema: esperado {expected}, encontrado {found}")]
    IncompatibleSchema { expected: String, found: String },

    #[error("Repositório não encontrado: {0}")]
    RepositoryNotFound(String),

    #[error("Erro de validação: {0}")]
    ValidationError(String),

    #[error("Erros de validação ({}): {}", .0.len(), .0.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(", "))]
    ValidationErrors(Vec<ValidationErrorDetail>),

    #[error("Erro de persistência: {0}")]
    StorageError(String),
}

impl From<Vec<ValidationErrorDetail>> for NexoError {
    fn from(errors: Vec<ValidationErrorDetail>) -> Self {
        NexoError::ValidationErrors(errors)
    }
}

pub type Result<T> = std::result::Result<T, NexoError>;
