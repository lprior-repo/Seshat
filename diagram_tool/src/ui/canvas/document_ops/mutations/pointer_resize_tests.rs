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
    use crate::ui::canvas::document_ops::mutations::pointer_resize::handle_resizing;
    use canvas_domain::interaction_reducer::ResizeHandle;
    use diagram_models::document::{
        DiagramDocument, LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
    };
    use dioxus::prelude::*;
    use im::HashMap;

    fn test_node(x: f64, y: f64, w: f64, h: f64, locked: bool) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: String::from("N"),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(w),
            height: OrderedFloat(h),
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
    fn given_unlocked_node_and_threshold_not_reached_then_no_resize() {
        let mut vdom = VirtualDom::new(|| {
            let mut doc = DiagramDocument::default();
            let node_id = NodeId::new("n1".to_string());
            doc.document.nodes = doc
                .document
                .nodes
                .update(node_id.clone(), test_node(10.0, 10.0, 50.0, 50.0, false));

            let mut doc_signal = Signal::new(doc);
            let mut history_signal = Signal::new(History::new());
            let mut did_resize = false;

            let mut originals = HashMap::new();
            originals.insert(node_id.clone(), (10.0, 10.0, 50.0, 50.0));

            handle_resizing(
                &mut doc_signal,
                &mut history_signal,
                10.0,
                10.0,
                &ResizeHandle::Se,
                &(10.0, 10.0, 50.0, 50.0),
                &originals,
                &(10.0, 10.0),
                &mut did_resize,
                &None,
            );

            assert!(!did_resize);
            assert_eq!(history_signal.read().can_undo(), false);
            let doc = doc_signal.read();
            assert!(doc.document.nodes.contains_key(&node_id));
            if let Some(node) = doc.document.nodes.get(&node_id) {
                assert_eq!(node.width.0, 50.0);
            }
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_locked_node_and_threshold_reached_then_no_history_pushed_and_no_changes() {
        let mut vdom = VirtualDom::new(|| {
            let mut doc = DiagramDocument::default();
            let node_id = NodeId::new("n1".to_string());
            doc.document.nodes = doc
                .document
                .nodes
                .update(node_id.clone(), test_node(10.0, 10.0, 50.0, 50.0, true));

            let mut doc_signal = Signal::new(doc);
            let mut history_signal = Signal::new(History::new());
            let mut did_resize = false;

            let mut originals = HashMap::new();
            originals.insert(node_id.clone(), (10.0, 10.0, 50.0, 50.0));

            handle_resizing(
                &mut doc_signal,
                &mut history_signal,
                30.0,
                30.0,
                &ResizeHandle::Se,
                &(10.0, 10.0, 50.0, 50.0),
                &originals,
                &(10.0, 10.0),
                &mut did_resize,
                &None,
            );

            assert!(!did_resize);
            assert_eq!(history_signal.read().can_undo(), false);
            let doc = doc_signal.read();
            assert!(doc.document.nodes.contains_key(&node_id));
            if let Some(node) = doc.document.nodes.get(&node_id) {
                assert_eq!(node.width.0, 50.0);
            }
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_unlocked_node_and_se_handle_resize_then_node_expands() {
        let mut vdom = VirtualDom::new(|| {
            let mut doc = DiagramDocument::default();
            doc.editor_state.zoom = OrderedFloat(1.0);
            doc.editor_state.camera_x = OrderedFloat(0.0);
            doc.editor_state.camera_y = OrderedFloat(0.0);
            let node_id = NodeId::new("n1".to_string());
            doc.document.nodes = doc
                .document
                .nodes
                .update(node_id.clone(), test_node(10.0, 10.0, 50.0, 50.0, false));

            let mut doc_signal = Signal::new(doc);
            let mut history_signal = Signal::new(History::new());
            let mut did_resize = false;

            let mut originals = HashMap::new();
            originals.insert(node_id.clone(), (10.0, 10.0, 50.0, 50.0));

            handle_resizing(
                &mut doc_signal,
                &mut history_signal,
                30.0,
                40.0,
                &ResizeHandle::Se,
                &(10.0, 10.0, 50.0, 50.0),
                &originals,
                &(10.0, 10.0),
                &mut did_resize,
                &None,
            );

            assert!(did_resize);
            let doc = doc_signal.read();
            assert!(doc.document.nodes.contains_key(&node_id));
            if let Some(node) = doc.document.nodes.get(&node_id) {
                assert_eq!(node.x.0, 10.0);
                assert_eq!(node.y.0, 10.0);
                assert_eq!(node.width.0, 70.0);
                assert_eq!(node.height.0, 80.0);
            }
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_nw_handle_resize_then_position_and_size_change() {
        let mut vdom = VirtualDom::new(|| {
            let mut doc = DiagramDocument::default();
            doc.editor_state.zoom = OrderedFloat(1.0);
            doc.editor_state.camera_x = OrderedFloat(0.0);
            doc.editor_state.camera_y = OrderedFloat(0.0);
            let node_id = NodeId::new("n1".to_string());
            doc.document.nodes = doc
                .document
                .nodes
                .update(node_id.clone(), test_node(50.0, 50.0, 50.0, 50.0, false));

            let mut doc_signal = Signal::new(doc);
            let mut history_signal = Signal::new(History::new());
            let mut did_resize = true;

            let mut originals = HashMap::new();
            originals.insert(node_id.clone(), (50.0, 50.0, 50.0, 50.0));

            handle_resizing(
                &mut doc_signal,
                &mut history_signal,
                40.0,
                30.0,
                &ResizeHandle::Nw,
                &(50.0, 50.0, 50.0, 50.0),
                &originals,
                &(50.0, 50.0),
                &mut did_resize,
                &None,
            );

            let doc = doc_signal.read();
            assert!(doc.document.nodes.contains_key(&node_id));
            if let Some(node) = doc.document.nodes.get(&node_id) {
                assert_eq!(node.x.0, 40.0);
                assert_eq!(node.y.0, 30.0);
                assert_eq!(node.width.0, 60.0);
                assert_eq!(node.height.0, 70.0);
            }
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_shrink_below_minimum_then_clamps_to_24() {
        let mut vdom = VirtualDom::new(|| {
            let mut doc = DiagramDocument::default();
            doc.editor_state.zoom = OrderedFloat(1.0);
            doc.editor_state.camera_x = OrderedFloat(0.0);
            doc.editor_state.camera_y = OrderedFloat(0.0);
            let node_id = NodeId::new("n1".to_string());
            doc.document.nodes = doc
                .document
                .nodes
                .update(node_id.clone(), test_node(10.0, 10.0, 50.0, 50.0, false));

            let mut doc_signal = Signal::new(doc);
            let mut history_signal = Signal::new(History::new());
            let mut did_resize = true;

            let mut originals = HashMap::new();
            originals.insert(node_id.clone(), (10.0, 10.0, 50.0, 50.0));

            handle_resizing(
                &mut doc_signal,
                &mut history_signal,
                -40.0,
                -40.0,
                &ResizeHandle::Se,
                &(10.0, 10.0, 50.0, 50.0),
                &originals,
                &(10.0, 10.0),
                &mut did_resize,
                &None,
            );

            let doc = doc_signal.read();
            assert!(doc.document.nodes.contains_key(&node_id));
            if let Some(node) = doc.document.nodes.get(&node_id) {
                assert_eq!(node.width.0, 24.0);
                assert_eq!(node.height.0, 24.0);
            }
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_aspect_ratio_then_maintains_proportions() {
        let mut vdom = VirtualDom::new(|| {
            let mut doc = DiagramDocument::default();
            doc.editor_state.zoom = OrderedFloat(1.0);
            doc.editor_state.camera_x = OrderedFloat(0.0);
            doc.editor_state.camera_y = OrderedFloat(0.0);
            let node_id = NodeId::new("n1".to_string());
            doc.document.nodes = doc
                .document
                .nodes
                .update(node_id.clone(), test_node(10.0, 10.0, 100.0, 50.0, false));

            let mut doc_signal = Signal::new(doc);
            let mut history_signal = Signal::new(History::new());
            let mut did_resize = true;

            let mut originals = HashMap::new();
            originals.insert(node_id.clone(), (10.0, 10.0, 100.0, 50.0));

            let aspect_ratio = Some(2.0);

            handle_resizing(
                &mut doc_signal,
                &mut history_signal,
                110.0,
                60.0,
                &ResizeHandle::Se,
                &(10.0, 10.0, 100.0, 50.0),
                &originals,
                &(10.0, 10.0),
                &mut did_resize,
                &aspect_ratio,
            );

            let doc = doc_signal.read();
            assert!(doc.document.nodes.contains_key(&node_id));
            if let Some(node) = doc.document.nodes.get(&node_id) {
                assert_eq!(node.width.0, 200.0);
                assert_eq!(node.height.0, 100.0);
            }
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }

    #[test]
    fn given_snap_to_grid_then_snaps_resize() {
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
                .update(node_id.clone(), test_node(10.0, 10.0, 50.0, 50.0, false));

            let mut doc_signal = Signal::new(doc);
            let mut history_signal = Signal::new(History::new());
            let mut did_resize = true;

            let mut originals = HashMap::new();
            originals.insert(node_id.clone(), (10.0, 10.0, 50.0, 50.0));

            handle_resizing(
                &mut doc_signal,
                &mut history_signal,
                25.0,
                15.0,
                &ResizeHandle::Se,
                &(10.0, 10.0, 50.0, 50.0),
                &originals,
                &(10.0, 10.0),
                &mut did_resize,
                &None,
            );

            let doc = doc_signal.read();
            assert!(doc.document.nodes.contains_key(&node_id));
            if let Some(node) = doc.document.nodes.get(&node_id) {
                assert_eq!(node.width.0, 70.0);
                assert_eq!(node.height.0, 50.0);
            }
            rsx! { div {} }
        });
        let _ = vdom.rebuild_in_place();
    }
}
