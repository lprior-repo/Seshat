#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

mod actions;
pub mod auto_save;
mod export_actions;
mod persistence;
mod persistence_compat;

use crate::history::History;
use crate::models::document::{ArrowType, DiagramDocument, EdgeStyle};
use crate::models::envelope::EventEnvelope;
use crate::mutation::error::MutationError;
use crate::ui::editor::ToolMode;
use crate::ui::panels::PanelVisibility;
use crate::ui::theme::{
    ThemeMode, ACCENT, ACCENT_SOFT, BG_BASE, BG_ELEVATED, BG_SURFACE, BORDER, BORDER_SUBTLE, ERROR,
    TEXT_MAIN, TEXT_MUTED,
};
use crate::ui::toast::{use_toast, ToastQueue};
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
    let _clipboard_signal = use_context::<Signal<Option<crate::ui::commands::ClipboardData>>>();
    let mut tool_signal = use_context::<Signal<ToolMode>>();
    let viewport_size_signal = use_context::<Signal<(f64, f64)>>();
    let mut theme_mode_signal = use_context::<Signal<ThemeMode>>();
    let mut panel_visibility = use_context::<Signal<PanelVisibility>>();
    let toasts = use_context::<Signal<ToastQueue>>();
    let toast = use_toast();
    let edge_style_signal = use_context::<Signal<EdgeStyle>>();
    let arrow_type_signal = use_context::<Signal<ArrowType>>();
    let mut validate_trigger = use_context::<Signal<u64>>();
    let toolbar_stats = use_context::<Signal<ToolbarStats>>();
    let stats = *toolbar_stats.read();
    let db_tx = use_context::<Option<Coroutine<EventEnvelope>>>();

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

    let delete_color = if stats.selected_count > 0 {
        ERROR
    } else {
        TEXT_MAIN
    };
    let delete_opacity = if stats.selected_count > 0 { "1" } else { "0.6" };
    let zoom_percent = (doc_signal.read().editor_state.zoom.0 * 100.0).round();

    let undo_disabled = !history_signal.read().can_undo();
    let undo_opacity = if undo_disabled { "0.4" } else { "1" };
    let undo_cursor = if undo_disabled {
        "not-allowed"
    } else {
        "pointer"
    };

    let redo_disabled = !history_signal.read().can_redo();
    let redo_opacity = if redo_disabled { "0.4" } else { "1" };
    let redo_cursor = if redo_disabled {
        "not-allowed"
    } else {
        "pointer"
    };

    rsx! {
        div {
            "data-testid": "toolbar-root",
            class: "toolbar",
            style: "height: 56px; background: linear-gradient(180deg, {BG_SURFACE} 0%, {BG_ELEVATED} 100%); color: {TEXT_MAIN}; display: flex; align-items: center; padding: 0 12px; gap: 8px; border-bottom: 1px solid {BORDER_SUBTLE}; box-shadow: 0 4px 16px color-mix(in oklch, black 22%, transparent); overflow-x: auto;",

            for mode in [ToolMode::Select, ToolMode::Pan, ToolMode::Edge, ToolMode::Subgraph, ToolMode::Text, ToolMode::Draw] {
                {
                    let active = *tool_signal.read() == mode;
                    let bg = if active { ACCENT_SOFT } else { "transparent" };
                    let border = if active { format!("1px solid {ACCENT}") } else { format!("1px solid {BORDER}") };
                    let test_id = match mode {
                        ToolMode::Select => "tool-select",
                        ToolMode::Pan => "tool-pan",
                        ToolMode::Edge => "tool-edge",
                        ToolMode::Subgraph => "tool-subgraph",
                        ToolMode::Text => "tool-text",
                        ToolMode::Draw => "tool-draw",
                    };
                    rsx! {
                        button {
                            "data-testid": "{test_id}",
                            style: "padding: 6px 10px; border-radius: 6px; border: {border}; background: {bg}; color: {TEXT_MAIN}; cursor: pointer; font-size: 12px;",
                            onclick: move |_| tool_signal.set(mode),
                            "{mode.label()}"
                        }
                    }
                }
            }

            button {
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: move |_| actions::auto_layout(doc_signal, history_signal, toast),
                "Auto-Arrange"
            }

            div { style: "width: 1px; height: 20px; background: {BORDER};" }

            button {
                "data-testid": "toolbar-undo",
                disabled: undo_disabled,
                style: "padding: 6px 10px; cursor: {undo_cursor}; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN}; opacity: {undo_opacity};",
                onclick: move |_| actions::undo(doc_signal, history_signal),
                "Undo"
            }
            button {
                "data-testid": "toolbar-redo",
                disabled: redo_disabled,
                style: "padding: 6px 10px; cursor: {redo_cursor}; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN}; opacity: {redo_opacity};",
                onclick: move |_| actions::redo(doc_signal, history_signal),
                "Redo"
            }

            button {
                "data-testid": "zoom-in",
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: move |_| actions::zoom_in(doc_signal, history_signal, viewport_size_signal),
                "+"
            }
            button {
                "data-testid": "zoom-reset",
                "data-zoom-percent": "{zoom_percent:.0}",
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {ACCENT}; background: color-mix(in oklch, {ACCENT_SOFT} 65%, {BG_BASE}); color: {TEXT_MAIN}; min-width: 72px;",
                onclick: move |_| {
                    actions::zoom_reset(doc_signal, history_signal, viewport_size_signal);
                },
                title: "Reset zoom",
                span {
                    "data-testid": "zoom-percent",
                    "{zoom_percent:.0}%"
                }
            }
            button {
                "data-testid": "zoom-out",
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: move |_| actions::zoom_out(doc_signal, history_signal, viewport_size_signal),
                "-"
            }

            div { style: "width: 1px; height: 20px; background: {BORDER};" }

            button {
                "data-testid": "toolbar-delete",
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {delete_color}; opacity: {delete_opacity};",
                onclick: move |_| actions::delete_selection(doc_signal, history_signal),
                disabled: stats.selected_count == 0,
                "Delete"
            }
            button {
                "data-testid": "toolbar-copy",
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: move |_| actions::copy_selection(doc_signal),
                disabled: stats.selected_count == 0,
                "Copy"
            }
            button {
                "data-testid": "toolbar-paste",
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: move |_| actions::paste_selection(doc_signal, history_signal),
                disabled: !actions::can_paste(),
                "Paste"
            }

            button {
                "data-testid": "toolbar-send-backward",
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: move |_| actions::send_backward(doc_signal, history_signal),
                disabled: stats.selected_count == 0,
                "Back"
            }
            button {
                "data-testid": "toolbar-bring-forward",
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: move |_| actions::bring_forward(doc_signal, history_signal),
                disabled: stats.selected_count == 0,
                "Forward"
            }
            button {
                "data-testid": "toolbar-send-to-back",
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: move |_| actions::send_to_back(doc_signal, history_signal, db_tx),
                disabled: stats.selected_count == 0,
                "To Back"
            }
            button {
                "data-testid": "toolbar-bring-to-front",
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: move |_| actions::bring_to_front(doc_signal, history_signal, db_tx),
                disabled: stats.selected_count == 0,
                "To Front"
            }

            div { style: "width: 1px; height: 20px; background: {BORDER};" }

            // Alignment buttons - require 2+ selected nodes
            button {
                "data-testid": "toolbar-align-left",
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: move |_| actions::align_left(doc_signal, history_signal),
                disabled: stats.selected_count < 2,
                "Left"
            }
            button {
                "data-testid": "toolbar-align-center-h",
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: move |_| actions::align_center_horizontal(doc_signal, history_signal),
                disabled: stats.selected_count < 2,
                "H-Center"
            }
            button {
                "data-testid": "toolbar-align-right",
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: move |_| actions::align_right(doc_signal, history_signal),
                disabled: stats.selected_count < 2,
                "Right"
            }
            button {
                "data-testid": "toolbar-align-top",
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: move |_| actions::align_top(doc_signal, history_signal),
                disabled: stats.selected_count < 2,
                "Top"
            }
            button {
                "data-testid": "toolbar-align-middle-v",
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: move |_| actions::align_middle_vertical(doc_signal, history_signal),
                disabled: stats.selected_count < 2,
                "V-Center"
            }
            button {
                "data-testid": "toolbar-align-bottom",
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: move |_| actions::align_bottom(doc_signal, history_signal),
                disabled: stats.selected_count < 2,
                "Bottom"
            }

            // Distribution buttons - require 3+ selected nodes
            button {
                "data-testid": "toolbar-distribute-h",
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: move |_| actions::distribute_horizontal(doc_signal, history_signal),
                disabled: stats.selected_count < 3,
                "Dist H"
            }
            button {
                "data-testid": "toolbar-distribute-v",
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: move |_| actions::distribute_vertical(doc_signal, history_signal),
                disabled: stats.selected_count < 3,
                "Dist V"
            }

            div { style: "width: 1px; height: 20px; background: {BORDER};" }

            button {
                "data-testid": "toolbar-validate",
                style: "padding: 5px 10px; cursor: pointer; background: {ACCENT}; border: none; border-radius: 4px; color: {BG_BASE};",
                onclick: move |_| {
                    validate_trigger.with_mut(|t| *t = t.saturating_add(1));
                    let mut panels = *panel_visibility.read();
                    panels.validation = true;
                    panel_visibility.set(panels);
                },
                "Validate"
            }

            div { style: "width: 1px; height: 20px; background: {BORDER};" }

            button {
                "data-testid": "toolbar-save",
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
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
            button {
                "data-testid": "toolbar-open",
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
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

            div { style: "width: 1px; height: 20px; background: {BORDER};" }

            select {
                style: "padding: 6px 8px; min-width: 110px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN}; cursor: pointer; font-size: 12px;",
                value: "{theme_mode_signal.read().persisted_key()}",
                onchange: move |evt| {
                    if let Some(next) = ThemeMode::from_persisted_key(&evt.value()) {
                        theme_mode_signal.set(next);
                    }
                },
                for mode in [ThemeMode::System, ThemeMode::Light, ThemeMode::Dark] {
                    option { value: "{mode.persisted_key()}", "{mode.label()} theme" }
                }
            }

            for (label, stable_test_id, enabled, setter) in [
                ("Icons", "panel-icons-toggle", panel_visibility.read().sidebar, 0_u8),
                ("Mini", "panel-mini-toggle", panel_visibility.read().minimap, 1_u8),
                ("Valid", "panel-valid-toggle", panel_visibility.read().validation, 2_u8),
            ] {
                {
                    let bg = if enabled { ACCENT_SOFT } else { BG_BASE };
                    let border = if enabled { format!("1px solid {ACCENT}") } else { format!("1px solid {BORDER}") };
                    rsx! {
                        button {
                            "data-testid": "{stable_test_id}",
                            style: "padding: 6px 8px; cursor: pointer; border-radius: 6px; border: {border}; background: {bg}; color: {TEXT_MAIN}; font-size: 11px;",
                            onclick: move |_| {
                                panel_visibility.with_mut(|panels| {
                                    match setter {
                                        0 => panels.sidebar = !panels.sidebar,
                                        1 => panels.minimap = !panels.minimap,
                                        _ => panels.validation = !panels.validation,
                                    }
                                });
                            },
                            "{label}"
                        }
                    }
                }
            }

            {
                let grid_enabled = doc_signal.read().editor_state.show_grid;
                let grid_bg = if grid_enabled { ACCENT_SOFT } else { BG_BASE };
                let grid_border = if grid_enabled { format!("1px solid {ACCENT}") } else { format!("1px solid {BORDER}") };
                rsx! {
                    button {
                        "data-testid": "grid-toggle",
                        "data-checked": "{grid_enabled}",
                        style: "padding: 6px 8px; cursor: pointer; border-radius: 6px; border: {grid_border}; background: {grid_bg}; color: {TEXT_MAIN}; font-size: 11px;",
                        onclick: move |_| actions::toggle_grid(doc_signal),
                        "Grid"
                    }
                }
            }

            div { style: "flex: 1;" }

            button {
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: move |_| export_actions::export_png(doc_signal, toasts),
                "Export PNG"
            }
            button {
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: move |_| export_actions::export_svg(doc_signal, toasts),
                "Export SVG"
            }
            button {
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: move |_| export_actions::export_json(doc_signal, toasts),
                "Export JSON"
            }

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

const fn mutation_error_code(err: &MutationError) -> &'static str {
    match err {
        MutationError::Schema(_) => "schema_error",
        MutationError::Semantic(_) => "semantic_error",
    }
}
