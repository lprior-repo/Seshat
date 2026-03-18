//! Distribution operations - distribute nodes evenly

use std::collections::BTreeSet;

use dioxus::prelude::*;

use crate::history::History;
use diagram_models::document::{DiagramDocument, NodeId, OrderedFloat};

/// Axis for distribution operations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DistributionAxis {
    Horizontal,
    Vertical,
}

/// Distribute selected nodes evenly along the specified axis.
///
/// # Preconditions
/// - At least 3 nodes must be selected (distribution requires 3+ to be meaningful)
/// - Selected nodes must have valid (finite) positions
/// - Nodes must be movable (not locked, or are subgraphs)
///
/// # Postconditions
/// - Outermost nodes remain at original bounds
/// - Interior nodes are repositioned to create equal spacing
/// - Node dimensions are preserved
/// - History is updated for undo support
///
/// # Invariants
/// - Distribution does not change node size
/// - Horizontal distribution preserves Y positions
/// - Vertical distribution preserves X positions
/// - Z-order is preserved
#[must_use]
pub fn apply_distribute_selection(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    axis: DistributionAxis,
) -> bool {
    let current = doc_signal.read().clone();

    // Get selected nodes that are movable (not locked, or are subgraphs)
    let selected_nodes: Vec<NodeId> = selected_node_ids(&current)
        .into_iter()
        .filter(|id| {
            current.document.nodes.get(id).is_some_and(|node| {
                let coords_finite = node.x.0.is_finite() && node.y.0.is_finite();
                let movable = node.lock_state.is_movable(&node.kind);
                coords_finite && movable
            })
        })
        .collect();

    // Need at least 3 nodes to distribute
    if selected_nodes.len() < 3 {
        return false;
    }

    // Collect node data: (id, position, size) sorted by position along axis
    let mut node_data: Vec<(NodeId, f64, f64)> = selected_nodes
        .iter()
        .filter_map(|id| {
            current.document.nodes.get(id).map(|node| {
                let (pos, size) = match axis {
                    DistributionAxis::Horizontal => (node.x.0, node.width.0),
                    DistributionAxis::Vertical => (node.y.0, node.height.0),
                };
                (id.clone(), pos, size)
            })
        })
        .collect();

    // Sort by position along the distribution axis
    node_data.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    if node_data.len() < 3 {
        return false;
    }

    // Calculate the space to distribute
    // Safety: len() >= 3 checked at line 77, so first/last are guaranteed to be Some
    let Some(first) = node_data.first() else {
        return false;
    };
    let Some(last) = node_data.last() else {
        return false;
    };
    let total_extent = (last.1 + last.2) - first.1;

    let total_sizes: f64 = node_data.iter().map(|(_, _, size)| size).sum();
    let spacing = (total_extent - total_sizes) / (node_data.len() - 1) as f64;

    if !spacing.is_finite() || spacing < 0.0 {
        return false;
    }

    let history = history_signal.read().clone();
    *history_signal.write() = history.push(current);

    doc_signal.with_mut(|doc| {
        // Update positions: first node stays, last node stays, interior nodes are spaced
        for (i, (node_id, _, _)) in node_data.iter().enumerate() {
            if i == 0 || i == node_data.len() - 1 {
                continue; // Skip first and last
            }

            if let Some(node) = doc.document.nodes.get_mut(node_id) {
                if !node.lock_state.is_movable(&node.kind) {
                    continue;
                }

                // Calculate new position based on accumulated spacing
                let new_pos = (i as f64).mul_add(spacing, first.1)
                    + (0..i).map(|j| node_data[j].2).sum::<f64>();

                match axis {
                    DistributionAxis::Horizontal => {
                        node.x = OrderedFloat(new_pos);
                    }
                    DistributionAxis::Vertical => {
                        node.y = OrderedFloat(new_pos);
                    }
                }
            }
        }
        doc.revision = doc.revision.increment();
    });

    true
}

// Private helper function

fn selected_node_ids(doc: &DiagramDocument) -> BTreeSet<NodeId> {
    doc.editor_state
        .selected_items
        .iter()
        .map(|id| diagram_models::document::NodeId::new(id.clone()))
        .filter(|id| doc.document.nodes.contains_key(id))
        .collect()
}
