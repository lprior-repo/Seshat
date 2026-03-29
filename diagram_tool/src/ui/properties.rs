#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
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
        "cursor-pointer"
    } else {
        "cursor-not-allowed"
    };
    let delete_opacity = if selected_total > 0 { "opacity-100" } else { "opacity-60" };

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
            class: "properties-panel w-[260px] max-w-[40vw] bg-surface p-2.5 border-l border-[var(--border-subtle)] flex flex-col gap-2.5 min-h-0 overflow-hidden",

            h3 {
                class: "m-0 text-xs tracking-[0.08em] uppercase text-muted-foreground",
                "Properties"
            }
            p {
                class: "m-0 text-[11px] text-[var(--text-dim)]",
                "{selected_node_count} node(s), {selected_edge_count} edge(s) selected"
            }

            div {
                class: "flex-1 min-h-0 overflow-y-auto flex flex-col gap-2.5 pr-0.5",

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
                        class: "p-1.5 rounded-md border border-dashed border-border text-[var(--text-dim)] text-[11px]",
                        "Multiple items selected. Use delete to remove all selected items."
                    }
                }
            }

            div {
                class: "pt-2 border-t border-[var(--border-subtle)]",
                button {
                    class: "w-full py-[7px] px-2.5 {delete_cursor} rounded-md border border-border bg-[var(--bg-base)] text-red-400 {delete_opacity}",
                    disabled: selected_total == 0,
                    onclick: delete_selected,
                    "Delete Selected"
                }
            }
        }
    }
}
