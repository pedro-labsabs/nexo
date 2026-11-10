use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ViewMode {
    #[default]
    Dashboard,
    Graph,
    Table,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Repository {
    pub id: String,
    pub name: String,
    pub full_name: String,
    pub is_private: bool,
    pub description: Option<String>,
    pub primary_language: Option<String>,
    pub stars_count: u32,
    pub forks_count: u32,
}

impl Repository {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        full_name: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            full_name: full_name.into(),
            is_private: false,
            description: None,
            primary_language: None,
            stars_count: 0,
            forks_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NexoState {
    pub app_version: String,
    pub view_mode: ViewMode,
    pub dark_theme: bool,
    pub selected_repo_id: Option<String>,
    pub interaction_count: u64,
    pub error_message: Option<String>,
}

impl Default for NexoState {
    fn default() -> Self {
        Self {
            app_version: "0.1.0".to_string(),
            view_mode: ViewMode::Dashboard,
            dark_theme: true,
            selected_repo_id: None,
            interaction_count: 0,
            error_message: None,
        }
    }
}

impl NexoState {
    pub fn increment_interaction(&mut self) {
        self.interaction_count = self.interaction_count.saturating_add(1);
    }

    pub fn toggle_theme(&mut self) {
        self.dark_theme = !self.dark_theme;
        self.increment_interaction();
    }

    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
        self.increment_interaction();
    }

    pub fn set_error(&mut self, message: impl Into<String>) {
        self.error_message = Some(message.into());
        self.increment_interaction();
    }

    pub fn clear_error(&mut self) {
        self.error_message = None;
        self.increment_interaction();
    }
}
