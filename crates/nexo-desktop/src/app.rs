//! Interface desktop nativa do NEXO construída com o framework Rune.

use nexo_domain::{NexoState, ViewMode};
use rune::tok::{motion, type_};
use rune::{Cx, El, Key, KeyEvent, Tx, col, el, row, text};

use crate::theme::get_theme;

/// Estado do aplicativo desktop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub domain: NexoState,
    pub status_message: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            domain: NexoState::default(),
            status_message: "Pronto para exploração offline".to_string(),
        }
    }
}

/// Constrói a árvore de visualização Rune baseada no estado atual.
pub fn view(s: &AppState, _: &mut Cx<AppState>) -> El {
    let t = get_theme(s.domain.dark_theme);

    // 1. Cabeçalho
    let title = text("NEXO — Observatório Local")
        .font(type_::TITLE)
        .color(t.text);

    let version_badge = text(format!("v{}", s.domain.app_version))
        .font(type_::LABEL)
        .color(t.accent);

    let header_left = row().center().gap(8.0).child(title).child(version_badge);

    // Botão de alternância de tema
    let theme_label = text(if s.domain.dark_theme {
        "🌙 Escuro"
    } else {
        "☀️ Claro"
    })
    .font(type_::LABEL)
    .color(t.text);

    let theme_btn = el()
        .key("theme_switch")
        .role("button")
        .focusable(true)
        .padding(8.0)
        .radius(6.0)
        .fill(t.control_off)
        .motion(motion::SNAPPY)
        .on_press(|s: &mut AppState| {
            s.domain.toggle_theme();
            s.status_message = format!(
                "Tema alternado para {}",
                if s.domain.dark_theme {
                    "escuro"
                } else {
                    "claro"
                }
            );
        })
        .child(theme_label);

    let header = row()
        .w_full()
        .padding(16.0)
        .fill(t.bg)
        .child(header_left)
        .child(el().grow(1.0))
        .child(theme_btn);

    // 2. Barra de navegação por abas/modos
    let btn_dash_label = text("📊 Visão Geral").font(type_::BODY).color(
        if s.domain.view_mode == ViewMode::Dashboard {
            t.accent
        } else {
            t.text_secondary
        },
    );

    let btn_dash = el()
        .key("tab_dashboard")
        .role("tab")
        .focusable(true)
        .padding(8.0)
        .radius(4.0)
        .fill(if s.domain.view_mode == ViewMode::Dashboard {
            t.control_off
        } else {
            t.bg
        })
        .on_press(|s: &mut AppState| {
            s.domain.set_view_mode(ViewMode::Dashboard);
            s.status_message = "Visualizando painel de métricas".to_string();
        })
        .child(btn_dash_label);

    let btn_graph_label =
        text("🕸️ Grafo")
            .font(type_::BODY)
            .color(if s.domain.view_mode == ViewMode::Graph {
                t.accent
            } else {
                t.text_secondary
            });

    let btn_graph = el()
        .key("tab_graph")
        .role("tab")
        .focusable(true)
        .padding(8.0)
        .radius(4.0)
        .fill(if s.domain.view_mode == ViewMode::Graph {
            t.control_off
        } else {
            t.bg
        })
        .on_press(|s: &mut AppState| {
            s.domain.set_view_mode(ViewMode::Graph);
            s.status_message = "Visualizando grafo de dependências".to_string();
        })
        .child(btn_graph_label);

    let btn_table_label =
        text("📋 Repositórios")
            .font(type_::BODY)
            .color(if s.domain.view_mode == ViewMode::Table {
                t.accent
            } else {
                t.text_secondary
            });

    let btn_table = el()
        .key("tab_table")
        .role("tab")
        .focusable(true)
        .padding(8.0)
        .radius(4.0)
        .fill(if s.domain.view_mode == ViewMode::Table {
            t.control_off
        } else {
            t.bg
        })
        .on_press(|s: &mut AppState| {
            s.domain.set_view_mode(ViewMode::Table);
            s.status_message = "Visualizando tabela de repositórios".to_string();
        })
        .child(btn_table_label);

    let nav_bar = row()
        .w_full()
        .gap(8.0)
        .padding(8.0)
        .fill(t.bg)
        .child(btn_dash)
        .child(btn_graph)
        .child(btn_table);

    // 3. Conteúdo principal
    let content_text = match s.domain.view_mode {
        ViewMode::Dashboard => "Modo Dashboard: Métricas de integridade e atividade",
        ViewMode::Graph => "Modo Grafo: Exploração de conexões entre repositórios",
        ViewMode::Table => "Modo Tabela: Detalhes tabulares e anotações locais",
    };

    let main_desc = text(content_text).font(type_::BODY).color(t.text);

    // Botão de interação principal (contador)
    let action_btn_text = text(format!("Interagir ({})", s.domain.interaction_count))
        .font(type_::LABEL)
        .color(t.text);

    let action_btn = el()
        .key("action_button")
        .role("button")
        .focusable(true)
        .padding(10.0)
        .radius(8.0)
        .fill(t.accent)
        .motion(motion::SNAPPY)
        .on_press(|s: &mut AppState| {
            s.domain.increment_interaction();
            s.status_message = format!(
                "Ação executada! Total de interações: {}",
                s.domain.interaction_count
            );
        })
        .child(action_btn_text);

    // Botão de simulação de erro
    let error_btn_text = text("Simular Erro").font(type_::LABEL).color(t.text);

    let error_btn = el()
        .key("trigger_error_btn")
        .role("button")
        .focusable(true)
        .padding(10.0)
        .radius(8.0)
        .fill(t.control_off)
        .on_press(|s: &mut AppState| {
            s.domain
                .set_error("Erro de teste: Falha intencional de verificação");
            s.status_message = "Erro reportado na interface".to_string();
        })
        .child(error_btn_text);

    let action_row = row().gap(12.0).child(action_btn).child(error_btn);

    let mut content_col = col()
        .w_full()
        .grow(1.0)
        .padding(20.0)
        .gap(16.0)
        .fill(t.card)
        .child(main_desc)
        .child(action_row);

    // Exibição de erro com espaçamento adequado se presente
    if let Some(ref err_msg) = s.domain.error_message {
        let err_label = text(format!("⚠️ {}", err_msg))
            .font(type_::BODY)
            .color(t.destructive);

        let clear_btn_label = text("Fechar").font(type_::LABEL).color(t.text);

        let clear_btn = el()
            .key("clear_error_btn")
            .role("button")
            .focusable(true)
            .padding(6.0)
            .radius(4.0)
            .fill(t.bg)
            .on_press(|s: &mut AppState| {
                s.domain.clear_error();
                s.status_message = "Erro limpo".to_string();
            })
            .child(clear_btn_label);

        let err_box = row()
            .key("error_banner")
            .w_full()
            .padding(12.0)
            .radius(8.0)
            .fill(t.bg)
            .child(err_label)
            .child(el().grow(1.0))
            .child(clear_btn);

        content_col = content_col.child(err_box);
    }

    // 4. Rodapé / Status bar com espaçamento garantido
    let status_label = text(&s.status_message)
        .font(type_::LABEL)
        .color(t.text_secondary);

    let interactions_label = text(format!("Interações: {}", s.domain.interaction_count))
        .font(type_::LABEL)
        .color(t.text_secondary);

    let footer = row()
        .key("status_bar")
        .w_full()
        .padding(10.0)
        .fill(t.bg)
        .child(status_label)
        .child(el().grow(1.0))
        .child(interactions_label);

    // Montagem do layout completo
    col()
        .key("app_root")
        .w_full()
        .h_full()
        .fill(t.bg)
        .child(header)
        .child(nav_bar)
        .child(content_col)
        .child(footer)
}

/// Trata eventos globais de teclado no nível da janela/aplicativo.
pub fn handle_key(s: &mut AppState, ev: &KeyEvent, _: &mut Tx) -> bool {
    if ev.key == Key::Escape && s.domain.error_message.is_some() {
        s.domain.clear_error();
        s.status_message = "Erro limpo via teclado (Escape)".to_string();
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rune::{Env, Key, KeyEvent, Mods, Runtime};

    fn make_test_runtime() -> Runtime<AppState> {
        let env = Env {
            system_dark: true,
            scale: 2.0,
        };
        let mut rt = Runtime::new(AppState::default(), view, (900.0, 600.0), env);
        rt.on_key = Some(handle_key);
        rt.frame(0.0);
        rt
    }

    fn press_key(rt: &mut Runtime<AppState>, key: Key) {
        rt.key(KeyEvent {
            key,
            mods: Mods::default(),
        });
        rt.frame(0.0);
    }

    fn focused_key(rt: &Runtime<AppState>) -> Option<String> {
        rt.elements()
            .into_iter()
            .find(|e| e.focused)
            .and_then(|e| e.key)
    }

    #[test]
    fn app_initializes_with_default_state() {
        let rt = make_test_runtime();
        assert_eq!(rt.state.domain.app_version, "0.1.0");
        assert_eq!(rt.state.domain.view_mode, ViewMode::Dashboard);
        assert!(rt.state.domain.dark_theme);
        assert_eq!(rt.state.domain.interaction_count, 0);
        assert_eq!(rt.state.domain.error_message, None);
        assert_eq!(rt.state.status_message, "Pronto para exploração offline");
    }

    #[test]
    fn clicking_theme_switch_toggles_theme() {
        let mut rt = make_test_runtime();
        let (x, y) = rt
            .find("theme_switch")
            .expect("theme_switch element not found");
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        for _ in 0..6 {
            rt.frame(1.0 / 60.0);
        }

        assert!(!rt.state.domain.dark_theme);
        assert_eq!(rt.state.domain.interaction_count, 1);
        assert!(rt.state.status_message.contains("claro"));
    }

    #[test]
    fn clicking_action_button_increments_counter() {
        let mut rt = make_test_runtime();
        let (x, y) = rt.find("action_button").expect("action_button not found");
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        for _ in 0..6 {
            rt.frame(1.0 / 60.0);
        }

        assert_eq!(rt.state.domain.interaction_count, 1);
        assert!(rt.state.status_message.contains("Total de interações: 1"));

        // Second click
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        for _ in 0..6 {
            rt.frame(1.0 / 60.0);
        }

        assert_eq!(rt.state.domain.interaction_count, 2);
    }

    #[test]
    fn tab_switching_updates_view_mode() {
        let mut rt = make_test_runtime();

        // Switch to Graph tab
        let (gx, gy) = rt.find("tab_graph").expect("tab_graph not found");
        rt.pointer_down(gx, gy);
        rt.pointer_up(gx, gy);
        for _ in 0..6 {
            rt.frame(1.0 / 60.0);
        }

        assert_eq!(rt.state.domain.view_mode, ViewMode::Graph);
        assert_eq!(rt.state.domain.interaction_count, 1);

        // Switch to Table tab
        let (tx, ty) = rt.find("tab_table").expect("tab_table not found");
        rt.pointer_down(tx, ty);
        rt.pointer_up(tx, ty);
        for _ in 0..6 {
            rt.frame(1.0 / 60.0);
        }

        assert_eq!(rt.state.domain.view_mode, ViewMode::Table);
        assert_eq!(rt.state.domain.interaction_count, 2);
    }

    #[test]
    fn error_lifecycle_display_and_clear() {
        let mut rt = make_test_runtime();
        assert!(rt.find("error_banner").is_none());

        // Trigger error
        let (ex, ey) = rt
            .find("trigger_error_btn")
            .expect("trigger_error_btn not found");
        rt.pointer_down(ex, ey);
        rt.pointer_up(ex, ey);
        for _ in 0..6 {
            rt.frame(1.0 / 60.0);
        }

        assert!(rt.state.domain.error_message.is_some());
        assert!(rt.find("error_banner").is_some());

        // Clear error
        let (cx, cy) = rt
            .find("clear_error_btn")
            .expect("clear_error_btn not found");
        rt.pointer_down(cx, cy);
        rt.pointer_up(cx, cy);
        for _ in 0..6 {
            rt.frame(1.0 / 60.0);
        }

        assert!(rt.state.domain.error_message.is_none());
        assert!(rt.find("error_banner").is_none());
    }

    #[test]
    fn keyboard_tab_navigation_and_space_activation() {
        let mut rt = make_test_runtime();

        // 1. Pressionar Tab move o foco para o primeiro controle focável (theme_switch)
        press_key(&mut rt, Key::Tab);
        assert_eq!(
            focused_key(&rt).as_deref(),
            Some("theme_switch"),
            "Tab deve focar theme_switch"
        );

        // 2. Acionar theme_switch com Space (semântica oficial Rune para botões/roles)
        press_key(&mut rt, Key::Text(" ".into()));
        for _ in 0..6 {
            rt.frame(1.0 / 60.0);
        }
        assert!(
            !rt.state.domain.dark_theme,
            "Space no theme_switch deve alternar tema para claro"
        );
        assert_eq!(rt.state.domain.interaction_count, 1);

        // 3. Tab para tab_dashboard
        press_key(&mut rt, Key::Tab);
        assert_eq!(focused_key(&rt).as_deref(), Some("tab_dashboard"));

        // 4. Tab para tab_graph
        press_key(&mut rt, Key::Tab);
        assert_eq!(focused_key(&rt).as_deref(), Some("tab_graph"));

        // 5. Ativar tab_graph com Space
        press_key(&mut rt, Key::Text(" ".into()));
        for _ in 0..6 {
            rt.frame(1.0 / 60.0);
        }
        assert_eq!(
            rt.state.domain.view_mode,
            ViewMode::Graph,
            "Space em tab_graph deve mudar view_mode"
        );
        assert_eq!(rt.state.domain.interaction_count, 2);

        // 6. Tab para tab_table
        press_key(&mut rt, Key::Tab);
        assert_eq!(focused_key(&rt).as_deref(), Some("tab_table"));

        // 7. Ativar tab_table com Space
        press_key(&mut rt, Key::Text(" ".into()));
        for _ in 0..6 {
            rt.frame(1.0 / 60.0);
        }
        assert_eq!(
            rt.state.domain.view_mode,
            ViewMode::Table,
            "Space em tab_table deve mudar view_mode"
        );
        assert_eq!(rt.state.domain.interaction_count, 3);

        // 8. Tab para action_button
        press_key(&mut rt, Key::Tab);
        assert_eq!(focused_key(&rt).as_deref(), Some("action_button"));

        // 9. Ativar action_button com Space
        press_key(&mut rt, Key::Text(" ".into()));
        for _ in 0..6 {
            rt.frame(1.0 / 60.0);
        }
        assert_eq!(
            rt.state.domain.interaction_count, 4,
            "Space em action_button deve incrementar interações"
        );

        // 10. Tab para trigger_error_btn
        press_key(&mut rt, Key::Tab);
        assert_eq!(focused_key(&rt).as_deref(), Some("trigger_error_btn"));

        // 11. Disparar erro com Space
        press_key(&mut rt, Key::Text(" ".into()));
        for _ in 0..6 {
            rt.frame(1.0 / 60.0);
        }
        assert!(rt.state.domain.error_message.is_some());
        assert!(rt.find("error_banner").is_some());

        // 12. Tab para o botão clear_error_btn que acabou de aparecer
        press_key(&mut rt, Key::Tab);
        assert_eq!(
            focused_key(&rt).as_deref(),
            Some("clear_error_btn"),
            "Tab deve focar clear_error_btn"
        );

        // 13. Fechar erro via teclado com Space
        press_key(&mut rt, Key::Text(" ".into()));
        for _ in 0..6 {
            rt.frame(1.0 / 60.0);
        }
        assert!(
            rt.state.domain.error_message.is_none(),
            "Space em clear_error_btn deve limpar o erro"
        );
    }

    #[test]
    fn keyboard_escape_dismisses_error_banner() {
        let mut rt = make_test_runtime();

        // Disparar erro diretamente no estado
        rt.update(|s, _| s.domain.set_error("Erro acidental"));
        rt.frame(0.0);
        assert!(rt.find("error_banner").is_some());

        // Pressionar Escape deve invocar o handler on_key e limpar o erro
        press_key(&mut rt, Key::Escape);
        for _ in 0..6 {
            rt.frame(1.0 / 60.0);
        }
        assert!(rt.state.domain.error_message.is_none());
        assert!(rt.find("error_banner").is_none());
        assert!(rt.state.status_message.contains("Escape"));
    }

    #[test]
    fn layout_responsive_separates_elements_across_resolutions() {
        // Testar em 3 resoluções distintas: compacta (640x480), padrão (900x600) e widescreen (1920x1080)
        let resolutions = [(640.0, 480.0), (900.0, 600.0), (1920.0, 1080.0)];

        for (w, h) in resolutions {
            let env = Env {
                system_dark: true,
                scale: 2.0,
            };
            let mut rt = Runtime::new(AppState::default(), view, (w, h), env);
            rt.frame(0.0);

            // Verificar posições dos elementos no rodapé (status_bar)
            let elements = rt.elements();
            let status_bar = elements
                .iter()
                .find(|e| e.key.as_deref() == Some("status_bar"))
                .expect("status_bar not found");
            assert!(
                status_bar.rect[2] > 0.0,
                "status_bar deve ter largura positiva em {}x{}",
                w,
                h
            );

            // Renderizar 10 frames sem qualquer erro
            for _ in 0..10 {
                rt.frame(1.0 / 60.0);
            }
        }
    }

    #[test]
    fn multiple_frames_render_without_panic() {
        let mut rt = make_test_runtime();
        for _ in 0..120 {
            rt.frame(1.0 / 60.0);
        }
        assert_eq!(rt.state.domain.interaction_count, 0);
    }
}
