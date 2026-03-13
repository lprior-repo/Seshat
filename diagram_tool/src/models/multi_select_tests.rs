use crate::models::document::{
    DiagramDocument, DocumentData, EditorState, Node, NodeId, NodeKind, OrderedFloat,
};
use crate::models::multi_select::{
    copy_selection, delete_selection, move_selection, paste_selection, resize_selection, Error,
    NonEmptyVec, Rect, Vector2D,
};
use im::HashMap;

fn create_node(id: &str, x: f64, y: f64, width: f64, height: f64, locked: bool) -> Node {
    Node {
        kind: NodeKind::Node,
        icon: "".to_string(),
        label: id.to_string(),
        x: OrderedFloat::new_unchecked(x),
        y: OrderedFloat::new_unchecked(y),
        width: OrderedFloat::new_unchecked(width),
        height: OrderedFloat::new_unchecked(height),
        font_size: None,
        font_weight: None,
        locked,
        parent: None,
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        z_index: 0,
        style: None,
        collapsed: None,
    }
}

fn setup_doc() -> DiagramDocument {
    let mut nodes = HashMap::new();
    nodes.insert(
        NodeId::new("A".to_string()),
        create_node("A", 10.0, 10.0, 50.0, 50.0, false),
    );
    nodes.insert(
        NodeId::new("B".to_string()),
        create_node("B", 20.0, 20.0, 30.0, 30.0, false),
    );

    let doc_data = DocumentData {
        nodes,
        edges: HashMap::new(),
    };

    let mut editor_state = EditorState::default();
    editor_state.selected_items.insert("A".to_string());
    editor_state.selected_items.insert("B".to_string());

    DiagramDocument {
        version: 2,
        revision: crate::models::document::Revision::INITIAL,
        document: doc_data,
        editor_state,
    }
}

#[cfg(test)]
mod mul_tests {
    use super::*;

    #[test]
    fn test_mul001_drag_3_selected_nodes_preserves_relative_spacing() {
        let mut doc = setup_doc();
        doc.document.nodes.insert(
            NodeId::new("C".to_string()),
            create_node("C", 100.0, 100.0, 40.0, 40.0, false),
        );

        let selection = NonEmptyVec::try_from(vec![
            NodeId::new("A".to_string()),
            NodeId::new("B".to_string()),
            NodeId::new("C".to_string()),
        ])
        .unwrap();

        let delta = Vector2D { x: 10.0, y: 20.0 };
        let result = move_selection(&mut doc, selection, delta);

        assert!(result.is_ok());

        let node_a = doc.document.nodes.get(&NodeId::new("A".to_string())).unwrap();
        let node_b = doc.document.nodes.get(&NodeId::new("B".to_string())).unwrap();
        let node_c = doc.document.nodes.get(&NodeId::new("C".to_string())).unwrap();

        assert_eq!(node_a.x.0, 20.0);
        assert_eq!(node_a.y.0, 30.0);
        assert_eq!(node_b.x.0, 30.0);
        assert_eq!(node_b.y.0, 40.0);
        assert_eq!(node_c.x.0, 110.0);
        assert_eq!(node_c.y.0, 120.0);

        assert_eq!(node_b.x.0 - node_a.x.0, 10.0);
        assert_eq!(node_c.x.0 - node_b.x.0, 80.0);
    }

    #[test]
    fn test_mul002_mixed_selection_drag() {
        let mut doc = setup_doc();
        
        let selection = NonEmptyVec::try_from(vec![
            NodeId::new("A".to_string()),
            NodeId::new("B".to_string()),
        ])
        .unwrap();

        let delta = Vector2D { x: 50.0, y: -30.0 };
        let result = move_selection(&mut doc, selection, delta);

        assert!(result.is_ok());

        let node_a = doc.document.nodes.get(&NodeId::new("A".to_string())).unwrap();
        let node_b = doc.document.nodes.get(&NodeId::new("B".to_string())).unwrap();

        assert_eq!(node_a.x.0, 60.0);
        assert_eq!(node_a.y.0, -20.0);
        assert_eq!(node_b.x.0, 70.0);
        assert_eq!(node_b.y.0, -10.0);

        assert_eq!(node_b.x.0 - node_a.x.0, 10.0);
        assert_eq!(node_b.y.0 - node_a.y.0, 10.0);
    }

    #[test]
    fn test_mul003_drag_across_container_boundary() {
        let mut doc = setup_doc();

        doc.document.nodes.insert(
            NodeId::new("container".to_string()),
            create_node("container", 0.0, 0.0, 200.0, 200.0, false),
        );

        doc.document.nodes.get_mut(&NodeId::new("A".to_string())).unwrap().parent = 
            Some(NodeId::new("container".to_string()));
        doc.document.nodes.get_mut(&NodeId::new("B".to_string())).unwrap().parent = 
            Some(NodeId::new("container".to_string()));

        let selection = NonEmptyVec::try_from(vec![
            NodeId::new("A".to_string()),
            NodeId::new("B".to_string()),
        ])
        .unwrap();

        let delta = Vector2D { x: 20.0, y: 20.0 };
        let result = move_selection(&mut doc, selection, delta);

        assert!(result.is_ok());

        let node_a = doc.document.nodes.get(&NodeId::new("A".to_string())).unwrap();
        let node_b = doc.document.nodes.get(&NodeId::new("B".to_string())).unwrap();

        assert_eq!(node_a.x.0, 30.0);
        assert_eq!(node_a.y.0, 30.0);
        assert_eq!(node_b.x.0, 40.0);
        assert_eq!(node_b.y.0, 40.0);
    }

    #[test]
    fn test_mul004_one_locked_item_stays_put() {
        let mut doc = setup_doc();
        doc.document.nodes.get_mut(&NodeId::new("B".to_string())).unwrap().locked = true;

        let selection = NonEmptyVec::try_from(vec![
            NodeId::new("A".to_string()),
            NodeId::new("B".to_string()),
        ])
        .unwrap();

        let delta = Vector2D { x: 10.0, y: 10.0 };
        let result = move_selection(&mut doc, selection, delta);

        assert_eq!(result, Err(Error::ItemLocked));

        let node_a = doc.document.nodes.get(&NodeId::new("A".to_string())).unwrap();
        let node_b = doc.document.nodes.get(&NodeId::new("B".to_string())).unwrap();

        assert_eq!(node_a.x.0, 10.0);
        assert_eq!(node_a.y.0, 10.0);
        assert_eq!(node_b.x.0, 20.0);
        assert_eq!(node_b.y.0, 20.0);
    }

    #[test]
    fn test_mul005_grid_snap_with_multi_select() {
        let mut doc = setup_doc();

        let selection = NonEmptyVec::try_from(vec![
            NodeId::new("A".to_string()),
            NodeId::new("B".to_string()),
        ])
        .unwrap();

        let raw_delta = Vector2D { x: 7.0, y: 13.0 };
        let grid_size = 10.0;
        let snapped_x = (raw_delta.x / grid_size).round() * grid_size;
        let snapped_y = (raw_delta.y / grid_size).round() * grid_size;
        let snapped_delta = Vector2D { x: snapped_x, y: snapped_y };

        let result = move_selection(&mut doc, selection, snapped_delta);

        assert!(result.is_ok());

        let node_a = doc.document.nodes.get(&NodeId::new("A".to_string())).unwrap();
        let node_b = doc.document.nodes.get(&NodeId::new("B".to_string())).unwrap();

        assert_eq!(node_a.x.0, 20.0);
        assert_eq!(node_a.y.0, 20.0);
        assert_eq!(node_b.x.0, 30.0);
        assert_eq!(node_b.y.0, 30.0);
    }

    #[test]
    fn test_mul006_resize_from_nw_corner() {
        let mut doc = setup_doc();
        let selection = NonEmptyVec::try_from(vec![
            NodeId::new("A".to_string()),
            NodeId::new("B".to_string()),
        ])
        .unwrap();

        let new_bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 80.0,
        };
        let result = resize_selection(&mut doc, selection, new_bounds);

        assert!(result.is_ok());

        let node_a = doc.document.nodes.get(&NodeId::new("A".to_string())).unwrap();
        let node_b = doc.document.nodes.get(&NodeId::new("B".to_string())).unwrap();

        assert_eq!(node_a.x.0, 0.0);
        assert_eq!(node_a.y.0, 0.0);
        assert_eq!(node_a.width.0, 80.0);
        assert_eq!(node_a.height.0, 80.0);
    }

    #[test]
    fn test_mul007_multi_select_resize_maintains_relative_positions() {
        let mut doc = setup_doc();
        
        doc.document.nodes.insert(
            NodeId::new("C".to_string()),
            create_node("C", 100.0, 100.0, 40.0, 40.0, false),
        );

        let selection = NonEmptyVec::try_from(vec![
            NodeId::new("A".to_string()),
            NodeId::new("B".to_string()),
            NodeId::new("C".to_string()),
        ])
        .unwrap();

        let new_bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 200.0,
        };
        let result = resize_selection(&mut doc, selection, new_bounds);

        assert!(result.is_ok());

        let node_a = doc.document.nodes.get(&NodeId::new("A".to_string())).unwrap();
        let node_b = doc.document.nodes.get(&NodeId::new("B".to_string())).unwrap();
        let node_c = doc.document.nodes.get(&NodeId::new("C".to_string())).unwrap();

        let b_to_a = node_b.x.0 - node_a.x.0;
        let c_to_b = node_c.x.0 - node_b.x.0;
        
        assert!((b_to_a - 10.0).abs() < 0.1);
        assert!((c_to_b - 80.0).abs() < 0.1);
    }

    #[test]
    fn test_mul008_resize_clamps_to_minimum_size() {
        let mut doc = setup_doc();
        let selection = NonEmptyVec::try_from(vec![
            NodeId::new("A".to_string()),
        ])
        .unwrap();

        let new_bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 5.0,
            height: 5.0,
        };
        let result = resize_selection(&mut doc, selection, new_bounds);

        assert!(result.is_ok());

        let node_a = doc.document.nodes.get(&NodeId::new("A".to_string())).unwrap();
        assert!(node_a.width.0 >= 10.0);
        assert!(node_a.height.0 >= 10.0);
    }

    #[test]
    fn test_mul009_resize_expands_selection_bounds() {
        let mut doc = setup_doc();
        let selection = NonEmptyVec::try_from(vec![
            NodeId::new("A".to_string()),
            NodeId::new("B".to_string()),
        ])
        .unwrap();

        let new_bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 200.0,
        };
        let result = resize_selection(&mut doc, selection, new_bounds);

        assert!(result.is_ok());

        let node_a = doc.document.nodes.get(&NodeId::new("A".to_string())).unwrap();
        let node_b = doc.document.nodes.get(&NodeId::new("B".to_string())).unwrap();

        assert!(node_a.width.0 > 50.0);
        assert!(node_b.width.0 > 30.0);
    }

    #[test]
    fn test_mul010_resize_with_text_nodes() {
        let mut doc = setup_doc();
        
        doc.document.nodes.get_mut(&NodeId::new("A".to_string())).unwrap().kind = NodeKind::Text;
        doc.document.nodes.get_mut(&NodeId::new("B".to_string())).unwrap().kind = NodeKind::Text;

        let selection = NonEmptyVec::try_from(vec![
            NodeId::new("A".to_string()),
            NodeId::new("B".to_string()),
        ])
        .unwrap();

        let new_bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let result = resize_selection(&mut doc, selection, new_bounds);

        assert!(result.is_ok());
    }

    #[test]
    fn test_move_preserves_relative_positions() {
        let mut doc = setup_doc();
        let selection = NonEmptyVec::try_from(vec![
            NodeId::new("A".to_string()),
            NodeId::new("B".to_string()),
        ])
        .unwrap();

        let res = move_selection(&mut doc, selection, Vector2D { x: 5.0, y: 5.0 });
        assert_eq!(res, Ok(()));

        let node_a = doc
            .document
            .nodes
            .get(&NodeId::new("A".to_string()))
            .unwrap();
        let node_b = doc
            .document
            .nodes
            .get(&NodeId::new("B".to_string()))
            .unwrap();

        assert_eq!(node_a.x.0, 15.0);
        assert_eq!(node_a.y.0, 15.0);
        assert_eq!(node_b.x.0, 25.0);
        assert_eq!(node_b.y.0, 25.0);

        assert_eq!(node_b.x.0 - node_a.x.0, 10.0);
    }

    #[test]
    fn test_resize_scales_items_proportionally() {
        let mut doc = setup_doc();
        let selection = NonEmptyVec::try_from(vec![
            NodeId::new("A".to_string()),
            NodeId::new("B".to_string()),
        ])
        .unwrap();

        let new_bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let res = resize_selection(&mut doc, selection, new_bounds);
        assert_eq!(res, Ok(()));

        let node_a = doc
            .document
            .nodes
            .get(&NodeId::new("A".to_string()))
            .unwrap();
        let node_b = doc
            .document
            .nodes
            .get(&NodeId::new("B".to_string()))
            .unwrap();

        assert_eq!(node_a.x.0, 0.0);
        assert_eq!(node_a.width.0, 100.0);
        assert_eq!(node_b.x.0, 20.0);
        assert_eq!(node_b.width.0, 60.0);
    }

    #[test]
    fn test_delete_removes_all_selected_items() {
        let mut doc = setup_doc();
        let selection = NonEmptyVec::try_from(vec![
            NodeId::new("A".to_string()),
            NodeId::new("B".to_string()),
        ])
        .unwrap();

        let res = delete_selection(&mut doc, selection);
        assert_eq!(res, Ok(()));

        assert!(!doc
            .document
            .nodes
            .contains_key(&NodeId::new("A".to_string())));
        assert!(!doc
            .document
            .nodes
            .contains_key(&NodeId::new("B".to_string())));
        assert!(doc.editor_state.selected_items.is_empty());
    }

    #[test]
    fn test_copy_paste_duplicates_selection_with_offset() {
        let mut doc = setup_doc();
        let selection = NonEmptyVec::try_from(vec![
            NodeId::new("A".to_string()),
            NodeId::new("B".to_string()),
        ])
        .unwrap();

        let clipboard = copy_selection(&doc, selection).unwrap();
        assert_eq!(clipboard.nodes.len(), 2);

        let pasted_ids = paste_selection(&mut doc, &clipboard, Vector2D { x: 10.0, y: 10.0 }).unwrap();
        assert_eq!(pasted_ids.len(), 2);

        let new_a = doc.document.nodes.get(&pasted_ids[0]).unwrap();
        assert_eq!(new_a.x.0, 20.0);

        let new_b = doc.document.nodes.get(&pasted_ids[1]).unwrap();
        assert_eq!(new_b.x.0, 30.0);

        assert!(doc
            .editor_state
            .selected_items
            .contains(&pasted_ids[0].to_string()));
        assert!(doc
            .editor_state
            .selected_items
            .contains(&pasted_ids[1].to_string()));
        assert!(!doc.editor_state.selected_items.contains("A"));
    }

    #[test]
    fn test_p2_violation_returns_item_locked_error() {
        let mut doc = setup_doc();
        doc.document
            .nodes
            .get_mut(&NodeId::new("A".to_string()))
            .unwrap()
            .locked = true;

        let selection = NonEmptyVec::try_from(vec![
            NodeId::new("A".to_string()),
            NodeId::new("B".to_string()),
        ])
        .unwrap();

        let res = delete_selection(&mut doc, selection.clone());
        assert_eq!(res, Err(Error::ItemLocked));

        let res = move_selection(&mut doc, selection, Vector2D { x: 5.0, y: 5.0 });
        assert_eq!(res, Err(Error::ItemLocked));
    }

    #[test]
    fn test_p3_violation_returns_invalid_hierarchy_error() {
        let mut doc = setup_doc();
        doc.document
            .nodes
            .get_mut(&NodeId::new("B".to_string()))
            .unwrap()
            .parent = Some(NodeId::new("A".to_string()));

        let selection = NonEmptyVec::try_from(vec![
            NodeId::new("A".to_string()),
            NodeId::new("B".to_string()),
        ])
        .unwrap();

        let res = move_selection(&mut doc, selection, Vector2D { x: 5.0, y: 5.0 });
        assert_eq!(res, Err(Error::InvalidHierarchy));
    }
}
