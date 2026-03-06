use crate::geometry::AABB;
use crate::models::document::{DiagramDocument, NodeId, OrderedFloat};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum TransformError {
    #[error("No items selected to align")]
    EmptySelection,
    #[error("Locked nodes cannot be transformed: {0}")]
    LockedNode(NodeId),
}

/// Aligns all selected nodes to the minimum X boundary of the selection group.
pub fn align_left(doc: &mut DiagramDocument) -> Result<(), TransformError> {
    let selected = doc.editor_state.selected_items.clone();
    if selected.is_empty() {
        return Err(TransformError::EmptySelection);
    }

    // 1. Find min X across all selected nodes
    let mut min_x = f64::MAX;
    for id_str in &selected {
        let id = NodeId::new(id_str.clone());
        if let Some(node) = doc.document.nodes.get(&id) {
            if node.locked {
                return Err(TransformError::LockedNode(id));
            }
            if node.x.0 < min_x {
                min_x = node.x.0;
            }
        }
    }

    // 2. Apply min X to all selected nodes
    for id_str in &selected {
        let id = NodeId::new(id_str.clone());
        if let Some(node) = doc.document.nodes.get_mut(&id) {
            node.x = OrderedFloat::new_unchecked(min_x);
        }
    }

    Ok(())
}

/// Distributes selected nodes horizontally with equal spacing
pub fn distribute_horizontal(doc: &mut DiagramDocument) -> Result<(), TransformError> {
    let selected = doc.editor_state.selected_items.clone();
    if selected.len() < 3 {
        // Need at least 3 nodes to distribute
        return Ok(());
    }

    // 1. Extract and sort by X coordinate
    let mut nodes: Vec<(NodeId, f64, f64)> = Vec::new(); // (id, x, width)
    for id_str in &selected {
        let id = NodeId::new(id_str.clone());
        if let Some(node) = doc.document.nodes.get(&id) {
            if node.locked {
                return Err(TransformError::LockedNode(id));
            }
            nodes.push((id, node.x.0, node.width.0));
        }
    }

    nodes.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let first = nodes.first().unwrap();
    let last = nodes.last().unwrap();

    let total_width = (last.1 + last.2) - first.1;
    let sum_of_node_widths: f64 = nodes.iter().map(|n| n.2).sum();
    let available_space = total_width - sum_of_node_widths;
    let spacing = available_space / (nodes.len() as f64 - 1.0);

    let mut current_x = first.1;

    // Apply new positions
    for (id, _x, width) in nodes {
        if let Some(node) = doc.document.nodes.get_mut(&id) {
            node.x = OrderedFloat::new_unchecked(current_x);
            current_x += width + spacing;
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "transform_tests.rs"]
mod tests;
