use crate::document::{DiagramDocument, NodeId};

use super::helpers::{
    apply_resize_scale, apply_scale_to_selection, check_invalid_hierarchy, check_locked_items,
    compute_bounding_box, compute_resize_scales, remove_node_from_doc, to_ordered,
    validate_scale_factor, verify_all_removed,
};
use super::types::{Error, NonEmptyVec, Rect, Vector2D};

/// Moves the selection by the given delta.
///
/// # Errors
///
/// Returns `Error` if items are locked, hierarchy is invalid, or a node is not found.
pub fn move_selection(
    doc: &mut DiagramDocument,
    selection: &NonEmptyVec<NodeId>,
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

/// Resizes the selection.
///
/// # Errors
///
/// Returns `Error` if items are locked, hierarchy is invalid, or bounds computation fails.
pub fn resize_selection(
    doc: &mut DiagramDocument,
    selection: &NonEmptyVec<NodeId>,
    new_bounds: Rect,
) -> Result<(), Error> {
    let selection_slice = selection.as_slice();
    check_locked_items(doc, selection_slice)?;
    check_invalid_hierarchy(doc, selection_slice)?;

    let (min_x, min_y, new_x, new_y, scale_x, scale_y) =
        compute_resize_scales(doc, selection_slice, new_bounds)?;

    for id in selection_slice {
        if let Some(node) = doc.document.nodes.get_mut(id) {
            apply_resize_scale(node, min_x, min_y, new_x, new_y, scale_x, scale_y)?;
        }
    }

    Ok(())
}

/// Deletes the selection.
///
/// # Errors
///
/// Returns `Error` if items are locked, a node is not found, or postcondition is violated.
pub fn delete_selection(
    doc: &mut DiagramDocument,
    selection: &NonEmptyVec<NodeId>,
) -> Result<(), Error> {
    let selection_slice = selection.as_slice();
    check_locked_items(doc, selection_slice)?;

    for id in selection_slice {
        if !doc.document.nodes.contains_key(id) {
            return Err(Error::NodeNotFound);
        }
    }

    for id in selection_slice {
        remove_node_from_doc(doc, id);
    }

    verify_all_removed(doc, selection_slice)
}

/// Computes the centroid of the selection.
///
/// # Errors
///
/// Returns `Error` if bounding box computation fails.
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

/// Scales the selection around its centroid.
///
/// # Errors
///
/// Returns `Error` if scaling fails.
pub fn scale_selection_around_centroid(
    doc: &mut DiagramDocument,
    selection: &NonEmptyVec<NodeId>,
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
