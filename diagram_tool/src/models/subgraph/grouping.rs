//! Node grouping operations
//!
//! Operations for creating subgraphs from nodes and ungrouping.

use crate::models::document::{Node, NodeId, NodeKind, OrderedFloat, Point};

use super::types::CanvasState;
use super::types::{
    calculate_container_bounds, create_empty_subgraph, BoundingBox, Error, Padding,
};

type GroupTransformError = super::transform::GroupTransformError;

/// Creates a new subgraph encapsulating pre-selected nodes.
///
/// # Errors
/// Returns `Error::NodeNotFound` if any selected node doesn't exist.
/// Returns `Error::CircularDependency` if it creates a cycle.
/// Returns `Error::InvariantViolation` if reparenting fails to persist.
pub fn create_subgraph_from_nodes(
    id: NodeId,
    child_ids: &[NodeId],
    canvas: &mut CanvasState,
) -> Result<Node, Error> {
    let children = collect_children(canvas, child_ids)?;
    let bounds = calculate_bounds_with_padding(&children)?;
    let subgraph = create_subgraph_node(&id, &bounds)?;

    canvas.nodes = canvas.nodes.update(id.clone(), subgraph.clone());

    reparent_children_to_subgraph(child_ids, &id, canvas)?;
    validate_reparenting(child_ids, &id, canvas)?;

    Ok(subgraph)
}

fn collect_children(canvas: &CanvasState, child_ids: &[NodeId]) -> Result<Vec<Node>, Error> {
    child_ids
        .iter()
        .map(|cid| {
            canvas
                .nodes
                .get(cid)
                .cloned()
                .ok_or_else(|| Error::NodeNotFound(cid.clone()))
        })
        .collect()
}

fn calculate_bounds_with_padding(children: &[Node]) -> Result<BoundingBox, Error> {
    calculate_container_bounds(
        children,
        Padding {
            top: 20,
            right: 20,
            bottom: 20,
            left: 20,
        },
    )
}

fn create_subgraph_node(id: &NodeId, bounds: &BoundingBox) -> Result<Node, Error> {
    let min_width = 100.0;
    let min_height = 60.0;

    let position = Point {
        x: OrderedFloat::new(bounds.min_x).map_err(|_| Error::InvariantViolation)?,
        y: OrderedFloat::new(bounds.min_y).map_err(|_| Error::InvariantViolation)?,
    };

    let empty_subgraph = create_empty_subgraph(id.clone(), position)?;

    let width = OrderedFloat::new(f64::max(min_width, bounds.max_x - bounds.min_x))
        .map_err(|_| Error::InvariantViolation)?;
    let height = OrderedFloat::new(f64::max(min_height, bounds.max_y - bounds.min_y))
        .map_err(|_| Error::InvariantViolation)?;

    Ok(Node {
        width,
        height,
        ..empty_subgraph
    })
}

fn reparent_children_to_subgraph(
    child_ids: &[NodeId],
    parent_id: &NodeId,
    canvas: &mut CanvasState,
) -> Result<(), Error> {
    child_ids.iter().try_for_each(|child_id| {
        super::reparenting::set_node_parent(child_id.clone(), parent_id.clone(), canvas)
    })
}

fn validate_reparenting(
    child_ids: &[NodeId],
    parent_id: &NodeId,
    canvas: &CanvasState,
) -> Result<(), Error> {
    let all_reparented = child_ids.iter().all(|cid| {
        canvas
            .nodes
            .get(cid)
            .and_then(|n| n.parent.as_ref())
            .map_or(false, |pid| pid == parent_id)
    });

    if !all_reparented {
        return Err(Error::InvariantViolation);
    }
    Ok(())
}

/// Groups existing nodes into a new container node.
///
/// # Errors
/// Returns errors based on the contract (`EmptySelection`, `NodeNotFound`, `NodeLocked`, etc.).
#[allow(clippy::needless_pass_by_value)]
pub fn group_nodes(
    canvas: &mut CanvasState,
    group_id: NodeId,
    child_ids: &[NodeId],
) -> Result<Node, Error> {
    validate_non_empty_selection(child_ids)?;
    validate_children_unlocked(canvas, child_ids)?;

    let child_bounds = capture_child_bounds(canvas, child_ids);
    let subgraph = create_subgraph_from_nodes(group_id.clone(), child_ids, canvas)?;

    validate_subgraph_contains_children(&subgraph, &child_bounds)?;
    validate_children_reparented(canvas, child_ids, &group_id)?;

    Ok(subgraph)
}

fn validate_non_empty_selection(child_ids: &[NodeId]) -> Result<(), Error> {
    if child_ids.is_empty() {
        return Err(Error::EmptySelection);
    }
    Ok(())
}

fn validate_children_unlocked(canvas: &CanvasState, child_ids: &[NodeId]) -> Result<(), Error> {
    for id in child_ids {
        let node = canvas
            .nodes
            .get(id)
            .ok_or_else(|| Error::NodeNotFound(id.clone()))?;
        if node.locked {
            return Err(Error::NodeLocked(id.clone()));
        }
    }
    Ok(())
}

fn capture_child_bounds(canvas: &CanvasState, child_ids: &[NodeId]) -> Vec<(f64, f64, f64, f64)> {
    child_ids
        .iter()
        .filter_map(|id| canvas.nodes.get(id))
        .map(|n| (n.x.0, n.y.0, n.width.0, n.height.0))
        .collect()
}

fn validate_subgraph_contains_children(
    subgraph: &Node,
    child_bounds: &[(f64, f64, f64, f64)],
) -> Result<(), Error> {
    for (cx, cy, cw, ch) in child_bounds {
        if subgraph.x.0 > *cx
            || subgraph.y.0 > *cy
            || (subgraph.x.0 + subgraph.width.0) < (cx + cw)
            || (subgraph.y.0 + subgraph.height.0) < (cy + ch)
        {
            return Err(Error::InvariantViolation);
        }
    }
    Ok(())
}

fn validate_children_reparented(
    canvas: &CanvasState,
    child_ids: &[NodeId],
    group_id: &NodeId,
) -> Result<(), Error> {
    for id in child_ids {
        let parent = canvas.nodes.get(id).and_then(|n| n.parent.as_ref());
        if parent != Some(group_id) {
            return Err(Error::InvariantViolation);
        }
    }
    Ok(())
}

/// Removes a container and reparents its children to its parent.
///
/// # Errors
/// Returns errors based on contract (`NodeNotFound`, `InvalidNodeType`, `NodeLocked`, etc.).
#[allow(clippy::needless_pass_by_value)]
pub fn ungroup_nodes(canvas: &mut CanvasState, group_id: NodeId) -> Result<Vec<NodeId>, Error> {
    let group = validate_group_exists(canvas, &group_id)?;
    validate_group_is_subgraph(&group)?;
    validate_group_unlocked(&group, &group_id)?;

    let group_parent = group.parent.clone();
    let children = find_group_children(canvas, &group_id);

    remove_group(canvas, &group_id)?;
    reparent_children_to_grandparent(canvas, &children, &group_parent);

    validate_children_exist(canvas, &children)?;

    Ok(children)
}

fn validate_group_exists(canvas: &CanvasState, group_id: &NodeId) -> Result<Node, Error> {
    canvas
        .nodes
        .get(group_id)
        .cloned()
        .ok_or_else(|| Error::NodeNotFound(group_id.clone()))
}

fn validate_group_is_subgraph(group: &Node) -> Result<(), Error> {
    if group.kind != NodeKind::Subgraph {
        return Err(Error::InvalidNodeType);
    }
    Ok(())
}

fn validate_group_unlocked(group: &Node, group_id: &NodeId) -> Result<(), Error> {
    if group.locked {
        return Err(Error::NodeLocked(group_id.clone()));
    }
    Ok(())
}

fn find_group_children(canvas: &CanvasState, group_id: &NodeId) -> Vec<NodeId> {
    canvas
        .nodes
        .iter()
        .filter(|(_, n)| n.parent.as_ref() == Some(group_id))
        .map(|(id, _)| id.clone())
        .collect()
}

fn remove_group(canvas: &mut CanvasState, group_id: &NodeId) -> Result<(), Error> {
    let _removed_group = canvas
        .nodes
        .remove(group_id)
        .ok_or_else(|| Error::NodeNotFound(group_id.clone()))?;
    Ok(())
}

fn reparent_children_to_grandparent(
    canvas: &mut CanvasState,
    children: &[NodeId],
    new_parent: &Option<NodeId>,
) {
    for child_id in children {
        if let Some(child) = canvas.nodes.get(child_id) {
            let updated_child = Node {
                parent: new_parent.clone(),
                ..child.clone()
            };
            canvas.nodes = canvas.nodes.update(child_id.clone(), updated_child);
        }
    }
}

fn validate_children_exist(canvas: &CanvasState, children: &[NodeId]) -> Result<(), Error> {
    for child_id in children {
        if !canvas.nodes.contains_key(child_id) {
            return Err(Error::InvariantViolation);
        }
    }
    Ok(())
}
