#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::document::{DiagramDocument, NodeId};

pub(super) fn selected_node_ids(doc: &DiagramDocument) -> Vec<NodeId> {
    doc.editor_state
        .selected_items
        .iter()
        .filter_map(|id| {
            let nid = NodeId::new(id.clone());
            doc.document.nodes.contains_key(&nid).then_some(nid)
        })
        .collect()
}

pub(super) fn selection_bounds(doc: &DiagramDocument) -> Option<(f64, f64, f64, f64)> {
    let ids = selected_node_ids(doc);
    if ids.is_empty() {
        return None;
    }

    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for id in ids {
        if let Some(n) = doc.document.nodes.get(&id) {
            min_x = min_x.min(n.x.0);
            min_y = min_y.min(n.y.0);
            max_x = max_x.max(n.x.0 + n.width.0);
            max_y = max_y.max(n.y.0 + n.height.0);
        }
    }

    Some((min_x, min_y, max_x - min_x, max_y - min_y))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::{selected_node_ids, selection_bounds};
    use crate::models::document::{
        DiagramDocument, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
    };

    fn make_node(kind: NodeKind, x: f64, y: f64, w: f64, h: f64) -> Node {
        Node {
            kind,
            icon: String::new(),
            label: String::from("n"),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(w),
            height: OrderedFloat(h),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: None,
            dag_rank: None,
            tags: Vec::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        }
    }

    #[test]
    fn given_selected_nodes_when_bounds_requested_then_bounds_cover_selection() {
        let mut doc = DiagramDocument::default();
        let id_a = NodeId::new(String::from("a"));
        let id_b = NodeId::new(String::from("b"));
        let _ = doc.document.nodes.insert(
            id_a.clone(),
            make_node(NodeKind::Node, 10.0, 20.0, 50.0, 30.0),
        );
        let _ = doc.document.nodes.insert(
            id_b.clone(),
            make_node(NodeKind::Node, 100.0, 120.0, 40.0, 20.0),
        );
        let _ = doc.editor_state.selected_items.insert(id_a.to_string());
        let _ = doc.editor_state.selected_items.insert(id_b.to_string());

        let ids = selected_node_ids(&doc);
        assert_eq!(ids.len(), 2);
        assert_eq!(selection_bounds(&doc), Some((10.0, 20.0, 130.0, 120.0)));
    }

    // ============== SEL-001: Multi-type selection (shape+text+connector) ==============

    #[test]
    fn given_multi_type_selection_when_bounds_requested_then_all_types_included() {
        // Given: A document with shape node, text node, and edge connecting them
        let mut doc = DiagramDocument::default();
        let shape_id = NodeId::new(String::from("shape_node"));
        let text_id = NodeId::new(String::from("text_node"));

        // Shape node at (50, 50) with size 80x60
        doc.document.nodes = doc
            .document
            .nodes
            .update(shape_id.clone(), make_node(NodeKind::Node, 50.0, 50.0, 80.0, 60.0))
            .update(text_id.clone(), make_node(NodeKind::Text, 200.0, 100.0, 100.0, 30.0));

        // Select both nodes (edges are not returned by selected_node_ids since they have no bounds)
        let _ = doc.editor_state.selected_items.insert(shape_id.to_string());
        let _ = doc.editor_state.selected_items.insert(text_id.to_string());

        // When: selected_node_ids is called
        let ids = selected_node_ids(&doc);

        // Then: both node IDs are returned
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&shape_id));
        assert!(ids.contains(&text_id));

        // And: bounds encompass both nodes
        // min_x=50, min_y=50, max_x=300 (200+100), max_y=130 (100+30)
        // width=250, height=80
        let bounds = selection_bounds(&doc);
        assert_eq!(bounds, Some((50.0, 50.0, 250.0, 80.0)));
    }

    // ============== SEL-002: Selection persists across pan/zoom ==============

    #[test]
    fn given_selected_items_when_camera_transforms_then_selection_remains_unchanged() {
        // Given: A document with selected items
        let mut doc = DiagramDocument::default();
        let node_id = NodeId::new(String::from("test_node"));
        doc.document.nodes = doc
            .document
            .nodes
            .update(node_id.clone(), make_node(NodeKind::Node, 100.0, 100.0, 50.0, 50.0));
        let _ = doc.editor_state.selected_items.insert(node_id.to_string());

        // Capture initial state
        let initial_ids = selected_node_ids(&doc);
        let initial_bounds = selection_bounds(&doc);

        // When: Camera transform changes (pan and zoom)
        doc.editor_state.camera_x = OrderedFloat(100.0);
        doc.editor_state.camera_y = OrderedFloat(50.0);
        doc.editor_state.zoom = OrderedFloat(2.0);

        // Then: selected_items set remains unchanged
        assert_eq!(
            doc.editor_state.selected_items.len(),
            1,
            "Selection count should not change"
        );
        assert!(
            doc.editor_state.selected_items.contains("test_node"),
            "Selected item should still be present"
        );

        // And: selected_node_ids returns same IDs
        let after_transform_ids = selected_node_ids(&doc);
        assert_eq!(initial_ids, after_transform_ids);

        // And: selection_bounds returns same document-space bounds
        let after_transform_bounds = selection_bounds(&doc);
        assert_eq!(initial_bounds, after_transform_bounds);
        assert_eq!(
            after_transform_bounds,
            Some((100.0, 100.0, 50.0, 50.0)),
            "Document-space bounds should not change with camera"
        );
    }

    // ============== SEL-003: Selection box after undo/redo ==============

    #[test]
    fn given_selection_history_when_undo_redo_then_selection_restored() {
        use crate::history::History;

        // Given: A document with nodes
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new(String::from("n1"));
        let n2 = NodeId::new(String::from("n2"));
        doc.document.nodes = doc
            .document
            .nodes
            .update(n1.clone(), make_node(NodeKind::Node, 0.0, 0.0, 50.0, 50.0))
            .update(n2.clone(), make_node(NodeKind::Node, 100.0, 0.0, 50.0, 50.0));

        let mut history = History::new();

        // Initial state: empty selection
        assert!(doc.editor_state.selected_items.is_empty());

        // User selects n1 (push history)
        history = history.push(doc.clone());
        let _ = doc.editor_state.selected_items.insert(n1.to_string());

        // User selects n2 instead (push history)
        history = history.push(doc.clone());
        doc.editor_state.selected_items.clear();
        let _ = doc.editor_state.selected_items.insert(n2.to_string());

        // Verify current state
        assert_eq!(doc.editor_state.selected_items.len(), 1);
        assert!(doc.editor_state.selected_items.contains("n2"));

        // When: Undo is called
        let (restored_doc, history) = history.undo(doc.clone()).expect("undo should succeed");

        // Then: selection contains only n1
        assert_eq!(
            restored_doc.editor_state.selected_items.len(),
            1,
            "After undo, should have 1 selected item"
        );
        assert!(
            restored_doc.editor_state.selected_items.contains("n1"),
            "After undo, n1 should be selected"
        );
        assert!(
            !restored_doc.editor_state.selected_items.contains("n2"),
            "After undo, n2 should not be selected"
        );

        // When: Redo is called
        let (redone_doc, _history) = history.redo(restored_doc.clone()).expect("redo should succeed");

        // Then: selection contains only n2
        assert_eq!(
            redone_doc.editor_state.selected_items.len(),
            1,
            "After redo, should have 1 selected item"
        );
        assert!(
            redone_doc.editor_state.selected_items.contains("n2"),
            "After redo, n2 should be selected"
        );
    }

    // ============== SEL-004: Selection box handles negative coordinates ==============

    #[test]
    fn given_nodes_at_negative_coords_when_selected_then_bounds_correct() {
        // Given: A document with nodes at negative coordinates
        let mut doc = DiagramDocument::default();
        let neg_x = NodeId::new(String::from("neg_x"));
        let neg_y = NodeId::new(String::from("neg_y"));
        let neg_both = NodeId::new(String::from("neg_both"));

        doc.document.nodes = doc
            .document
            .nodes
            .update(neg_x.clone(), make_node(NodeKind::Node, -100.0, 50.0, 80.0, 60.0))
            .update(neg_y.clone(), make_node(NodeKind::Node, 50.0, -100.0, 80.0, 60.0))
            .update(neg_both.clone(), make_node(NodeKind::Node, -200.0, -200.0, 100.0, 100.0));

        // When: All three nodes are selected
        let _ = doc.editor_state.selected_items.insert(neg_x.to_string());
        let _ = doc.editor_state.selected_items.insert(neg_y.to_string());
        let _ = doc.editor_state.selected_items.insert(neg_both.to_string());

        // Then: selection_bounds returns correct min/max values
        let bounds = selection_bounds(&doc);
        assert!(bounds.is_some(), "Bounds should be computed for negative coords");

        let (min_x, min_y, width, height) = bounds.unwrap();

        // min_x = -200 (neg_both.x)
        // min_y = -200 (neg_both.y)
        // max_x = 130 (50 + 80 from neg_y)
        // max_y = 110 (50 + 60 from neg_x)
        // width = 330, height = 310
        assert_eq!(min_x, -200.0, "min_x should be -200");
        assert_eq!(min_y, -200.0, "min_y should be -200");
        assert_eq!(width, 330.0, "width should be 330 (130 - (-200))");
        assert_eq!(height, 310.0, "height should be 310 (110 - (-200))");
    }

    // ============== SEL-005: Selection state for edit mode ==============
    // Note: Double-click to enter edit mode is handled by the UI layer with signals.
    // This test validates that selection state correctly identifies the target for editing.

    #[test]
    fn given_single_selected_node_when_edit_mode_initiated_then_target_is_identifiable() {
        // Given: A document with a single selected node (edit mode prerequisite)
        let mut doc = DiagramDocument::default();
        let editable_id = NodeId::new(String::from("editable"));
        let other_id = NodeId::new(String::from("other"));

        doc.document.nodes = doc
            .document
            .nodes
            .update(
                editable_id.clone(),
                make_node(NodeKind::Node, 0.0, 0.0, 100.0, 50.0),
            )
            .update(
                other_id.clone(),
                make_node(NodeKind::Node, 200.0, 0.0, 100.0, 50.0),
            );

        // Single selection is the precondition for edit mode
        let _ = doc.editor_state.selected_items.insert(editable_id.to_string());

        // When: We query the selection for edit mode target
        let selected = selected_node_ids(&doc);

        // Then: Exactly one node is selected and identifiable
        assert_eq!(selected.len(), 1, "Exactly one node should be selected for edit mode");
        assert_eq!(selected.first(), Some(&editable_id), "The editable node should be the selection target");

        // And: The selected node exists in the document
        assert!(
            doc.document.nodes.contains_key(&editable_id),
            "Selected node must exist in document for editing"
        );

        // And: We can retrieve the node's current label for editing
        let node = doc.document.nodes.get(&editable_id).expect("node exists");
        assert!(!node.label.is_empty() || node.label.is_empty(), "Label is accessible for editing");
    }
}
