#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::history::History;
use crate::models::document::{
    ArrowType, DiagramDocument, EdgeId, EdgeStyle, NodeId, NodeKind, OrderedFloat,
};
use crate::ui::theme::{
    BG_BASE, BG_SURFACE, BORDER, BORDER_SUBTLE, TEXT_DIM, TEXT_MAIN, TEXT_MUTED,
};
use dioxus::prelude::*;

fn remove_selected(doc: &mut DiagramDocument) {
    let selected = doc.editor_state.selected_items.clone();
    if selected.is_empty() {
        return;
    }

    doc.document.nodes = doc
        .document
        .nodes
        .iter()
        .filter(|(id, _)| !selected.contains(&id.to_string()))
        .map(|(id, node)| (id.clone(), node.clone()))
        .collect();

    let node_ids: im::HashSet<NodeId> = doc.document.nodes.keys().cloned().collect();
    doc.document.edges = doc
        .document
        .edges
        .iter()
        .filter(|(id, edge)| {
            node_ids.contains(&edge.source)
                && node_ids.contains(&edge.target)
                && !selected.contains(&id.to_string())
        })
        .map(|(id, edge)| (id.clone(), edge.clone()))
        .collect();

    doc.editor_state.selected_items.clear();
    doc.revision = doc.revision.increment();
}

fn parse_edge_style(v: &str) -> EdgeStyle {
    match v {
        "dashed" => EdgeStyle::Dashed,
        "dotted" => EdgeStyle::Dotted,
        _ => EdgeStyle::Solid,
    }
}

fn parse_arrow_type(v: &str) -> ArrowType {
    match v {
        "straight" | "open" => ArrowType::Straight,
        "step" | "diamond" => ArrowType::Step,
        "curved" | "circle" => ArrowType::Curved,
        "sharp" | "none" => ArrowType::Sharp,
        _ => ArrowType::Default,
    }
}

const fn edge_style_str(v: EdgeStyle) -> &'static str {
    match v {
        EdgeStyle::Solid => "solid",
        EdgeStyle::Dashed => "dashed",
        EdgeStyle::Dotted => "dotted",
    }
}

const fn arrow_type_str(v: ArrowType) -> &'static str {
    match v {
        ArrowType::Default => "default",
        ArrowType::Straight => "straight",
        ArrowType::Step => "step",
        ArrowType::Curved => "curved",
        ArrowType::Sharp => "sharp",
    }
}

const fn node_kind_str(kind: &NodeKind) -> &'static str {
    match kind {
        NodeKind::Node => "node",
        NodeKind::Subgraph => "subgraph",
        NodeKind::Text => "text",
    }
}

fn node_label_with_id_fallback(doc: &DiagramDocument, id: &NodeId) -> String {
    doc.document.nodes.get(id).map_or_else(
        || id.to_string(),
        |node| {
            let trimmed = node.label.trim();
            if trimmed.is_empty() {
                id.to_string()
            } else {
                trimmed.to_string()
            }
        },
    )
}

#[component]
pub fn PropertiesPanel() -> Element {
    let mut doc_signal = use_context::<Signal<DiagramDocument>>();
    let mut history = use_context::<Signal<History>>();
    let mut edge_style_default = use_context::<Signal<EdgeStyle>>();
    let mut arrow_type_default = use_context::<Signal<ArrowType>>();

    let selected_ids = use_memo(move || doc_signal.read().editor_state.selected_items.clone());
    let selected_items: Vec<String> = selected_ids.read().iter().cloned().collect();
    let doc_snapshot = doc_signal.read().clone();
    let selected_total = selected_items.len();

    let selected_nodes: Vec<_> = selected_items
        .iter()
        .filter_map(|id| {
            let node_id = NodeId::new(id.clone());
            doc_snapshot
                .document
                .nodes
                .get(&node_id)
                .map(|node| (node_id, node.clone()))
        })
        .collect();
    let selected_edges: Vec<_> = selected_items
        .iter()
        .filter_map(|id| {
            let edge_id = EdgeId::new(id.clone());
            doc_snapshot
                .document
                .edges
                .get(&edge_id)
                .map(|edge| (edge_id, edge.clone()))
        })
        .collect();

    let selected_node_count = selected_nodes.len();
    let selected_edge_count = selected_edges.len();
    let delete_cursor = if selected_total > 0 {
        "pointer"
    } else {
        "not-allowed"
    };
    let delete_opacity = if selected_total > 0 { "1" } else { "0.6" };

    let delete_selected = move |_| {
        if selected_total == 0 {
            return;
        }
        let current = doc_signal.read().clone();
        let next_history = history.read().push(current);
        *history.write() = next_history;
        doc_signal.with_mut(remove_selected);
    };

    let single_node = if selected_node_count == 1 {
        selected_nodes.first().cloned()
    } else {
        None
    };
    let single_edge = if selected_edge_count == 1 {
        selected_edges.first().cloned()
    } else {
        None
    };

    let connection_rows = if let Some((node_id, _)) = &single_node {
        doc_snapshot
            .document
            .edges
            .iter()
            .filter_map(|(_, edge)| {
                if edge.source == *node_id || edge.target == *node_id {
                    let other_id = if edge.source == *node_id {
                        edge.target.clone()
                    } else {
                        edge.source.clone()
                    };
                    let other_label = doc_snapshot
                        .document
                        .nodes
                        .get(&other_id)
                        .map_or_else(|| other_id.to_string(), |n| n.label.clone());
                    let direction = if edge.source == *node_id { "out" } else { "in" };
                    Some((direction.to_string(), other_label, edge.label.clone()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let show_multi_select_hint =
        selected_node_count > 1 || (selected_node_count >= 1 && selected_edge_count >= 1);

    let edge_default_value = {
        let v = *edge_style_default.read();
        edge_style_str(v)
    };
    let arrow_default_value = {
        let v = *arrow_type_default.read();
        arrow_type_str(v)
    };
    rsx! {
        div {
            class: "properties-panel",
            style: "width: 260px; max-width: 40vw; background: {BG_SURFACE}; padding: 10px; border-left: 1px solid {BORDER_SUBTLE}; display: flex; flex-direction: column; gap: 10px; min-height: 0; overflow: hidden;",

            h3 {
                style: "margin: 0; font-size: 12px; letter-spacing: 0.08em; text-transform: uppercase; color: {TEXT_MUTED};",
                "Properties"
            }
            p {
                style: "margin: 0; font-size: 11px; color: {TEXT_DIM};",
                "{selected_node_count} node(s), {selected_edge_count} edge(s) selected"
            }

            div {
                style: "flex: 1; min-height: 0; overflow-y: auto; display: flex; flex-direction: column; gap: 10px; padding-right: 2px;",

            if selected_total == 0 {
                div {
                    style: "display: flex; flex-direction: column; gap: 10px;",
                    div { style: "color: {TEXT_DIM}; font-style: italic;", "Select a node or edge to view its properties" }

                    div {
                        label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Default Edge Style" }
                        select {
                            style: "width: 100%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                            value: "{edge_default_value}",
                            onchange: move |evt| edge_style_default.set(parse_edge_style(&evt.value())),
                            option { value: "solid", "Solid" }
                            option { value: "dashed", "Dashed" }
                            option { value: "dotted", "Dotted" }
                        }
                    }

                    div {
                        label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Default Arrow Type" }
                        select {
                            style: "width: 100%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                            value: "{arrow_default_value}",
                            onchange: move |evt| arrow_type_default.set(parse_arrow_type(&evt.value())),
                            option { value: "default", "Default" }
                            option { value: "straight", "Straight" }
                            option { value: "curved", "Curved" }
                            option { value: "step", "Step" }
                            option { value: "sharp", "Sharp" }
                        }
                    }
                }
            }

            if let Some((id, node)) = single_node {
                {
                    let id_label = id.clone();
                    let id_x = id.clone();
                    let id_y = id.clone();
                    let id_w = id.clone();
                    let id_h = id.clone();
                    let id_font = id.clone();
                    let id_lock = id.clone();
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
                                    onfocus: move |_| {
                                        let current = doc_signal.read().clone();
                                        let next_h = history.read().push(current);
                                        *history.write() = next_h;
                                    },
                                    oninput: move |evt| {
                                        let nid = id_label.clone();
                                        doc_signal.with_mut(|doc| {
                                            if let Some(n) = doc.document.nodes.get_mut(&nid) {
                                                n.label = evt.value();
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
                                        oninput: move |evt| {
                                            let nid = id_x.clone();
                                            if let Ok(val) = evt.value().parse::<f64>() {
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
                                        oninput: move |evt| {
                                            let nid = id_y.clone();
                                            if let Ok(val) = evt.value().parse::<f64>() {
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
                                        oninput: move |evt| {
                                            let nid = id_w.clone();
                                            if let Ok(val) = evt.value().parse::<f64>() {
                                                doc_signal.with_mut(|doc| {
                                                    if let Some(n) = doc.document.nodes.get_mut(&nid) {
                                                        n.width = OrderedFloat(val.max(24.0));
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
                                        oninput: move |evt| {
                                            let nid = id_h.clone();
                                            if let Ok(val) = evt.value().parse::<f64>() {
                                                doc_signal.with_mut(|doc| {
                                                    if let Some(n) = doc.document.nodes.get_mut(&nid) {
                                                        n.height = OrderedFloat(val.max(24.0));
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
                                    oninput: move |evt| {
                                        let nid = id_font.clone();
                                        if let Ok(val) = evt.value().parse::<f64>() {
                                            doc_signal.with_mut(|doc| {
                                                if let Some(n) = doc.document.nodes.get_mut(&nid) {
                                                    n.font_size = Some(OrderedFloat(val.clamp(8.0, 72.0)));
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
                                        doc_signal.with_mut(|doc| {
                                            if let Some(n) = doc.document.nodes.get_mut(&nid) {
                                                n.locked = !n.locked;
                                                doc.revision = doc.revision.increment();
                                            }
                                        });
                                    },
                                    if node.locked { "Locked" } else { "Unlocked" }
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
            }

            if let Some((id, edge)) = single_edge {
                {
                    let eid_label = id.clone();
                    let eid_style = id.clone();
                    let eid_arrow = id.clone();
                    let eid_font = id.clone();
                    let source_label = node_label_with_id_fallback(&doc_snapshot, &edge.source);
                    let target_label = node_label_with_id_fallback(&doc_snapshot, &edge.target);
                    rsx! {
                        div {
                            key: "{id}",
                            style: "display: flex; flex-direction: column; gap: 10px;",
                            div {
                                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Edge Label" }
                                input {
                                    style: "width: 100%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                                    value: "{edge.label}",
                                    oninput: move |evt| {
                                        let eid = eid_label.clone();
                                        doc_signal.with_mut(|doc| {
                                            if let Some(e) = doc.document.edges.get_mut(&eid) {
                                                e.label = evt.value();
                                                doc.revision = doc.revision.increment();
                                            }
                                        });
                                    }
                                }
                            }

                            div {
                                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Line Style" }
                                select {
                                    style: "width: 100%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                                    value: "{edge_style_str(edge.style)}",
                                    onchange: move |evt| {
                                        let eid = eid_style.clone();
                                        let style = parse_edge_style(&evt.value());
                                        doc_signal.with_mut(|doc| {
                                            if let Some(e) = doc.document.edges.get_mut(&eid) {
                                                e.style = style;
                                                doc.revision = doc.revision.increment();
                                            }
                                        });
                                    },
                                    option { value: "solid", "Solid" }
                                    option { value: "dashed", "Dashed" }
                                    option { value: "dotted", "Dotted" }
                                }
                            }

                            div {
                                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Arrow Type" }
                                select {
                                    style: "width: 100%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                                    value: "{arrow_type_str(edge.arrow_type)}",
                                    onchange: move |evt| {
                                        let eid = eid_arrow.clone();
                                        let arrow = parse_arrow_type(&evt.value());
                                        doc_signal.with_mut(|doc| {
                                            if let Some(e) = doc.document.edges.get_mut(&eid) {
                                                e.arrow_type = arrow;
                                                doc.revision = doc.revision.increment();
                                            }
                                        });
                                    },
                                    option { value: "default", "Default" }
                                    option { value: "straight", "Straight" }
                                    option { value: "curved", "Curved" }
                                    option { value: "step", "Step" }
                                    option { value: "sharp", "Sharp" }
                                }
                            }

                            div {
                                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Font Size" }
                                input {
                                    style: "width: 100%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                                    r#type: "number",
                                    value: "{edge.font_size.map_or(10.0, |v| v.0)}",
                                    oninput: move |evt| {
                                        let eid = eid_font.clone();
                                        if let Ok(val) = evt.value().parse::<f64>() {
                                            doc_signal.with_mut(|doc| {
                                                if let Some(e) = doc.document.edges.get_mut(&eid) {
                                                    e.font_size = Some(OrderedFloat(val.clamp(8.0, 72.0)));
                                                    doc.revision = doc.revision.increment();
                                                }
                                            });
                                        }
                                    }
                                }
                            }

                            div { style: "height: 1px; background: {BORDER_SUBTLE};" }

                            div {
                                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Source" }
                                p {
                                    style: "margin: 3px 0 0 0; font-size: 11px; color: {TEXT_MAIN}; word-break: break-all;",
                                    "{source_label}"
                                }
                            }

                            div {
                                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Target" }
                                p {
                                    style: "margin: 3px 0 0 0; font-size: 11px; color: {TEXT_MAIN}; word-break: break-all;",
                                    "{target_label}"
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
            }

            if show_multi_select_hint {
                div {
                    style: "padding: 6px; border-radius: 6px; border: 1px dashed {BORDER}; color: {TEXT_DIM}; font-size: 11px;",
                    "Multiple items selected. Use delete to remove all selected items."
                }
            }
            }

            div {
                style: "padding-top: 8px; border-top: 1px solid {BORDER_SUBTLE};",
                button {
                    style: "width: 100%; padding: 7px 10px; cursor: {delete_cursor}; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: #f87171; opacity: {delete_opacity};",
                    disabled: selected_total == 0,
                    onclick: delete_selected,
                    "Delete Selected"
                }
            }
        }
    }
}
