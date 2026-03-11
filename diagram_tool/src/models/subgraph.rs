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
    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),
    #[error("Circular dependency detected")]
    CircularDependency,
    #[error("Invalid transform scale")]
    InvalidTransform,
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

#[cfg(test)]
#[path = "subgraph_tests.rs"]
mod tests;
