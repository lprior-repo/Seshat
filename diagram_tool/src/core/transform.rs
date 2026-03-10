use crate::geometry::operations::compute_subgraph_bounds;
use crate::models::document::{DiagramDocument, NodeId, NodeKind, OrderedFloat};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TransformError {
    #[error("No items selected to align")]
    EmptySelection,
    #[error("Locked nodes cannot be transformed: {0}")]
    LockedNode(NodeId),
}

pub enum AlignmentAxis {
    Horizontal,
    Vertical,
}

pub enum AlignmentMode {
    Start,
    Center,
    End,
}

/// Recomputes bounds for all containers that are ancestors of the given nodes.
///
/// # Returns
/// Number of containers whose bounds were updated.
fn recompute_container_bounds(doc: &mut DiagramDocument, moved_node_ids: &[NodeId]) -> usize {
    // Find unique parent containers of the moved nodes
    let mut containers_to_update: Vec<NodeId> = Vec::new();

    for node_id in moved_node_ids {
        if let Some(node) = doc.document.nodes.get(node_id) {
            if let Some(parent_id) = &node.parent {
                // Check if this parent is a subgraph container
                if let Some(parent) = doc.document.nodes.get(parent_id) {
                    if parent.kind == NodeKind::Subgraph {
                        // Only add if not already in list
                        if !containers_to_update.contains(parent_id) {
                            containers_to_update.push(parent_id.clone());
                        }
                    }
                }
            }
        }
    }

    // For each container, recompute bounds from children
    let mut updated_count = 0;
    for container_id in containers_to_update {
        // Collect all children bounds
        let children_bounds: Vec<(f64, f64, f64, f64)> = doc
            .document
            .nodes
            .iter()
            .filter(|(_, node)| node.parent.as_ref() == Some(&container_id))
            .map(|(_, node)| (node.x.0, node.y.0, node.width.0, node.height.0))
            .collect();

        // Compute new bounds
        if let Some((x, y, width, height)) = compute_subgraph_bounds(children_bounds) {
            if let Some(container) = doc.document.nodes.get_mut(&container_id) {
                // Add padding to the computed bounds
                let padding = 24.0;
                container.x = OrderedFloat(x - padding);
                container.y = OrderedFloat(y - padding);
                container.width = OrderedFloat(width + padding * 2.0);
                container.height = OrderedFloat(height + padding * 2.0);
                updated_count += 1;
            }
        }
    }

    updated_count
}

/// Aligns selected nodes along the specified axis.
///
/// # Errors
///
/// Returns `TransformError::EmptySelection` if fewer than 2 nodes are selected.
/// Returns `TransformError::LockedNode` if any selected node is locked.
pub fn align_selection(
    doc: &mut DiagramDocument,
    axis: &AlignmentAxis,
    mode: &AlignmentMode,
) -> Result<(), TransformError> {
    let selected = doc.editor_state.selected_items.clone();
    if selected.len() < 2 {
        return Err(TransformError::EmptySelection);
    }

    // 1. Calculate boundaries and check constraints
    let mut min_val = f64::MAX;
    let mut max_val = f64::MIN;

    for id_str in &selected {
        let id = NodeId::new(id_str.clone());
        if let Some(node) = doc.document.nodes.get(&id) {
            if node.locked && node.kind != NodeKind::Subgraph {
                return Err(TransformError::LockedNode(id));
            }

            match axis {
                AlignmentAxis::Horizontal => {
                    min_val = min_val.min(node.x.0);
                    max_val = max_val.max(node.x.0 + node.width.0);
                }
                AlignmentAxis::Vertical => {
                    min_val = min_val.min(node.y.0);
                    max_val = max_val.max(node.y.0 + node.height.0);
                }
            }
        }
    }

    let center_val = min_val + (max_val - min_val) / 2.0;

    // 2. Apply alignment
    let mut transformed_ids: Vec<NodeId> = Vec::new();
    for id_str in &selected {
        let id = NodeId::new(id_str.clone());
        transformed_ids.push(id.clone());
        if let Some(node) = doc.document.nodes.get_mut(&id) {
            match axis {
                AlignmentAxis::Horizontal => {
                    let new_x = match mode {
                        AlignmentMode::Start => min_val,
                        AlignmentMode::Center => center_val - (node.width.0 / 2.0),
                        AlignmentMode::End => max_val - node.width.0,
                    };
                    node.x = OrderedFloat::new_unchecked(new_x);
                }
                AlignmentAxis::Vertical => {
                    let new_y = match mode {
                        AlignmentMode::Start => min_val,
                        AlignmentMode::Center => center_val - (node.height.0 / 2.0),
                        AlignmentMode::End => max_val - node.height.0,
                    };
                    node.y = OrderedFloat::new_unchecked(new_y);
                }
            }
        }
    }

    // Recompute container bounds after alignment (GEO-025)
    let _ = recompute_container_bounds(doc, &transformed_ids);

    Ok(())
}

/// Distributes selected nodes evenly along the specified axis.
///
/// # Errors
///
/// Returns `TransformError::EmptySelection` if fewer than 3 nodes are selected.
pub fn distribute_selection(
    doc: &mut DiagramDocument,
    axis: &AlignmentAxis,
) -> Result<(), TransformError> {
    let selected = doc.editor_state.selected_items.clone();
    if selected.len() < 3 {
        return Ok(());
    }

    let mut nodes: Vec<(NodeId, f64, f64)> = Vec::new();
    for id_str in &selected {
        let id = NodeId::new(id_str.clone());
        if let Some(node) = doc.document.nodes.get(&id) {
            if node.locked && node.kind != NodeKind::Subgraph {
                return Err(TransformError::LockedNode(id));
            }
            match axis {
                AlignmentAxis::Horizontal => nodes.push((id, node.x.0, node.width.0)),
                AlignmentAxis::Vertical => nodes.push((id, node.y.0, node.height.0)),
            }
        }
    }

    nodes.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let Some(first) = nodes.first() else {
        return Ok(());
    };
    let Some(last) = nodes.last() else {
        return Ok(());
    };

    let total_span = (last.1 + last.2) - first.1;
    let sum_of_extents: f64 = nodes.iter().map(|n| n.2).sum();
    let available_space = total_span - sum_of_extents;
    #[allow(clippy::cast_precision_loss)]
    let spacing = available_space / (nodes.len() as f64 - 1.0);

    let mut current_pos = first.1;
    let mut transformed_ids: Vec<NodeId> = Vec::new();

    for (id, _pos, extent) in nodes {
        transformed_ids.push(id.clone());
        if let Some(node) = doc.document.nodes.get_mut(&id) {
            match axis {
                AlignmentAxis::Horizontal => {
                    node.x = OrderedFloat::new_unchecked(current_pos);
                }
                AlignmentAxis::Vertical => {
                    node.y = OrderedFloat::new_unchecked(current_pos);
                }
            }
            current_pos += extent + spacing;
        }
    }

    // Recompute container bounds after distribution (GEO-025)
    let _ = recompute_container_bounds(doc, &transformed_ids);

    Ok(())
}

#[cfg(test)]
#[path = "transform_tests.rs"]
mod tests;
