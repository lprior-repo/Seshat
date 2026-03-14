//! Selection operations - select all, clear, delete, group, ungroup, nudge

use std::collections::BTreeSet;

use dioxus::prelude::*;
use uuid::Uuid;

use crate::history::History;
use crate::models::document::{DiagramDocument, Node, NodeId, NodeKind, OrderedFloat};
use crate::models::envelope::EventEnvelope;

/// Select all nodes and edges in the document
pub fn apply_select_all(mut doc_signal: Signal<DiagramDocument>) {
    doc_signal.with_mut(|doc| {
        doc.editor_state.selected_items = doc
            .document
            .nodes
            .keys()
            .map(ToString::to_string)
            .chain(doc.document.edges.keys().map(ToString::to_string))
            .collect();
    });
}

/// Clear the current selection
pub fn apply_clear_selection(mut doc_signal: Signal<DiagramDocument>) {
    doc_signal.with_mut(|doc| {
        doc.editor_state.selected_items.clear();
    });
}

/// Delete all selected nodes and edges
#[must_use]
pub fn apply_delete_selected(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
) -> bool {
    let selected = doc_signal.read().editor_state.selected_items.clone();
    if selected.is_empty() {
        return false;
    }

    let history = history_signal.read().clone();
    *history_signal.write() = history.push(doc_signal.read().clone());

    doc_signal.with_mut(|doc| {
        let deleted_node_ids =
            selected_nodes_from_selection(&doc.editor_state.selected_items, &doc.document.nodes);
        doc.document.nodes = doc
            .document
            .nodes
            .iter()
            .filter(|(id, _)| !selected.contains(&id.to_string()))
            .map(|(id, node)| {
                let mut next = node.clone();
                next.parent = reparent_if_deleted(next.parent, &deleted_node_ids);
                (id.clone(), next)
            })
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
    });
    true
}

/// Nudge selected nodes by the given delta
#[must_use]
pub fn apply_nudge_selection(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    dx: f64,
    dy: f64,
    push_undo: bool,
) -> bool {
    let selected_nodes = {
        let doc = doc_signal.read();
        selected_node_ids(&doc)
    };
    if selected_nodes.is_empty() || (dx == 0.0 && dy == 0.0) {
        return false;
    }

    if push_undo {
        let history = history_signal.read().clone();
        *history_signal.write() = history.push(doc_signal.read().clone());
    }
    doc_signal.with_mut(|doc| {
        for node_id in selected_nodes {
            if let Some(node) = doc.document.nodes.get_mut(&node_id) {
                if node.locked && node.kind != NodeKind::Subgraph {
                    continue;
                }
                node.x = OrderedFloat(node.x.0 + dx);
                node.y = OrderedFloat(node.y.0 + dy);
            }
        }
        doc.revision = doc.revision.increment();
    });
    true
}

/// Group selected nodes into a parent container
#[must_use]
pub fn apply_group_selection(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
) -> bool {
    let selected_nodes = {
        let doc = doc_signal.read();
        selected_node_ids(&doc)
            .into_iter()
            .filter(|id| {
                doc.document
                    .nodes
                    .get(id)
                    .is_some_and(|node| node.kind != NodeKind::Subgraph)
            })
            .collect::<Vec<_>>()
    };
    if selected_nodes.len() < 2 {
        return false;
    }

    let (min_x, min_y, max_x, max_y) = {
        let doc = doc_signal.read();
        selected_nodes.iter().fold(
            (
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
            ),
            |(min_x, min_y, max_x, max_y), node_id| {
                doc.document
                    .nodes
                    .get(node_id)
                    .map_or((min_x, min_y, max_x, max_y), |node| {
                        (
                            min_x.min(node.x.0),
                            min_y.min(node.y.0),
                            max_x.max(node.x.0 + node.width.0),
                            max_y.max(node.y.0 + node.height.0),
                        )
                    })
            },
        )
    };

    if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
        return false;
    }

    let history = history_signal.read().clone();
    *history_signal.write() = history.push(doc_signal.read().clone());

    let group_id = NodeId::new(Uuid::new_v4().to_string());
    let member_ids = selected_nodes;
    doc_signal.with_mut(|doc| {
        for node_id in &member_ids {
            if let Some(node) = doc.document.nodes.get_mut(node_id) {
                node.parent = Some(group_id.clone());
            }
        }

        // Create a group node (Subgraph)
        let group_node = Node {
            kind: NodeKind::Subgraph,
            icon: String::new(),
            label: "Group".to_string(),
            x: OrderedFloat(min_x),
            y: OrderedFloat(min_y),
            width: OrderedFloat(max_x - min_x),
            height: OrderedFloat(max_y - min_y),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };
        let _ = doc.document.nodes.insert(group_id.clone(), group_node);

        doc.editor_state.selected_items.clear();
        let _ = doc.editor_state.selected_items.insert(group_id.to_string());
        doc.revision = doc.revision.increment();
    });
    true
}

/// Ungroup a selected group node, releasing its children
#[must_use]
pub fn apply_ungroup_selection(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    _db_tx: Option<Coroutine<EventEnvelope>>,
) -> bool {
    let selected = doc_signal.read().editor_state.selected_items.clone();

    let group_ids: Vec<NodeId> = selected
        .iter()
        .filter_map(|id| NodeId::new(id.clone()).into())
        .filter(|id| {
            doc_signal
                .read()
                .document
                .nodes
                .get(id)
                .is_some_and(|node| node.kind == NodeKind::Subgraph)
        })
        .collect();

    if group_ids.is_empty() {
        return false;
    }

    let history = history_signal.read().clone();
    *history_signal.write() = history.push(doc_signal.read().clone());

    doc_signal.with_mut(|doc| {
        for group_id in &group_ids {
            // Get member IDs before removing the group
            let member_ids: Vec<NodeId> = doc
                .document
                .nodes
                .iter()
                .filter(|(_, node)| node.parent.as_ref() == Some(group_id))
                .map(|(id, _)| id.clone())
                .collect();

            // Clear parent reference for all members
            for member_id in &member_ids {
                if let Some(node) = doc.document.nodes.get_mut(member_id) {
                    node.parent = None;
                }
            }

            // Update selection to include the members
            for member_id in &member_ids {
                let _ = doc
                    .editor_state
                    .selected_items
                    .insert(member_id.to_string());
            }

            // Remove the group node
            doc.document.nodes.remove(group_id);
        }
        doc.revision = doc.revision.increment();
    });
    true
}

// Private helper functions

fn selected_node_ids(doc: &DiagramDocument) -> BTreeSet<NodeId> {
    doc.editor_state
        .selected_items
        .iter()
        .map(|id| NodeId::new(id.clone()))
        .filter(|id| doc.document.nodes.contains_key(id))
        .collect()
}

fn selected_nodes_from_selection(
    selected: &im::HashSet<String>,
    nodes: &im::HashMap<NodeId, Node>,
) -> BTreeSet<NodeId> {
    selected
        .iter()
        .map(|id| NodeId::new(id.clone()))
        .filter(|id| nodes.contains_key(id))
        .collect()
}

fn reparent_if_deleted(parent: Option<NodeId>, deleted_ids: &BTreeSet<NodeId>) -> Option<NodeId> {
    parent.and_then(|parent_id| {
        if deleted_ids.contains(&parent_id) {
            None
        } else {
            Some(parent_id)
        }
    })
}
