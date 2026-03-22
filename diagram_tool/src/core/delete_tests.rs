#[cfg(test)]
mod tests {
    use crate::core::delete::delete_selected;
    use diagram_models::document::{
        DiagramDocument, Edge, EdgeId, LockState, Node, NodeId, NodeKind, OrderedFloat,
    };
    use im::Vector;

    fn create_test_node(id: &str, parent: Option<NodeId>) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: id.to_string(),
            x: OrderedFloat(0.0),
            y: OrderedFloat(0.0),
            width: OrderedFloat(100.0),
            height: OrderedFloat(100.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent,
            dag_rank: None,
            tags: Vector::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        }
    }

    fn create_test_edge(source: NodeId, target: NodeId) -> Edge {
        Edge {
            source,
            target,
            label: String::new(),
            style: Default::default(),
            arrow_type: Default::default(),
            label_offset_t: OrderedFloat(0.5),
            directed: true,
            bend_points: Vector::new(),
            tags: Vector::new(),
            metadata: im::HashMap::new(),
            color: None,
            thickness: OrderedFloat(1.0),
            font_size: None,
            source_port: None,
            target_port: None,
        }
    }

    #[test]
    fn given_selected_node_when_deleted_then_connected_edges_removed() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());
        let e1 = EdgeId::new("e1".to_string());

        doc.document
            .nodes
            .insert(n1.clone(), create_test_node("n1", None));
        doc.document
            .nodes
            .insert(n2.clone(), create_test_node("n2", None));
        doc.document
            .edges
            .insert(e1.clone(), create_test_edge(n1.clone(), n2.clone()));

        doc.editor_state
            .selected_items
            .insert(n1.as_str().to_string());

        let changed = delete_selected(&mut doc);

        assert!(changed);
        assert!(!doc.document.nodes.contains_key(&n1));
        assert!(doc.document.nodes.contains_key(&n2));
        assert!(!doc.document.edges.contains_key(&e1));
        assert!(doc.editor_state.selected_items.is_empty());
    }

    #[test]
    fn given_selected_subgraph_when_deleted_then_children_reparented_to_none() {
        let mut doc = DiagramDocument::default();
        let sg = NodeId::new("sg".to_string());
        let child = NodeId::new("child".to_string());

        let mut parent_node = create_test_node("sg", None);
        parent_node.kind = NodeKind::Subgraph;

        doc.document.nodes.insert(sg.clone(), parent_node);
        doc.document
            .nodes
            .insert(child.clone(), create_test_node("child", Some(sg.clone())));

        doc.editor_state
            .selected_items
            .insert(sg.as_str().to_string());

        delete_selected(&mut doc);

        assert!(!doc.document.nodes.contains_key(&sg));
        let surviving_child = doc.document.nodes.get(&child).unwrap();
        assert_eq!(surviving_child.parent, None);
    }
}
