#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]
#![allow(dead_code)]

mod async_sync;
mod autosave_hooks;
mod tabs;
mod types;
mod validation;

pub use types::{DiagramTab, DraggedIconPayload};

use crate::history::History;
use crate::hooks::e2e_reset::use_e2e_reset_hook;
use crate::hooks::keyboard::use_global_keyboard;
use crate::ui::canvas::Canvas;
use crate::ui::commands::ClipboardData;
use crate::ui::editor::ToolMode;
use crate::ui::minimap::Minimap;
use crate::ui::mobile::{use_sidebar_mobile_bridge, SidebarUiState};
use crate::ui::panels::PanelVisibility;
use crate::ui::sidebar::Sidebar;
use crate::ui::theme_provider::ThemeProvider;
use crate::ui::toast::{ToastQueue, Toaster};
use crate::ui::toolbar::{Toolbar, ToolbarStats};
use diagram_models::document::{ArrowType, DiagramDocument, EdgeStyle};

use crate::ui::ValidationPanel;
use dioxus::prelude::*;

#[allow(clippy::too_many_lines)]
pub fn App() -> Element {
    use_context_provider(|| Signal::new(DiagramDocument::default()));
    let _dragging_icon = use_context_provider(|| Signal::new(Option::<DraggedIconPayload>::None));
    use_context_provider(|| Signal::new(History::new()));
    use_context_provider(|| Signal::new(Option::<ClipboardData>::None));
    use_context_provider(|| Signal::new(ToolMode::Select));
    use_context_provider(|| Signal::new(EdgeStyle::Solid));
    use_context_provider(|| Signal::new(ArrowType::Default));
    use_context_provider(|| Signal::new(ToastQueue::default()));
    use_context_provider(|| Signal::new(PanelVisibility::default()));
    use_context_provider(|| Signal::new(ToolbarStats::default()));
    use_context_provider(|| Signal::new(SidebarUiState::default()));
    use_context_provider(|| Signal::new((1200.0_f64, 800.0_f64)));
    use_context_provider(|| Signal::new(0_u64));
    use_context_provider(|| Signal::new(Option::<crate::ui::toast::AiConflictState>::None));
    use_context_provider(|| Signal::new(false));
    use_context_provider(|| Signal::new(std::collections::HashSet::<String>::new()));

    let mut doc_signal = use_context::<Signal<DiagramDocument>>();
    let history_signal = use_context::<Signal<History>>();
    let validate_trigger = use_context::<Signal<u64>>();
    let sidebar_ui = use_context::<Signal<SidebarUiState>>();
    let panels = use_context::<Signal<PanelVisibility>>();
    let mut toolbar_stats = use_context::<Signal<ToolbarStats>>();

    let keyboard_db_tx = async_sync::provide_db_event_context();
    use_global_keyboard(keyboard_db_tx);
    use_e2e_reset_hook();

    async_sync::use_conflict_toast_effect();

    #[cfg(all(feature = "async-db", not(target_arch = "wasm32")))]
    async_sync::use_store_sync_loop(doc_signal);

    let tabs_state = tabs::use_tabs_logic(doc_signal, history_signal);
    let mut switch_tab = {
        let state = tabs_state;
        move |target_id: String| tabs::switch_tab(target_id, &state, doc_signal, history_signal)
    };
    let mut add_tab = {
        let state = tabs_state;
        move |_| tabs::add_tab(&state, doc_signal, history_signal)
    };
    let mut close_tab = {
        let state = tabs_state;
        move |close_id: String| tabs::close_tab(close_id, &state, doc_signal, history_signal)
    };

    use_sidebar_mobile_bridge(sidebar_ui, panels);

    #[cfg(target_arch = "wasm32")]
    autosave_hooks::use_auto_save(doc_signal);

    let validation_issues = validation::use_validation_state(doc_signal, validate_trigger);

    use_effect(move || {
        let doc = doc_signal.read();
        let next = ToolbarStats {
            selected_count: doc.editor_state.selected_items.len(),
            node_count: doc.document.nodes.len(),
            edge_count: doc.document.edges.len(),
        };
        if *toolbar_stats.read() != next {
            toolbar_stats.set(next);
        }
    });

    let active_tab_id = tabs_state.active_tab_id;
    let tab_names = tabs_state.tab_names;

    rsx! {
        ThemeProvider {
            div {
                style: "display: flex; flex-direction: row; background: var(--bg-surface); border-bottom: 1px solid var(--border); overflow-x: auto; padding: 4px 8px 0 8px; gap: 4px;",
                for (id, name) in tab_names.read().iter() {
                    {
                        let is_active = *id == *active_tab_id.read();
                        let bg = if is_active { "var(--bg-base)" } else { "var(--bg-surface)" };
                        let fg = if is_active { "var(--text-main)" } else { "var(--text-muted)" };
                        rsx! {
                            div {
                                key: "{id}",
                                style: "display: flex; align-items: center; padding: 6px 12px; border-radius: 6px 6px 0 0; cursor: pointer; border: 1px solid var(--border); border-bottom: none; background: {bg}; color: {fg}; min-width: max-content;",
                                onclick: {
                                    let target_id = id.clone();
                                    move |_| switch_tab(target_id.clone())
                                },
                                span { "{name}" }
                                if tab_names.read().len() > 1 {
                                    button {
                                        style: "margin-left: 8px; padding: 2px 4px; background: transparent; border: none; cursor: pointer; color: var(--text-muted); font-size: 14px; line-height: 1;",
                                        onclick: {
                                            let close_id = id.clone();
                                            move |evt| {
                                                evt.stop_propagation();
                                                close_tab(close_id.clone());
                                            }
                                        },
                                        "×"
                                    }
                                }
                            }
                        }
                    }
                }
                button {
                    style: "margin-left: 4px; padding: 4px 12px; background: transparent; border: 1px dashed var(--border); border-radius: 6px 6px 0 0; border-bottom: none; cursor: pointer; color: var(--text-muted); font-size: 16px;",
                    onclick: add_tab,
                    "+"
                }
            }
            Toolbar {}
            Toaster {}

            div {
                display: "flex",
                flex: "1",
                overflow: "hidden",
                min_width: "0",

                if panels.read().sidebar {
                    Sidebar {}
                }
                div {
                    display: "flex",
                    flex: "1",
                    position: "relative",
                    Canvas {}
                    if panels.read().minimap {
                        Minimap {}
                    }
                }
                if panels.read().validation {
                    ValidationPanel { issues: validation_issues }
                }
            }
        }
    }
}
