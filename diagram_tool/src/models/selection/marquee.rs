use crate::geometry::primitives::AABB;
use crate::models::document::{DiagramDocument, NodeId};
use crate::models::selection::types::{Rect, SelectionError};
use crate::models::spatial_index::build_spatial_index;
use im::HashSet;

pub fn compute_marquee_selection(
    doc: &DiagramDocument,
    marquee: Rect,
) -> Result<HashSet<NodeId>, SelectionError> {
    if marquee.width < 0.0 || marquee.height < 0.0 {
        return Err(SelectionError::InvalidMarqueeBounds);
    }

    let index = build_spatial_index(&doc.document.nodes);
    let marquee_aabb = AABB::new(
        marquee.x,
        marquee.y,
        marquee.x + marquee.width,
        marquee.y + marquee.height,
    );

    let mut selected = HashSet::new();
    let m_right = marquee.x + marquee.width;
    let m_bottom = marquee.y + marquee.height;

    let mut parents = HashSet::new();
    for node in doc.document.nodes.values() {
        if let Some(parent_id) = &node.parent {
            parents.insert(parent_id.clone());
        }
    }

    let candidates = crate::models::spatial_index::gather_candidates(&index, &marquee_aabb);

    for id in candidates {
        let node = doc
            .document
            .nodes
            .get(&id)
            .ok_or(SelectionError::NodeNotFound)?;
        let nx = node.x.0;
        let ny = node.y.0;
        let nw = node.width.0;
        let nh = node.height.0;

        let rotation = node
            .metadata
            .get("rotation")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);

        let (min_x, min_y, max_x, max_y) = if rotation == 0.0 {
            (nx, ny, nx + nw, ny + nh)
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

            let mut node_min_x = f64::INFINITY;
            let mut node_min_y = f64::INFINITY;
            let mut node_max_x = f64::NEG_INFINITY;
            let mut node_max_y = f64::NEG_INFINITY;

            for &(px, py) in &corners {
                if px < node_min_x {
                    node_min_x = px;
                }
                if py < node_min_y {
                    node_min_y = py;
                }
                if px > node_max_x {
                    node_max_x = px;
                }
                if py > node_max_y {
                    node_max_y = py;
                }
            }
            (node_min_x, node_min_y, node_max_x, node_max_y)
        };

        let is_parent =
            parents.contains(&id) || node.kind == crate::models::document::NodeKind::Subgraph;

        let is_selected = if is_parent {
            min_x >= marquee.x && max_x <= m_right && min_y >= marquee.y && max_y <= m_bottom
        } else {
            !(min_x > m_right || max_x < marquee.x || min_y > m_bottom || max_y < marquee.y)
        };

        if is_selected {
            selected.insert(id.clone());
        }
    }

    Ok(selected)
}
