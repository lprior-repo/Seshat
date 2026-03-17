#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::history::History;
use crate::ui::dispatch::dispatch_update_node_style;
use crate::ui::properties_helpers::{node_kind_str, node_style_str, parse_node_style};
use crate::ui::theme::{BG_BASE, BORDER, BORDER_SUBTLE, TEXT_DIM, TEXT_MAIN, TEXT_MUTED};
use diagram_models::document::{DiagramDocument, LockState, Node, NodeId, OrderedFloat};
use diagram_models::envelope::EventEnvelope;
use dioxus::prelude::*;

#[derive(PartialEq, Clone, Props)]
pub struct NodePanelProps {
    pub id: NodeId,
    pub node: Node,
    pub connection_rows: Vec<(String, String, String)>,
}

#[component]
#[allow(clippy::approx_constant, clippy::float_cmp)]
pub fn NodePanel(props: NodePanelProps) -> Element {
    let id = props.id;
    let node = props.node;
    let connection_rows = props.connection_rows;
    let mut doc_signal = use_context::<Signal<DiagramDocument>>();
    let mut history = use_context::<Signal<History>>();
    let db_tx = use_context::<Option<Coroutine<EventEnvelope>>>();

    let id_label = id.clone();
    let id_x = id.clone();
    let id_y = id.clone();
    let id_w = id.clone();
    let id_h = id.clone();
    let id_font = id.clone();
    let id_lock = id.clone();
    let id_style = id.clone();

    rsx! {
        div {
            key: "{id}",
            style: "display: flex; flex-direction: column; gap: 10px;",
            div {
                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Label" }
                input {
                    "data-testid": "node-label-input",
                    style: "width: 100%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                    value: "{node.label}",
                    onchange: move |evt| {
                        let new_label = evt.value();
                        let nid = id_label.clone();
                        let has_changes = doc_signal.read()
                            .document
                            .nodes
                            .get(&nid)
                            .is_some_and(|n| n.label != new_label);
                        if has_changes {
                            let current = doc_signal.read().clone();
                            let next_h = history.read().push(current);
                            *history.write() = next_h;
                        }
                        doc_signal.with_mut(|doc| {
                            if let Some(n) = doc.document.nodes.get_mut(&nid) {
                                n.label = new_label;
                                doc.revision = doc.revision.increment();
                            }
                        });
                    }
                }
            }

            div {
                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Kind" }
                div {
                    style: "margin-top: 3px; display: inline-block; border: 1px solid {BORDER}; border-radius: 999px; padding: 2px 8px; font-size: 11px; color: {TEXT_MAIN};",
                    "{node_kind_str(&node.kind)}"
                }
            }

            div {
                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Style" }
                select {
                    style: "width: 100%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                    value: "{node_style_str(&node.style)}",
                    onchange: move |evt| {
                        let nid = id_style.clone();
                        let new_style = match parse_node_style(&evt.value()) {
                            Ok(style) => style,
                            Err(_) => return,
                        };
                        let has_changes = doc_signal.read()
                            .document
                            .nodes
                            .get(&nid)
                            .is_some_and(|n| n.style.as_ref() != Some(&new_style));
                        if has_changes {
                            let current = doc_signal.read().clone();
                            let next_h = history.read().push(current);
                            *history.write() = next_h;
                            dispatch_update_node_style(&db_tx, nid.as_str(), new_style.clone()).ok();
                        }
                        doc_signal.with_mut(|doc| {
                            if let Some(n) = doc.document.nodes.get_mut(&nid) {
                                n.style = Some(new_style);
                                doc.revision = doc.revision.increment();
                            }
                        });
                    },
                    option { value: "box", "Box" }
                    option { value: "cloud", "Cloud" }
                    option { value: "cylinder", "Cylinder" }
                    option { value: "dashed", "Dashed" }
                }
            }

            if !node.icon.is_empty() {
                div {
                    label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Icon" }
                    p {
                        style: "margin: 3px 0 0 0; font-size: 11px; color: {TEXT_MAIN}; word-break: break-all;",
                        "{node.icon}"
                    }
                }
            }

            div {
                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Position" }
                div { style: "display: flex; gap: 5px;",
                    input {
                        style: "width: 50%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                        r#type: "number",
                        value: "{node.x}",
                        onchange: move |evt| {
                            let nid = id_x.clone();
                            if let Ok(val) = evt.value().parse::<f64>() {
                                let has_changes = doc_signal.read()
                                    .document
                                    .nodes
                                    .get(&nid)
                                    .is_some_and(|n| n.x.0 != val);
                                if has_changes {
                                    let current = doc_signal.read().clone();
                                    let next_h = history.read().push(current);
                                    *history.write() = next_h;
                                }
                                doc_signal.with_mut(|doc| {
                                    if let Some(n) = doc.document.nodes.get_mut(&nid) {
                                        n.x = OrderedFloat(val);
                                        doc.revision = doc.revision.increment();
                                    }
                                });
                            }
                        }
                    }
                    input {
                        style: "width: 50%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                        r#type: "number",
                        value: "{node.y}",
                        onchange: move |evt| {
                            let nid = id_y.clone();
                            if let Ok(val) = evt.value().parse::<f64>() {
                                let has_changes = doc_signal.read()
                                    .document
                                    .nodes
                                    .get(&nid)
                                    .is_some_and(|n| n.y.0 != val);
                                if has_changes {
                                    let current = doc_signal.read().clone();
                                    let next_h = history.read().push(current);
                                    *history.write() = next_h;
                                }
                                doc_signal.with_mut(|doc| {
                                    if let Some(n) = doc.document.nodes.get_mut(&nid) {
                                        n.y = OrderedFloat(val);
                                        doc.revision = doc.revision.increment();
                                    }
                                });
                            }
                        }
                    }
                }
            }

            div {
                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Size" }
                div { style: "display: flex; gap: 5px;",
                    input {
                        style: "width: 50%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                        r#type: "number",
                        value: "{node.width}",
                        onchange: move |evt| {
                            let nid = id_w.clone();
                            if let Ok(val) = evt.value().parse::<f64>() {
                                let clamped_val = val.max(24.0);
                                let has_changes = doc_signal.read()
                                    .document
                                    .nodes
                                    .get(&nid)
                                    .is_some_and(|n| n.width.0 != clamped_val);
                                if has_changes {
                                    let current = doc_signal.read().clone();
                                    let next_h = history.read().push(current);
                                    *history.write() = next_h;
                                }
                                doc_signal.with_mut(|doc| {
                                    if let Some(n) = doc.document.nodes.get_mut(&nid) {
                                        n.width = OrderedFloat(clamped_val);
                                        doc.revision = doc.revision.increment();
                                    }
                                });
                            }
                        }
                    }
                    input {
                        style: "width: 50%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                        r#type: "number",
                        value: "{node.height}",
                        onchange: move |evt| {
                            let nid = id_h.clone();
                            if let Ok(val) = evt.value().parse::<f64>() {
                                let clamped_val = val.max(24.0);
                                let has_changes = doc_signal.read()
                                    .document
                                    .nodes
                                    .get(&nid)
                                    .is_some_and(|n| n.height.0 != clamped_val);
                                if has_changes {
                                    let current = doc_signal.read().clone();
                                    let next_h = history.read().push(current);
                                    *history.write() = next_h;
                                }
                                doc_signal.with_mut(|doc| {
                                    if let Some(n) = doc.document.nodes.get_mut(&nid) {
                                        n.height = OrderedFloat(clamped_val);
                                        doc.revision = doc.revision.increment();
                                    }
                                });
                            }
                        }
                    }
                }
            }

            div {
                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Font Size" }
                input {
                    style: "width: 100%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                    r#type: "number",
                    value: "{node.font_size.map_or(11.0, |v| v.0)}",
                    onchange: move |evt| {
                        let nid = id_font.clone();
                        if let Ok(val) = evt.value().parse::<f64>() {
                            let clamped_val = val.clamp(8.0, 72.0);
                            let has_changes = doc_signal.read()
                                .document
                                .nodes
                                .get(&nid)
                                .is_some_and(|n| n.font_size.map_or(11.0, |fs| fs.0) != clamped_val);
                            if has_changes {
                                let current = doc_signal.read().clone();
                                let next_h = history.read().push(current);
                                *history.write() = next_h;
                            }
                            doc_signal.with_mut(|doc| {
                                if let Some(n) = doc.document.nodes.get_mut(&nid) {
                                    n.font_size = Some(OrderedFloat(clamped_val));
                                    doc.revision = doc.revision.increment();
                                }
                            });
                        }
                    }
                }
            }

            div {
                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Lock" }
                button {
                    style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                    onclick: move |_| {
                        let nid = id_lock.clone();
                        let current = doc_signal.read().clone();
                        let next_h = history.read().push(current);
                        *history.write() = next_h;
                        doc_signal.with_mut(|doc| {
                            if let Some(n) = doc.document.nodes.get_mut(&nid) {
                                n.lock_state = if n.lock_state.is_locked() {
                                    LockState::Unlocked
                                } else {
                                    LockState::Locked
                                };
                                doc.revision = doc.revision.increment();
                            }
                        });
                    },
                    if node.lock_state.is_locked() { "Locked" } else { "Unlocked" }
                }
            }

            if !node.tags.is_empty() {
                div {
                    label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Tags" }
                    div {
                        style: "margin-top: 4px; display: flex; flex-wrap: wrap; gap: 4px;",
                        for tag in node.tags.clone() {
                            span {
                                key: "{id}-{tag}",
                                style: "display: inline-flex; align-items: center; padding: 2px 6px; border-radius: 999px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN}; font-size: 10px;",
                                "{tag}"
                            }
                        }
                    }
                }
            }

            div { style: "height: 1px; background: {BORDER_SUBTLE};" }

            div {
                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Connections" }
                div {
                    style: "margin-top: 4px; display: flex; flex-direction: column; gap: 4px;",
                    if connection_rows.is_empty() {
                        div {
                            style: "font-size: 11px; color: {TEXT_DIM}; font-style: italic;",
                            "No connections"
                        }
                    }
                    for (index, (direction, other_label, edge_label)) in connection_rows.iter().enumerate() {
                        div {
                            key: "{id}-connection-{index}",
                            style: "padding: 4px 6px; border-radius: 6px; border: 1px solid {BORDER_SUBTLE}; background: {BG_BASE}; font-size: 11px; color: {TEXT_MAIN};",
                            if edge_label.is_empty() {
                                "{direction} -> {other_label}"
                            } else {
                                "{direction} -> {other_label} ({edge_label})"
                            }
                        }
                    }
                }
            }

            div {
                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "ID" }
                p {
                    style: "margin: 3px 0 0 0; font-size: 10px; color: {TEXT_DIM}; word-break: break-all;",
                    "{id}"
                }
            }
        }
    }
}
