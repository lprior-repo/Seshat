#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

use crate::app::AppState;
use crate::ui::commands::{
    apply_delete_selected, apply_redo, apply_toggle_edge_direction, apply_undo, apply_zoom_in,
    apply_zoom_out, apply_zoom_reset,
};
use crate::ui::editor::ToolMode;
use dioxus::prelude::*;

use super::{open_workspace, save_workspace, WorkspaceSignals};

#[derive(Clone, PartialEq, Props)]
struct MenuActionButtonProps {
    test_id: &'static str,
    label: String,
    disabled: bool,
    onclick: EventHandler<MouseEvent>,
}

#[component]
fn MenuActionButton(props: MenuActionButtonProps) -> Element {
    let disabled_class = if props.disabled {
        "cursor-not-allowed opacity-40"
    } else {
        "cursor-pointer hover:bg-[var(--bg-elevated)]"
    };

    rsx! {
        button {
            "data-testid": props.test_id,
            role: "menuitem",
            disabled: props.disabled,
            class: "w-full rounded-md px-3 py-2 text-left text-[13px] text-foreground outline-none transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-[var(--accent)] focus-visible:outline-offset-2 {disabled_class}",
            "aria-label": props.label,
            onclick: move |evt| {
                if !props.disabled {
                    props.onclick.call(evt);
                }
            },
            "{props.label}"
        }
    }
}

#[component]
pub fn MobileToolbarMenu() -> Element {
    let app_state = use_context::<AppState>();
    let mut doc_signal = app_state.document;
    let session_signal = app_state.session;
    let history_signal = app_state.history;
    let mut tool_signal = app_state.tool_mode;
    let mut arrow_type_signal = app_state.arrow_type;
    let toasts = app_state.toasts;
    let viewport_size_signal = app_state.viewport_size;
    let toolbar_stats = app_state.toolbar_stats;
    let stats = *toolbar_stats.read();

    let mut menu_open = use_signal(|| false);
    let is_open = *menu_open.read();
    let undo_disabled = !history_signal.read().can_undo();
    let redo_disabled = !history_signal.read().can_redo();
    let show_grid = doc_signal.read().editor_state.show_grid;

    rsx! {
        div { class: "relative md:hidden",
            button {
                "data-testid": "toolbar-more",
                class: "h-9 px-2.5 flex items-center justify-center rounded-md border border-border bg-[var(--toolbar-bg)] text-foreground hover:bg-[var(--bg-elevated)] cursor-pointer text-[12px] transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-[var(--accent)] focus-visible:outline-offset-2",
                title: "More toolbar actions",
                "aria-label": "More toolbar actions",
                "aria-expanded": "{is_open}",
                "aria-controls": "toolbar-more-menu",
                onclick: move |_| menu_open.set(!is_open),
                "More"
            }

            if is_open {
                div {
                    id: "toolbar-more-menu",
                    "data-testid": "toolbar-more-menu",
                    role: "menu",
                    class: "absolute right-0 top-11 z-[1200] w-[calc(100vw-16px)] max-w-[280px] rounded-xl border border-border bg-surface p-2 shadow-xl",
                    div { class: "grid grid-cols-2 gap-1",
                        MenuActionButton {
                            test_id: "mobile-tool-subgraph",
                            label: String::from("Subgraph"),
                            disabled: false,
                            onclick: move |_| {
                                tool_signal.set(ToolMode::Subgraph);
                                menu_open.set(false);
                            }
                        }
                        MenuActionButton {
                            test_id: "mobile-tool-grid",
                            label: if show_grid { String::from("Hide grid") } else { String::from("Show grid") },
                            disabled: false,
                            onclick: move |_| {
                                doc_signal.with_mut(|doc| {
                                    doc.editor_state.show_grid = !doc.editor_state.show_grid;
                                });
                                menu_open.set(false);
                            }
                        }
                        MenuActionButton {
                            test_id: "mobile-toolbar-undo",
                            label: String::from("Undo"),
                            disabled: undo_disabled,
                            onclick: move |_| {
                                apply_undo(doc_signal, history_signal);
                                menu_open.set(false);
                            }
                        }
                        MenuActionButton {
                            test_id: "mobile-toolbar-redo",
                            label: String::from("Redo"),
                            disabled: redo_disabled,
                            onclick: move |_| {
                                apply_redo(doc_signal, history_signal);
                                menu_open.set(false);
                            }
                        }
                        MenuActionButton {
                            test_id: "mobile-zoom-in",
                            label: String::from("Zoom in"),
                            disabled: false,
                            onclick: move |_| {
                                let _ = apply_zoom_in(doc_signal, history_signal, *viewport_size_signal.read());
                                menu_open.set(false);
                            }
                        }
                        MenuActionButton {
                            test_id: "mobile-zoom-reset",
                            label: String::from("Reset zoom"),
                            disabled: false,
                            onclick: move |_| {
                                let _ = apply_zoom_reset(doc_signal, history_signal, *viewport_size_signal.read());
                                menu_open.set(false);
                            }
                        }
                        MenuActionButton {
                            test_id: "mobile-zoom-out",
                            label: String::from("Zoom out"),
                            disabled: false,
                            onclick: move |_| {
                                let _ = apply_zoom_out(doc_signal, history_signal, *viewport_size_signal.read());
                                menu_open.set(false);
                            }
                        }
                        MenuActionButton {
                            test_id: "mobile-toolbar-delete",
                            label: String::from("Delete selection"),
                            disabled: stats.selected_count == 0,
                            onclick: move |_| {
                                let _ = apply_delete_selected(doc_signal, history_signal);
                                menu_open.set(false);
                            }
                        }
                        MenuActionButton {
                            test_id: "mobile-style-arrow-type",
                            label: match *arrow_type_signal.read() {
                                diagram_models::document::ArrowType::Default => String::from("Arrow: Default"),
                                diagram_models::document::ArrowType::Sharp => String::from("Arrow: Sharp"),
                                diagram_models::document::ArrowType::Curved => String::from("Arrow: Curved"),
                                diagram_models::document::ArrowType::Step => String::from("Arrow: Step"),
                                diagram_models::document::ArrowType::Straight => String::from("Arrow: Straight"),
                            },
                            disabled: false,
                            onclick: move |_| {
                                use diagram_models::document::ArrowType;
                                let next_type = match *arrow_type_signal.read() {
                                    ArrowType::Default => ArrowType::Sharp,
                                    ArrowType::Sharp => ArrowType::Curved,
                                    ArrowType::Curved => ArrowType::Step,
                                    ArrowType::Step => ArrowType::Straight,
                                    ArrowType::Straight => ArrowType::Default,
                                };
                                arrow_type_signal.set(next_type);
                                let _ = crate::ui::commands::apply_arrow_type_to_selection(doc_signal, history_signal, next_type);
                                menu_open.set(false);
                            }
                        }
                        MenuActionButton {
                            test_id: "mobile-style-arrow",
                            label: String::from("Toggle edge direction"),
                            disabled: false,
                            onclick: move |_| {
                                let _ = apply_toggle_edge_direction(doc_signal, history_signal);
                                menu_open.set(false);
                            }
                        }
                        MenuActionButton {
                            test_id: "mobile-toolbar-export",
                            label: String::from("Export workspace"),
                            disabled: false,
                            onclick: move |_| {
                                save_workspace(doc_signal, session_signal, toasts);
                                menu_open.set(false);
                            }
                        }
                        MenuActionButton {
                            test_id: "mobile-toolbar-import",
                            label: String::from("Import workspace"),
                            disabled: false,
                            onclick: move |_| {
                                let signals = WorkspaceSignals {
                                    doc: doc_signal,
                                    session: session_signal,
                                    history: history_signal,
                                };
                                open_workspace(signals, toasts);
                                menu_open.set(false);
                            }
                        }
                    }
                }
            }
        }
    }
}
