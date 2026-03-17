use crate::document::{DiagramDocument, Node, NodeId, OrderedFloat};
use std::collections::HashSet;

use super::types::{Error, Rect, Vector2D};

pub(super) fn to_ordered(value: f64) -> Result<OrderedFloat, Error> {
    OrderedFloat::new(value).map_err(|_| Error::InvalidScale)
}

pub(super) fn compute_bounding_box(
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

pub(super) fn check_locked_items(doc: &DiagramDocument, selection: &[NodeId]) -> Result<(), Error> {
    for id in selection {
        let node = doc.document.nodes.get(id).ok_or(Error::NodeNotFound)?;
        if node.lock_state.is_locked() {
            return Err(Error::ItemLocked);
        }
    }
    Ok(())
}

pub(super) fn check_invalid_hierarchy(
    doc: &DiagramDocument,
    selection: &[NodeId],
) -> Result<(), Error> {
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

pub(super) fn scale_node(
    node: &mut Node,
    centroid: Vector2D,
    scale_factor: f64,
) -> Result<(), Error> {
    let rel_x = node.x.0 - centroid.x;
    let rel_y = node.y.0 - centroid.y;

    node.x = to_ordered(rel_x.mul_add(scale_factor, centroid.x))?;
    node.y = to_ordered(rel_y.mul_add(scale_factor, centroid.y))?;
    node.width = to_ordered(node.width.0 * scale_factor)?;
    node.height = to_ordered(node.height.0 * scale_factor)?;

    Ok(())
}

pub(super) fn generate_unique_id(base_label: &str, doc: &DiagramDocument) -> NodeId {
    let mut idx = 1;
    loop {
        let new_id = NodeId::new(format!("{base_label}_{idx}"));
        if !doc.document.nodes.contains_key(&new_id) {
            return new_id;
        }
        idx += 1;
    }
}

pub(super) fn apply_resize_scale(
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

pub(super) fn compute_resize_scales(
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

    let _ = to_ordered(new_bounds.x)?;
    let _ = to_ordered(new_bounds.y)?;

    Ok((min_x, min_y, new_bounds.x, new_bounds.y, scale_x, scale_y))
}

pub(super) fn verify_all_removed(
    doc: &DiagramDocument,
    selection_slice: &[NodeId],
) -> Result<(), Error> {
    for id in selection_slice {
        if doc.document.nodes.contains_key(id) {
            return Err(Error::PostconditionViolated);
        }
    }
    Ok(())
}

pub(super) fn remove_node_from_doc(doc: &mut DiagramDocument, id: &NodeId) {
    doc.document.nodes.remove(id);
    doc.editor_state.selected_items.remove(&id.to_string());
}

pub(super) fn add_node_to_doc(doc: &mut DiagramDocument, new_node: Node, new_id: NodeId) -> NodeId {
    doc.document.nodes.insert(new_id.clone(), new_node);
    doc.editor_state.selected_items.insert(new_id.to_string());
    new_id
}

pub(super) fn validate_scale_factor(scale_factor: f64) -> Result<(), Error> {
    if scale_factor > 0.0 && scale_factor.is_finite() {
        Ok(())
    } else {
        Err(Error::InvalidScale)
    }
}

pub(super) fn apply_scale_to_selection(
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
