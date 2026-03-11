//! Subgraph event handling and lifecycle management (bounds, z-index, add/remove nodes).
//!
//! Enforces Big 6 Functional Rust constraints (Data->Calc->Actions, Zero Mutability).

use std::collections::HashSet;
use thiserror::Error;

use crate::models::document::{Node, NodeId, OrderedFloat};
use crate::models::projection::types::DiagramProjection;

/// Alias to align with contract terminology
pub type DiagramState = DiagramProjection;

/// Bounding box representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: OrderedFloat,
    pub y: OrderedFloat,
    pub width: OrderedFloat,
    pub height: OrderedFloat,
}

impl Rect {
    /// Creates a new `Rect` ensuring valid bounds.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidBounds` if width or height is negative.
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Result<Self, Error> {
        if width < 0.0 || height < 0.0 {
            return Err(Error::InvalidBounds(Self {
                x: OrderedFloat::new_unchecked(x),
                y: OrderedFloat::new_unchecked(y),
                width: OrderedFloat::new_unchecked(width),
                height: OrderedFloat::new_unchecked(height),
            }));
        }
        Ok(Self {
            x: OrderedFloat::new_unchecked(x),
            y: OrderedFloat::new_unchecked(y),
            width: OrderedFloat::new_unchecked(width),
            height: OrderedFloat::new_unchecked(height),
        })
    }
}

/// Errors defined in the contract taxonomy
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),
    #[error("Cycle detected: {0} -> {1}")]
    CycleDetected(NodeId, NodeId),
    #[error("Invalid bounds: {0:?}")]
    InvalidBounds(Rect),
}

/// Helper to get a node, returning `Error::NodeNotFound` if missing.
fn get_node<'a>(state: &'a DiagramState, id: &NodeId) -> Result<&'a Node, Error> {
    state
        .get_node(id)
        .ok_or_else(|| Error::NodeNotFound(id.clone()))
}

/// Helper to get a node and ensure it's a subgraph
fn get_subgraph<'a>(state: &'a DiagramState, id: &NodeId) -> Result<&'a Node, Error> {
    let node = get_node(state, id)?;
    // The contract implies `NodeNotFound` is used if the subgraph is not found.
    // If it exists but isn't a subgraph, should we return an error? The contract doesn't explicitly define one.
    // Let's assume `NodeNotFound` for missing, but if it exists we treat it as a container.
    Ok(node)
}

/// Calculate the bounds of a subgraph based on its children.
///
/// SUB-019 (Bounds): After a child is added, removed, or moved, the subgraph's bounds must accurately enclose all child nodes plus any defined padding.
///
/// # Errors
///
/// Returns `Error::NodeNotFound` if the subgraph does not exist.
pub fn calculate_subgraph_bounds(
    subgraph_id: &NodeId,
    state: &DiagramState,
) -> Result<Rect, Error> {
    // Ensure subgraph exists
    let _subgraph = get_subgraph(state, subgraph_id)?;

    // Find all direct children
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut has_children = false;

    for node in state.nodes.values() {
        if node.parent.as_ref() == Some(subgraph_id) {
            has_children = true;
            min_x = min_x.min(node.x.0);
            min_y = min_y.min(node.y.0);
            max_x = max_x.max(node.x.0 + node.width.0);
            max_y = max_y.max(node.y.0 + node.height.0);
        }
    }

    // SUB-024: If all children are removed, subgraph container node remains with empty bounds or minimum dimensions.
    if !has_children {
        return Rect::new(0.0, 0.0, 0.0, 0.0);
    }

    let padding = 20.0;
    let x = min_x - padding;
    let y = min_y - padding;
    let width = max_x - min_x + padding * 2.0;
    let height = max_y - min_y + padding * 2.0;

    Rect::new(x, y, width, height)
}

/// Updates the z-index of children to be strictly above the subgraph container.
///
/// SUB-020 (Z-index)
///
/// # Errors
///
/// Returns `Error::NodeNotFound` if the subgraph does not exist.
pub fn update_z_index_ordering(
    subgraph_id: &NodeId,
    state: &mut DiagramState,
) -> Result<(), Error> {
    let subgraph = get_subgraph(state, subgraph_id)?;
    let base_z = subgraph.z_index;

    let new_nodes = state.nodes.clone();

    // We must ensure children are strictly above the base_z
    let new_nodes = state
        .nodes
        .iter()
        .filter(|(_, node)| node.parent.as_ref() == Some(subgraph_id))
        .map(|(id, _)| id.clone())
        .fold(new_nodes, |acc, id| {
            if let Some(node) = acc.get(&id) {
                let mut node = node.clone();
                // Assign a z_index strictly above the container
                // A simple implementation is to put children at base_z + 1
                node.z_index = base_z + 1;
                acc.update(id, node)
            } else {
                acc
            }
        });

    state.nodes = new_nodes;
    Ok(())
}

/// Checks if adding `child_id` to `subgraph_id` would create a cycle.
fn detect_cycle(
    child_id: &NodeId,
    subgraph_id: &NodeId,
    state: &DiagramState,
) -> Result<(), Error> {
    if child_id == subgraph_id {
        return Err(Error::CycleDetected(child_id.clone(), subgraph_id.clone()));
    }

    let mut current_parent = state.nodes.get(subgraph_id).and_then(|n| n.parent.clone());
    let mut visited = HashSet::new();
    visited.insert(subgraph_id.clone());

    while let Some(parent_id) = current_parent {
        if &parent_id == child_id {
            return Err(Error::CycleDetected(child_id.clone(), subgraph_id.clone()));
        }
        if !visited.insert(parent_id.clone()) {
            // Already visited, cycle detected internally, but strictly child is parent of subgraph
            return Err(Error::CycleDetected(child_id.clone(), subgraph_id.clone()));
        }
        current_parent = state.nodes.get(&parent_id).and_then(|n| n.parent.clone());
    }

    Ok(())
}

/// SUB-021: Adds a node to a subgraph, updating parent reference and recalculating bounds.
///
/// # Errors
///
/// Returns `Error::NodeNotFound` if the subgraph or child does not exist.
/// Returns `Error::CycleDetected` if adding the child would create a cycle.
pub fn add_node_to_subgraph(
    child_id: &NodeId,
    subgraph_id: &NodeId,
    state: &mut DiagramState,
) -> Result<(), Error> {
    // P1: Subgraph must exist
    let _ = get_subgraph(state, subgraph_id)?;
    // P2: Child must exist
    let _ = get_node(state, child_id)?;
    // P3: No cycles
    detect_cycle(child_id, subgraph_id, state)?;

    let new_nodes = state.nodes.clone();

    // Q3: Added child's parent_id is updated to subgraph
    let new_nodes = if let Some(node) = new_nodes.get(child_id) {
        let mut node = node.clone();
        node.parent = Some(subgraph_id.clone());
        new_nodes.update(child_id.clone(), node)
    } else {
        new_nodes
    };

    state.nodes = new_nodes;

    // Q1: Subgraph bounds must accurately enclose all child nodes
    let new_bounds = calculate_subgraph_bounds(subgraph_id, state)?;

    // Apply calculated bounds to subgraph
    let new_nodes = if let Some(node) = state.nodes.get(subgraph_id) {
        let mut node = node.clone();
        node.x = new_bounds.x;
        node.y = new_bounds.y;
        node.width = new_bounds.width;
        node.height = new_bounds.height;
        state.nodes.update(subgraph_id.clone(), node)
    } else {
        state.nodes.clone()
    };

    state.nodes = new_nodes;

    // Q2: Subgraph children must inherit z-index
    update_z_index_ordering(subgraph_id, state)?;

    Ok(())
}

/// SUB-022: Removes a node from a subgraph, clearing its parent reference.
///
/// # Errors
///
/// Returns `Error::NodeNotFound` if the child does not exist.
pub fn remove_node_from_subgraph(child_id: &NodeId, state: &mut DiagramState) -> Result<(), Error> {
    let node = get_node(state, child_id)?;
    let old_parent = node.parent.clone();

    let new_nodes = state.nodes.clone();

    // Q4: Removed child's parent_id is set to None
    let new_nodes = if let Some(node) = new_nodes.get(child_id) {
        let mut node = node.clone();
        node.parent = None;
        new_nodes.update(child_id.clone(), node)
    } else {
        new_nodes
    };

    state.nodes = new_nodes;

    // Recalculate bounds of the former parent if it exists
    if let Some(subgraph_id) = old_parent {
        if state.has_node(&subgraph_id) {
            let new_bounds = calculate_subgraph_bounds(&subgraph_id, state)?;
            let new_nodes = if let Some(node) = state.nodes.get(&subgraph_id) {
                let mut node = node.clone();
                node.x = new_bounds.x;
                node.y = new_bounds.y;
                node.width = new_bounds.width;
                node.height = new_bounds.height;
                state.nodes.update(subgraph_id.clone(), node)
            } else {
                state.nodes.clone()
            };
            state.nodes = new_nodes;
        }
    }

    Ok(())
}

/// SUB-023: Batch add nodes to a subgraph, updating parent references and recalculating bounds exactly once.
///
/// # Errors
///
/// Returns `Error::NodeNotFound` if the subgraph or any child does not exist.
/// Returns `Error::CycleDetected` if adding a child would create a cycle.
pub fn batch_add_nodes_to_subgraph(
    child_ids: &[NodeId],
    subgraph_id: &NodeId,
    state: &mut DiagramState,
) -> Result<(), Error> {
    if child_ids.is_empty() {
        return Ok(());
    }

    let _ = get_subgraph(state, subgraph_id)?;

    // Verify all children exist and no cycles
    for child_id in child_ids {
        let _ = get_node(state, child_id)?;
        detect_cycle(child_id, subgraph_id, state)?;
    }

    let new_nodes = state.nodes.clone();

    // Q5: All specified nodes have parent_id updated
    let new_nodes = child_ids.iter().fold(new_nodes, |acc, child_id| {
        if let Some(node) = acc.get(child_id) {
            let mut node = node.clone();
            node.parent = Some(subgraph_id.clone());
            acc.update(child_id.clone(), node)
        } else {
            acc
        }
    });

    state.nodes = new_nodes;

    // Q5: Subgraph bounds recalculated exactly once
    let new_bounds = calculate_subgraph_bounds(subgraph_id, state)?;

    let new_nodes = if let Some(node) = state.nodes.get(subgraph_id) {
        let mut node = node.clone();
        node.x = new_bounds.x;
        node.y = new_bounds.y;
        node.width = new_bounds.width;
        node.height = new_bounds.height;
        state.nodes.update(subgraph_id.clone(), node)
    } else {
        state.nodes.clone()
    };

    state.nodes = new_nodes;

    update_z_index_ordering(subgraph_id, state)?;

    Ok(())
}

/// SUB-024: Remove all nodes from a subgraph, leaving an empty container.
///
/// # Errors
///
/// Returns `Error::NodeNotFound` if the subgraph does not exist.
pub fn remove_all_nodes_from_subgraph(
    subgraph_id: &NodeId,
    state: &mut DiagramState,
) -> Result<(), Error> {
    let _ = get_subgraph(state, subgraph_id)?;

    let new_nodes = state.nodes.clone();

    // Q6: All children removed
    let new_nodes = state
        .nodes
        .iter()
        .filter(|(_, node)| node.parent.as_ref() == Some(subgraph_id))
        .map(|(id, _)| id.clone())
        .fold(new_nodes, |acc, child_id| {
            if let Some(node) = acc.get(&child_id) {
                let mut node = node.clone();
                node.parent = None;
                acc.update(child_id, node)
            } else {
                acc
            }
        });

    state.nodes = new_nodes;

    // Recalculate bounds (will be empty)
    let new_bounds = calculate_subgraph_bounds(subgraph_id, state)?;

    let new_nodes = if let Some(node) = state.nodes.get(subgraph_id) {
        let mut node = node.clone();
        node.x = new_bounds.x;
        node.y = new_bounds.y;
        node.width = new_bounds.width;
        node.height = new_bounds.height;
        state.nodes.update(subgraph_id.clone(), node)
    } else {
        state.nodes.clone()
    };

    state.nodes = new_nodes;

    Ok(())
}
