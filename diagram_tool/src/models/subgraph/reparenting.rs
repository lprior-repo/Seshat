//! Node reparenting operations
//!
//! Contains operations for setting parent references, cycle detection,
//! and validation of reparenting operations.

use crate::models::document::{NodeId, NodeKind};

use super::types::CanvasState;
use super::types::Error;

fn check_cycle(canvas: &CanvasState, child_id: &NodeId, parent_id: &NodeId) -> bool {
    if child_id == parent_id {
        return true;
    }
    canvas
        .nodes
        .get(parent_id)
        .and_then(|p| p.parent.as_ref())
        .map_or(false, |next_parent| {
            check_cycle(canvas, child_id, next_parent)
        })
}

fn validate_child_exists(canvas: &CanvasState, child_id: &NodeId) -> Result<(), Error> {
    if !canvas.nodes.contains_key(child_id) {
        return Err(Error::NodeNotFound(child_id.clone()));
    }
    Ok(())
}

fn validate_parent_is_subgraph(canvas: &CanvasState, parent_id: &NodeId) -> Result<(), Error> {
    let parent_node = canvas.nodes.get(parent_id);
    let is_subgraph = parent_node.map_or(false, |n| n.kind == NodeKind::Subgraph);
    if !is_subgraph {
        return Err(Error::InvalidNodeType);
    }
    Ok(())
}

fn validate_no_cycle(
    canvas: &CanvasState,
    child_id: &NodeId,
    parent_id: &NodeId,
) -> Result<(), Error> {
    if check_cycle(canvas, child_id, parent_id) {
        return Err(Error::CircularDependency);
    }
    Ok(())
}

fn update_node_parent(
    canvas: &mut CanvasState,
    child_id: NodeId,
    parent_id: NodeId,
) -> Result<(), Error> {
    let updated_node = canvas
        .nodes
        .get(&child_id)
        .cloned()
        .map(|n| crate::models::document::Node {
            parent: Some(parent_id),
            ..n
        })
        .ok_or_else(|| Error::NodeNotFound(child_id.clone()))?;

    canvas.nodes = canvas.nodes.update(child_id, updated_node);
    Ok(())
}

/// Sets the parent of a node to a container node, checking for acyclic properties.
/// For drag-and-drop into a subgraph, the parent must be a NodeKind::Subgraph.
///
/// # Errors
/// Returns `Error::NodeNotFound` if child or parent don't exist.
/// Returns `Error::InvalidNodeType` if parent is not a Subgraph container.
/// Returns `Error::CircularDependency` if assigning the parent creates a cycle.
pub fn set_node_parent(
    child_id: NodeId,
    parent_id: NodeId,
    canvas: &mut CanvasState,
) -> Result<(), Error> {
    // P1: Child must exist
    validate_child_exists(canvas, &child_id)?;
    // P2: Parent must exist
    if !canvas.nodes.contains_key(&parent_id) {
        return Err(Error::NodeNotFound(parent_id));
    }

    // P3: Parent must be a Subgraph container (for drag-and-drop reparenting)
    validate_parent_is_subgraph(canvas, &parent_id)?;

    // P4: Check for cycle
    validate_no_cycle(canvas, &child_id, &parent_id)?;

    update_node_parent(canvas, child_id, parent_id)
}

/// Removes the parent reference from a node, effectively moving it to root level.
/// This is used when dragging a node out of a subgraph to the canvas root.
///
/// # Errors
/// Returns `Error::NodeNotFound` if the child doesn't exist.
pub fn unparent_node(child_id: NodeId, canvas: &mut CanvasState) -> Result<(), Error> {
    // P1: Child must exist
    if !canvas.nodes.contains_key(&child_id) {
        return Err(Error::NodeNotFound(child_id));
    }

    let updated_node = canvas
        .nodes
        .get(&child_id)
        .cloned()
        .map(|n| crate::models::document::Node { parent: None, ..n })
        .ok_or_else(|| Error::NodeNotFound(child_id.clone()))?;

    canvas.nodes = canvas.nodes.update(child_id, updated_node);
    Ok(())
}
