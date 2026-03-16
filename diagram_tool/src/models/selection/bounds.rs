use crate::models::document::{DiagramDocument, NodeId};
use crate::models::selection::types::{SelectionBounds, SelectionError};

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

    let (min_x, min_y, max_x, max_y) = selected_ids.iter().fold(
        (
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ),
        |(min_x, min_y, max_x, max_y), id_str| {
            let node_id =
                NodeId::try_new(id_str.clone()).unwrap_or_else(|_| NodeId::new(String::new()));
            if let Some(node) = doc.document.nodes.get(&node_id) {
                let nx = node.x.0;
                let ny = node.y.0;
                let nw = node.width.0;
                let nh = node.height.0;

                let rotation = node
                    .metadata
                    .get("rotation")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(0.0);

                if rotation == 0.0 {
                    (
                        min_x.min(nx),
                        min_y.min(ny),
                        max_x.max(nx + nw),
                        max_y.max(ny + nh),
                    )
                } else {
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
                        (min_x, min_y, max_x, max_y),
                        |(mx, my, m1, m2), &(px, py)| {
                            (mx.min(px), my.min(py), m1.max(px), m2.max(py))
                        },
                    )
                }
            } else {
                (min_x, min_y, max_x, max_y)
            }
        },
    );

    // Node id validation without early return in fold
    for id_str in selected_ids {
        let node_id = NodeId::try_new(id_str.clone()).map_err(|_| SelectionError::NodeNotFound)?;
        if !doc.document.nodes.contains_key(&node_id) {
            return Err(SelectionError::NodeNotFound);
        }
    }

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
