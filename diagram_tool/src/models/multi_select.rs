use crate::models::document::{DiagramDocument, Node, NodeId, OrderedFloat};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Error {
    #[error("Empty selection")]
    EmptySelection,
    #[error("Item is locked")]
    ItemLocked,
    #[error("Invalid hierarchy")]
    InvalidHierarchy,
    #[error("Postcondition violated")]
    PostconditionViolated,
    #[error("Node not found")]
    NodeNotFound,
    #[error("Invalid scale factor: must be positive and finite")]
    InvalidScale,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonEmptyVec<T>(Vec<T>);

impl<T> NonEmptyVec<T> {
    pub fn try_from(vec: Vec<T>) -> Result<Self, Error> {
        if vec.is_empty() {
            Err(Error::EmptySelection)
        } else {
            Ok(Self(vec))
        }
    }

    #[must_use]
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }

    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }
}

impl<T> IntoIterator for NonEmptyVec<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2D {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardData {
    pub nodes: Vec<Node>,
}

/// Helper: Convert f64 to `OrderedFloat` safely
fn to_ordered(value: f64) -> Result<OrderedFloat, Error> {
    OrderedFloat::new(value).map_err(|_| Error::InvalidScale)
}

/// Helper: Compute bounding box of a selection
fn compute_bounding_box(
    doc: &DiagramDocument,
    selection: &[NodeId],
) -> Result<(f64, f64, f64, f64), Error> {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for id in selection {
        let node = doc.document.nodes.get(id).ok_or(Error::NodeNotFound)?;
        let nx = node.x.0;
        let ny = node.y.0;
        let nw = node.width.0;
        let nh = node.height.0;

        min_x = min_x.min(nx);
        min_y = min_y.min(ny);
        max_x = max_x.max(nx + nw);
        max_y = max_y.max(ny + nh);
    }

    Ok((min_x, min_y, max_x, max_y))
}

fn check_locked_items(doc: &DiagramDocument, selection: &[NodeId]) -> Result<(), Error> {
    for id in selection {
        let node = doc.document.nodes.get(id).ok_or(Error::NodeNotFound)?;
        if node.lock_state.is_locked() {
            return Err(Error::ItemLocked);
        }
    }
    Ok(())
}

fn check_invalid_hierarchy(doc: &DiagramDocument, selection: &[NodeId]) -> Result<(), Error> {
    let selected_set: HashSet<_> = selection.iter().collect();

    for id in selection {
        let mut current_id = id;
        loop {
            let node = doc
                .document
                .nodes
                .get(current_id)
                .ok_or(Error::NodeNotFound)?;
            if let Some(parent_id) = &node.parent {
                if selected_set.contains(parent_id) {
                    return Err(Error::InvalidHierarchy);
                }
                current_id = parent_id;
            } else {
                break;
            }
        }
    }
    Ok(())
}

/// Helper: Apply scale factor to a node relative to a centroid
fn scale_node(node: &mut Node, centroid: Vector2D, scale_factor: f64) -> Result<(), Error> {
    let rel_x = node.x.0 - centroid.x;
    let rel_y = node.y.0 - centroid.y;

    node.x = to_ordered(rel_x.mul_add(scale_factor, centroid.x))?;
    node.y = to_ordered(rel_y.mul_add(scale_factor, centroid.y))?;
    node.width = to_ordered(node.width.0 * scale_factor)?;
    node.height = to_ordered(node.height.0 * scale_factor)?;

    Ok(())
}

/// Helper: Generate unique node ID
fn generate_unique_id(base_label: &str, doc: &DiagramDocument) -> NodeId {
    let mut idx = 1;
    loop {
        let new_id = NodeId::new(format!("{base_label}_{idx}"));
        if !doc.document.nodes.contains_key(&new_id) {
            return new_id;
        }
        idx += 1;
    }
}

pub fn move_selection(
    doc: &mut DiagramDocument,
    selection: NonEmptyVec<NodeId>,
    delta: Vector2D,
) -> Result<(), Error> {
    let selection_slice = selection.as_slice();
    check_locked_items(doc, selection_slice)?;
    check_invalid_hierarchy(doc, selection_slice)?;

    let delta_x = to_ordered(delta.x)?;
    let delta_y = to_ordered(delta.y)?;

    for id in selection_slice {
        if let Some(node) = doc.document.nodes.get_mut(id) {
            node.x = node.x + delta_x;
            node.y = node.y + delta_y;
        } else {
            return Err(Error::NodeNotFound);
        }
    }

    Ok(())
}

/// Helper: Apply resize scaling to a single node
fn apply_resize_scale(
    node: &mut Node,
    min_x: f64,
    min_y: f64,
    new_x: f64,
    new_y: f64,
    scale_x: f64,
    scale_y: f64,
) -> Result<(), Error> {
    let rel_x = node.x.0 - min_x;
    let rel_y = node.y.0 - min_y;

    node.x = to_ordered(rel_x.mul_add(scale_x, new_x))?;
    node.y = to_ordered(rel_y.mul_add(scale_y, new_y))?;
    node.width = to_ordered(node.width.0 * scale_x)?;
    node.height = to_ordered(node.height.0 * scale_y)?;

    Ok(())
}

/// Helper: Extract resize scale factors from old and new bounds
fn compute_resize_scales(
    doc: &DiagramDocument,
    selection_slice: &[NodeId],
    new_bounds: Rect,
) -> Result<(f64, f64, f64, f64, f64, f64), Error> {
    let (min_x, min_y, max_x, max_y) = compute_bounding_box(doc, selection_slice)?;
    let old_width = max_x - min_x;
    let old_height = max_y - min_y;

    if old_width == 0.0 || old_height == 0.0 {
        return Err(Error::InvalidScale);
    }

    let scale_x = new_bounds.width / old_width;
    let scale_y = new_bounds.height / old_height;

    // Validate new bounds coordinates
    let _ = to_ordered(new_bounds.x)?;
    let _ = to_ordered(new_bounds.y)?;

    Ok((min_x, min_y, new_bounds.x, new_bounds.y, scale_x, scale_y))
}

pub fn resize_selection(
    doc: &mut DiagramDocument,
    selection: NonEmptyVec<NodeId>,
    new_bounds: Rect,
) -> Result<(), Error> {
    let selection_slice = selection.as_slice();
    check_locked_items(doc, selection_slice)?;
    check_invalid_hierarchy(doc, selection_slice)?;

    let scales = compute_resize_scales(doc, selection_slice, new_bounds)?;

    let (min_x, min_y, new_x, new_y, scale_x, scale_y) = scales;

    for id in selection_slice {
        if let Some(node) = doc.document.nodes.get_mut(id) {
            apply_resize_scale(node, min_x, min_y, new_x, new_y, scale_x, scale_y)?;
        }
    }

    Ok(())
}

/// Helper: Verify all nodes removed from document
fn verify_all_removed(doc: &DiagramDocument, selection_slice: &[NodeId]) -> Result<(), Error> {
    for id in selection_slice {
        if doc.document.nodes.contains_key(id) {
            return Err(Error::PostconditionViolated);
        }
    }
    Ok(())
}

/// Helper: Remove a node from document and selection
fn remove_node_from_doc(doc: &mut DiagramDocument, id: &NodeId) {
    doc.document.nodes.remove(id);
    doc.editor_state.selected_items.remove(&id.to_string());
}

pub fn delete_selection(
    doc: &mut DiagramDocument,
    selection: NonEmptyVec<NodeId>,
) -> Result<(), Error> {
    let selection_slice = selection.as_slice();
    check_locked_items(doc, selection_slice)?;

    // Verify all nodes exist
    for id in selection_slice {
        if !doc.document.nodes.contains_key(id) {
            return Err(Error::NodeNotFound);
        }
    }

    // Remove all nodes
    for id in selection_slice {
        remove_node_from_doc(doc, id);
    }

    // Postcondition: verify all removed
    verify_all_removed(doc, selection_slice)
}

pub fn copy_selection(
    doc: &DiagramDocument,
    selection: NonEmptyVec<NodeId>,
) -> Result<ClipboardData, Error> {
    let selection_slice = selection.as_slice();
    let mut copied_nodes = Vec::new();

    for id in selection_slice {
        let node = doc.document.nodes.get(id).ok_or(Error::NodeNotFound)?;
        copied_nodes.push(node.clone());
    }

    Ok(ClipboardData {
        nodes: copied_nodes,
    })
}

/// Helper: Add node to document and selection
fn add_node_to_doc(doc: &mut DiagramDocument, new_node: Node, new_id: NodeId) -> NodeId {
    doc.document.nodes.insert(new_id.clone(), new_node);
    doc.editor_state.selected_items.insert(new_id.to_string());
    new_id
}

pub fn paste_selection(
    doc: &mut DiagramDocument,
    clipboard: &ClipboardData,
    offset: Vector2D,
) -> Result<Vec<NodeId>, Error> {
    doc.editor_state.selected_items.clear();

    let offset_x = to_ordered(offset.x)?;
    let offset_y = to_ordered(offset.y)?;

    let mut new_ids = Vec::new();
    for node in &clipboard.nodes {
        let mut new_node = node.clone();
        new_node.x = new_node.x + offset_x;
        new_node.y = new_node.y + offset_y;
        let new_id = generate_unique_id(&node.label, doc);
        let id = add_node_to_doc(doc, new_node, new_id);
        new_ids.push(id);
    }

    Ok(new_ids)
}

/// Compute the centroid (geometric center) of a multi-selection's bounding box
pub fn compute_selection_centroid(
    doc: &DiagramDocument,
    selection: &[NodeId],
) -> Result<Vector2D, Error> {
    let (min_x, min_y, max_x, max_y) = compute_bounding_box(doc, selection)?;

    Ok(Vector2D {
        x: f64::midpoint(min_x, max_x),
        y: f64::midpoint(min_y, max_y),
    })
}

/// Helper: Validate scale factor
fn validate_scale_factor(scale_factor: f64) -> Result<(), Error> {
    if scale_factor > 0.0 && scale_factor.is_finite() {
        Ok(())
    } else {
        Err(Error::InvalidScale)
    }
}

/// Helper: Apply scaling to all nodes in selection
fn apply_scale_to_selection(
    doc: &mut DiagramDocument,
    selection_slice: &[NodeId],
    centroid: Vector2D,
    scale_factor: f64,
) -> Result<(), Error> {
    for id in selection_slice {
        if let Some(node) = doc.document.nodes.get_mut(id) {
            scale_node(node, centroid, scale_factor)?;
        }
    }
    Ok(())
}

/// Scale a multi-selection around its common centroid
pub fn scale_selection_around_centroid(
    doc: &mut DiagramDocument,
    selection: NonEmptyVec<NodeId>,
    scale_factor: f64,
) -> Result<(), Error> {
    validate_scale_factor(scale_factor)?;

    let selection_slice = selection.as_slice();
    check_locked_items(doc, selection_slice)?;
    check_invalid_hierarchy(doc, selection_slice)?;

    let centroid = compute_selection_centroid(doc, selection_slice)?;
    apply_scale_to_selection(doc, selection_slice, centroid, scale_factor)?;

    Ok(())
}
