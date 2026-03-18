use diagram_models::document::{DiagramDocument, EdgeId, Node, NodeId, NodeKind};
use dioxus::prelude::*;

use crate::history::History;
use crate::ui::canvas::document_ops::{fit_icon_side, node_image_data_url};
use crate::ui::theme::{ACCENT, BG_BASE, TEXT_MAIN, TEXT_MUTED};

use super::inline_edit::InlineEdit;

#[derive(Props, Clone, PartialEq)]
pub struct NodeContentProps {
    pub node: Node,
    pub id: NodeId,
    pub is_editing_node: bool,
    pub font_px: f64,
    pub provider_top: String,
    pub node_initials: String,
    pub width: f64,
    pub height: f64,
    pub zoom: f64,
    pub doc_signal: Signal<DiagramDocument>,
    pub history_signal: Signal<History>,
    pub editing_node: Signal<Option<NodeId>>,
    pub editing_edge: Signal<Option<EdgeId>>,
    pub edit_value: Signal<String>,
    pub db_tx: Option<Coroutine<diagram_models::envelope::EventEnvelope>>,
}

#[component]
pub fn NodeContent(props: NodeContentProps) -> Element {
    let node = props.node;
    let is_editing_node = props.is_editing_node;
    let font_px = props.font_px;

    let mut editing_node = props.editing_node;
    let mut editing_edge = props.editing_edge;
    let mut edit_value = props.edit_value;

    if node.kind == NodeKind::Text {
        if is_editing_node {
            rsx! {
                InlineEdit {
                    edit_value: props.edit_value,
                    style: "width: 100%; padding: 2px 4px; border-radius: 2px; border: 1px solid {ACCENT}; background: transparent; color: {TEXT_MAIN}; font-size: {font_px}px; text-align: center;".to_string(),
                    doc_signal: props.doc_signal,
                    history_signal: props.history_signal,
                    editing_node: props.editing_node,
                    editing_edge: props.editing_edge,
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
                            editing_edge.set(None);
                            editing_node.set(Some(id_edit_text.clone()));
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
                style: "position: absolute; top: 0; left: 0; right: 0; height: 32px; border-bottom: 1px solid var(--border); display: flex; align-items: center; padding: 0 12px; background: color-mix(in oklch, var(--node-bg-subgraph) 80%, transparent); border-radius: 9px 9px 0 0; pointer-events: none;",
                span {
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
                    editing_node: props.editing_node,
                    editing_edge: props.editing_edge,
                    db_tx: props.db_tx,
                }
            } else {
                {
                    let id_edit_subgraph = props.id.clone();
                    rsx! {
                        span {
                            "data-testid": "node-label",
                            style: "position:absolute; top:8px; left:10px; font-size:{font_px}px; color:{TEXT_MUTED};",
                            ondoubleclick: {
                                let edit_label = node.label.clone();
                                move |evt| {
                                    evt.stop_propagation();
                                    editing_edge.set(None);
                                    editing_node.set(Some(id_edit_subgraph.clone()));
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
                style: "position:absolute; left:0; right:0; top:0; height:4px; border-radius:8px 8px 0 0; background:{provider_top}; opacity:0.75;"
            }

            if props.zoom >= 0.3 {
                {
                    node_image.clone().map_or_else(
                        || {
                            rsx! {
                                span {
                                    style: "font-size: {font_px * 1.1}px; color: {provider_top}; font-weight: 700; font-family: monospace;",
                                    "{props.node_initials}"
                                }
                            }
                        },
                        |icon_src| {
                            rsx! {
                                img {
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
                    editing_node: props.editing_node,
                    editing_edge: props.editing_edge,
                    db_tx: props.db_tx,
                }
            } else if props.zoom >= 0.3 {
                {
                    let id_edit_node = props.id.clone();
                    rsx! {
                        span {
                            "data-testid": "node-label",
                            style: "position:absolute; left:0; right:0; bottom:-18px; text-align:center; font-size:{font_px}px; color:{TEXT_MAIN};",
                            ondoubleclick: {
                                let edit_label = node.label.clone();
                                move |evt| {
                                    evt.stop_propagation();
                                    editing_edge.set(None);
                                    editing_node.set(Some(id_edit_node.clone()));
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
