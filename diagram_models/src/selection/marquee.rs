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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{
        DiagramDocument, DocumentData, EditorState, LockState, Node, NodeKind, OrderedFloat,
    };
    use im::HashMap;

    fn setup_doc() -> DiagramDocument {
        let mut nodes = HashMap::new();

        let parent_id = NodeId::new("p1".to_string());
        let child_id = NodeId::new("c1".to_string());

        let p1_node = Node {
            kind: NodeKind::Subgraph,
            icon: String::new(),
            label: "p1".to_string(),
            x: OrderedFloat(0.0),
            y: OrderedFloat(0.0),
            width: OrderedFloat(100.0),
            height: OrderedFloat(100.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };

        let c1_node = Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: "c1".to_string(),
            x: OrderedFloat(10.0),
            y: OrderedFloat(10.0),
            width: OrderedFloat(50.0),
            height: OrderedFloat(50.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: Some(parent_id.clone()),
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };

        nodes.insert(parent_id, p1_node);
        nodes.insert(child_id, c1_node);

        DiagramDocument {
            version: 1,
            revision: crate::document::Revision::INITIAL,
            document: DocumentData {
                nodes,
                edges: HashMap::new(),
            },
            editor_state: EditorState::default(),
        }
    }

    #[test]
    fn test_compute_marquee_selection_invalid_bounds() {
        let doc = setup_doc();
        let marquee = Rect {
            x: 0.0,
            y: 0.0,
            width: -10.0,
            height: 10.0,
        };
        assert_eq!(
            compute_marquee_selection(&doc, marquee),
            Err(SelectionError::InvalidMarqueeBounds)
        );
    }

    #[test]
    fn test_compute_marquee_selection_fully_encloses() {
        let doc = setup_doc();
        let marquee = Rect {
            x: -10.0,
            y: -10.0,
            width: 120.0,
            height: 120.0,
        };
        let selected = compute_marquee_selection(&doc, marquee).unwrap();
        assert!(selected.contains(&NodeId::new("p1".to_string())));
        assert!(selected.contains(&NodeId::new("c1".to_string())));
    }

    #[test]
    fn test_compute_marquee_selection_intersects_child_only() {
        let doc = setup_doc();
        let marquee = Rect {
            x: 10.0,
            y: 10.0,
            width: 20.0,
            height: 20.0,
        };
        let selected = compute_marquee_selection(&doc, marquee).unwrap();
        assert!(selected.contains(&NodeId::new("c1".to_string())));
        assert!(!selected.contains(&NodeId::new("p1".to_string())));
    }

    #[test]
    fn test_compute_marquee_selection_no_intersection() {
        let doc = setup_doc();
        let marquee = Rect {
            x: 200.0,
            y: 200.0,
            width: 10.0,
            height: 10.0,
        };
        let selected = compute_marquee_selection(&doc, marquee).unwrap();
        assert!(selected.is_empty());
    }
}
