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
    use crate::layout::grid::algorithm::calculate_grid_layout;
    use diagram_models::document::{
        DiagramDocument, LockState, Node, NodeId, NodeKind, OrderedFloat,
    };
    use im::HashMap;

    fn create_test_node(x: f64, y: f64, locked: bool, parent: Option<NodeId>) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: String::new(),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(100.0),
            height: OrderedFloat(100.0),
            font_size: None,
            font_weight: None,
            lock_state: if locked {
                LockState::Locked
            } else {
                LockState::Unlocked
            },
            parent,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        }
    }

    #[test]
    fn given_empty_document_when_calculate_grid_then_returns_empty_document() {
        let doc = DiagramDocument::default();
        let result = calculate_grid_layout(&doc, 100.0);
        assert!(result.document.nodes.is_empty());
    }

    #[test]
    fn given_only_locked_nodes_when_calculate_grid_then_returns_unchanged() -> Result<(), String> {
        let mut doc = DiagramDocument::default();
        doc.document.nodes.insert(
            NodeId::new("n1".to_string()),
            create_test_node(0.0, 0.0, true, None),
        );
        let result = calculate_grid_layout(&doc, 100.0);
        assert_eq!(
            result
                .document
                .nodes
                .get(&NodeId::new("n1".to_string()))
                .ok_or("Node not found")?
                .x
                .0,
            0.0
        );
        Ok(())
    }

    #[test]
    fn given_unlocked_nodes_when_calculate_grid_then_positions_in_grid() -> Result<(), String> {
        let mut doc = DiagramDocument::default();
        doc.document.nodes.insert(
            NodeId::new("n1".to_string()),
            create_test_node(15.0, 25.0, false, None),
        );
        doc.document.nodes.insert(
            NodeId::new("n2".to_string()),
            create_test_node(35.0, 45.0, false, None),
        );
        doc.document.nodes.insert(
            NodeId::new("n3".to_string()),
            create_test_node(55.0, 65.0, false, None),
        );

        // 3 nodes -> sqrt(3) ceil -> 2 columns
        let result = calculate_grid_layout(&doc, 100.0);

        let n1 = result
            .document
            .nodes
            .get(&NodeId::new("n1".to_string()))
            .ok_or("n1 not found")?;
        let n2 = result
            .document
            .nodes
            .get(&NodeId::new("n2".to_string()))
            .ok_or("n2 not found")?;
        let n3 = result
            .document
            .nodes
            .get(&NodeId::new("n3".to_string()))
            .ok_or("n3 not found")?;

        // Check that they are snapped to 100.0 increments
        assert_eq!(n1.x.0 % 100.0, 0.0);
        assert_eq!(n1.y.0 % 100.0, 0.0);
        assert_eq!(n2.x.0 % 100.0, 0.0);
        assert_eq!(n2.y.0 % 100.0, 0.0);
        assert_eq!(n3.x.0 % 100.0, 0.0);
        assert_eq!(n3.y.0 % 100.0, 0.0);

        Ok(())
    }

    #[test]
    fn given_locked_nodes_occupying_cells_when_calculate_grid_then_unlocked_nodes_avoid_them(
    ) -> Result<(), String> {
        let mut doc = DiagramDocument::default();
        // Occupies 0,0
        doc.document.nodes.insert(
            NodeId::new("locked".to_string()),
            create_test_node(0.0, 0.0, true, None),
        );
        // Needs a spot
        doc.document.nodes.insert(
            NodeId::new("unlocked".to_string()),
            create_test_node(0.0, 0.0, false, None),
        );

        let result = calculate_grid_layout(&doc, 100.0);

        let unlocked = result
            .document
            .nodes
            .get(&NodeId::new("unlocked".to_string()))
            .ok_or("unlocked not found")?;
        // Should not be at 0,0
        assert!(unlocked.x.0 != 0.0 || unlocked.y.0 != 0.0);

        Ok(())
    }

    #[test]
    fn given_nodes_with_parents_when_calculate_grid_then_maintains_relative_offsets(
    ) -> Result<(), String> {
        let mut doc = DiagramDocument::default();
        let parent_id = NodeId::new("parent".to_string());
        let child_id = NodeId::new("child".to_string());

        doc.document
            .nodes
            .insert(parent_id.clone(), create_test_node(10.0, 10.0, false, None));
        doc.document.nodes.insert(
            child_id.clone(),
            create_test_node(20.0, 20.0, false, Some(parent_id.clone())),
        );

        let result = calculate_grid_layout(&doc, 100.0);

        let parent = result
            .document
            .nodes
            .get(&parent_id)
            .ok_or("parent not found")?;
        let child = result
            .document
            .nodes
            .get(&child_id)
            .ok_or("child not found")?;

        // Parent moved to a grid intersection (e.g. 0,0)
        let parent_dx = parent.x.0 - 10.0;
        let parent_dy = parent.y.0 - 10.0;

        // Child should have moved by the same delta
        assert_eq!(child.x.0, 20.0 + parent_dx);
        assert_eq!(child.y.0, 20.0 + parent_dy);

        Ok(())
    }

    #[test]
    #[should_panic(expected = "cell_size must be positive and finite")]
    fn given_invalid_cell_size_when_calculate_grid_then_panics() {
        let doc = DiagramDocument::default();
        let _ = calculate_grid_layout(&doc, -10.0);
    }
}
