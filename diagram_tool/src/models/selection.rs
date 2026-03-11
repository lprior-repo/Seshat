use crate::models::document::{DiagramDocument, NodeId};
use im::HashSet;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SelectionError {
    #[error("Node not found in document")]
    NodeNotFound,
    #[error("Movement exceeded drag threshold")]
    MovementExceededDragThreshold,
    #[error("Node is not editable")]
    NodeNotEditable,
    #[error("Invalid marquee bounds: negative width or height")]
    InvalidMarqueeBounds,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectionBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Result<Self, SelectionError> {
        if width < 0.0 || height < 0.0 {
            Err(SelectionError::InvalidMarqueeBounds)
        } else {
            Ok(Self {
                x,
                y,
                width,
                height,
            })
        }
    }
}

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
                    .and_then(|v| v.as_f64())
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
                        |(mx, my, M1, M2), &(px, py)| {
                            (mx.min(px), my.min(py), M1.max(px), M2.max(py))
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

pub fn handle_long_press(
    doc: &mut DiagramDocument,
    target: NodeId,
    movement: f64,
) -> Result<(), SelectionError> {
    if movement >= 5.0 {
        return Err(SelectionError::MovementExceededDragThreshold);
    }

    // Must ensure node exists to not add invalid node IDs to selection
    if !doc.document.nodes.contains_key(&target) {
        return Err(SelectionError::NodeNotFound);
    }

    doc.editor_state.selected_items.insert(target.to_string());
    Ok(())
}

pub fn handle_double_click(
    doc: &mut DiagramDocument,
    target: NodeId,
) -> Result<(), SelectionError> {
    let node = doc
        .document
        .nodes
        .get(&target)
        .ok_or(SelectionError::NodeNotFound)?;

    if node.locked {
        return Err(SelectionError::NodeNotEditable);
    }

    doc.editor_state.edit_mode_target = Some(target.to_string());
    Ok(())
}

pub fn compute_marquee_selection(
    doc: &DiagramDocument,
    marquee: Rect,
) -> Result<HashSet<NodeId>, SelectionError> {
    if marquee.width < 0.0 || marquee.height < 0.0 {
        return Err(SelectionError::InvalidMarqueeBounds);
    }

    let mut selected = HashSet::new();

    let m_right = marquee.x + marquee.width;
    let m_bottom = marquee.y + marquee.height;

    let mut parents = HashSet::new();
    for node in doc.document.nodes.values() {
        if let Some(parent_id) = &node.parent {
            parents.insert(parent_id.clone());
        }
    }

    for (id, node) in &doc.document.nodes {
        let nx = node.x.0;
        let ny = node.y.0;
        let nw = node.width.0;
        let nh = node.height.0;

        let rotation = node
            .metadata
            .get("rotation")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        // Simple AABB intersection for now, even if rotated, we use its AABB
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
            parents.contains(id) || node.kind == crate::models::document::NodeKind::Subgraph;

        let is_selected = if is_parent {
            // Must be fully enclosed
            min_x >= marquee.x && max_x <= m_right && min_y >= marquee.y && max_y <= m_bottom
        } else {
            // Must intersect
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
    use super::{
        compute_marquee_selection, compute_selection_bounds, handle_double_click,
        handle_long_press, Rect, SelectionBounds, SelectionError,
    };
    use crate::models::document::{
        DiagramDocument, DocumentData, EditorState, Node, NodeId, NodeKind, OrderedFloat,
    };
    use im::HashMap;
    use serde_json::json;

    fn setup_doc() -> DiagramDocument {
        let mut nodes = HashMap::new();
        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());

        let n1_node = Node {
            kind: NodeKind::Node,
            icon: "".to_string(),
            label: "n1".to_string(),
            x: OrderedFloat(0.0),
            y: OrderedFloat(0.0),
            width: OrderedFloat(100.0),
            height: OrderedFloat(100.0),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };

        let n2_node = Node {
            kind: NodeKind::Node,
            icon: "".to_string(),
            label: "n2".to_string(),
            x: OrderedFloat(200.0),
            y: OrderedFloat(200.0),
            width: OrderedFloat(50.0),
            height: OrderedFloat(50.0),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };

        nodes.insert(n1.clone(), n1_node);
        nodes.insert(n2.clone(), n2_node);

        let doc_data = DocumentData {
            nodes,
            edges: HashMap::new(),
        };

        DiagramDocument {
            version: 2,
            revision: crate::models::document::Revision::INITIAL,
            document: doc_data,
            editor_state: EditorState::default(),
        }
    }

    #[test]
    fn test_sel_021_bounding_box_covers_rotated_nodes() {
        let mut doc = setup_doc();
        // Rotate n1 by 45 degrees
        let mut meta = HashMap::new();
        meta.insert("rotation".to_string(), json!(std::f64::consts::FRAC_PI_4));
        doc.document
            .nodes
            .get_mut(&NodeId::new("n1".to_string()))
            .unwrap()
            .metadata = meta;

        doc.editor_state.selected_items.insert("n1".to_string());

        let bounds = compute_selection_bounds(&doc).unwrap();
        // 100x100 box at 0,0 rotated 45deg around center 50,50
        // distance from center to corner is sqrt(50^2 + 50^2) = 70.7106
        // So bounds width = 70.7106 * 2 = 141.421

        assert!((bounds.width - 141.421).abs() < 0.1);
        assert!((bounds.height - 141.421).abs() < 0.1);
    }

    #[test]
    fn test_sel_022_long_press_adds_node_to_selection_without_drag() {
        let mut doc = setup_doc();
        let res = handle_long_press(&mut doc, NodeId::new("n1".to_string()), 2.0);
        assert!(res.is_ok());
        assert!(doc.editor_state.selected_items.contains("n1"));
    }

    #[test]
    fn test_sel_023_double_click_enters_edit_mode_on_shape() {
        let mut doc = setup_doc();
        let res = handle_double_click(&mut doc, NodeId::new("n1".to_string()));
        assert!(res.is_ok());
        assert_eq!(doc.editor_state.edit_mode_target.as_deref(), Some("n1"));
    }

    #[test]
    fn test_sel_024_selection_persists_across_camera_zoom_and_pan() {
        let mut doc = setup_doc();
        doc.editor_state.selected_items.insert("n1".to_string());

        let selection_before = doc.editor_state.selected_items.clone();

        // Zoom and pan
        doc.editor_state.camera_x = OrderedFloat(100.0);
        doc.editor_state.zoom = OrderedFloat(2.0);

        let selection_after = doc.editor_state.selected_items.clone();

        assert_eq!(selection_before, selection_after);
    }

    #[test]
    fn test_sel_025_marquee_selects_nodes_inside_subgraphs() {
        let mut doc = setup_doc();
        let child = Node {
            kind: NodeKind::Node,
            icon: "".to_string(),
            label: "child".to_string(),
            x: OrderedFloat(50.0),
            y: OrderedFloat(50.0),
            width: OrderedFloat(10.0),
            height: OrderedFloat(10.0),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: Some(NodeId::new("n1".to_string())),
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };
        doc.document
            .nodes
            .insert(NodeId::new("child".to_string()), child);

        let marquee = Rect::new(45.0, 45.0, 20.0, 20.0).unwrap();
        let selected = compute_marquee_selection(&doc, marquee).unwrap();

        assert!(selected.contains(&NodeId::new("child".to_string())));
        assert!(selected.contains(&NodeId::new("n1".to_string()))); // n1 bounds 0,0 100,100, overlaps
        assert!(!selected.contains(&NodeId::new("n2".to_string())));
    }

    #[test]
    fn test_returns_error_when_computing_bounds_for_missing_nodes() {
        let mut doc = setup_doc();
        doc.editor_state
            .selected_items
            .insert("n3_missing".to_string());

        let res = compute_selection_bounds(&doc);
        assert_eq!(res.unwrap_err(), SelectionError::NodeNotFound);
    }

    #[test]
    fn test_long_press_fails_when_movement_exceeds_threshold() {
        let mut doc = setup_doc();
        let res = handle_long_press(&mut doc, NodeId::new("n1".to_string()), 15.0);
        assert_eq!(
            res.unwrap_err(),
            SelectionError::MovementExceededDragThreshold
        );
    }

    #[test]
    fn test_double_click_fails_on_uneditable_nodes() {
        let mut doc = setup_doc();
        doc.document
            .nodes
            .get_mut(&NodeId::new("n1".to_string()))
            .unwrap()
            .locked = true;

        let res = handle_double_click(&mut doc, NodeId::new("n1".to_string()));
        assert_eq!(res.unwrap_err(), SelectionError::NodeNotEditable);
    }

    #[test]
    fn test_p1_violation_returns_node_not_found() {
        let mut doc = setup_doc();
        doc.editor_state.selected_items.insert("n3".to_string());
        let res = compute_selection_bounds(&doc);
        assert_eq!(res, Err(SelectionError::NodeNotFound));
    }

    #[test]
    fn test_p2_violation_returns_movement_exceeded_drag_threshold() {
        let mut doc = setup_doc();
        let res = handle_long_press(&mut doc, NodeId::new("n1".to_string()), 6.0);
        assert_eq!(res, Err(SelectionError::MovementExceededDragThreshold));
    }

    #[test]
    fn test_p3_violation_returns_node_not_editable() {
        let mut doc = setup_doc();
        doc.document
            .nodes
            .get_mut(&NodeId::new("n1".to_string()))
            .unwrap()
            .locked = true;
        let res = handle_double_click(&mut doc, NodeId::new("n1".to_string()));
        assert_eq!(res, Err(SelectionError::NodeNotEditable));
    }

    #[test]
    fn test_p5_violation_returns_marquee_invalid() {
        let res = Rect::new(0.0, 0.0, -10.0, 10.0);
        assert_eq!(res.unwrap_err(), SelectionError::InvalidMarqueeBounds);
    }

    #[test]
    fn test_sel_021_bounding_box_with_mixed_rotated_and_unrotated_nodes() {
        let mut doc = setup_doc();
        // n1 is unrotated: 0,0 -> 100,100
        // n2 is rotated 90 deg: 200,200 -> 250,250
        // bounding box of both should envelop both
        let mut meta = HashMap::new();
        meta.insert("rotation".to_string(), json!(std::f64::consts::FRAC_PI_2));
        doc.document
            .nodes
            .get_mut(&NodeId::new("n2".to_string()))
            .unwrap()
            .metadata = meta;

        doc.editor_state.selected_items.insert("n1".to_string());
        doc.editor_state.selected_items.insert("n2".to_string());

        let bounds = compute_selection_bounds(&doc).unwrap();

        // Unrotated n1: 0, 0 to 100, 100
        // Rotated n2 by 90deg doesn't change its AABB size since it's square: 200, 200 to 250, 250
        assert!((bounds.x - 0.0).abs() < 0.1);
        assert!((bounds.y - 0.0).abs() < 0.1);
        assert!((bounds.width - 250.0).abs() < 0.1);
        assert!((bounds.height - 250.0).abs() < 0.1);
    }

    #[test]
    fn test_sel_025_marquee_partially_overlapping_parent_selects_fully_enclosed_child() {
        let mut doc = setup_doc();
        // Group A
        let group_a = Node {
            kind: NodeKind::Node,
            icon: "".to_string(),
            label: "Group A".to_string(),
            x: OrderedFloat(0.0),
            y: OrderedFloat(0.0),
            width: OrderedFloat(100.0),
            height: OrderedFloat(100.0),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };
        doc.document
            .nodes
            .insert(NodeId::new("group_a".to_string()), group_a);

        // Child B
        let child_b = Node {
            kind: NodeKind::Node,
            icon: "".to_string(),
            label: "Child B".to_string(),
            x: OrderedFloat(10.0),
            y: OrderedFloat(10.0),
            width: OrderedFloat(20.0),
            height: OrderedFloat(20.0),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: Some(NodeId::new("group_a".to_string())),
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };
        doc.document
            .nodes
            .insert(NodeId::new("child_b".to_string()), child_b);

        // Node C
        let node_c = Node {
            kind: NodeKind::Node,
            icon: "".to_string(),
            label: "Node C".to_string(),
            x: OrderedFloat(150.0),
            y: OrderedFloat(10.0),
            width: OrderedFloat(20.0),
            height: OrderedFloat(20.0),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };
        doc.document
            .nodes
            .insert(NodeId::new("node_c".to_string()), node_c);

        // Marquee hits Child B fully, and partially hits Group A
        let marquee = Rect::new(5.0, 5.0, 200.0, 30.0).unwrap();
        let selected = compute_marquee_selection(&doc, marquee).unwrap();

        assert!(selected.contains(&NodeId::new("child_b".to_string())));
        assert!(selected.contains(&NodeId::new("node_c".to_string())));
        assert!(!selected.contains(&NodeId::new("group_a".to_string())));
    }
}
