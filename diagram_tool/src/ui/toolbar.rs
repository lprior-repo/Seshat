#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

mod actions;
pub mod auto_save;
pub mod components;
mod export_actions;
mod persistence;
mod persistence_compat;

use crate::history::History;
use crate::mutation::error::MutationError;
use crate::ui::editor::ToolMode;
use crate::ui::panels::PanelVisibility;
use crate::ui::theme::{ACCENT, BG_ELEVATED, BG_SURFACE, BORDER_SUBTLE, TEXT_MAIN, TEXT_MUTED};
use crate::ui::toast::{use_toast, ToastQueue};
use components::{
    AlignmentGroup, Divider, EditGroup, ExportGroup, HistoryZoomGroup, ToolSelectionGroup,
    ViewAndThemeGroup,
};
use diagram_models::document::{ArrowType, DiagramDocument, EdgeStyle};
use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub struct ToolbarStats {
    pub selected_count: usize,
    pub node_count: usize,
    pub edge_count: usize,
}

#[component]
pub fn Toolbar() -> Element {
    let doc_signal = use_context::<Signal<DiagramDocument>>();
    let history_signal = use_context::<Signal<History>>();
    let tool_signal = use_context::<Signal<ToolMode>>();
    let edge_style_signal = use_context::<Signal<EdgeStyle>>();
    let arrow_type_signal = use_context::<Signal<ArrowType>>();
    let toasts = use_context::<Signal<ToastQueue>>();
    let toast = use_toast();
    let mut panel_visibility = use_context::<Signal<PanelVisibility>>();
    let mut validate_trigger = use_context::<Signal<u64>>();

    let toolbar_stats = use_context::<Signal<ToolbarStats>>();
    let stats = *toolbar_stats.read();

    let save_label = if cfg!(target_arch = "wasm32") {
        "Save to Server"
    } else {
        "Save"
    };
    let open_label = if cfg!(target_arch = "wasm32") {
        "Import JSON"
    } else {
        "Open"
    };

    rsx! {
        div {
            "data-testid": "toolbar-root",
            class: "toolbar",
            style: "height: 56px; background: linear-gradient(180deg, {BG_SURFACE} 0%, {BG_ELEVATED} 100%); color: {TEXT_MAIN}; display: flex; align-items: center; padding: 0 12px; gap: 8px; border-bottom: 1px solid {BORDER_SUBTLE}; box-shadow: 0 4px 16px color-mix(in oklch, black 22%, transparent); overflow-x: auto;",

            ToolSelectionGroup {}

            components::ToolbarButton {
                test_id: "toolbar-auto-layout",
                onclick: move |_| actions::auto_layout(doc_signal, history_signal, toast),
                "Auto-Arrange"
            }

            Divider {}
            HistoryZoomGroup {}

            Divider {}
            EditGroup {}

            Divider {}
            AlignmentGroup {}

            Divider {}
            button {
                "data-testid": "toolbar-validate",
                style: "padding: 5px 10px; cursor: pointer; background: {ACCENT}; border: none; border-radius: 4px; color: {crate::ui::theme::BG_BASE};",
                onclick: move |_| {
                    validate_trigger.with_mut(|t| *t = t.saturating_add(1));
                    let mut panels = *panel_visibility.read();
                    panels.validation = true;
                    panel_visibility.set(panels);
                },
                "Validate"
            }

            Divider {}
            components::ToolbarButton {
                test_id: "toolbar-save",
                onclick: move |_| {
                    persistence::save_workspace(
                        doc_signal,
                        tool_signal,
                        edge_style_signal,
                        arrow_type_signal,
                        toasts,
                    );
                },
                "{save_label}"
            }
            components::ToolbarButton {
                test_id: "toolbar-open",
                onclick: move |_| {
                    persistence::open_workspace(
                        doc_signal,
                        history_signal,
                        tool_signal,
                        edge_style_signal,
                        arrow_type_signal,
                        toasts,
                    );
                },
                "{open_label}"
            }

            Divider {}
            ViewAndThemeGroup {}

            div { style: "flex: 1;" }
            ExportGroup {}

            span {
                "data-testid": "node-count",
                "data-count": "{stats.node_count}",
                style: "font-size: 11px; color: {TEXT_MUTED}; margin-left: 8px;",
                span {
                    "data-testid": "counter-nodes",
                    "{stats.node_count} nodes"
                }
            }
            span {
                "data-testid": "edge-count",
                "data-count": "{stats.edge_count}",
                style: "font-size: 11px; color: {TEXT_MUTED};",
                span {
                    "data-testid": "counter-edges",
                    "{stats.edge_count} edges"
                }
            }
            span {
                "data-testid": "selected-count",
                "data-count": "{stats.selected_count}",
                style: "font-size: 11px; color: {TEXT_MUTED};",
                span {
                    "data-testid": "counter-selected",
                    "{stats.selected_count} selected"
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
