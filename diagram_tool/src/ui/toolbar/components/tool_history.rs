use crate::history::History;
use crate::ui::editor::ToolMode;
use crate::ui::theme::{ACCENT, ACCENT_SOFT, BG_BASE, BORDER, TEXT_MAIN};
use crate::ui::toolbar::actions;
use crate::ui::toolbar::components::base::ToolbarButton;
use diagram_models::document::DiagramDocument;
use dioxus::prelude::*;

#[component]
pub fn ToolSelectionGroup() -> Element {
    let mut tool_signal = use_context::<Signal<ToolMode>>();

    rsx! {
        for mode in [
            ToolMode::Select,
            ToolMode::Pan,
            ToolMode::Edge,
            ToolMode::Subgraph,
            ToolMode::Text,
            ToolMode::Draw,
        ] {
            {
                let active = *tool_signal.read() == mode;
                let bg = if active { ACCENT_SOFT } else { "transparent" };
                let border = if active { ACCENT } else { BORDER };
                let test_id = match mode {
                    ToolMode::Select => "tool-select",
                    ToolMode::Pan => "tool-pan",
                    ToolMode::Edge => "tool-edge",
                    ToolMode::Subgraph => "tool-subgraph",
                    ToolMode::Text => "tool-text",
                    ToolMode::Draw => "tool-draw",
                };
                rsx! {
                    ToolbarButton {
                        test_id,
                        onclick: move |_| tool_signal.set(mode),
                        bg,
                        border,
                        extra_style: "font-size: 12px;",
                        "{mode.label()}"
                    }
                }
            }
        }
    }
}

#[component]
pub fn HistoryZoomGroup() -> Element {
    let doc_signal = use_context::<Signal<DiagramDocument>>();
    let history_signal = use_context::<Signal<History>>();
    let viewport_size_signal = use_context::<Signal<(f64, f64)>>();

    let undo_disabled = !history_signal.read().can_undo();
    let redo_disabled = !history_signal.read().can_redo();
    let zoom_percent = (doc_signal.read().editor_state.zoom.0 * 100.0).round();

    rsx! {
        ToolbarButton {
            test_id: "toolbar-undo",
            disabled: undo_disabled,
            onclick: move |_| actions::undo(doc_signal, history_signal),
            "Undo"
        }
        ToolbarButton {
            test_id: "toolbar-redo",
            disabled: redo_disabled,
            onclick: move |_| actions::redo(doc_signal, history_signal),
            "Redo"
        }
        ToolbarButton {
            test_id: "zoom-in",
            onclick: move |_| actions::zoom_in(doc_signal, history_signal, viewport_size_signal),
            "+"
        }
        button {
            "data-testid": "zoom-reset",
            "data-zoom-percent": "{zoom_percent:.0}",
            style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {ACCENT}; background: color-mix(in oklch, {ACCENT_SOFT} 65%, {BG_BASE}); color: {TEXT_MAIN}; min-width: 72px;",
            onclick: move |_| actions::zoom_reset(doc_signal, history_signal, viewport_size_signal),
            title: "Reset zoom",
            span {
                "data-testid": "zoom-percent",
                "{zoom_percent:.0}%"
            }
        }
        ToolbarButton {
            test_id: "zoom-out",
            onclick: move |_| actions::zoom_out(doc_signal, history_signal, viewport_size_signal),
            "-"
        }
    }
}
