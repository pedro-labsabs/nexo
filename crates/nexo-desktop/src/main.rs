//! Ponto de entrada do aplicativo nativo NEXO desktop.

use nexo_desktop::app::{AppState, handle_key, view};
use rune::AppSpec;

fn main() {
    rune::run(AppSpec::<AppState> {
        on_key: Some(handle_key),
        scenes: &[
            ("default", |_| {}),
            ("dark", |s| s.domain.dark_theme = true),
            ("light", |s| s.domain.dark_theme = false),
            ("error", |s| {
                s.domain
                    .set_error("Erro de inicialização sintético para testes");
            }),
        ],
        ..AppSpec::new("NEXO", (900.0, 600.0), AppState::default, view)
    });
}
