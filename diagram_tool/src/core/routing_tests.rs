#[cfg(test)]
mod tests {
    use crate::core::routing::{create_edge, RoutingError};
    use crate::models::document::{DiagramDocument, EdgeId, Node, NodeId, NodeKind, OrderedFloat};

    fn test_node() -> Node {
        Node {
            kind: NodeKind::Text,
            icon: String::new(),
            label: "Test".to_string(),
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
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_returns_error_when_source_node_missing() {
        let mut doc = DiagramDocument::default();
        let t1 = NodeId::new("t1".to_string());
        doc.document.nodes.insert(t1.clone(), test_node());

        let err = create_edge(
            &mut doc,
            NodeId::new("s1".to_string()),
            t1,
            EdgeId::new("e1".to_string()),
        )
        .unwrap_err();
        assert_eq!(
            err,
            RoutingError::SourceNotFound(NodeId::new("s1".to_string()))
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_returns_error_when_attempting_self_loop() {
        let mut doc = DiagramDocument::default();
        let s1 = NodeId::new("s1".to_string());
        doc.document.nodes.insert(s1.clone(), test_node());

        let err = create_edge(
            &mut doc,
            s1.clone(),
            s1.clone(),
            EdgeId::new("e1".to_string()),
        )
        .unwrap_err();
        assert_eq!(err, RoutingError::SelfLoop(s1));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_adding_edge_that_creates_cycle_returns_cycle_detected_error() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("1".to_string());
        let n2 = NodeId::new("2".to_string());
        let n3 = NodeId::new("3".to_string());

        doc.document.nodes.insert(n1.clone(), test_node());
        doc.document.nodes.insert(n2.clone(), test_node());
        doc.document.nodes.insert(n3.clone(), test_node());

        create_edge(
            &mut doc,
            n1.clone(),
            n2.clone(),
            EdgeId::new("e1".to_string()),
        )
        .unwrap();
        create_edge(
            &mut doc,
            n2.clone(),
            n3.clone(),
            EdgeId::new("e2".to_string()),
        )
        .unwrap();

        let err = create_edge(
            &mut doc,
            n3.clone(),
            n1.clone(),
            EdgeId::new("e3".to_string()),
        )
        .unwrap_err();
        assert_eq!(err, RoutingError::CycleDetected);
    }
}
