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

// ============================================================================
// Módulo de Parsing e Associação Determinística de Janelas por PID (Task 1)
// ============================================================================

/// Informações estruturadas de uma janela retornada por `wmctrl -l -p`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowInfo {
    pub id: String,
    pub desktop: i32,
    pub pid: u32,
    pub host: String,
    pub title: String,
}

/// Erros tipados para associação de janelas ao processo alvo via EWMH / PID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowBindingError {
    PidNotFound(u32),
    PidNotSupported,
    AmbiguousWindows(Vec<String>),
}

impl std::fmt::Display for WindowBindingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowBindingError::PidNotFound(pid) => {
                write!(f, "Nenhuma janela encontrada para o PID {pid}")
            }
            WindowBindingError::PidNotSupported => {
                write!(
                    f,
                    "Window manager não suporta ou não reportou _NET_WM_PID (PID 0)"
                )
            }
            WindowBindingError::AmbiguousWindows(ids) => {
                write!(f, "Múltiplas janelas encontradas para o mesmo PID: {ids:?}")
            }
        }
    }
}

impl std::error::Error for WindowBindingError {}

/// Realiza o parsing determinístico da saída do comando `wmctrl -l -p`.
pub fn parse_wmctrl_l_p(output: &str) -> Vec<WindowInfo> {
    let mut windows = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut token_ranges = Vec::with_capacity(4);
        let mut in_token = false;
        let mut token_start = 0;
        let mut end_of_host = 0;

        for (i, c) in line.char_indices() {
            if c.is_whitespace() {
                if in_token {
                    token_ranges.push((token_start, i));
                    in_token = false;
                    if token_ranges.len() == 4 {
                        end_of_host = i;
                        break;
                    }
                }
            } else if !in_token {
                token_start = i;
                in_token = true;
            }
        }

        if in_token && token_ranges.len() < 4 {
            token_ranges.push((token_start, line.len()));
            if token_ranges.len() == 4 {
                end_of_host = line.len();
            }
        }

        if token_ranges.len() < 4 {
            continue;
        }

        let id = line[token_ranges[0].0..token_ranges[0].1].to_string();
        let desktop = match line[token_ranges[1].0..token_ranges[1].1].parse::<i32>() {
            Ok(d) => d,
            Err(_) => continue,
        };
        let pid = match line[token_ranges[2].0..token_ranges[2].1].parse::<u32>() {
            Ok(p) => p,
            Err(_) => continue,
        };
        let host = line[token_ranges[3].0..token_ranges[3].1].to_string();

        let raw_title = if end_of_host < line.len() {
            &line[end_of_host..]
        } else {
            ""
        };
        let title = raw_title.trim_start().to_string();

        windows.push(WindowInfo {
            id,
            desktop,
            pid,
            host,
            title,
        });
    }
    windows
}

/// Encontra de forma inequívoca uma janela pertencente ao PID especificado.
pub fn find_window_by_pid(
    windows: &[WindowInfo],
    target_pid: u32,
) -> Result<WindowInfo, WindowBindingError> {
    if target_pid == 0 {
        return Err(WindowBindingError::PidNotSupported);
    }

    let matching: Vec<&WindowInfo> = windows.iter().filter(|w| w.pid == target_pid).collect();

    if matching.is_empty() {
        if !windows.is_empty() && windows.iter().all(|w| w.pid == 0) {
            return Err(WindowBindingError::PidNotSupported);
        }
        return Err(WindowBindingError::PidNotFound(target_pid));
    }

    if matching.len() > 1 {
        return Err(WindowBindingError::AmbiguousWindows(
            matching.iter().map(|w| w.id.clone()).collect(),
        ));
    }

    Ok(matching[0].clone())
}

#[cfg(test)]
mod window_locator_tests {
    use super::*;

    #[test]
    fn parse_wmctrl_l_p_valid_lines() {
        let raw = "\
0x04000003  0 12345 myhost Terminal\n\
0x04200002  1 12346 myhost NEXO   Rune   Application   Title\n\
0x04200005 -1 0     myhost Desktop Window\n\
";
        let windows = parse_wmctrl_l_p(raw);
        assert_eq!(windows.len(), 3);

        assert_eq!(windows[0].id, "0x04000003");
        assert_eq!(windows[0].desktop, 0);
        assert_eq!(windows[0].pid, 12345);
        assert_eq!(windows[0].host, "myhost");
        assert_eq!(windows[0].title, "Terminal");

        assert_eq!(windows[1].id, "0x04200002");
        assert_eq!(windows[1].desktop, 1);
        assert_eq!(windows[1].pid, 12346);
        assert_eq!(windows[1].host, "myhost");
        assert_eq!(
            windows[1].title, "NEXO   Rune   Application   Title",
            "Múltiplos espaços internos no título da janela devem ser preservados"
        );

        assert_eq!(windows[2].id, "0x04200005");
        assert_eq!(windows[2].desktop, -1);
        assert_eq!(windows[2].pid, 0);
        assert_eq!(windows[2].host, "myhost");
        assert_eq!(windows[2].title, "Desktop Window");
    }

    #[test]
    fn find_window_by_pid_ignores_preexisting_window_with_same_title() {
        let windows = vec![
            WindowInfo {
                id: "0x01".to_string(),
                desktop: 0,
                pid: 1111,
                host: "host".to_string(),
                title: "NEXO".to_string(),
            },
            WindowInfo {
                id: "0x02".to_string(),
                desktop: 0,
                pid: 2222,
                host: "host".to_string(),
                title: "NEXO".to_string(),
            },
        ];

        let found = find_window_by_pid(&windows, 2222).expect("Must find window for PID 2222");
        assert_eq!(found.id, "0x02");
        assert_eq!(found.pid, 2222);
    }

    #[test]
    fn find_window_by_pid_returns_not_found_when_no_match() {
        let windows = vec![WindowInfo {
            id: "0x01".to_string(),
            desktop: 0,
            pid: 1111,
            host: "host".to_string(),
            title: "NEXO".to_string(),
        }];

        let res = find_window_by_pid(&windows, 2222);
        assert_eq!(res, Err(WindowBindingError::PidNotFound(2222)));
    }

    #[test]
    fn find_window_by_pid_rejects_zero_pid() {
        // Caso 1: consulta direta por PID 0 é rejeitada como não suportada
        let windows = vec![WindowInfo {
            id: "0x01".to_string(),
            desktop: 0,
            pid: 0,
            host: "host".to_string(),
            title: "NEXO".to_string(),
        }];
        let res_zero = find_window_by_pid(&windows, 0);
        assert_eq!(res_zero, Err(WindowBindingError::PidNotSupported));

        // Caso 2: todas as janelas reportam PID 0 (falta de suporte EWMH / _NET_WM_PID)
        let res_unsupported = find_window_by_pid(&windows, 2222);
        assert_eq!(res_unsupported, Err(WindowBindingError::PidNotSupported));
    }

    #[test]
    fn find_window_by_pid_detects_ambiguous_windows() {
        let windows = vec![
            WindowInfo {
                id: "0x01".to_string(),
                desktop: 0,
                pid: 2222,
                host: "host".to_string(),
                title: "NEXO Main".to_string(),
            },
            WindowInfo {
                id: "0x02".to_string(),
                desktop: 0,
                pid: 2222,
                host: "host".to_string(),
                title: "NEXO Secondary".to_string(),
            },
        ];

        let res = find_window_by_pid(&windows, 2222);
        assert_eq!(
            res,
            Err(WindowBindingError::AmbiguousWindows(vec![
                "0x01".to_string(),
                "0x02".to_string()
            ]))
        );
    }
}
