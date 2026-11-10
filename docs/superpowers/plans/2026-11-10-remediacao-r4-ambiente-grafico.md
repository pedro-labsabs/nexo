# NEXO — PR #10 | Remediação R4 — Validação de Ambiente Gráfico

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Sanar as deficiências de validação de ambiente gráfico e comunicação com o gerenciador de janelas EWMH/X11 em `crates/nexo-desktop/tests/lifecycle_tests.rs`, implementando distinção entre X11/EWMH e Wayland puro, verificação estrita de exit code do `wmctrl`, garantia de que falhas do aplicativo nunca são mascaradas como skip, e cobertura completa dos 7 cenários de regressão obrigatórios.

**Architecture:** 
1. Refatorar `EnvironmentSupport` e `check_environment_support` para aceitar um probe de comunicação com o window manager, distinguindo sessões headless (`MissingDisplay`), Wayland puro sem ponte X11 (`PureWaylandWithoutX11`), ausência do binário `wmctrl` (`MissingWmctrl`), falha de comunicação na execução do `wmctrl` (`WmctrlCommunicationFailed(code)`) e ambiente suportado (`Supported`).
2. Adicionar a variante `WmctrlExecutionFailed(i32)` em `WindowBindingError` e checar rigorosamente `out.status.success()` em `fetch_window` e `close_window_by_id`, eliminando mascaramento de erros de comunicação como `PidNotFound`.
3. Implementar testes unitários e de integração para os 7 cenários obrigatórios de regressão, assegurando distinção inequívoca entre testes executados, testes ignorados por pré-requisito e testes reprovados.
4. Atualizar o ADR 0002 (`docs/architecture/0002-rune-pin-and-linux-deps.md`) documentando as limitações e especificidades de X11, XWayland e Wayland puro, além de atualizar a contagem real de testes verificados.

**Tech Stack:** Rust 1.99.1 (Rune upstream pin `42f2d35`), Vitest/TypeScript, X11/EWMH (`wmctrl`), Linux.

**Spec:** Issue #2 (Milestone M1), PR #10, Especificação da Remediação R4.

## Global Constraints
- Preservar integralmente todas as implementações anteriores aprovadas: 18 testes headless Rust, 30 testes TypeScript, build e lints limpos.
- Nunca mascarar falha do processo filho como skip (falhas reais devem causar falha do teste com logs anexados).
- Nunca fechar qualquer janela sem associação comprovada por PID ao processo filho.
- Usar fechamento cirúrgico estrito por Window ID (`wmctrl -i -c <window_id>`).
- Não alterar escopo da issue #2, não abrir nova PR, não realizar merge ou fechar a issue antes da auditoria final.

---

### Task 1: Refatoração de `EnvironmentSupport` e Probe de Comunicação EWMH

**Files:**
- Modify: `crates/nexo-desktop/tests/lifecycle_tests.rs:50-80`
- Test: `crates/nexo-desktop/tests/lifecycle_tests.rs`

**Interfaces:**
- Produces:
  ```rust
  #[derive(Debug, PartialEq, Eq, Clone)]
  pub enum EnvironmentSupport {
      Supported,
      MissingDisplay,
      PureWaylandWithoutX11,
      MissingWmctrl,
      WmctrlCommunicationFailed(i32),
  }

  pub fn check_environment_support<F>(
      display_var: Option<&str>,
      wayland_var: Option<&str>,
      has_wmctrl: bool,
      probe_wmctrl: F,
  ) -> EnvironmentSupport
  where
      F: FnOnce() -> Result<(), i32>;
  ```

- [ ] **Step 1: Escrever teste de unidade falhando para `check_environment_support` com distinção de Wayland puro e falha de comunicação**

Em `crates/nexo-desktop/tests/lifecycle_tests.rs`, atualizar `regression_missing_ewmh_or_display_handled_cleanly` para testar as novas variantes:
```rust
#[test]
fn regression_missing_ewmh_or_display_handled_cleanly() {
    // 1. Ausência de DISPLAY e WAYLAND_DISPLAY
    assert_eq!(
        check_environment_support(None, None, true, || Ok(())),
        EnvironmentSupport::MissingDisplay
    );
    assert_eq!(
        check_environment_support(Some(""), Some(""), true, || Ok(())),
        EnvironmentSupport::MissingDisplay
    );

    // 2. Wayland puro sem ponte X11/XWayland (WAYLAND_DISPLAY presente, DISPLAY ausente)
    assert_eq!(
        check_environment_support(None, Some("wayland-0"), true, || Ok(())),
        EnvironmentSupport::PureWaylandWithoutX11
    );
    assert_eq!(
        check_environment_support(Some(""), Some("wayland-0"), true, || Ok(())),
        EnvironmentSupport::PureWaylandWithoutX11
    );

    // 3. DISPLAY presente mas wmctrl ausente
    assert_eq!(
        check_environment_support(Some(":0"), None, false, || Ok(())),
        EnvironmentSupport::MissingWmctrl
    );

    // 4. DISPLAY e wmctrl presentes, mas wmctrl falha na comunicação com o WM
    assert_eq!(
        check_environment_support(Some(":0"), None, true, || Err(1)),
        EnvironmentSupport::WmctrlCommunicationFailed(1)
    );

    // 5. Suporte X11/EWMH completo e funcional
    assert_eq!(
        check_environment_support(Some(":0"), None, true, || Ok(())),
        EnvironmentSupport::Supported
    );
    // Suporte com XWayland (ambos DISPLAY e WAYLAND_DISPLAY presentes e probe OK)
    assert_eq!(
        check_environment_support(Some(":0"), Some("wayland-0"), true, || Ok(())),
        EnvironmentSupport::Supported
    );
}
```

- [ ] **Step 2: Executar o teste para verificar a falha de compilação/asserção**

Run: `cargo test -p nexo-desktop --test lifecycle_tests regression_missing_ewmh_or_display_handled_cleanly`  
Expected: FAIL com erro de tipo/assinatura em `check_environment_support`.

- [ ] **Step 3: Implementar `EnvironmentSupport` e `check_environment_support`**

Em `crates/nexo-desktop/tests/lifecycle_tests.rs`:
```rust
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EnvironmentSupport {
    Supported,
    MissingDisplay,
    PureWaylandWithoutX11,
    MissingWmctrl,
    WmctrlCommunicationFailed(i32),
}

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
```

Atualizar o ponto de chamada em `native_window_process_lifecycle_clean_exit`:
```rust
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
```

- [ ] **Step 4: Executar os testes para confirmar que passam**

Run: `cargo test -p nexo-desktop --test lifecycle_tests`  
Expected: PASS (todos os testes passando).

- [ ] **Step 5: Commit das alterações da Task 1**

```bash
git add crates/nexo-desktop/tests/lifecycle_tests.rs
git commit -m "fix(desktop): distinguir Wayland puro e validar comunicação EWMH antes do teste de janela"
```

---

### Task 2: Verificação Estrita de Exit Status do `wmctrl` em Runtime e Erradicação de Mascaramento

**Files:**
- Modify: `crates/nexo-desktop/tests/lifecycle_tests.rs:80-270`
- Test: `crates/nexo-desktop/tests/lifecycle_tests.rs`

**Interfaces:**
- Produces:
  ```rust
  #[derive(Debug, PartialEq, Eq)]
  pub enum WindowBindingError {
      PidNotFound(u32),
      AmbiguousWindows(Vec<String>),
      PidNotSupported,
      WmctrlExecutionFailed(i32),
  }
  ```

- [ ] **Step 1: Escrever teste de unidade falhando para erro de execução do `wmctrl` durante o polling**

Adicionar teste unitário em `lifecycle_tests.rs`:
```rust
#[test]
fn regression_wmctrl_execution_failure_is_not_masked_as_pid_not_found() {
    let mut child = Command::new("sleep")
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
```

- [ ] **Step 2: Executar o teste para verificar a falha de compilação**

Run: `cargo test -p nexo-desktop --test lifecycle_tests regression_wmctrl_execution_failure_is_not_masked_as_pid_not_found`  
Expected: FAIL com `no variant WmctrlExecutionFailed in WindowBindingError`.

- [ ] **Step 3: Adicionar a variante `WmctrlExecutionFailed(i32)` e tratar explicitamente o status de saída**

Em `crates/nexo-desktop/tests/lifecycle_tests.rs`:
1. Atualizar `WindowBindingError`:
```rust
#[derive(Debug, PartialEq, Eq)]
pub enum WindowBindingError {
    PidNotFound(u32),
    AmbiguousWindows(Vec<String>),
    PidNotSupported,
    WmctrlExecutionFailed(i32),
}
```
2. Atualizar o `Display` de `WindowBindingError`:
```rust
WindowBindingError::WmctrlExecutionFailed(code) => {
    write!(f, "Falha na execução do utilitário wmctrl (código: {code})")
}
```
3. Em `poll_child_window`:
```rust
match fetch_window() {
    Ok(window) => return Ok(window),
    Err(WindowBindingError::PidNotFound(_)) => {
        // Continua aguardando a criação e mapeamento da janela
    }
    Err(err) => {
        // Erro fatal de comunicação, ambiguidade ou falta de suporte: encerra o polling imediatamente
        return Err(LifecycleError::WindowBinding(err));
    }
}
```
4. Em `fetch_window` dentro de `native_window_process_lifecycle_clean_exit`:
```rust
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
```
5. Em `close_window_by_id`:
```rust
pub fn close_window_by_id(window_id: &str) -> std::io::Result<ExitStatus> {
    Command::new("wmctrl")
        .args(["-i", "-c", window_id])
        .status()
}
```

- [ ] **Step 4: Executar todos os testes para garantir aprovação**

Run: `cargo test -p nexo-desktop --test lifecycle_tests`  
Expected: PASS.

- [ ] **Step 5: Commit das alterações da Task 2**

```bash
git add crates/nexo-desktop/tests/lifecycle_tests.rs
git commit -m "fix(desktop): verificar exit code do wmctrl e impedir mascaramento de erro como ausência de janela"
```

---

### Task 3: Bateria de Regressões dos 7 Cenários Obrigatórios

**Files:**
- Modify: `crates/nexo-desktop/tests/lifecycle_tests.rs`
- Test: `crates/nexo-desktop/tests/lifecycle_tests.rs`

**Interfaces:**
- Testes cobrindo expressamente:
  1. `regression_scenario_1_x11_ewmh_functional_supported`
  2. `regression_scenario_2_missing_display_and_wayland`
  3. `regression_scenario_3_pure_wayland_without_x11`
  4. `regression_scenario_4_wmctrl_present_but_execution_fails`
  5. `regression_scenario_5_child_premature_exit_preserved_as_failure`
  6. `regression_scenario_6_preexisting_windows_preserved`
  7. `regression_scenario_7_clean_exit_by_exact_window_id_and_pid`

- [ ] **Step 1: Escrever os 7 testes de regressão dedicados**

Em `crates/nexo-desktop/tests/lifecycle_tests.rs`, estruturar as 7 funções de regressão com nomes e asserções explícitas:

```rust
/// Cenário 1: Sessão X11 com EWMH funcional é identificada como Supported.
#[test]
fn regression_scenario_1_x11_ewmh_functional_supported() {
    let support = check_environment_support(Some(":0"), None, true, || Ok(()));
    assert_eq!(support, EnvironmentSupport::Supported);
}

/// Cenário 2: Ambiente sem DISPLAY e sem WAYLAND_DISPLAY retorna MissingDisplay.
#[test]
fn regression_scenario_2_missing_display_and_wayland() {
    assert_eq!(
        check_environment_support(None, None, true, || Ok(())),
        EnvironmentSupport::MissingDisplay
    );
    assert_eq!(
        check_environment_support(Some("  "), Some(""), true, || Ok(())),
        EnvironmentSupport::MissingDisplay
    );
}

/// Cenário 3: Wayland puro sem acesso a X11/EWMH retorna PureWaylandWithoutX11.
#[test]
fn regression_scenario_3_pure_wayland_without_x11() {
    assert_eq!(
        check_environment_support(None, Some("wayland-0"), true, || Ok(())),
        EnvironmentSupport::PureWaylandWithoutX11
    );
    assert_eq!(
        check_environment_support(Some(""), Some("wayland-1"), false, || Ok(())),
        EnvironmentSupport::PureWaylandWithoutX11
    );
}

/// Cenário 4: Presença de wmctrl, mas falha na sua execução retorna WmctrlCommunicationFailed.
#[test]
fn regression_scenario_4_wmctrl_present_but_execution_fails() {
    assert_eq!(
        check_environment_support(Some(":0"), None, true, || Err(1)),
        EnvironmentSupport::WmctrlCommunicationFailed(1)
    );
    assert_eq!(
        check_environment_support(Some(":1"), Some("wayland-0"), true, || Err(127)),
        EnvironmentSupport::WmctrlCommunicationFailed(127)
    );
}

/// Cenário 5: Processo filho com falha de inicialização em ambiente comprovadamente suportado
/// resulta em erro real de PrematureExit, nunca sendo convertido em skip.
#[test]
fn regression_scenario_5_child_premature_exit_preserved_as_failure() {
    let mut child = Command::new("false")
        .spawn()
        .expect("Failed to spawn process that exits with error");
    let mut guard = ChildGuard(Some(child));

    let dummy_fetch = || Ok(WindowInfo {
        id: "0x123".into(),
        desktop: 0,
        pid: 9999,
        host: "host".into(),
        title: "NEXO".into(),
    });

    let result = poll_child_window(
        guard.0.as_mut().unwrap(),
        Duration::from_millis(500),
        Duration::from_millis(20),
        dummy_fetch,
    );

    match result {
        Err(LifecycleError::PrematureExit(code)) => {
            assert_ne!(code, 0, "Deve preservar falha real do processo (código != 0)");
        }
        other => panic!("Esperava PrematureExit com código de erro, obteve: {other:?}"),
    }
}

/// Cenário 6: Preservação de janelas preexistentes com o mesmo título NEXO.
#[test]
fn regression_scenario_6_preexisting_windows_preserved() {
    let windows = vec![
        WindowInfo {
            id: "0x01".into(),
            desktop: 0,
            pid: 1001,
            host: "host".into(),
            title: "NEXO".into(),
        },
        WindowInfo {
            id: "0x02".into(),
            desktop: 0,
            pid: 1002,
            host: "host".into(),
            title: "NEXO".into(),
        },
    ];

    // Busca janela do PID 2000 (o filho atual do teste)
    let result = find_window_by_pid(&windows, 2000);
    assert_eq!(
        result,
        Err(WindowBindingError::PidNotFound(2000)),
        "Janelas preexistentes de outros PIDs não devem ser associadas"
    );
}

/// Cenário 7: Encerramento bem-sucedido por Window ID e PID associados.
#[test]
fn regression_scenario_7_clean_exit_by_exact_window_id_and_pid() {
    let target_pid = 4321;
    let windows = vec![
        WindowInfo {
            id: "0x01".into(),
            desktop: 0,
            pid: 1111,
            host: "host".into(),
            title: "NEXO".into(),
        },
        WindowInfo {
            id: "0x0a1b2c".into(),
            desktop: 0,
            pid: target_pid,
            host: "host".into(),
            title: "NEXO".into(),
        },
    ];

    let found = find_window_by_pid(&windows, target_pid).expect("Deve encontrar janela associada ao PID");
    assert_eq!(found.id, "0x0a1b2c", "Deve associar o Window ID exato da janela do processo");
    assert_eq!(found.pid, target_pid);
}
```

- [ ] **Step 2: Executar a suíte de testes de regressão**

Run: `cargo test -p nexo-desktop --test lifecycle_tests regression_scenario_`  
Expected: PASS (todos os 7 cenários passando com sucesso).

- [ ] **Step 3: Commit das regressões da Task 3**

```bash
git add crates/nexo-desktop/tests/lifecycle_tests.rs
git commit -m "test(desktop): adicionar cobertura para os 7 cenários obrigatórios de validação gráfica"
```

---

### Task 4: Atualização Documental do ADR 0002

**Files:**
- Modify: `docs/architecture/0002-rune-pin-and-linux-deps.md`

- [ ] **Step 1: Documentar as especificidades e limitações de X11, XWayland e Wayland puro**

No ADR 0002:
1. Detalhar:
   - **X11:** Gerenciador de janelas com suporte padrão a EWMH (`_NET_WM_PID`, `_NET_CLOSE_WINDOW`). Totalmente compatível com `wmctrl`.
   - **XWayland:** Camada de compatibilidade X11 sobre compositores Wayland. Requer variável `DISPLAY` ativa e compositor com suporte EWMH exportado.
   - **Wayland Puro:** Sessões exclusivas Wayland (`WAYLAND_DISPLAY` sem `DISPLAY` ou sem suporte a chamadas EWMH via X11). Utilitários como `wmctrl` não conseguem enumerar janelas Wayland nativas nem obter `_NET_WM_PID`. O teste reconhece essa limitação via `PureWaylandWithoutX11` e emite skip diagnóstico explícito, nunca fechando janelas alheias e nunca declarando teste nativo executado sem evidência real.
2. Atualizar o número de testes documentados: registrar os testes Rust reais (34 testes) e TypeScript (30 testes).

- [ ] **Step 2: Revisar o documento com `git diff`**

Run: `git diff docs/architecture/0002-rune-pin-and-linux-deps.md`  
Expected: Redação precisa, técnica e aderente aos fatos.

- [ ] **Step 3: Commit da documentação da Task 4**

```bash
git add docs/architecture/0002-rune-pin-and-linux-deps.md
git commit -m "docs(adr): documentar requisitos de X11, XWayland e limitações de Wayland puro no ADR 0002"
```

---

### Task 5: Validação Integral, Push para PR #10 e Relatório R4

**Files:**
- Modify: N/A
- Verify: Workspace completo

- [ ] **Step 1: Executar validações de qualidade e integridade do código**

Run:
```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace --locked
npm test
```
Expected: Todas as verificações limpas, 0 warnings, 0 erros.

- [ ] **Step 2: Enviar commits para a PR #10 no GitHub**

```bash
git push origin feat/m1-rune-linux-foundation
```

- [ ] **Step 3: Apresentar o relatório final da Remediação R4**

Resumo das evidências dos 7 cenários, contagem exata de testes executados e status da PR #10 sem fechamento precoce.
