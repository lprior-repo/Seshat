#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::history::History;
use crate::models::document::{
    DiagramDocument, EdgeId, EdgeStyle, NodeId, NodeKind, NodeStyle, OrderedFloat,
};
use crate::models::envelope::EventEnvelope;
use crate::ui::dispatch::{dispatch_update_edge_style, dispatch_update_node_style};
use crate::ui::properties_helpers::{
    arrow_type_str, edge_style_str, node_kind_str, node_label_with_id_fallback, node_style_str,
    parse_arrow_type, parse_edge_style, parse_node_style, remove_selected, StyleError,
};
use crate::ui::theme::{
    BG_BASE, BG_SURFACE, BORDER, BORDER_SUBTLE, TEXT_DIM, TEXT_MAIN, TEXT_MUTED,
};
use crate::ui::toast::use_toast;
use dioxus::prelude::*;

#[component]
#[allow(clippy::approx_constant, clippy::float_cmp)]
pub fn PropertiesPanel() -> Element {
    let mut doc_signal = use_context::<Signal<DiagramDocument>>();
    let mut history = use_context::<Signal<History>>();
    let mut edge_style_default = use_context::<Signal<EdgeStyle>>();
    let mut arrow_type_default = use_context::<Signal<crate::models::document::ArrowType>>();
    let db_tx = use_context::<Option<Coroutine<EventEnvelope>>>();
    let toast = use_toast();

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
                                        // Only push history and update if value actually changed
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
                                        // Parse returns Result - invalid values from dropdown shouldn't happen but handle gracefully
                                        let new_style = match parse_node_style(&evt.value()) {
                                            Ok(style) => style,
                                            Err(_) => return, // Ignore invalid input from dropdown
                                        };
                                        // Only push history and dispatch if style actually changed
                                        // Compare Option<NodeStyle> with Some(NodeStyle) to correctly detect:
                                        // - None -> Some(Box) as a change
                                        // - Some(Cloud) -> Some(Box) as a change
                                        // - Some(Box) -> Some(Box) as NO change (idempotent)
                                        let has_changes = doc_signal.read()
                                            .document
                                            .nodes
                                            .get(&nid)
                                            .is_some_and(|n| n.style.as_ref() != Some(&new_style));
                                        if has_changes {
                                            let current = doc_signal.read().clone();
                                            let next_h = history.read().push(current);
                                            *history.write() = next_h;
                                            // Dispatch to db_tx
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
                                                // Only push history if value actually changed
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
                                                // Only push history if value actually changed
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
                                                // Only push history if value actually changed
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
                                                // Only push history if value actually changed
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
                                            // Only push history if value actually changed
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
                                        // Push history before mutating
                                        let current = doc_signal.read().clone();
                                        let next_h = history.read().push(current);
                                        *history.write() = next_h;
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
                                        let new_style = parse_edge_style(&evt.value());
                                        // Only push history and dispatch if style actually changed (idempotent)
                                        let has_changes = doc_signal.read()
                                            .document
                                            .edges
                                            .get(&eid)
                                            .is_some_and(|e| e.style != new_style);
                                        if has_changes {
                                            // Push history before mutation
                                            let current = doc_signal.read().clone();
                                            let next_h = history.read().push(current);
                                            *history.write() = next_h;
                                            // Dispatch to db_tx - show error toast if dispatch fails
                                            if let Err(e) = dispatch_update_edge_style(&db_tx, eid.as_str(), new_style) {
                                                let _ = toast.error("Failed to save", Some(format!("{:?}", e)));
                                            }
                                        }
                                        doc_signal.with_mut(|doc| {
                                            if let Some(e) = doc.document.edges.get_mut(&eid) {
                                                e.style = new_style;
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
