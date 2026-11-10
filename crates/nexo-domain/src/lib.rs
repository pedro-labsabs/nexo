pub mod error;
pub mod indicators;
pub mod model;
pub mod types;

pub use error::{NexoError, Result};
pub use indicators::{IndicatorDetail, IndicatorStatus, RepositoryIndicators, compute_indicators};
pub use model::{
    Affiliation, Organization, OwnerType, RepositoryOwner, Snapshot, SnapshotMetadata, TreeSignals,
};
pub use types::{NexoState, Repository, ViewMode};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_initializes_correctly() {
        let state = NexoState::default();
        assert_eq!(state.app_version, "0.1.0");
        assert_eq!(state.view_mode, ViewMode::Dashboard);
        assert!(state.dark_theme);
        assert_eq!(state.interaction_count, 0);
        assert_eq!(state.error_message, None);
    }

    #[test]
    fn state_transitions_update_counters() {
        let mut state = NexoState::default();
        state.toggle_theme();
        assert!(!state.dark_theme);
        assert_eq!(state.interaction_count, 1);

        state.set_view_mode(ViewMode::Graph);
        assert_eq!(state.view_mode, ViewMode::Graph);
        assert_eq!(state.interaction_count, 2);

        state.set_error("Falha de teste");
        assert_eq!(state.error_message.as_deref(), Some("Falha de teste"));
        assert_eq!(state.interaction_count, 3);

        state.clear_error();
        assert_eq!(state.error_message, None);
        assert_eq!(state.interaction_count, 4);
    }

    #[test]
    fn repository_creation_and_serde() {
        let repo = Repository::new("repo-1", "nexo", "pedro-labsabs/nexo");
        let json = serde_json::to_string(&repo).expect("serialization failed");
        let decoded: Repository = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(repo, decoded);
    }

    #[test]
    fn error_display_formats_portuguese() {
        let err = NexoError::InvalidSnapshot("arquivo corrompido".into());
        assert_eq!(
            err.to_string(),
            "Dados de snapshot inválidos: arquivo corrompido"
        );

        let err2 = NexoError::IncompatibleSchema {
            expected: "1.0".into(),
            found: "0.9".into(),
        };
        assert_eq!(
            err2.to_string(),
            "Versão incompatível do schema: esperado 1.0, encontrado 0.9"
        );
    }
}
