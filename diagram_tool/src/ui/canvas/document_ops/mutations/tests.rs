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
    use super::super::rubber_band::apply_rubber_band_release;
    use diagram_models::document::{
        DiagramDocument, LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
    };
    use im::HashMap;

    fn node_at(x: f64, y: f64) -> Node {
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
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    fn given_rubber_band_release_when_applied_then_selection_is_committed() {
        let mut doc = DiagramDocument::default();
        let node_id = NodeId::new(String::from("n1"));
        doc.document.nodes = doc
            .document
            .nodes
            .update(node_id.clone(), node_at(10.0, 10.0));

        apply_rubber_band_release(&mut doc, (0.0, 0.0), (80.0, 80.0), false);

        assert!(doc
            .editor_state
            .selected_items
            .contains(&node_id.to_string()));
    }

    #[cfg(kani)]
    #[kani::proof]
    fn given_noop_rubber_band_when_released_then_selection_is_preserved() {
        let mut doc = DiagramDocument::default();
        let node_id = NodeId::new(String::from("n1"));
        doc.document.nodes = doc
            .document
            .nodes
            .update(node_id.clone(), node_at(10.0, 10.0));
        doc.editor_state.selected_items =
            doc.editor_state.selected_items.update(node_id.to_string());

        apply_rubber_band_release(&mut doc, (10.0, 10.0), (10.0, 10.0), false);

        assert!(doc
            .editor_state
            .selected_items
            .contains(&node_id.to_string()));
    }

    #[cfg(kani)]
    #[kani::proof]
    fn given_existing_selection_when_rubber_band_released_then_selection_is_cleared() {
        let mut doc = DiagramDocument::default();
        // Create two nodes
        let node1_id = NodeId::new(String::from("n1"));
        let node2_id = NodeId::new(String::from("n2"));
        doc.document.nodes = doc
            .document
            .nodes
            .update(node1_id.clone(), node_at(10.0, 10.0))
            .update(node2_id.clone(), node_at(100.0, 100.0));
        // Select node1 first
        doc.editor_state.selected_items =
            doc.editor_state.selected_items.update(node1_id.to_string());

        // Apply rubber band that only contains node2
        apply_rubber_band_release(&mut doc, (50.0, 50.0), (150.0, 150.0), false);

        // Selection should be cleared and only node2 should be selected
        assert!(!doc
            .editor_state
            .selected_items
            .contains(&node1_id.to_string()));
        assert!(doc
            .editor_state
            .selected_items
            .contains(&node2_id.to_string()));
    }
}
