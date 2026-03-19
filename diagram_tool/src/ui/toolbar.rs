#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

pub mod auto_save;
mod persistence;
mod persistence_compat;

use crate::history::History;
use crate::mutation::error::MutationError;
use crate::ui::commands::{
    apply_delete_selected, apply_redo, apply_undo, apply_zoom_in, apply_zoom_out, apply_zoom_reset,
};
use crate::ui::editor::ToolMode;
use crate::ui::icons::{Icon, IconKind};
use crate::ui::theme::{ACCENT, ACCENT_SOFT, BG_SURFACE, BORDER_SUBTLE, TEXT_MAIN, TEXT_MUTED};
use crate::ui::toast::ToastQueue;
use diagram_models::document::{ArrowType, DiagramDocument, EdgeStyle};
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
            style: "width: 1px; height: 24px; background: {BORDER_SUBTLE}; margin: 0 4px;"
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
) -> Element {
    let is_active = active.unwrap_or(false);
    let is_disabled = disabled.unwrap_or(false);
    let bg = if is_active {
        ACCENT_SOFT
    } else {
        "transparent"
    };
    let border = if is_active { ACCENT } else { "transparent" };
    let fill = if is_active { Some(ACCENT) } else { color };
    let opacity = if is_disabled { "0.4" } else { "1.0" };
    let cursor = if is_disabled {
        "not-allowed"
    } else {
        "pointer"
    };

    rsx! {
        button {
            "data-testid": test_id,
            style: "width: 36px; height: 36px; display: flex; align-items: center; justify-content: center; border-radius: 6px; border: 1px solid {border}; background: {bg}; cursor: {cursor}; opacity: {opacity}; padding: 0; outline: none; margin: 0 2px; color: {TEXT_MAIN};",
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
) -> Element {
    let is_active = active.unwrap_or(false);
    let is_disabled = disabled.unwrap_or(false);
    let bg = if is_active {
        ACCENT_SOFT
    } else {
        "transparent"
    };
    let border = if is_active { ACCENT } else { "transparent" };
    let fill = if is_active { ACCENT } else { TEXT_MAIN };
    let opacity = if is_disabled { "0.4" } else { "1.0" };
    let cursor = if is_disabled {
        "not-allowed"
    } else {
        "pointer"
    };

    rsx! {
        button {
            "data-testid": test_id,
            style: "width: 36px; height: 36px; display: flex; align-items: center; justify-content: center; border-radius: 6px; border: 1px solid {border}; background: {bg}; cursor: {cursor}; opacity: {opacity}; padding: 0; outline: none; margin: 0 2px; color: {fill}; font-size: 18px; font-weight: 500; font-family: monospace;",
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
pub fn Toolbar() -> Element {
    let doc_signal = use_context::<Signal<DiagramDocument>>();
    let history_signal = use_context::<Signal<History>>();
    let mut tool_signal = use_context::<Signal<ToolMode>>();
    let edge_style_signal = use_context::<Signal<EdgeStyle>>();
    let arrow_type_signal = use_context::<Signal<ArrowType>>();
    let toasts = use_context::<Signal<ToastQueue>>();

    let toolbar_stats = use_context::<Signal<ToolbarStats>>();
    let stats = *toolbar_stats.read();
    let viewport_size_signal = use_context::<Signal<(f64, f64)>>();

    let undo_disabled = !history_signal.read().can_undo();
    let redo_disabled = !history_signal.read().can_redo();
    let zoom_percent = (doc_signal.read().editor_state.zoom.0 * 100.0).round();

    rsx! {
        div {
            "data-testid": "toolbar-root",
            class: "toolbar",
            style: "height: 56px; background: {BG_SURFACE}; display: flex; align-items: center; padding: 0 16px; border-bottom: 1px solid {BORDER_SUBTLE}; overflow-x: auto;",

            // Import/Export
            IconButton {
                test_id: "toolbar-open",
                onclick: move |_| {
                    persistence::open_workspace(
                        doc_signal, history_signal, tool_signal, edge_style_signal, arrow_type_signal, toasts
                    );
                },
                icon: IconKind::FolderOpen
            }
            IconButton {
                test_id: "toolbar-save",
                onclick: move |_| {
                    persistence::save_workspace(
                        doc_signal, tool_signal, edge_style_signal, arrow_type_signal, toasts
                    );
                },
                icon: IconKind::Save
            }

            Divider {}

            // Stats
            div {
                style: "color: {TEXT_MUTED}; font-family: monospace; font-size: 13px; margin: 0 8px; white-space: nowrap;",
                span { "data-testid": "counter-nodes", "data-count": "{stats.node_count}", "{stats.node_count} nodes " }
                span { "data-testid": "counter-edges", "data-count": "{stats.edge_count}", "{stats.edge_count} edges " }
                span { "data-testid": "counter-selected", "data-count": "{stats.selected_count}", "Rev {stats.revision}" }
            }

            Divider {}

            // Tools
            IconButton {
                test_id: "tool-select",
                active: *tool_signal.read() == ToolMode::Select,
                onclick: move |_| tool_signal.set(ToolMode::Select),
                icon: IconKind::Select
            }
            IconButton {
                test_id: "tool-pan",
                active: *tool_signal.read() == ToolMode::Pan,
                onclick: move |_| tool_signal.set(ToolMode::Pan),
                icon: IconKind::Pan
            }
            IconButton {
                test_id: "tool-edge",
                active: *tool_signal.read() == ToolMode::Edge,
                onclick: move |_| tool_signal.set(ToolMode::Edge),
                icon: IconKind::Edge
            }
            IconButton {
                test_id: "tool-subgraph",
                active: *tool_signal.read() == ToolMode::Subgraph,
                onclick: move |_| tool_signal.set(ToolMode::Subgraph),
                icon: IconKind::Subgraph
            }
            TextButton {
                test_id: "tool-text",
                active: *tool_signal.read() == ToolMode::Text,
                onclick: move |_| tool_signal.set(ToolMode::Text),
                text: "T"
            }

            Divider {}

            // History
            IconButton {
                test_id: "toolbar-undo",
                disabled: undo_disabled,
                onclick: move |_| { apply_undo(doc_signal, history_signal); },
                icon: IconKind::Undo,
                color: None
            }
            IconButton {
                test_id: "toolbar-redo",
                disabled: redo_disabled,
                onclick: move |_| { apply_redo(doc_signal, history_signal); },
                icon: IconKind::Redo,
                color: None
            }

            Divider {}

            // Zoom
            IconButton {
                test_id: "zoom-in",
                onclick: move |_| { let _ = apply_zoom_in(doc_signal, history_signal, *viewport_size_signal.read()); },
                icon: IconKind::ZoomIn,
                color: None
            }
            div {
                "data-testid": "zoom-reset",
                "data-zoom-percent": "{zoom_percent:.0}",
                style: "color: {TEXT_MUTED}; font-family: monospace; font-size: 13px; margin: 0 8px; cursor: pointer; user-select: none;",
                onclick: move |_| { let _ = apply_zoom_reset(doc_signal, history_signal, *viewport_size_signal.read()); },
                title: "Reset zoom",
                "{zoom_percent:.0}%"
            }
            IconButton {
                test_id: "zoom-out",
                onclick: move |_| { let _ = apply_zoom_out(doc_signal, history_signal, *viewport_size_signal.read()); },
                icon: IconKind::ZoomOut,
                color: None
            }

            Divider {}

            // Delete
            IconButton {
                test_id: "toolbar-delete",
                disabled: stats.selected_count == 0,
                onclick: move |_| { let _ = apply_delete_selected(doc_signal, history_signal); },
                icon: IconKind::Trash,
                color: Some("#ef4444")
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
