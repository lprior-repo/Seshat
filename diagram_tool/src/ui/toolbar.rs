#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

pub mod auto_save;
mod persistence;
mod persistence_compat;

#[cfg(test)]
mod tests;

use crate::app::AppState;
use crate::mutation::error::MutationError;
use crate::ui::commands::{
    apply_delete_selected, apply_redo, apply_toggle_edge_direction, apply_undo, apply_zoom_in,
    apply_zoom_out, apply_zoom_reset,
};
use crate::ui::editor::ToolMode;
use crate::ui::icons::{Icon, IconKind};
use crate::ui::theme::ACCENT;
use diagram_models::envelope::EventEnvelope;
use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub struct ToolbarStats {
    pub selected_count: usize,
    pub node_count: usize,
    pub edge_count: usize,
    pub revision: u64,
}

#[component]
fn Divider() -> Element {
    rsx! {
        div {
            class: "w-[1px] h-6 bg-[var(--border-subtle)] mx-1"
        }
    }
}

#[component]
fn IconButton(
    test_id: &'static str,
    active: Option<bool>,
    onclick: EventHandler<MouseEvent>,
    disabled: Option<bool>,
    icon: IconKind,
    color: Option<&'static str>,
    active_bg: Option<&'static str>,
    title: Option<&'static str>,
) -> Element {
    let is_active = active.unwrap_or(false);
    let is_disabled = disabled.unwrap_or(false);
    let bg = if is_active {
        active_bg.unwrap_or("bg-[var(--accent-soft)]")
    } else {
        "bg-transparent hover:bg-white/5"
    };
    let border = if is_active {
        "border-[var(--accent)]"
    } else {
        "border-transparent"
    };
    let fill = if is_active { Some(ACCENT) } else { color };
    let opacity = if is_disabled {
        "opacity-40"
    } else {
        "opacity-100"
    };
    let cursor = if is_disabled {
        "cursor-not-allowed"
    } else {
        "cursor-pointer"
    };
    let tooltip = title.unwrap_or("");

    rsx! {
        button {
            "data-testid": test_id,
            class: "w-9 h-9 flex items-center justify-center rounded-md border {border} {bg} {cursor} {opacity} p-0 outline-none mx-0.5 text-foreground transition-colors",
            title: "{tooltip}",
            onclick: move |evt| {
                if !is_disabled {
                    onclick.call(evt);
                }
            },
            Icon { kind: icon, color: fill, size: 20 }
        }
    }
}

#[component]
fn TextButton(
    test_id: &'static str,
    active: Option<bool>,
    onclick: EventHandler<MouseEvent>,
    disabled: Option<bool>,
    text: &'static str,
    title: Option<&'static str>,
) -> Element {
    let is_active = active.unwrap_or(false);
    let is_disabled = disabled.unwrap_or(false);
    let bg = if is_active {
        "bg-[var(--accent-soft)]"
    } else {
        "bg-transparent hover:bg-white/5"
    };
    let border = if is_active {
        "border-[var(--accent)]"
    } else {
        "border-transparent"
    };
    let fill = if is_active {
        "text-[var(--accent)]"
    } else {
        "text-foreground"
    };
    let opacity = if is_disabled {
        "opacity-40"
    } else {
        "opacity-100"
    };
    let cursor = if is_disabled {
        "cursor-not-allowed"
    } else {
        "cursor-pointer"
    };
    let tooltip = title.unwrap_or("");

    rsx! {
        button {
            "data-testid": test_id,
            class: "w-9 h-9 flex items-center justify-center rounded-md border {border} {bg} {cursor} {opacity} p-0 outline-none mx-0.5 {fill} text-[18px] font-medium font-mono transition-colors",
            title: "{tooltip}",
            onclick: move |evt| {
                if !is_disabled {
                    onclick.call(evt);
                }
            },
            "{text}"
        }
    }
}

#[component]
fn LabelButton(
    test_id: &'static str,
    icon: Option<IconKind>,
    text: &'static str,
    onclick: Option<EventHandler<MouseEvent>>,
) -> Element {
    rsx! {
        button {
            "data-testid": test_id,
            class: "h-9 px-2 flex items-center justify-center rounded-md border border-transparent bg-transparent hover:bg-white/5 cursor-pointer p-0 outline-none mx-0.5 text-foreground text-[13px] transition-colors gap-1.5",
            onclick: move |evt| {
                if let Some(handler) = &onclick {
                    handler.call(evt);
                }
            },
            if let Some(i) = icon {
                Icon { kind: i, size: 16 }
            }
            span { "{text}" }
        }
    }
}

#[component]
pub fn Toolbar() -> Element {
    let app_state = use_context::<AppState>();
    let db_tx = use_context::<Option<Coroutine<EventEnvelope>>>();
    let mut doc_signal = app_state.document;
    let history_signal = app_state.history;
    let mut tool_signal = app_state.tool_mode;
    let edge_style_signal = app_state.edge_style;
    let mut arrow_type_signal = app_state.arrow_type;
    let toasts = app_state.toasts;

    let toolbar_stats = app_state.toolbar_stats;
    let stats = *toolbar_stats.read();
    let viewport_size_signal = app_state.viewport_size;

    let undo_disabled = !history_signal.read().can_undo();
    let redo_disabled = !history_signal.read().can_redo();
    let doc = doc_signal.read();
    let zoom_percent = (doc.editor_state.zoom.0 * 100.0).round();
    let show_grid = doc.editor_state.show_grid;
    drop(doc);

    rsx! {
        div {
            "data-testid": "toolbar-root",
            class: "h-14 shrink-0 bg-surface flex items-center justify-between px-4 border-b border-border-subtle w-full",

            // Left section: Tools and actions
            div {
                class: "flex items-center",

                // Tools
                IconButton {
                    test_id: "tool-select",
                    active: *tool_signal.read() == ToolMode::Select,
                    onclick: move |_| tool_signal.set(ToolMode::Select),
                    icon: IconKind::Select,
                    title: "Select (V)"
                }
                IconButton {
                    test_id: "tool-pan",
                    active: *tool_signal.read() == ToolMode::Pan,
                    onclick: move |_| tool_signal.set(ToolMode::Pan),
                    icon: IconKind::Pan,
                    title: "Pan (H)"
                }
                IconButton {
                    test_id: "tool-edge",
                    active: *tool_signal.read() == ToolMode::Edge,
                    onclick: move |_| tool_signal.set(ToolMode::Edge),
                    icon: IconKind::Edge,
                    title: "Edge (L)"
                }
                IconButton {
                    test_id: "tool-subgraph",
                    active: *tool_signal.read() == ToolMode::Subgraph,
                    onclick: move |_| tool_signal.set(ToolMode::Subgraph),
                    icon: IconKind::Subgraph,
                    title: "Subgraph (R)"
                }
                TextButton {
                    test_id: "tool-text",
                    active: *tool_signal.read() == ToolMode::Text,
                    onclick: move |_| tool_signal.set(ToolMode::Text),
                    text: "T",
                    title: "Text (T)"
                }
                IconButton {
                    test_id: "tool-grid",
                    active: show_grid,
                    active_bg: "bg-[oklch(0.4_0.1_160)]", // Darker green highlight for grid
                    onclick: move |_| {
                        let mut d = doc_signal.write();
                        d.editor_state.show_grid = !d.editor_state.show_grid;
                    },
                    icon: IconKind::Grid,
                    title: "Toggle Grid"
                }

                Divider {}

                // History
                IconButton {
                    test_id: "toolbar-undo",
                    disabled: undo_disabled,
                    onclick: move |_| { apply_undo(doc_signal, history_signal); },
                    icon: IconKind::Undo,
                    color: None,
                    title: "Undo"
                }
                IconButton {
                    test_id: "toolbar-redo",
                    disabled: redo_disabled,
                    onclick: move |_| { apply_redo(doc_signal, history_signal); },
                    icon: IconKind::Redo,
                    color: None,
                    title: "Redo"
                }

                Divider {}

                // Zoom
                IconButton {
                    test_id: "zoom-in",
                    onclick: move |_| { let _ = apply_zoom_in(doc_signal, history_signal, *viewport_size_signal.read()); },
                    icon: IconKind::ZoomIn,
                    color: None,
                    title: "Zoom In"
                }
                div {
                    "data-testid": "zoom-reset",
                    "data-zoom-percent": "{zoom_percent:.0}",
                    class: "text-foreground font-mono text-[13px] mx-2 cursor-pointer select-none hover:text-[var(--accent)] transition-colors w-[45px] text-center",
                    onclick: move |_| { let _ = apply_zoom_reset(doc_signal, history_signal, *viewport_size_signal.read()); },
                    title: "Reset zoom",
                    "{zoom_percent:.0}%"
                }
                IconButton {
                    test_id: "zoom-out",
                    onclick: move |_| { let _ = apply_zoom_out(doc_signal, history_signal, *viewport_size_signal.read()); },
                    icon: IconKind::ZoomOut,
                    color: None,
                    title: "Zoom Out"
                }

                Divider {}

                // Delete
                IconButton {
                    test_id: "toolbar-delete",
                    disabled: stats.selected_count == 0,
                    onclick: move |_| {
                        let selected_nodes: Vec<String> = {
                            let doc = doc_signal.read();
                            canvas_domain::selection_geometry::selected_node_ids(&doc)
                                .into_iter()
                                .map(|id| id.to_string())
                                .collect()
                        };
                        let dispatch_result = crate::ui::dispatch::dispatch_node_delete_batch(&db_tx, &selected_nodes);
                        match dispatch_result {
                            Ok(_) => crate::ui::commands::apply_clear_selection(doc_signal),
                            Err(_) => {
                                let _ = apply_delete_selected(doc_signal, history_signal);
                            }
                        }
                    },
                    icon: IconKind::Trash,
                    color: Some("#ef4444"),
                    title: "Delete"
                }

                Divider {}

                // Style settings
                LabelButton {
                    test_id: "style-line",
                    icon: Some(IconKind::Minus), // Assuming we have Minus or Line icon
                    text: "Solid",
                    onclick: None,
                }
                LabelButton {
                    test_id: "style-arrow-type",
                    icon: Some(IconKind::ChevronRight),
                    text: match *arrow_type_signal.read() {
                        diagram_models::document::ArrowType::Default => "Default",
                        diagram_models::document::ArrowType::Sharp => "Sharp",
                        diagram_models::document::ArrowType::Curved => "Curved",
                        diagram_models::document::ArrowType::Step => "Step",
                        diagram_models::document::ArrowType::Straight => "Straight",
                    },
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
                    }
                }
                IconButton {
                    test_id: "style-arrow",
                    onclick: move |_| {
                        let _ = apply_toggle_edge_direction(doc_signal, history_signal);
                    },
                    icon: IconKind::ArrowRight,
                    color: None,
                    disabled: None,
                    active: None,
                    active_bg: None,
                }
            }

            // Right section: Export/Stats
            div {
                class: "flex items-center gap-2",

                div {
                    class: "flex items-center",
                    IconButton {
                        test_id: "toolbar-export",
                        onclick: move |_| {
                            persistence::save_workspace(
                                doc_signal, tool_signal, edge_style_signal, arrow_type_signal, toasts
                            );
                        },
                        icon: IconKind::Upload, // Map export to Upload icon (arrow pointing up from bracket)
                        title: "Export"
                    }
                    IconButton {
                        test_id: "toolbar-import",
                        onclick: move |_| {
                            persistence::open_workspace(
                                doc_signal, history_signal, tool_signal, edge_style_signal, arrow_type_signal, toasts
                            );
                        },
                        icon: IconKind::Download, // Map import to Download icon (arrow pointing down to bracket)
                        title: "Import"
                    }
                }

                div {
                    class: "w-[1px] h-6 bg-[var(--border-subtle)] ml-1 mr-3"
                }

                div {
                    class: "text-muted-foreground font-mono text-[12px] flex gap-3 whitespace-nowrap",
                    span { "data-testid": "counter-nodes", "data-count": "{stats.node_count}", "{stats.node_count} nodes" }
                    span { "data-testid": "counter-edges", "data-count": "{stats.edge_count}", "{stats.edge_count} edges" }
                    span { "data-testid": "counter-selected", "data-count": "{stats.selected_count}", "Rev {stats.revision}" }
                }
            }
        }
    }
}

pub const fn mutation_error_code(err: &MutationError) -> &'static str {
    match err {
        MutationError::Schema(_) => "schema_error",
        MutationError::Semantic(_) => "semantic_error",
    }
}
