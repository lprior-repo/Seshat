//! Alignment operations - align nodes horizontally or vertically

use std::collections::BTreeSet;

use dioxus::prelude::*;

use crate::history::History;
use crate::models::document::{DiagramDocument, LockState, NodeId, NodeKind, OrderedFloat};

/// Axis for alignment operations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignmentAxis {
    Horizontal,
    Vertical,
}

/// Mode for alignment operations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignmentMode {
    Start,  // Left (Horizontal) or Top (Vertical)
    Center, // Center (Horizontal) or Middle (Vertical)
    End,    // Right (Horizontal) or Bottom (Vertical)
}

/// Align selected nodes along the specified axis using the given mode.
///
/// Returns `true` if alignment was performed, `false` if:
/// - Fewer than 2 nodes are selected
/// - All selected nodes are locked
/// - Any selected node has non-finite coordinates
///
/// # Invariants
/// - Node dimensions (width, height) are never modified
/// - Z-order is preserved
/// - Locked nodes are skipped (unless they are Subgraphs)
pub fn apply_align_selection(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    axis: AlignmentAxis,
    mode: AlignmentMode,
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

    // Need at least 2 nodes to align
    if selected_nodes.len() < 2 {
        return false;
    }

    // Calculate bounding box
    let (min_pos, max_pos, max_extent) = match axis {
        AlignmentAxis::Horizontal => {
            let positions: Vec<(f64, f64)> = selected_nodes
                .iter()
                .filter_map(|id| current.document.nodes.get(id))
                .map(|node| (node.x.0, node.x.0 + node.width.0))
                .collect();

            if positions
                .iter()
                .any(|(p, e)| !p.is_finite() || !e.is_finite())
            {
                return false;
            }

            let min_x = positions
                .iter()
                .map(|(p, _)| *p)
                .fold(f64::INFINITY, f64::min);
            let max_right = positions
                .iter()
                .map(|(_, e)| *e)
                .fold(f64::NEG_INFINITY, f64::max);

            if !min_x.is_finite() || !max_right.is_finite() {
                return false;
            }

            (min_x, max_right, max_right - min_x)
        }
        AlignmentAxis::Vertical => {
            let positions: Vec<(f64, f64)> = selected_nodes
                .iter()
                .filter_map(|id| current.document.nodes.get(id))
                .map(|node| (node.y.0, node.y.0 + node.height.0))
                .collect();

            if positions
                .iter()
                .any(|(p, e)| !p.is_finite() || !e.is_finite())
            {
                return false;
            }

            let min_y = positions
                .iter()
                .map(|(p, _)| *p)
                .fold(f64::INFINITY, f64::min);
            let max_bottom = positions
                .iter()
                .map(|(_, e)| *e)
                .fold(f64::NEG_INFINITY, f64::max);

            if !min_y.is_finite() || !max_bottom.is_finite() {
                return false;
            }

            (min_y, max_bottom, max_bottom - min_y)
        }
    };

    let history = history_signal.read().clone();
    *history_signal.write() = history.push(current);

    doc_signal.with_mut(|doc| {
        for node_id in &selected_nodes {
            if let Some(node) = doc.document.nodes.get_mut(node_id) {
                // Double-check movability (should be redundant but defensive)
                if !node.lock_state.is_movable(&node.kind) {
                    continue;
                }

                match (axis, mode) {
                    (AlignmentAxis::Horizontal, AlignmentMode::Start) => {
                        // Align Left: set x to min_x
                        node.x = OrderedFloat(min_pos);
                    }
                    (AlignmentAxis::Horizontal, AlignmentMode::Center) => {
                        // Align Center H: center the node within the bounding box
                        let center_x = min_pos + max_extent / 2.0;
                        node.x = OrderedFloat(center_x - node.width.0 / 2.0);
                    }
                    (AlignmentAxis::Horizontal, AlignmentMode::End) => {
                        // Align Right: set x so right edge aligns with max_right
                        node.x = OrderedFloat(max_pos - node.width.0);
                    }
                    (AlignmentAxis::Vertical, AlignmentMode::Start) => {
                        // Align Top: set y to min_y
                        node.y = OrderedFloat(min_pos);
                    }
                    (AlignmentAxis::Vertical, AlignmentMode::Center) => {
                        // Align Middle V: center the node within the bounding box
                        let center_y = min_pos + max_extent / 2.0;
                        node.y = OrderedFloat(center_y - node.height.0 / 2.0);
                    }
                    (AlignmentAxis::Vertical, AlignmentMode::End) => {
                        // Align Bottom: set y so bottom edge aligns with max_bottom
                        node.y = OrderedFloat(max_pos - node.height.0);
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
        .map(|id| NodeId::new(id.clone()))
        .filter(|id| doc.document.nodes.contains_key(id))
        .collect()
}
