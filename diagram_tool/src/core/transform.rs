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
    for id_str in &selected {
        let id = NodeId::new(id_str.clone());
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

    for (id, _pos, extent) in nodes {
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

    Ok(())
}

#[cfg(test)]
#[path = "transform_tests.rs"]
mod tests;
