//! Subgraph event handling and lifecycle management (bounds, z-index, add/remove nodes).
//!
//! Enforces Big 6 Functional Rust constraints (Data->Calc->Actions, Zero Mutability).

pub mod types;

pub use types::{DiagramState, Error, Rect};

use crate::models::document::{Node, NodeId, NodeKind};
use im::HashMap;

/// Helper to get a node, returning `Error::NodeNotFound` if missing.
fn get_node<'a>(state: &'a DiagramState, id: &NodeId) -> Result<&'a Node, Error> {
    state
        .get_node(id)
        .ok_or_else(|| Error::NodeNotFound(id.clone()))
}

/// Helper to get a node and ensure it's a subgraph
fn get_subgraph<'a>(state: &'a DiagramState, id: &NodeId) -> Result<&'a Node, Error> {
    let node = get_node(state, id)?;
    if node.kind != NodeKind::Subgraph {
        return Err(Error::NodeNotFound(id.clone()));
    }
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

    let (min_x, min_y, max_x, max_y, has_children) = collect_child_bounds(subgraph_id, state);

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

/// Initial bounds accumulator state
const fn initial_bounds() -> (f64, f64, f64, f64, bool) {
    (
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
        false,
    )
}

/// Extract bounding coords from a node
fn node_bounds(node: &Node) -> (f64, f64, f64, f64) {
    (
        node.x.0,
        node.y.0,
        node.x.0 + node.width.0,
        node.y.0 + node.height.0,
    )
}

/// Collects bounding box coordinates from all direct children of a subgraph.
fn collect_child_bounds(subgraph_id: &NodeId, state: &DiagramState) -> (f64, f64, f64, f64, bool) {
    state
        .nodes
        .values()
        .filter(|n| n.parent.as_ref() == Some(subgraph_id))
        .map(node_bounds)
        .fold(initial_bounds(), |(mx, my, m_x, m_y, _), (x, y, rx, ry)| {
            (mx.min(x), my.min(y), m_x.max(rx), m_y.max(ry), true)
        })
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
    let base_z = get_subgraph(state, subgraph_id)?.z_index;
    let new_nodes = state
        .nodes
        .iter()
        .filter(|(_, node)| node.parent.as_ref() == Some(subgraph_id))
        .fold(state.nodes.clone(), |nodes, (id, n)| {
            let mut updated = n.clone();
            updated.z_index = base_z + 1;
            nodes.update(id.clone(), updated)
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

    let mut visited = std::collections::HashSet::new();
    visited.insert(subgraph_id.clone());

    let mut current = state.nodes.get(subgraph_id).and_then(|n| n.parent.clone());
    while let Some(parent_id) = current {
        if &parent_id == child_id || !visited.insert(parent_id.clone()) {
            return Err(Error::CycleDetected(child_id.clone(), subgraph_id.clone()));
        }
        current = state.nodes.get(&parent_id).and_then(|n| n.parent.clone());
    }
    Ok(())
}

/// Helper: Update a node's parent reference in the state.
fn update_node_parent(
    nodes: HashMap<NodeId, Node>,
    node_id: &NodeId,
    new_parent: Option<NodeId>,
) -> Result<HashMap<NodeId, Node>, Error> {
    nodes
        .get(node_id)
        .map(|node| {
            let mut updated = node.clone();
            updated.parent = new_parent;
            nodes.update(node_id.clone(), updated)
        })
        .ok_or_else(|| Error::NodeNotFound(node_id.clone()))
}

/// Helper: Apply bounds to a subgraph node.
fn apply_bounds_to_subgraph(
    nodes: HashMap<NodeId, Node>,
    subgraph_id: &NodeId,
    bounds: Rect,
) -> Result<HashMap<NodeId, Node>, Error> {
    nodes
        .get(subgraph_id)
        .map(|node| {
            let mut updated = node.clone();
            updated.x = bounds.x;
            updated.y = bounds.y;
            updated.width = bounds.width;
            updated.height = bounds.height;
            nodes.update(subgraph_id.clone(), updated)
        })
        .ok_or_else(|| Error::NodeNotFound(subgraph_id.clone()))
}

/// SUB-021: Adds a node to a subgraph, updating parent reference and recalculating bounds.
pub fn add_node_to_subgraph(
    child_id: &NodeId,
    subgraph_id: &NodeId,
    state: &mut DiagramState,
) -> Result<(), Error> {
    let _ = get_subgraph(state, subgraph_id)?;
    let _ = get_node(state, child_id)?;
    detect_cycle(child_id, subgraph_id, state)?;
    let nodes = update_node_parent(state.nodes.clone(), child_id, Some(subgraph_id.clone()))?;
    state.nodes = nodes;
    let bounds = calculate_subgraph_bounds(subgraph_id, state)?;
    let nodes = apply_bounds_to_subgraph(state.nodes.clone(), subgraph_id, bounds)?;
    state.nodes = nodes;
    update_z_index_ordering(subgraph_id, state)
}

/// SUB-022: Removes a node from a subgraph, clearing its parent reference.
///
/// # Errors
///
/// Returns `Error::NodeNotFound` if the child does not exist.
pub fn remove_node_from_subgraph(child_id: &NodeId, state: &mut DiagramState) -> Result<(), Error> {
    let node = get_node(state, child_id)?;
    let old_parent = node.parent.clone();

    // Clear the parent reference
    let nodes = update_node_parent(state.nodes.clone(), child_id, None)?;
    state.nodes = nodes;

    // Recalculate bounds of the former parent if it exists
    if let Some(subgraph_id) = old_parent.filter(|id| state.has_node(id)) {
        let bounds = calculate_subgraph_bounds(&subgraph_id, state)?;
        let nodes = apply_bounds_to_subgraph(state.nodes.clone(), &subgraph_id, bounds)?;
        state.nodes = nodes;
    }

    Ok(())
}

/// SUB-023: Batch add nodes to a subgraph, updating parent references and recalculating bounds exactly once.
pub fn batch_add_nodes_to_subgraph(
    child_ids: &[NodeId],
    subgraph_id: &NodeId,
    state: &mut DiagramState,
) -> Result<(), Error> {
    if child_ids.is_empty() {
        return Ok(());
    }
    let _ = get_subgraph(state, subgraph_id)?;
    child_ids.iter().try_for_each(|cid| {
        let _ = get_node(state, cid)?;
        detect_cycle(cid, subgraph_id, state)
    })?;
    let nodes = child_ids.iter().try_fold(state.nodes.clone(), |acc, cid| {
        update_node_parent(acc, cid, Some(subgraph_id.clone()))
    })?;
    state.nodes = nodes;
    let bounds = calculate_subgraph_bounds(subgraph_id, state)?;
    state.nodes = apply_bounds_to_subgraph(state.nodes.clone(), subgraph_id, bounds)?;
    update_z_index_ordering(subgraph_id, state)
}

/// SUB-024: Remove all nodes from a subgraph, leaving an empty container.
pub fn remove_all_nodes_from_subgraph(
    subgraph_id: &NodeId,
    state: &mut DiagramState,
) -> Result<(), Error> {
    let _ = get_subgraph(state, subgraph_id)?;
    let children: Vec<NodeId> = state
        .nodes
        .iter()
        .filter(|(_, node)| node.parent.as_ref() == Some(subgraph_id))
        .map(|(id, _)| id.clone())
        .collect();
    let nodes = children
        .iter()
        .try_fold(state.nodes.clone(), |acc, child_id| {
            update_node_parent(acc, child_id, None)
        })?;
    state.nodes = nodes;
    let bounds = calculate_subgraph_bounds(subgraph_id, state)?;
    state.nodes = apply_bounds_to_subgraph(state.nodes.clone(), subgraph_id, bounds)?;
    Ok(())
}
