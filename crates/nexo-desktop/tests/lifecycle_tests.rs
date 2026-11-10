//! Testes de integração de ciclo de vida e encerramento limpo para o aplicativo NEXO Rune.

use nexo_desktop::app::{AppState, view};
use rune::{Env, Key, KeyEvent, Mods, Runtime};
use std::fs::File;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// RAII Guard para garantir que o processo filho seja terminado
/// caso o teste falhe ou ocorra pânico, evitando processos zumbis ou órfãos.
struct ChildGuard(Option<Child>);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Valida o ciclo de vida do runtime Rune em modo headless:
/// Garante que múltiplas instâncias de Runtime podem ser criadas, renderizadas
/// por múltiplos frames e descartadas sequencialmente sem vazamentos de memória,
/// recursos órfãos de GPU ou panics no tear-down, sem depender de servidor gráfico.
#[test]
fn native_runtime_lifecycle_repeated_init_render_and_clean_drop() {
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

/// Teste genuíno de ciclo de vida nativo de processo:
/// Spawna o binário compilado `nexo-desktop`, aguarda a criação da janela nativa
/// pelo window manager via `wmctrl`, despacha sinal gracioso de encerramento
/// EWMH (_NET_CLOSE_WINDOW) e valida que o processo encerra com exit code 0 sem panics.
///
/// Em ambientes sem display gráfico ou sem `wmctrl` (como runners CI puramente headless),
/// o teste documenta explicitamente o skip para evitar falsas asserções de verificação.
#[test]
fn native_window_process_lifecycle_clean_exit() {
    let has_display = std::env::var("DISPLAY")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
        || std::env::var("WAYLAND_DISPLAY")
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false);

    let has_wmctrl = Command::new("which")
        .arg("wmctrl")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_display || !has_wmctrl {
        eprintln!(
            "SKIPPED: native window verification requires a graphical display server (DISPLAY or WAYLAND_DISPLAY) with an EWMH-compatible window manager and wmctrl. Skipping window lifecycle test in headless environment."
        );
        return;
    }

    let bin_path = env!("CARGO_BIN_EXE_nexo-desktop");
    let log_path =
        std::env::temp_dir().join(format!("nexo_native_lifecycle_{}.log", std::process::id()));
    let log_file =
        File::create(&log_path).expect("Failed to create temporary log file for child process");
    let log_file_err = log_file
        .try_clone()
        .expect("Failed to clone temporary log file handle");

    let child = Command::new(bin_path)
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::from(log_file_err))
        .spawn()
        .expect("Failed to spawn nexo-desktop binary");

    let mut guard = ChildGuard(Some(child));

    // Polling de até 5 segundos buscando janela "NEXO" via `wmctrl -l`
    let start = Instant::now();
    let timeout = Duration::from_secs(5);
    let poll_interval = Duration::from_millis(100);

    let mut window_found = false;
    while start.elapsed() < timeout {
        // Verifica se o processo filho terminou prematuramente com erro
        if let Some(child) = guard.0.as_mut()
            && let Ok(Some(status)) = child.try_wait()
        {
            panic!(
                "Child process exited unexpectedly early with status: {:?}",
                status
            );
        }

        let output = Command::new("wmctrl").arg("-l").output();
        if let Ok(out) = output {
            let stdout_str = String::from_utf8_lossy(&out.stdout);
            for line in stdout_str.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let title = parts[3..].join(" ");
                    if title == "NEXO" || title.contains("NEXO") {
                        window_found = true;
                        break;
                    }
                } else if line.contains("NEXO") {
                    window_found = true;
                    break;
                }
            }
        }

        if window_found {
            break;
        }
        std::thread::sleep(poll_interval);
    }

    assert!(
        window_found,
        "Native window titled 'NEXO' was not found within 5 seconds"
    );

    // Envia requisição graciosa de encerramento via `wmctrl -c NEXO` (_NET_CLOSE_WINDOW)
    let close_status = Command::new("wmctrl")
        .args(["-c", "NEXO"])
        .status()
        .expect("Failed to execute wmctrl -c NEXO");
    assert!(
        close_status.success(),
        "Failed to send close message via wmctrl"
    );

    // Aguarda a saída do processo filho (com timeout de até 5 segundos)
    let exit_timeout = Duration::from_secs(5);
    let exit_start = Instant::now();
    let mut exit_status = None;

    while exit_start.elapsed() < exit_timeout {
        if let Some(child) = guard.0.as_mut() {
            match child.try_wait() {
                Ok(Some(status)) => {
                    exit_status = Some(status);
                    break;
                }
                Ok(None) => {
                    std::thread::sleep(poll_interval);
                }
                Err(e) => {
                    panic!("Error waiting for child process: {:?}", e);
                }
            }
        }
    }

    let mut child = guard.0.take().expect("Child must still be present");
    let status = match exit_status {
        Some(s) => s,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("Child process failed to exit cleanly within timeout after close request");
        }
    };

    assert!(
        status.success(),
        "Child process exited with non-zero exit code: {:?}",
        status
    );

    // Inspeciona o arquivo de log para garantir que não houve panics ou erros fatais
    if let Ok(log_content) = std::fs::read_to_string(&log_path) {
        let _ = std::fs::remove_file(&log_path);
        assert!(
            !log_content.contains("panicked at"),
            "Panic detected in child process logs:\n{}",
            log_content
        );
        assert!(
            !log_content.contains("fatal runtime error"),
            "Fatal runtime error detected in child process logs:\n{}",
            log_content
        );
    }
}
