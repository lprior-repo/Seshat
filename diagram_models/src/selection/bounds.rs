use crate::document::{DiagramDocument, NodeId};
use crate::selection::types::{SelectionBounds, SelectionError};

/// Extracts rotation in radians from node metadata, returning 0.0 if not present.
#[inline]
#[must_use]
pub fn get_node_rotation(node: &crate::document::Node) -> f64 {
    node.metadata
        .get("rotation")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0)
}

/// Computes the axis-aligned bounding box for a rotated node.
#[inline]
#[must_use]
pub fn rotated_node_bounds(
    nx: f64,
    ny: f64,
    nw: f64,
    nh: f64,
    rotation: f64,
) -> (f64, f64, f64, f64) {
    if rotation == 0.0 {
        return (nx, ny, nx + nw, ny + nh);
    }

    let cx = nx + nw / 2.0;
    let cy = ny + nh / 2.0;
    let cos_r = rotation.cos();
    let sin_r = rotation.sin();

    let rotate = |px: f64, py: f64| -> (f64, f64) {
        let dx = px - cx;
        let dy = py - cy;
        (cx + dx * cos_r - dy * sin_r, cy + dx * sin_r + dy * cos_r)
    };

    let corners = [
        rotate(nx, ny),
        rotate(nx + nw, ny),
        rotate(nx, ny + nh),
        rotate(nx + nw, ny + nh),
    ];

    corners.iter().fold(
        (
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ),
        |(min_x, min_y, max_x, max_y), &(px, py)| {
            (min_x.min(px), min_y.min(py), max_x.max(px), max_y.max(py))
        },
    )
}

/// Computes bounds of the current selection.
///
/// # Errors
///
/// Returns `SelectionError` if a selected node is not found.
pub fn compute_selection_bounds(doc: &DiagramDocument) -> Result<SelectionBounds, SelectionError> {
    let selected_ids = &doc.editor_state.selected_items;

    if selected_ids.is_empty() {
        return Ok(SelectionBounds {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        });
    }

    // Single-pass validation + reduction via try_fold (eliminates intermediate Vec + mut)
    let (min_x, min_y, max_x, max_y) = selected_ids
        .iter()
        .map(|id_str| {
            let node_id =
                NodeId::try_new(id_str.clone()).map_err(|_| SelectionError::NodeNotFound)?;

            let node = doc
                .document
                .nodes
                .get(&node_id)
                .ok_or(SelectionError::NodeNotFound)?;

            Ok(rotated_node_bounds(
                node.x.0,
                node.y.0,
                node.width.0,
                node.height.0,
                get_node_rotation(node),
            ))
        })
        .try_fold(
            (
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
            ),
            |(min_x, min_y, max_x, max_y), item| {
                let (n_min_x, n_min_y, n_max_x, n_max_y) = item?;
                Ok((
                    min_x.min(n_min_x),
                    min_y.min(n_min_y),
                    max_x.max(n_max_x),
                    max_y.max(n_max_y),
                ))
            },
        )?;

    if min_x.is_infinite() {
        return Ok(SelectionBounds {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        });
    }

    Ok(SelectionBounds {
        x: min_x,
        y: min_y,
        width: max_x - min_x,
        height: max_y - min_y,
    })
}
