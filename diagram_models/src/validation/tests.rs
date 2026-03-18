#[cfg(test)]
mod tests {
    use crate::document::DiagramDocument;
    use crate::document::{
        ArrowType, Edge, EdgeId, EdgeStyle, LockState, Node, NodeId, NodeKind, NodeStyle,
        OrderedFloat,
    };
    use crate::validation::{validate_document, ValidationCode};
    use im::HashMap;

    fn make_node(id: &str) -> (NodeId, Node) {
        (
            NodeId::new(id.to_string()),
            Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: id.to_string(),
                x: OrderedFloat(0.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(64.0),
                height: OrderedFloat(64.0),
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
            },
        )
    }

    fn make_edge(id: &str, src: &str, tgt: &str) -> (EdgeId, Edge) {
        (
            EdgeId::new(id.to_string()),
            Edge {
                source: NodeId::new(src.to_string()),
                target: NodeId::new(tgt.to_string()),
                label: String::new(),
                style: EdgeStyle::default(),
                arrow_type: ArrowType::default(),
                label_offset_t: OrderedFloat(0.5),
                color: None,
                thickness: OrderedFloat(1.5),
                directed: true,
                bend_points: im::Vector::new(),
                tags: im::Vector::new(),
                metadata: HashMap::new(),
                font_size: None,
                source_port: None,
                target_port: None,
            },
        )
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_edge_to_nonexistent_node_when_validated_then_edge_dangling_error() {
        let mut doc = DiagramDocument::default();
        let (nid, node) = make_node("A");
        doc.document.nodes = doc.document.nodes.update(nid, node);
        let (eid, edge) = make_edge("e1", "A", "MISSING");
        doc.document.edges = doc.document.edges.update(eid, edge);

        let issues = validate_document(&doc);
        assert!(issues
            .iter()
            .any(|i| i.code == ValidationCode::EDGE_DANGLING));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_cycle_when_validated_then_dag_cycle_error() {
        let mut doc = DiagramDocument::default();
        let (aid, a) = make_node("A");
        let (bid, b) = make_node("B");
        doc.document.nodes = doc.document.nodes.update(aid, a).update(bid, b);
        let (e1id, e1) = make_edge("e1", "A", "B");
        let (e2id, e2) = make_edge("e2", "B", "A");
        doc.document.edges = doc.document.edges.update(e1id, e1).update(e2id, e2);

        let issues = validate_document(&doc);
        assert!(issues.iter().any(|i| i.code == ValidationCode::DAG_CYCLE));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_node_with_non_subgraph_parent_when_validated_then_invalid_parent_error() {
        let mut doc = DiagramDocument::default();
        let (aid, a) = make_node("A"); // kind: Node (not Subgraph)
        let (bid, mut b) = make_node("B");
        b.parent = Some(NodeId::new("A".to_string()));
        doc.document.nodes = doc.document.nodes.update(aid, a).update(bid, b);

        let issues = validate_document(&doc);
        assert!(issues
            .iter()
            .any(|i| i.code == ValidationCode::INVALID_PARENT));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_node_with_existing_subgraph_parent_when_validated_then_no_invalid_parent_issue() {
        let mut doc = DiagramDocument::default();
        let (parent_id, mut parent) = make_node("P");
        parent.kind = NodeKind::Subgraph;
        let (child_id, mut child) = make_node("C");
        child.parent = Some(parent_id.clone());
        doc.document.nodes = doc
            .document
            .nodes
            .update(parent_id, parent)
            .update(child_id, child);

        let issues = validate_document(&doc);
        assert!(!issues
            .iter()
            .any(|i| i.code == ValidationCode::INVALID_PARENT));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_valid_document_when_validated_then_no_issues() {
        let doc = DiagramDocument::default();
        let issues = validate_document(&doc);
        assert!(issues.is_empty());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_nan_node_geometry_when_validated_then_returns_error() {
        let mut doc = DiagramDocument::default();
        let (nid, mut node) = make_node("nan-node");
        node.x = OrderedFloat::new_unchecked(f64::NAN);
        node.y = OrderedFloat::new_unchecked(f64::NAN);
        node.width = OrderedFloat::new_unchecked(f64::NAN);
        node.height = OrderedFloat::new_unchecked(f64::NAN);
        doc.document.nodes = doc.document.nodes.update(nid, node);

        let issues = validate_document(&doc);
        assert!(
            issues
                .iter()
                .any(|i| i.code == ValidationCode::INVALID_NUMERIC),
            "Validation should report invalid-numeric for NaN geometry"
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_inf_node_geometry_when_validated_then_returns_error() {
        let mut doc = DiagramDocument::default();
        let (nid, mut node) = make_node("inf-node");
        node.x = OrderedFloat::new_unchecked(f64::INFINITY);
        node.y = OrderedFloat::new_unchecked(f64::NEG_INFINITY);
        node.width = OrderedFloat::new_unchecked(f64::INFINITY);
        node.height = OrderedFloat::new_unchecked(f64::INFINITY);
        doc.document.nodes = doc.document.nodes.update(nid, node);

        let issues = validate_document(&doc);
        assert!(
            issues
                .iter()
                .any(|i| i.code == ValidationCode::INVALID_NUMERIC),
            "Validation should report invalid-numeric for Inf geometry"
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_negative_node_dimensions_when_validated_then_returns_error() {
        let mut doc = DiagramDocument::default();
        let (nid, mut node) = make_node("neg-node");
        node.width = OrderedFloat::new_unchecked(-10.0);
        node.height = OrderedFloat::new_unchecked(-5.0);
        doc.document.nodes = doc.document.nodes.update(nid, node);

        let issues = validate_document(&doc);
        assert!(
            issues
                .iter()
                .any(|i| i.code == ValidationCode::INVALID_NUMERIC),
            "Validation should report invalid-numeric for negative dimensions"
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_valid_node_minimum_size_when_validated_then_accepts() {
        let mut doc = DiagramDocument::default();
        let (nid, node) = make_node("small-valid");
        let small_node = Node {
            width: OrderedFloat::new_unchecked(24.0),
            height: OrderedFloat::new_unchecked(24.0),
            ..node
        };
        doc.document.nodes = doc.document.nodes.update(nid, small_node);
        let issues = validate_document(&doc);
        assert!(issues
            .iter()
            .all(|i| i.code != ValidationCode::INTERNAL_ERROR));
    }
}
