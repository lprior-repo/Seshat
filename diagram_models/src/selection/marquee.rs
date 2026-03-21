use crate::document::{DiagramDocument, NodeId};
use crate::geometry::AABB;
use crate::selection::types::{Rect, SelectionError};
use crate::spatial_index::build_spatial_index;
use im::HashSet;

use super::bounds::{get_node_rotation, rotated_node_bounds};

/// Computes selection based on a marquee rectangle.
///
/// # Errors
///
/// Returns `SelectionError` if marquee bounds are invalid or a node is not found.
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

    // Build parent set once using iterator pattern
    let parents: std::collections::HashSet<NodeId> = doc
        .document
        .nodes
        .values()
        .filter_map(|node| node.parent.clone())
        .collect();

    let candidates = crate::spatial_index::gather_candidates(&index, &marquee_aabb);

    for id in candidates {
        let node = doc
            .document
            .nodes
            .get(&id)
            .ok_or(SelectionError::NodeNotFound)?;

        let rotation = get_node_rotation(node);
        let (min_x, min_y, max_x, max_y) =
            rotated_node_bounds(node.x.0, node.y.0, node.width.0, node.height.0, rotation);

        let is_parent = parents.contains(&id) || node.kind == crate::document::NodeKind::Subgraph;

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
