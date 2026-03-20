#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

pub mod default_panel;
pub mod edge_panel;
pub mod node_panel;

use crate::history::History;
use crate::ui::properties::default_panel::DefaultPanel;
use crate::ui::properties::edge_panel::EdgePanel;
use crate::ui::properties::node_panel::NodePanel;
use crate::ui::properties_helpers::{node_label_with_id_fallback, remove_selected};
use crate::ui::theme::{BG_BASE, BG_SURFACE, BORDER, BORDER_SUBTLE, TEXT_DIM, TEXT_MUTED};
use diagram_models::document::{DiagramDocument, EdgeId, NodeId};
use dioxus::prelude::*;

#[component]
#[allow(clippy::approx_constant, clippy::float_cmp)]
pub fn PropertiesPanel() -> Element {
    let app_state = use_context::<crate::app::AppState>();
    let mut doc_signal = app_state.document;
    let mut history = app_state.history;

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
                    DefaultPanel {}
                }

                if let Some((id, node)) = single_node {
                    NodePanel {
                        id: id,
                        node: node,
                        connection_rows: connection_rows
                    }
                }

                if let Some((id, edge)) = single_edge {
                    {
                        let source_label = node_label_with_id_fallback(&doc_snapshot, &edge.source);
                        let target_label = node_label_with_id_fallback(&doc_snapshot, &edge.target);
                        rsx! {
                            EdgePanel {
                                id: id,
                                edge: edge,
                                source_label: source_label,
                                target_label: target_label
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
