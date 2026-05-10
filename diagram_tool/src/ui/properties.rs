#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

use crate::ui::commands::apply_delete_selected;
use diagram_models::document::{DiagramDocument, Edge, EdgeId, Node, NodeId};
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Eq)]
struct ConnectionRow {
    direction: &'static str,
    peer_label: String,
    edge_label: String,
}

#[derive(Clone, PartialEq)]
struct PropertiesViewModel {
    selected_total: usize,
    selected_node_count: usize,
    selected_edge_count: usize,
    single_node: Option<(NodeId, Node)>,
    single_edge: Option<(EdgeId, Edge)>,
    connection_rows: Vec<ConnectionRow>,
    show_multi_select_hint: bool,
}

fn node_label_with_id_fallback(doc: &DiagramDocument, id: &NodeId) -> String {
    match doc.document.nodes.get(id) {
        Some(node) if !node.label.is_empty() => node.label.clone(),
        _ => id.to_string(),
    }
}

fn connection_row_for_edge(
    doc: &DiagramDocument,
    node_id: &NodeId,
    edge: &Edge,
) -> Option<ConnectionRow> {
    if edge.source == *node_id {
        Some(ConnectionRow {
            direction: "out",
            peer_label: node_label_with_id_fallback(doc, &edge.target),
            edge_label: edge.label.clone(),
        })
    } else if edge.target == *node_id {
        Some(ConnectionRow {
            direction: "in",
            peer_label: node_label_with_id_fallback(doc, &edge.source),
            edge_label: edge.label.clone(),
        })
    } else {
        None
    }
}

fn connection_rows_for_node(doc: &DiagramDocument, node_id: &NodeId) -> Vec<ConnectionRow> {
    doc.document
        .edges
        .values()
        .filter_map(|edge| connection_row_for_edge(doc, node_id, edge))
        .collect()
}

fn selected_item_ids(doc: &DiagramDocument) -> Vec<String> {
    doc.editor_state.selected_items.iter().cloned().collect()
}

fn selected_nodes(doc: &DiagramDocument, selected_items: &[String]) -> Vec<(NodeId, Node)> {
    selected_items
        .iter()
        .filter_map(|id| {
            let node_id = NodeId::new(id.clone());
            doc.document
                .nodes
                .get(&node_id)
                .map(|node| (node_id, node.clone()))
        })
        .collect()
}

fn selected_edges(doc: &DiagramDocument, selected_items: &[String]) -> Vec<(EdgeId, Edge)> {
    selected_items
        .iter()
        .filter_map(|id| {
            let edge_id = EdgeId::new(id.clone());
            doc.document
                .edges
                .get(&edge_id)
                .map(|edge| (edge_id, edge.clone()))
        })
        .collect()
}

fn properties_view_model(doc: &DiagramDocument) -> PropertiesViewModel {
    let selected_items = selected_item_ids(doc);
    let selected_total = selected_items.len();
    let selected_nodes = selected_nodes(doc, &selected_items);
    let selected_edges = selected_edges(doc, &selected_items);
    let selected_node_count = selected_nodes.len();
    let selected_edge_count = selected_edges.len();
    let single_node = single_node_selection(selected_node_count, &selected_nodes);
    let single_edge = single_edge_selection(selected_edge_count, &selected_edges);
    let connection_rows = match &single_node {
        Some((id, _)) => connection_rows_for_node(doc, id),
        None => Vec::new(),
    };

    PropertiesViewModel {
        selected_total,
        selected_node_count,
        selected_edge_count,
        single_node,
        single_edge,
        connection_rows,
        show_multi_select_hint: selected_node_count > 1
            || selected_node_count >= 1 && selected_edge_count >= 1,
    }
}

fn single_node_selection(count: usize, items: &[(NodeId, Node)]) -> Option<(NodeId, Node)> {
    if count == 1 {
        items.first().cloned()
    } else {
        None
    }
}

fn single_edge_selection(count: usize, items: &[(EdgeId, Edge)]) -> Option<(EdgeId, Edge)> {
    if count == 1 {
        items.first().cloned()
    } else {
        None
    }
}

fn delete_button_state(selected_total: usize) -> (&'static str, &'static str) {
    if selected_total > 0 {
        ("cursor-pointer", "opacity-100")
    } else {
        ("cursor-not-allowed", "opacity-60")
    }
}

#[component]
pub fn PropertiesPanel() -> Element {
    let app_state = use_context::<crate::app::AppState>();
    let doc_signal = app_state.document;
    let history = app_state.history;
    let doc_snapshot = doc_signal.read().clone();
    let view_model = properties_view_model(&doc_snapshot);
    let (delete_cursor, delete_opacity) = delete_button_state(view_model.selected_total);
    let delete_selected = move |_| {
        let _deleted = apply_delete_selected(doc_signal, history);
    };

    rsx! {
        aside {
            class: "properties-panel hidden lg:flex w-[260px] max-w-[40vw] bg-surface p-2.5 border-l border-[var(--border-subtle)] flex-col gap-2.5 min-h-0 overflow-hidden",
            "data-testid": "properties-panel",

            h3 { class: "m-0 text-xs tracking-[0.08em] uppercase text-muted-foreground", "Properties" }
            p {
                class: "m-0 text-[11px] text-[var(--text-dim)]",
                "{view_model.selected_node_count} node(s), {view_model.selected_edge_count} edge(s) selected"
            }

            PropertiesContent { doc_snapshot, view_model: view_model.clone() }

            div { class: "pt-2 border-t border-[var(--border-subtle)]",
                button {
                    class: "w-full py-[7px] px-2.5 {delete_cursor} rounded-md border border-border bg-[var(--bg-base)] text-red-400 {delete_opacity}",
                    disabled: view_model.selected_total == 0,
                    onclick: delete_selected,
                    "Delete Selected"
                }
            }
        }
    }
}

#[component]
fn PropertiesContent(doc_snapshot: DiagramDocument, view_model: PropertiesViewModel) -> Element {
    let selected_total = view_model.selected_total;
    let single_node = view_model.single_node;
    let single_edge = view_model.single_edge;
    let connection_rows = view_model.connection_rows;
    let show_multi_select_hint = view_model.show_multi_select_hint;

    rsx! {
        div { class: "flex-1 min-h-0 overflow-y-auto flex flex-col gap-2.5 pr-0.5",
            if selected_total == 0 {
                EmptyPanel {}
            }

            if let Some((id, node)) = single_node {
                NodePanel { id, node, connection_rows }
            }

            if let Some((id, edge)) = single_edge {
                EdgePanel {
                    id,
                    source_label: node_label_with_id_fallback(&doc_snapshot, &edge.source),
                    target_label: node_label_with_id_fallback(&doc_snapshot, &edge.target),
                    edge,
                }
            }

            if show_multi_select_hint {
                div {
                    class: "p-1.5 rounded-md border border-dashed border-border text-[var(--text-dim)] text-[11px]",
                    "Multiple items selected. Use delete to remove all selected items."
                }
            }
        }
    }
}

#[component]
fn EmptyPanel() -> Element {
    rsx! {
        div { class: "p-2 rounded-md border border-border bg-[var(--bg-base)] text-[12px] text-[var(--text-dim)]",
            p { class: "m-0 font-medium text-foreground", "No selection" }
            p { class: "m-0 mt-1", "Select a node or edge to inspect its metadata." }
        }
    }
}

#[component]
fn NodePanel(id: NodeId, node: Node, connection_rows: Vec<ConnectionRow>) -> Element {
    let kind = format!("{:?}", node.kind);
    let lock_state = format!("{:?}", node.lock_state);

    rsx! {
        div { class: "p-2 rounded-md border border-border bg-[var(--bg-base)] text-[12px] text-foreground space-y-1",
            h4 { class: "m-0 text-[12px]", "Node" }
            PropertyRow { label: "ID", value: id.to_string() }
            PropertyRow { label: "Label", value: node.label }
            PropertyRow { label: "Kind", value: kind }
            PropertyRow { label: "Position", value: format!("{:.0}, {:.0}", node.x.0, node.y.0) }
            PropertyRow { label: "Size", value: format!("{:.0} x {:.0}", node.width.0, node.height.0) }
            PropertyRow { label: "Lock", value: lock_state }
            if !connection_rows.is_empty() {
                div { class: "mt-2 pt-2 border-t border-[var(--border-subtle)] space-y-1",
                    p { class: "m-0 text-[11px] uppercase tracking-[0.08em] text-muted-foreground", "Connections" }
                    for row in connection_rows {
                        p { class: "m-0 text-[11px] text-[var(--text-dim)]",
                            "{row.direction} -> {row.peer_label} {row.edge_label}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn EdgePanel(id: EdgeId, edge: Edge, source_label: String, target_label: String) -> Element {
    let style = format!("{:?}", edge.style);
    let arrow_type = format!("{:?}", edge.arrow_type);

    rsx! {
        div { class: "p-2 rounded-md border border-border bg-[var(--bg-base)] text-[12px] text-foreground space-y-1",
            h4 { class: "m-0 text-[12px]", "Edge" }
            PropertyRow { label: "ID", value: id.to_string() }
            PropertyRow { label: "Label", value: edge.label }
            PropertyRow { label: "Source", value: source_label }
            PropertyRow { label: "Target", value: target_label }
            PropertyRow { label: "Style", value: style }
            PropertyRow { label: "Arrow", value: arrow_type }
        }
    }
}

#[component]
fn PropertyRow(label: &'static str, value: String) -> Element {
    let display_value = if value.is_empty() {
        "-".to_string()
    } else {
        value
    };

    rsx! {
        div { class: "grid grid-cols-[72px_1fr] gap-2 text-[11px]",
            span { class: "text-muted-foreground", "{label}" }
            span { class: "min-w-0 truncate", title: "{display_value}", "{display_value}" }
        }
    }
}
