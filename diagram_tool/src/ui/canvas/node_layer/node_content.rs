use diagram_models::document::{DiagramDocument, Node, NodeId, NodeKind};
use dioxus::prelude::*;

use crate::history::History;
use crate::ui::canvas::document_ops::{fit_icon_side, node_image_data_url};
use crate::ui::theme::{ACCENT, BG_BASE, TEXT_MAIN, TEXT_MUTED};

use super::inline_edit::InlineEdit;
use super::node_element::NodeInteractionState;

#[derive(Props, Clone, PartialEq)]
pub struct NodeContentProps {
    pub node: Node,
    pub id: NodeId,
    pub interaction_state: NodeInteractionState,
    pub font_px: f64,
    pub provider_top: String,
    pub node_initials: String,
    pub width: f64,
    pub height: f64,
    pub zoom: f64,
    pub doc_signal: Signal<DiagramDocument>,
    pub history_signal: Signal<History>,
    pub editor_state: Signal<crate::ui::canvas::state::EditorState>,
    pub edit_value: Signal<String>,
    pub db_tx: Option<Coroutine<diagram_models::envelope::EventEnvelope>>,
}

#[component]
pub fn NodeContent(props: NodeContentProps) -> Element {
    let node = props.node;
    let is_editing_node = props.interaction_state.is_editing();
    let font_px = props.font_px;

    let mut editor_state = props.editor_state;
    let mut edit_value = props.edit_value;

    if node.kind == NodeKind::Text {
        if is_editing_node {
            rsx! {
                InlineEdit {
                    edit_value: props.edit_value,
                    style: "width: 100%; padding: 2px 4px; border-radius: 2px; border: 1px solid {ACCENT}; background: transparent; color: {TEXT_MAIN}; font-size: {font_px}px; text-align: center;".to_string(),
                    doc_signal: props.doc_signal,
                    history_signal: props.history_signal,
                    editor_state: props.editor_state,
                    db_tx: props.db_tx,
                }
            }
        } else {
            let id_edit_text = props.id.clone();
            rsx! {
                span {
                    "data-testid": "node-label",
                    style: "font-size: {font_px}px; color: {TEXT_MAIN};",
                    ondoubleclick: {
                        let edit_label = node.label.clone();
                        move |evt| {
                            evt.stop_propagation();
                            editor_state.set(crate::ui::canvas::state::EditorState::EditingNode(id_edit_text.clone()));
                            edit_value.set(edit_label.clone());
                        }
                    },
                    "{node.label}"
                }
            }
        }
    } else if node.kind == NodeKind::Subgraph {
        rsx! {
            div {
                "data-testid": "subgraph-header",
                class: "absolute top-0 left-0 right-0 h-[32px] flex items-center px-[12px] rounded-t-[9px] pointer-events-none",
                style: "border-bottom: 1px solid var(--border); background: color-mix(in oklch, var(--node-bg-subgraph) 80%, transparent);",
                span {
                    "data-testid": "subgraph-header-text",
                    style: "font-size: 11px; font-weight: 500; text-transform: uppercase; letter-spacing: 0.04em; color: var(--text-muted); pointer-events: none;",
                    "{node.label}"
                }
            }
            if is_editing_node {
                InlineEdit {
                    edit_value: props.edit_value,
                    style: format!("position:absolute; top:8px; left:8px; right:8px; width: calc(100% - 16px); padding: 2px 4px; border-radius: 4px; border: 1px solid {ACCENT}; background: {BG_BASE}; color: {TEXT_MAIN}; font-size: {font_px}px;"),
                    doc_signal: props.doc_signal,
                    history_signal: props.history_signal,
                    editor_state: props.editor_state,
                    db_tx: props.db_tx,
                }
            } else {
                {
                    let id_edit_subgraph = props.id.clone();
                    rsx! {
                        span {
                            "data-testid": "node-label",
                            class: "absolute top-[8px] left-[10px]",
                            style: "font-size:{font_px}px; color:{TEXT_MUTED};",
                            ondoubleclick: {
                                let edit_label = node.label.clone();
                                move |evt| {
                                    evt.stop_propagation();
                                    editor_state.set(crate::ui::canvas::state::EditorState::EditingNode(id_edit_subgraph.clone()));
                                    edit_value.set(edit_label.clone());
                                }
                            },
                            "{node.label}"
                        }
                    }
                }
            }
        }
    } else {
        let provider_top = props.provider_top;
        let icon_w = fit_icon_side(props.width);
        let icon_h = fit_icon_side(props.height);
        let node_image = node_image_data_url(&node);
        rsx! {
            div {
                "data-testid": "node-color-bar",
                class: "absolute left-0 right-0 top-0 h-[4px] rounded-t-[8px] opacity-75",
                style: "background:{provider_top};"
            }

            if props.zoom >= 0.3 {
                {
                    node_image.clone().map_or_else(
                        || {
                            rsx! {
                                span {
                                    "data-testid": "node-initials",
                                    style: "font-size: {font_px * 1.1}px; color: {provider_top}; font-weight: 700; font-family: monospace;",
                                    "{props.node_initials}"
                                }
                            }
                        },
                        |icon_src| {
                            rsx! {
                                img {
                                    "data-testid": "node-icon",
                                    src: "{icon_src}",
                                    width: "{icon_w}px",
                                    height: "{icon_h}px",
                                    style: "object-fit: contain; pointer-events: none; user-select: none;"
                                }
                            }
                        },
                    )
                }
            }

            if is_editing_node {
                InlineEdit {
                    edit_value: props.edit_value,
                    style: format!("position:absolute; left:6px; right:6px; bottom:-22px; width: calc(100% - 12px); padding: 2px 4px; border-radius: 4px; border: 1px solid {ACCENT}; background: {BG_BASE}; color: {TEXT_MAIN}; font-size: {font_px}px; text-align:center;"),
                    doc_signal: props.doc_signal,
                    history_signal: props.history_signal,
                    editor_state: props.editor_state,
                    db_tx: props.db_tx,
                }
            } else if props.zoom >= 0.3 {
                {
                    let id_edit_node = props.id.clone();
                    rsx! {
                        span {
                            "data-testid": "node-label",
                            class: "absolute left-0 right-0 bottom-[-18px] text-center",
                            style: "font-size:{font_px}px; color:{TEXT_MAIN};",
                            ondoubleclick: {
                                let edit_label = node.label.clone();
                                move |evt| {
                                    evt.stop_propagation();
                                    editor_state.set(crate::ui::canvas::state::EditorState::EditingNode(id_edit_node.clone()));
                                    edit_value.set(edit_label.clone());
                                }
                            },
                            "{node.label}"
                        }
                    }
                }
            }
        }
    }
}
