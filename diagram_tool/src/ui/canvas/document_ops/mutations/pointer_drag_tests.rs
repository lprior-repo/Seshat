#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
#[cfg(test)]
mod tests {
    use crate::history::History;
    use crate::ui::canvas::document_ops::mutations::pointer_drag::handle_dragging;
    use diagram_models::document::{
        DiagramDocument, LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
    };
    use dioxus::prelude::*;
    use im::HashMap;

    fn test_node(x: f64, y: f64, locked: bool) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: String::from("N"),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(50.0),
            height: OrderedFloat(50.0),
            font_size: None,
            font_weight: None,
            lock_state: if locked {
                LockState::Locked
            } else {
                LockState::Unlocked
            },
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        }
    }

    #[test]
    fn given_unlocked_node_and_threshold_not_reached_then_no_movement() {
        let mut vdom = VirtualDom::new(|| {
            let mut doc = DiagramDocument::default();
            let node_id = NodeId::new("n1".to_string());
            doc.document.nodes = doc
                .document
                .nodes
                .update(node_id.clone(), test_node(10.0, 10.0, false));

            let mut doc_signal = Signal::new(doc);
            let mut history_signal = Signal::new(History::new());
            let mut did_move = false;

            let mut original_positions = HashMap::new();
            original_positions.insert(node_id.clone(), (10.0, 10.0));

            handle_dragging(
                &mut doc_signal,
                &mut history_signal,
                12.0,
                12.0,
                &(10.0, 10.0),
                &(10.0, 10.0),
                &original_positions,
                &mut did_move,
            );

            assert!(!did_move);
            assert_eq!(history_signal.read().can_undo(), false);
            let doc = doc_signal.read();
            assert!(doc.document.nodes.contains_key(&node_id));
            if let Some(node) = doc.document.nodes.get(&node_id) {
                assert_eq!(node.x.0, 10.0);
                assert_eq!(node.y.0, 10.0);
            }
            rsx! { div {} }
        });
        let () = vdom.rebuild_in_place();
    }

    #[test]
    fn given_locked_node_and_threshold_reached_then_no_movement() {
        let mut vdom = VirtualDom::new(|| {
            let mut doc = DiagramDocument::default();
            let node_id = NodeId::new("n1".to_string());
            doc.document.nodes = doc
                .document
                .nodes
                .update(node_id.clone(), test_node(10.0, 10.0, true));

            let mut doc_signal = Signal::new(doc);
            let mut history_signal = Signal::new(History::new());
            let mut did_move = false;

            let mut original_positions = HashMap::new();
            original_positions.insert(node_id.clone(), (10.0, 10.0));

            handle_dragging(
                &mut doc_signal,
                &mut history_signal,
                50.0,
                50.0,
                &(10.0, 10.0),
                &(10.0, 10.0),
                &original_positions,
                &mut did_move,
            );

            assert!(!did_move);
            assert_eq!(history_signal.read().can_undo(), false);
            let doc = doc_signal.read();
            assert!(doc.document.nodes.contains_key(&node_id));
            if let Some(node) = doc.document.nodes.get(&node_id) {
                assert_eq!(node.x.0, 10.0);
                assert_eq!(node.y.0, 10.0);
            }
            rsx! { div {} }
        });
        let () = vdom.rebuild_in_place();
    }

    #[test]
    fn given_unlocked_node_and_threshold_reached_then_node_moved_and_history_pushed() {
        let mut vdom = VirtualDom::new(|| {
            let mut doc = DiagramDocument::default();
            doc.editor_state.zoom = OrderedFloat(1.0);
            doc.editor_state.camera_x = OrderedFloat(0.0);
            doc.editor_state.camera_y = OrderedFloat(0.0);
            let node_id = NodeId::new("n1".to_string());
            doc.document.nodes = doc
                .document
                .nodes
                .update(node_id.clone(), test_node(10.0, 10.0, false));

            let mut doc_signal = Signal::new(doc);
            let mut history_signal = Signal::new(History::new());
            let mut did_move = false;

            let mut original_positions = HashMap::new();
            original_positions.insert(node_id.clone(), (10.0, 10.0));

            handle_dragging(
                &mut doc_signal,
                &mut history_signal,
                30.0,
                30.0,
                &(10.0, 10.0),
                &(10.0, 10.0),
                &original_positions,
                &mut did_move,
            );

            assert!(did_move);
            assert_eq!(history_signal.read().can_undo(), true);
            let doc = doc_signal.read();
            assert!(doc.document.nodes.contains_key(&node_id));
            if let Some(node) = doc.document.nodes.get(&node_id) {
                assert_eq!(node.x.0, 30.0);
                assert_eq!(node.y.0, 30.0);
            }
            rsx! { div {} }
        });
        let () = vdom.rebuild_in_place();
    }

    #[test]
    fn given_already_moving_and_locked_node_then_no_changes() {
        let mut vdom = VirtualDom::new(|| {
            let mut doc = DiagramDocument::default();
            let node_id = NodeId::new("n1".to_string());
            doc.document.nodes = doc
                .document
                .nodes
                .update(node_id.clone(), test_node(10.0, 10.0, true));

            let mut doc_signal = Signal::new(doc);
            let mut history_signal = Signal::new(History::new());
            let mut did_move = true;

            let mut original_positions = HashMap::new();
            original_positions.insert(node_id.clone(), (10.0, 10.0));

            handle_dragging(
                &mut doc_signal,
                &mut history_signal,
                30.0,
                30.0,
                &(10.0, 10.0),
                &(10.0, 10.0),
                &original_positions,
                &mut did_move,
            );

            assert!(did_move);
            assert_eq!(history_signal.read().can_undo(), false);
            let doc = doc_signal.read();
            assert!(doc.document.nodes.contains_key(&node_id));
            if let Some(node) = doc.document.nodes.get(&node_id) {
                assert_eq!(node.x.0, 10.0);
                assert_eq!(node.y.0, 10.0);
            }
            rsx! { div {} }
        });
        let () = vdom.rebuild_in_place();
    }

    #[test]
    fn given_snap_to_grid_then_node_snaps_to_grid() {
        let mut vdom = VirtualDom::new(|| {
            let mut doc = DiagramDocument::default();
            doc.editor_state.zoom = OrderedFloat(1.0);
            doc.editor_state.camera_x = OrderedFloat(0.0);
            doc.editor_state.camera_y = OrderedFloat(0.0);
            doc.editor_state.snap_to_grid = true;
            doc.editor_state.grid_size = diagram_models::document::GridSize(20.0);
            let node_id = NodeId::new("n1".to_string());
            doc.document.nodes = doc
                .document
                .nodes
                .update(node_id.clone(), test_node(10.0, 10.0, false));

            let mut doc_signal = Signal::new(doc);
            let mut history_signal = Signal::new(History::new());
            let mut did_move = true;

            let mut original_positions = HashMap::new();
            original_positions.insert(node_id.clone(), (10.0, 10.0));

            handle_dragging(
                &mut doc_signal,
                &mut history_signal,
                25.0,
                25.0,
                &(10.0, 10.0),
                &(10.0, 10.0),
                &original_positions,
                &mut did_move,
            );

            let doc = doc_signal.read();
            assert!(doc.document.nodes.contains_key(&node_id));
            if let Some(node) = doc.document.nodes.get(&node_id) {
                assert_eq!(node.x.0, 20.0);
                assert_eq!(node.y.0, 20.0);
            }
            rsx! { div {} }
        });
        let () = vdom.rebuild_in_place();
    }

    #[test]
    fn given_empty_selection_then_no_crash_and_no_history_push() {
        let mut vdom = VirtualDom::new(|| {
            let doc = DiagramDocument::default();
            let mut doc_signal = Signal::new(doc);
            let mut history_signal = Signal::new(History::new());
            let mut did_move = false;

            let original_positions = HashMap::new();

            handle_dragging(
                &mut doc_signal,
                &mut history_signal,
                50.0,
                50.0,
                &(10.0, 10.0),
                &(10.0, 10.0),
                &original_positions,
                &mut did_move,
            );

            assert!(!did_move);
            assert_eq!(history_signal.read().can_undo(), false);
            rsx! { div {} }
        });
        let () = vdom.rebuild_in_place();
    }
}
