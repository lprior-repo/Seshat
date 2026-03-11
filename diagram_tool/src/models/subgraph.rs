#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::document::{DocumentData, Node, NodeId, NodeKind, OrderedFloat, Point};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl BoundingBox {
    #[must_use]
    pub const fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("Invalid padding")]
    InvalidPadding,
    #[error("Empty selection")]
    EmptySelection,
    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),
    #[error("Circular dependency detected")]
    CircularDependency,
    #[error("Node locked: {0}")]
    NodeLocked(NodeId),
    #[error("Invalid transform scale")]
    InvalidTransform,
    #[error("Invalid node type")]
    InvalidNodeType,
    #[error("Invariant violation")]
    InvariantViolation,
}

pub type CanvasState = DocumentData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Padding {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositiveScale(OrderedFloat);

impl PositiveScale {
    /// Creates a new `PositiveScale` ensuring the value is strictly greater than zero.
    ///
    /// # Errors
    /// Returns `Error::InvalidTransform` if the value is zero or negative.
    pub fn try_new(value: OrderedFloat) -> Result<Self, Error> {
        if value.0 > 0.0 {
            Ok(Self(value))
        } else {
            Err(Error::InvalidTransform)
        }
    }

    #[must_use]
    pub const fn value(&self) -> f64 {
        self.0 .0
    }
}

/// Applies a viewport transform to a subgraph, scaling its position and dimensions.
///
/// # Errors
/// Cannot fail inherently as scale validity is enforced by `PositiveScale`, but returns Result for contract alignment.
pub fn apply_viewport_transform(subgraph: &Node, scale: PositiveScale) -> Result<Node, Error> {
    Ok(Node {
        x: OrderedFloat::new_unchecked(subgraph.x.0 * scale.value()),
        y: OrderedFloat::new_unchecked(subgraph.y.0 * scale.value()),
        width: OrderedFloat::new_unchecked(subgraph.width.0 * scale.value()),
        height: OrderedFloat::new_unchecked(subgraph.height.0 * scale.value()),
        ..subgraph.clone()
    })
}

/// Calculates the bounding box that encapsulates all given child nodes plus the specified padding.
///
/// # Errors
/// Returns `Error::InvariantViolation` if calculating bounds fails or no children exist.
pub fn calculate_container_bounds(
    children: &[Node],
    padding: Padding,
) -> Result<BoundingBox, Error> {
    if children.is_empty() {
        return Ok(BoundingBox::new(0.0, 0.0, 0.0, 0.0));
    }

    let min_x = children.iter().map(|n| n.x.0).fold(f64::INFINITY, f64::min);
    let min_y = children.iter().map(|n| n.y.0).fold(f64::INFINITY, f64::min);
    let max_x = children
        .iter()
        .map(|n| n.x.0 + n.width.0)
        .fold(f64::NEG_INFINITY, f64::max);
    let max_y = children
        .iter()
        .map(|n| n.y.0 + n.height.0)
        .fold(f64::NEG_INFINITY, f64::max);

    let bounds = BoundingBox::new(
        min_x - f64::from(padding.left),
        min_y - f64::from(padding.top),
        max_x + f64::from(padding.right),
        max_y + f64::from(padding.bottom),
    );

    // Q1 Postcondition validation - ensure container bounds encapsulate all children + padding
    let valid = children.iter().all(|n| {
        bounds.min_x <= n.x.0 - f64::from(padding.left)
            && bounds.min_y <= n.y.0 - f64::from(padding.top)
            && bounds.max_x >= n.x.0 + n.width.0 + f64::from(padding.right)
            && bounds.max_y >= n.y.0 + n.height.0 + f64::from(padding.bottom)
    });

    if !valid {
        return Err(Error::InvariantViolation);
    }

    Ok(bounds)
}

/// Creates a new empty subgraph container node with minimum dimensions.
///
/// # Errors
/// Returns error if invariants are violated.
pub fn create_empty_subgraph(_id: NodeId, position: Point) -> Result<Node, Error> {
    let node = Node {
        kind: NodeKind::Subgraph,
        icon: String::new(),
        label: String::new(),
        x: position.x,
        y: position.y,
        width: OrderedFloat::new_unchecked(100.0), // minimum width
        height: OrderedFloat::new_unchecked(60.0), // minimum height
        font_size: None,
        font_weight: None,
        locked: false,
        parent: None,
        dag_rank: None,
        tags: im::vector![],
        metadata: im::HashMap::new(),
        z_index: 0,
        style: None,
        collapsed: None,
    };

    // Q2 Postcondition validation
    if node.width.0 < 100.0 || node.height.0 < 60.0 {
        return Err(Error::InvariantViolation);
    }

    Ok(node)
}

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

/// Sets the parent of a node to a container node, checking for acyclic properties.
///
/// # Errors
/// Returns `Error::NodeNotFound` if child or parent don't exist.
/// Returns `Error::CircularDependency` if assigning the parent creates a cycle.
pub fn set_node_parent(
    child_id: NodeId,
    parent_id: NodeId,
    canvas: &mut CanvasState,
) -> Result<(), Error> {
    if !canvas.nodes.contains_key(&child_id) {
        return Err(Error::NodeNotFound(child_id));
    }
    if !canvas.nodes.contains_key(&parent_id) {
        return Err(Error::NodeNotFound(parent_id));
    }

    if check_cycle(canvas, &child_id, &parent_id) {
        return Err(Error::CircularDependency);
    }

    let updated_node = canvas
        .nodes
        .get(&child_id)
        .cloned()
        .map(|n| Node {
            parent: Some(parent_id),
            ..n
        })
        .ok_or_else(|| Error::NodeNotFound(child_id.clone()))?;

    canvas.nodes = canvas.nodes.update(child_id, updated_node);
    Ok(())
}

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
    let children_result: Result<Vec<Node>, Error> = child_ids
        .iter()
        .map(|cid| {
            canvas
                .nodes
                .get(cid)
                .cloned()
                .ok_or_else(|| Error::NodeNotFound(cid.clone()))
        })
        .collect();
    let children = children_result?;

    let bounds = calculate_container_bounds(
        &children,
        Padding {
            top: 20,
            right: 20,
            bottom: 20,
            left: 20,
        },
    )?;

    let min_width = 100.0;
    let min_height = 60.0;

    let subgraph = create_empty_subgraph(
        id.clone(),
        Point {
            x: OrderedFloat::new_unchecked(bounds.min_x),
            y: OrderedFloat::new_unchecked(bounds.min_y),
        },
    )
    .map(|n| Node {
        width: OrderedFloat::new_unchecked(f64::max(min_width, bounds.max_x - bounds.min_x)),
        height: OrderedFloat::new_unchecked(f64::max(min_height, bounds.max_y - bounds.min_y)),
        ..n
    })?;

    canvas.nodes = canvas.nodes.update(id.clone(), subgraph.clone());

    child_ids
        .iter()
        .try_for_each(|child_id| set_node_parent(child_id.clone(), id.clone(), canvas))?;

    // Q3 Postcondition validation
    let all_reparented = child_ids.iter().all(|cid| {
        canvas
            .nodes
            .get(cid)
            .and_then(|n| n.parent.as_ref())
            .map_or(false, |pid| pid == &id)
    });

    if !all_reparented {
        return Err(Error::InvariantViolation);
    }

    Ok(subgraph)
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GroupTransformError {
    #[error("Selection cannot be empty")]
    EmptySelection,
    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),
    #[error("Node locked: {0}")]
    NodeLocked(NodeId),
    #[error("Scale out of bounds")]
    OutOfBounds,
}

pub type Subgraph = CanvasState;

const MIN_DIMENSION: f64 = 1.0;
const MAX_COORDINATE: f64 = 1_000_000.0;

/// Scales a group of selected nodes relative to an anchor point.
///
/// # Errors
/// Returns `GroupTransformError` if selection is empty, a node is not found,
/// a node is locked, or if the resulting scale exceeds bounds.
pub fn scale_group(
    subgraph: &mut Subgraph,
    selection: &[NodeId],
    scale_factor: PositiveScale,
    anchor: Point,
) -> Result<(), GroupTransformError> {
    if selection.is_empty() {
        return Err(GroupTransformError::EmptySelection);
    }

    let scale = scale_factor.value();

    let updates: Result<Vec<(NodeId, Node)>, GroupTransformError> = selection
        .iter()
        .map(|id| {
            let node = subgraph
                .nodes
                .get(id)
                .ok_or_else(|| GroupTransformError::NodeNotFound(id.clone()))?;

            if node.locked {
                return Err(GroupTransformError::NodeLocked(id.clone()));
            }

            let new_x = anchor.x.0 + (node.x.0 - anchor.x.0) * scale;
            let new_y = anchor.y.0 + (node.y.0 - anchor.y.0) * scale;
            let new_w = (node.width.0 * scale).max(MIN_DIMENSION);
            let new_h = (node.height.0 * scale).max(MIN_DIMENSION);

            if !new_x.is_finite() || !new_y.is_finite() || !new_w.is_finite() || !new_h.is_finite()
            {
                return Err(GroupTransformError::OutOfBounds);
            }

            if new_x.abs() > MAX_COORDINATE
                || new_y.abs() > MAX_COORDINATE
                || new_w > MAX_COORDINATE
                || new_h > MAX_COORDINATE
            {
                return Err(GroupTransformError::OutOfBounds);
            }

            let updated_node = Node {
                x: OrderedFloat::new_unchecked(new_x),
                y: OrderedFloat::new_unchecked(new_y),
                width: OrderedFloat::new_unchecked(new_w),
                height: OrderedFloat::new_unchecked(new_h),
                ..node.clone()
            };

            Ok((id.clone(), updated_node))
        })
        .collect();

    let resolved_updates = updates?;

    subgraph.nodes = resolved_updates
        .into_iter()
        .fold(subgraph.nodes.clone(), |nodes, (id, node)| {
            nodes.update(id, node)
        });

    Ok(())
}

/// Modifiers for selection actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionModifiers {
    pub ctrl: bool,
}

/// The result of a selection evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionResult {
    NodeSelected(NodeId),
}

/// Evaluates a selection click, considering modifiers like Ctrl to bypass containers.
///
/// # Errors
/// Returns `Error::EmptySelection` if no node was hit.
#[allow(clippy::needless_pass_by_value)]
pub fn evaluate_selection(
    canvas: &CanvasState,
    click_pos: Point,
    modifiers: SelectionModifiers,
) -> Result<SelectionResult, Error> {
    use itertools::Itertools;

    let px = click_pos.x.0;
    let py = click_pos.y.0;

    let hit = canvas
        .nodes
        .iter()
        .sorted_by_key(|(_, n)| -n.z_index)
        .find(|(_id, n)| {
            let nx = n.x.0;
            let ny = n.y.0;
            let nw = n.width.0;
            let nh = n.height.0;

            let intersects = px >= nx && px <= nx + nw && py >= ny && py <= ny + nh;

            if !intersects {
                return false;
            }

            // If we have ctrl modifier, bypass containers
            if modifiers.ctrl && n.kind == NodeKind::Subgraph {
                return false;
            }

            // If a node is in a collapsed parent, it shouldn't be hit-testable
            if let Some(parent_id) = &n.parent {
                if let Some(parent) = canvas.nodes.get(parent_id) {
                    if parent.collapsed.unwrap_or(false) {
                        return false;
                    }
                }
            }

            true
        });

    if let Some((id, _)) = hit {
        Ok(SelectionResult::NodeSelected(id.clone()))
    } else {
        Err(Error::EmptySelection)
    }
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
    if child_ids.is_empty() {
        return Err(Error::EmptySelection);
    }

    for id in child_ids {
        let node = canvas
            .nodes
            .get(id)
            .ok_or_else(|| Error::NodeNotFound(id.clone()))?;
        if node.locked {
            return Err(Error::NodeLocked(id.clone()));
        }
    }

    // Capture child bounds before operation for invariant Q1 check
    let child_bounds: Vec<_> = child_ids
        .iter()
        .filter_map(|id| canvas.nodes.get(id))
        .map(|n| (n.x.0, n.y.0, n.width.0, n.height.0))
        .collect();

    let subgraph = create_subgraph_from_nodes(group_id.clone(), child_ids, canvas)?;

    // Q1 Invariant check
    for (cx, cy, cw, ch) in child_bounds {
        if subgraph.x.0 > cx
            || subgraph.y.0 > cy
            || (subgraph.x.0 + subgraph.width.0) < (cx + cw)
            || (subgraph.y.0 + subgraph.height.0) < (cy + ch)
        {
            return Err(Error::InvariantViolation);
        }
    }

    // Q2 Invariant check
    for id in child_ids {
        let parent = canvas.nodes.get(id).and_then(|n| n.parent.as_ref());
        if parent != Some(&group_id) {
            return Err(Error::InvariantViolation);
        }
    }

    Ok(subgraph)
}

/// Removes a container and reparents its children to its parent.
///
/// # Errors
/// Returns errors based on contract (`NodeNotFound`, `InvalidNodeType`, `NodeLocked`, etc.).
#[allow(clippy::needless_pass_by_value)]
pub fn ungroup_nodes(canvas: &mut CanvasState, group_id: NodeId) -> Result<Vec<NodeId>, Error> {
    let group = canvas
        .nodes
        .get(&group_id)
        .ok_or_else(|| Error::NodeNotFound(group_id.clone()))?;

    if group.kind != NodeKind::Subgraph {
        return Err(Error::InvalidNodeType);
    }

    if group.locked {
        return Err(Error::NodeLocked(group_id.clone()));
    }

    let group_parent = group.parent.clone();

    // Find all children
    let children: Vec<NodeId> = canvas
        .nodes
        .iter()
        .filter(|(_, n)| n.parent.as_ref() == Some(&group_id))
        .map(|(id, _)| id.clone())
        .collect();

    // Remove group
    let _removed_group = canvas
        .nodes
        .remove(&group_id)
        .ok_or_else(|| Error::NodeNotFound(group_id.clone()))?;

    // Reparent children to group's parent
    for child_id in &children {
        if let Some(child) = canvas.nodes.get(child_id) {
            let updated_child = Node {
                parent: group_parent.clone(),
                ..child.clone()
            };
            canvas.nodes = canvas.nodes.update(child_id.clone(), updated_child);
        }
    }

    // Q3 Validation
    for child_id in &children {
        if !canvas.nodes.contains_key(child_id) {
            return Err(Error::InvariantViolation);
        }
    }

    Ok(children)
}

/// Toggles the collapsed state of a container.
///
/// # Errors
/// Returns errors based on contract.
#[allow(clippy::needless_pass_by_value)]
pub fn toggle_collapse(canvas: &mut CanvasState, group_id: NodeId) -> Result<(), Error> {
    let group = canvas
        .nodes
        .get(&group_id)
        .ok_or_else(|| Error::NodeNotFound(group_id.clone()))?;

    if group.kind != NodeKind::Subgraph {
        return Err(Error::InvalidNodeType);
    }

    let is_collapsed = group.collapsed.unwrap_or(false);

    let updated_group = Node {
        collapsed: Some(!is_collapsed),
        ..group.clone()
    };

    canvas.nodes = canvas.nodes.update(group_id, updated_group);

    Ok(())
}

#[cfg(test)]
#[path = "subgraph_tests.rs"]
mod tests;
