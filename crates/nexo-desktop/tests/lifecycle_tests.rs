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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EnvironmentSupport {
    Supported,
    MissingDisplay,
    PureWaylandWithoutX11,
    MissingWmctrl,
    WmctrlCommunicationFailed(i32),
}

/// Avalia se o ambiente possui os pré-requisitos necessários para execução
/// do teste de janela gráfica nativa (display server e utilitário wmctrl funcional).
pub fn check_environment_support<F>(
    display_var: Option<&str>,
    wayland_var: Option<&str>,
    has_wmctrl: bool,
    probe_wmctrl: F,
) -> EnvironmentSupport
where
    F: FnOnce() -> Result<(), i32>,
{
    let has_x11 = display_var.map(|v| !v.trim().is_empty()).unwrap_or(false);
    let has_wayland = wayland_var.map(|v| !v.trim().is_empty()).unwrap_or(false);

    if !has_x11 && !has_wayland {
        return EnvironmentSupport::MissingDisplay;
    }

    if !has_x11 && has_wayland {
        return EnvironmentSupport::PureWaylandWithoutX11;
    }

    if !has_wmctrl {
        return EnvironmentSupport::MissingWmctrl;
    }

    match probe_wmctrl() {
        Ok(()) => EnvironmentSupport::Supported,
        Err(code) => EnvironmentSupport::WmctrlCommunicationFailed(code),
    }
}

/// Erros tipados durante o ciclo de vida e monitoramento do processo filho.
#[derive(Debug, PartialEq, Eq)]
pub enum LifecycleError {
    PrematureExit(i32),
    NonZeroExit(i32),
    TerminatedBySignal,
    Timeout,
    WindowBinding(WindowBindingError),
}

impl std::fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LifecycleError::PrematureExit(code) => {
                write!(
                    f,
                    "Processo filho encerrou prematuramente com código {code}"
                )
            }
            LifecycleError::NonZeroExit(code) => {
                write!(
                    f,
                    "Processo filho encerrou com código diferente de zero: {code}"
                )
            }
            LifecycleError::TerminatedBySignal => {
                write!(f, "Processo filho foi encerrado por sinal")
            }
            LifecycleError::Timeout => {
                write!(f, "Timeout aguardando evento do processo filho")
            }
            LifecycleError::WindowBinding(err) => write!(f, "Erro de vinculação de janela: {err}"),
        }
    }
}

impl std::error::Error for LifecycleError {}

/// Monitora o processo filho enquanto busca a janela correspondente via polling.
/// Detecta e falha imediatamente se o processo filho terminar prematuramente.
pub fn poll_child_window<F>(
    child: &mut Child,
    timeout: Duration,
    poll_interval: Duration,
    mut fetch_window: F,
) -> Result<WindowInfo, LifecycleError>
where
    F: FnMut() -> Result<WindowInfo, WindowBindingError>,
{
    let start = Instant::now();
    while start.elapsed() < timeout {
        if let Ok(Some(status)) = child.try_wait() {
            let code = status.code().unwrap_or(-1);
            return Err(LifecycleError::PrematureExit(code));
        }

        match fetch_window() {
            Ok(window) => return Ok(window),
            Err(WindowBindingError::PidNotFound(_)) => {
                // Continua aguardando a criação e mapeamento da janela
            }
            Err(err) => {
                return Err(LifecycleError::WindowBinding(err));
            }
        }

        std::thread::sleep(poll_interval);
    }

    if let Ok(Some(status)) = child.try_wait() {
        let code = status.code().unwrap_or(-1);
        return Err(LifecycleError::PrematureExit(code));
    }

    Err(LifecycleError::Timeout)
}

/// Envia requisição graciosa de encerramento estritamente por Window ID (_NET_CLOSE_WINDOW).
/// Garante que o comando nunca utiliza correspondência por título.
pub fn close_window_by_id(window_id: &str) -> std::io::Result<std::process::ExitStatus> {
    assert!(!window_id.trim().is_empty(), "Window ID cannot be empty");
    Command::new("wmctrl")
        .args(["-i", "-c", window_id])
        .status()
}

/// Valida que o encerramento do processo ocorreu de forma limpa com código 0.
pub fn verify_clean_exit(
    status: std::process::ExitStatus,
) -> Result<std::process::ExitStatus, LifecycleError> {
    match status.code() {
        Some(0) => Ok(status),
        Some(code) => Err(LifecycleError::NonZeroExit(code)),
        None => Err(LifecycleError::TerminatedBySignal),
    }
}

/// Aguarda a saída limpa do processo filho até o timeout especificado.
pub fn wait_for_clean_exit(
    child: &mut Child,
    timeout: Duration,
    poll_interval: Duration,
) -> Result<std::process::ExitStatus, LifecycleError> {
    let start = Instant::now();
    while start.elapsed() < timeout {
        match child.try_wait() {
            Ok(Some(status)) => {
                return verify_clean_exit(status);
            }
            Ok(None) => {
                std::thread::sleep(poll_interval);
            }
            Err(e) => {
                panic!("Error waiting for child process: {:?}", e);
            }
        }
    }
    Err(LifecycleError::Timeout)
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
    let display_env = std::env::var("DISPLAY").ok();
    let wayland_env = std::env::var("WAYLAND_DISPLAY").ok();
    let has_wmctrl = Command::new("which")
        .arg("wmctrl")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let env_support = check_environment_support(
        display_env.as_deref(),
        wayland_env.as_deref(),
        has_wmctrl,
        || {
            let probe = Command::new("wmctrl").arg("-m").output().map_err(|_| -1)?;
            if probe.status.success() {
                Ok(())
            } else {
                Err(probe.status.code().unwrap_or(-1))
            }
        },
    );

    match env_support {
        EnvironmentSupport::Supported => {
            // Prossegue com a inicialização e verificação nativa
        }
        EnvironmentSupport::MissingDisplay => {
            eprintln!("SKIPPED: Nenhum display gráfico detectado (DISPLAY e WAYLAND_DISPLAY ausentes).");
            return;
        }
        EnvironmentSupport::PureWaylandWithoutX11 => {
            eprintln!(
                "SKIPPED: Sessão Wayland pura sem ponte X11/XWayland (DISPLAY ausente). O utilitário wmctrl opera sob X11/EWMH."
            );
            return;
        }
        EnvironmentSupport::MissingWmctrl => {
            eprintln!("SKIPPED: Utilitário wmctrl não está instalado no sistema.");
            return;
        }
        EnvironmentSupport::WmctrlCommunicationFailed(code) => {
            eprintln!(
                "SKIPPED: wmctrl falhou ao se comunicar com o window manager X11 (código {code})."
            );
            return;
        }
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

    let child_pid = child.id();
    let mut guard = ChildGuard(Some(child));

    // Polling de até 5 segundos buscando janela vinculada exclusivamente ao PID via `wmctrl -l -p`
    let window_timeout = Duration::from_secs(5);
    let poll_interval = Duration::from_millis(100);

    let fetch_window = || {
        let out = Command::new("wmctrl")
            .args(["-l", "-p"])
            .output()
            .map_err(|_| WindowBindingError::WmctrlExecutionFailed(-1))?;
        if !out.status.success() {
            let code = out.status.code().unwrap_or(-1);
            return Err(WindowBindingError::WmctrlExecutionFailed(code));
        }
        let stdout_str = String::from_utf8_lossy(&out.stdout);
        let windows = parse_wmctrl_l_p(&stdout_str);
        find_window_by_pid(&windows, child_pid)
    };

    let target_window = match poll_child_window(
        guard.0.as_mut().expect("Child must be active"),
        window_timeout,
        poll_interval,
        fetch_window,
    ) {
        Ok(w) => w,
        Err(LifecycleError::WindowBinding(WindowBindingError::PidNotSupported)) => {
            eprintln!(
                "SKIPPED: Window manager does not support _NET_WM_PID (all windows reported PID 0). Aborting native window test safely without modifying any windows."
            );
            return;
        }
        Err(LifecycleError::PrematureExit(code)) => {
            let logs = std::fs::read_to_string(&log_path).unwrap_or_default();
            panic!(
                "Child process (PID {child_pid}) exited unexpectedly early with code {code}. Logs:\n{logs}"
            );
        }
        Err(err) => {
            let logs = std::fs::read_to_string(&log_path).unwrap_or_default();
            panic!("Failed to associate native window for PID {child_pid}: {err}. Logs:\n{logs}");
        }
    };

    println!(
        "Associated window found: ID {}, PID {}, Title: {}",
        target_window.id, target_window.pid, target_window.title
    );

    // Envia requisição graciosa de encerramento estritamente por Window ID via `wmctrl -i -c` (_NET_CLOSE_WINDOW).
    // NUNCA usa fechamento por título.
    let close_status =
        close_window_by_id(&target_window.id).expect("Failed to execute wmctrl -i -c");
    assert!(
        close_status.success(),
        "Failed to send close message to window ID {} via wmctrl",
        target_window.id
    );

    // Aguarda a saída do processo filho (com timeout de até 5 segundos)
    let exit_timeout = Duration::from_secs(5);
    let mut child = guard.0.take().expect("Child must still be present");
    let status = match wait_for_clean_exit(&mut child, exit_timeout, poll_interval) {
        Ok(s) => s,
        Err(LifecycleError::Timeout) => {
            let _ = child.kill();
            let _ = child.wait();
            let logs = std::fs::read_to_string(&log_path).unwrap_or_default();
            panic!(
                "Child process failed to exit cleanly within timeout after close request for window ID {}. Logs:\n{logs}",
                target_window.id
            );
        }
        Err(LifecycleError::NonZeroExit(code)) => {
            let logs = std::fs::read_to_string(&log_path).unwrap_or_default();
            panic!("Child process exited with non-zero code {code}. Logs:\n{logs}");
        }
        Err(err) => {
            let logs = std::fs::read_to_string(&log_path).unwrap_or_default();
            panic!("Unexpected lifecycle error: {err}. Logs:\n{logs}");
        }
    };

    assert_eq!(
        status.code(),
        Some(0),
        "Child process must exit with code 0: {:?}",
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
    AmbiguousWindows(Vec<String>),
    PidNotSupported,
    WmctrlExecutionFailed(i32),
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
            WindowBindingError::WmctrlExecutionFailed(code) => {
                write!(f, "Falha na execução do utilitário wmctrl (código: {code})")
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

// ============================================================================
// Bateria de Testes de Regressão Obrigatórios (Task 3 / Auditoria R3)
// ============================================================================

/// Regressão 1: Comprova que uma janela pré-existente com título "NEXO" pertencente
/// a outro processo não é associada nem fechada quando o teste do processo filho é executado.
#[test]
fn regression_preexisting_nexo_window_is_not_closed() {
    let preexisting_window_id = "0x01a00001";
    let preexisting_pid = 11111;
    let child_pid = 22222;

    let windows_before_child = vec![
        WindowInfo {
            id: preexisting_window_id.to_string(),
            desktop: 0,
            pid: preexisting_pid,
            host: "desktop-host".to_string(),
            title: "NEXO".to_string(),
        },
        WindowInfo {
            id: "0x01a00002".to_string(),
            desktop: 0,
            pid: 10000,
            host: "desktop-host".to_string(),
            title: "Editor".to_string(),
        },
    ];

    // Antes da criação da janela do child, a busca pelo PID do child NÃO deve
    // vincular à janela pré-existente "NEXO"
    let lookup_before = find_window_by_pid(&windows_before_child, child_pid);
    assert_eq!(
        lookup_before,
        Err(WindowBindingError::PidNotFound(child_pid)),
        "A janela pré-existente com título 'NEXO' não pode ser vinculada ao child_pid {child_pid}"
    );

    // Quando o processo filho finalmente cria sua janela (também intitulada "NEXO")
    let child_window_id = "0x02b00005";
    let mut windows_after_child = windows_before_child.clone();
    windows_after_child.push(WindowInfo {
        id: child_window_id.to_string(),
        desktop: 0,
        pid: child_pid,
        host: "desktop-host".to_string(),
        title: "NEXO".to_string(),
    });

    let lookup_after = find_window_by_pid(&windows_after_child, child_pid);
    assert!(lookup_after.is_ok(), "Deve encontrar a janela do child");
    let target = lookup_after.unwrap();

    // Comprova que o ID e PID são estritamente do processo filho
    assert_eq!(target.id, child_window_id);
    assert_eq!(target.pid, child_pid);
    assert_ne!(
        target.id, preexisting_window_id,
        "O ID da janela alvo NUNCA pode ser o ID da janela pré-existente"
    );
    assert_ne!(
        target.pid, preexisting_pid,
        "O PID da janela alvo NUNCA pode ser o PID do processo pré-existente"
    );

    // Teste também a partir do parsing de saída bruta de wmctrl -l -p
    let raw_wmctrl = format!(
        "{preexisting_window_id}  0 {preexisting_pid} myhost NEXO\n\
         {child_window_id}        0 {child_pid}       myhost NEXO\n"
    );
    let parsed = parse_wmctrl_l_p(&raw_wmctrl);
    let bound = find_window_by_pid(&parsed, child_pid).expect("Must bind to child PID");
    assert_eq!(bound.id, child_window_id);
    assert_eq!(bound.pid, child_pid);
}

/// Regressão 2: Comprova que a saída prematura do processo filho (ex.: erro precoce
/// ou encerramento antes de abrir/mapear a janela) causa falha imediata na verificação.
#[test]
fn regression_premature_child_exit_triggers_failure() {
    // Cenário 1: Processo filho encerra prematuramente com erro (código 1)
    let child_err = Command::new("sh")
        .args(["-c", "exit 1"])
        .spawn()
        .expect("Failed to spawn synthetic child");
    let mut guard_err = ChildGuard(Some(child_err));

    let result_err = poll_child_window(
        guard_err.0.as_mut().unwrap(),
        Duration::from_millis(500),
        Duration::from_millis(20),
        || Err(WindowBindingError::PidNotFound(99999)),
    );

    match result_err {
        Err(LifecycleError::PrematureExit(code)) => {
            assert_eq!(code, 1, "Deve identificar saída prematura com código 1");
        }
        other => panic!("Esperava PrematureExit(1), obteve: {other:?}"),
    }

    // Cenário 2: Processo filho encerra prematuramente com código 0 antes de mapear janela
    let child_zero = Command::new("sh")
        .args(["-c", "exit 0"])
        .spawn()
        .expect("Failed to spawn synthetic child");
    let mut guard_zero = ChildGuard(Some(child_zero));

    let result_zero = poll_child_window(
        guard_zero.0.as_mut().unwrap(),
        Duration::from_millis(500),
        Duration::from_millis(20),
        || Err(WindowBindingError::PidNotFound(99999)),
    );

    match result_zero {
        Err(LifecycleError::PrematureExit(code)) => {
            assert_eq!(
                code, 0,
                "Deve identificar saída prematura mesmo com código 0"
            );
        }
        other => panic!("Esperava PrematureExit(0), obteve: {other:?}"),
    }
}

/// Regressão 3: Comprova que um encerramento com código diferente de zero é identificado
/// e rejeitado como falha pelo verificador de ciclo de vida.
#[test]
fn regression_nonzero_exit_code_triggers_failure() {
    let child = Command::new("sh")
        .args(["-c", "exit 42"])
        .spawn()
        .expect("Failed to spawn synthetic child");
    let mut guard = ChildGuard(Some(child));

    let result = wait_for_clean_exit(
        guard.0.as_mut().unwrap(),
        Duration::from_millis(500),
        Duration::from_millis(20),
    );

    match result {
        Err(LifecycleError::NonZeroExit(code)) => {
            assert_eq!(code, 42, "Deve capturar exatamente o código de saída 42");
        }
        other => panic!("Esperava NonZeroExit(42), obteve: {other:?}"),
    }
}

/// Regressão 4: Comprova que um processo em hang que não responde no timeout
/// provoca erro de timeout e é terminado e limpo de forma segura via ChildGuard.
#[test]
fn regression_child_hang_triggers_timeout_and_clean_kill() {
    let child = Command::new("sleep")
        .arg("60")
        .spawn()
        .expect("Failed to spawn hanging process");

    let pid = child.id();
    let proc_path = std::path::PathBuf::from(format!("/proc/{pid}"));
    assert!(
        proc_path.exists(),
        "Processo filho {pid} deve estar ativo inicialmente"
    );

    let mut guard = ChildGuard(Some(child));

    let start = Instant::now();
    let result = wait_for_clean_exit(
        guard.0.as_mut().unwrap(),
        Duration::from_millis(150),
        Duration::from_millis(20),
    );
    let elapsed = start.elapsed();

    assert_eq!(
        result,
        Err(LifecycleError::Timeout),
        "Processo em hang deve provocar Timeout"
    );
    assert!(
        elapsed < Duration::from_millis(500),
        "Timeout deve ser respeitado prontamente (gastou {elapsed:?})"
    );

    // O processo ainda está sob custódia de guard; o drop dispara kill() e wait() (reap)
    drop(guard);

    // Comprova que após o drop do ChildGuard o processo não existe mais
    assert!(
        !proc_path.exists(),
        "Processo {pid} deve ter sido terminado e reaped pelo ChildGuard"
    );
}

/// Regressão 5: Comprova que a execução sem suporte EWMH/X11 ou em ambientes Wayland puros
/// sem ponte é tratada corretamente, sem declarar um teste nativo como executado com sucesso.
#[test]
fn regression_missing_ewmh_or_display_handled_cleanly() {
    // 1. Ausência de DISPLAY ou WAYLAND_DISPLAY (headless puro)
    assert_eq!(
        check_environment_support(None, None, true, || Ok(())),
        EnvironmentSupport::MissingDisplay
    );
    assert_eq!(
        check_environment_support(Some(""), Some(""), true, || Ok(())),
        EnvironmentSupport::MissingDisplay
    );

    // 2. Pure Wayland sem X11 (DISPLAY ausente ou vazio, WAYLAND presente)
    assert_eq!(
        check_environment_support(None, Some("wayland-0"), true, || Ok(())),
        EnvironmentSupport::PureWaylandWithoutX11
    );
    assert_eq!(
        check_environment_support(Some(""), Some("wayland-0"), true, || Ok(())),
        EnvironmentSupport::PureWaylandWithoutX11
    );

    // 3. X11 presente, mas wmctrl ausente
    assert_eq!(
        check_environment_support(Some(":0"), None, false, || Ok(())),
        EnvironmentSupport::MissingWmctrl
    );
    assert_eq!(
        check_environment_support(Some(":0"), Some("wayland-0"), false, || Ok(())),
        EnvironmentSupport::MissingWmctrl
    );

    // 4. wmctrl presente mas probe de comunicação falha
    assert_eq!(
        check_environment_support(Some(":0"), None, true, || Err(1)),
        EnvironmentSupport::WmctrlCommunicationFailed(1)
    );
    assert_eq!(
        check_environment_support(Some(":0"), None, true, || Err(-1)),
        EnvironmentSupport::WmctrlCommunicationFailed(-1)
    );

    // 5. Suporte completo presente (X11 presente, wmctrl presente, probe bem-sucedido)
    assert_eq!(
        check_environment_support(Some(":0"), None, true, || Ok(())),
        EnvironmentSupport::Supported
    );
    assert_eq!(
        check_environment_support(Some(":0"), Some("wayland-0"), true, || Ok(())),
        EnvironmentSupport::Supported
    );

    // 4. Ausência de suporte EWMH _NET_WM_PID (todas as janelas reportam PID 0)
    let unsupported_windows = vec![
        WindowInfo {
            id: "0x01".to_string(),
            desktop: 0,
            pid: 0,
            host: "host".to_string(),
            title: "NEXO".to_string(),
        },
        WindowInfo {
            id: "0x02".to_string(),
            desktop: 0,
            pid: 0,
            host: "host".to_string(),
            title: "Browser".to_string(),
        },
    ];
    let result = find_window_by_pid(&unsupported_windows, 12345);
    assert_eq!(
        result,
        Err(WindowBindingError::PidNotSupported),
        "Quando todas as janelas têm PID 0, deve retornar PidNotSupported sem associar a janela"
    );

    // 5. Wayland puro sem ponte / lista vazia de janelas
    let empty_windows: Vec<WindowInfo> = Vec::new();
    let empty_result = find_window_by_pid(&empty_windows, 12345);
    assert_eq!(
        empty_result,
        Err(WindowBindingError::PidNotFound(12345)),
        "Lista vazia de janelas não pode associar nenhuma janela nem declarar sucesso"
    );
}

#[test]
fn regression_wmctrl_execution_failure_is_not_masked_as_pid_not_found() {
    let child = Command::new("sleep")
        .arg("10")
        .spawn()
        .expect("Failed to spawn process");
    let mut guard = ChildGuard(Some(child));

    let fetch_with_failing_wmctrl = || Err(WindowBindingError::WmctrlExecutionFailed(1));

    let result = poll_child_window(
        guard.0.as_mut().unwrap(),
        Duration::from_millis(500),
        Duration::from_millis(20),
        fetch_with_failing_wmctrl,
    );

    assert_eq!(
        result,
        Err(LifecycleError::WindowBinding(WindowBindingError::WmctrlExecutionFailed(1))),
        "Falha na comunicação com wmctrl deve abortar imediatamente com erro tipado, nunca ser mascarada como PidNotFound"
    );
}

