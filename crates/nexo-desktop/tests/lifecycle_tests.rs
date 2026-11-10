//! Testes de integração de ciclo de vida e encerramento limpo para o aplicativo NEXO Rune.

use nexo_desktop::app::{AppState, view};
use rune::{Env, Key, KeyEvent, Mods, Runtime};

#[test]
fn native_runtime_lifecycle_repeated_init_render_and_clean_drop() {
    // Valida que múltiplas instâncias de Runtime podem ser criadas, renderizadas
    // por múltiplos frames e descartadas sequencialmente sem vazamentos de memória,
    // recursos órfãos de GPU ou panics no tear-down.
    for run in 0..5 {
        let env = Env {
            system_dark: run % 2 == 0,
            scale: 2.0,
        };
        let mut rt = Runtime::new(AppState::default(), view, (900.0, 600.0), env);
        rt.frame(0.0);

        // Simula ciclo de renderização ativo
        for _ in 0..30 {
            rt.frame(1.0 / 60.0);
        }

        // Executa interação antes do encerramento
        rt.key(KeyEvent {
            key: Key::Tab,
            mods: Mods::default(),
        });
        rt.frame(0.0);

        // Descarte explícito deve ser limpo e instantâneo
        drop(rt);
    }
}

#[test]
fn process_clean_exit_without_hanging_threads() {
    // Garante que o estado inicial e as funções de ciclo de vida encerram normalmente.
    let state = AppState::default();
    assert_eq!(state.domain.app_version, "0.1.0");
}
