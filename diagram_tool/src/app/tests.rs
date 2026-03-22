#[cfg(test)]
mod tests {
    use crate::app::validation::collect_validation_issues;
    use diagram_models::document::{
        DiagramDocument, LockState, Node, NodeId, NodeKind, OrderedFloat,
    };
    use im::Vector;

    fn create_test_node(id: &str) -> Node {
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
            parent: None,
            dag_rank: None,
            tags: Vector::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        }
    }

    #[test]
    fn given_empty_document_when_validating_then_no_issues_returned() {
        let doc = DiagramDocument::default();
        let issues = collect_validation_issues(&doc);
        // Schema requires empty arrays at minimum, which default provides
        assert!(issues.is_empty(), "Empty document should be valid");
    }

    #[test]
    fn given_document_with_invalid_node_width_when_validating_then_issues_returned() {
        let mut doc = DiagramDocument::default();
        let mut node = create_test_node("n1");
        node.width = OrderedFloat(-10.0); // Invalid width < 0
        doc.document
            .nodes
            .insert(NodeId::new("n1".to_string()), node);

        let issues = collect_validation_issues(&doc);
        assert!(
            !issues.is_empty(),
            "Document with negative width node must have validation issues"
        );
    }

    #[test]
    fn given_document_with_invalid_node_height_when_validating_then_issues_returned() {
        let mut doc = DiagramDocument::default();
        let mut node = create_test_node("n1");
        node.height = OrderedFloat(-10.0); // Invalid height < 0
        doc.document
            .nodes
            .insert(NodeId::new("n1".to_string()), node);

        let issues = collect_validation_issues(&doc);
        assert!(
            !issues.is_empty(),
            "Document with negative height node must have validation issues"
        );
    }

    #[test]
    fn given_document_with_valid_node_when_validating_then_no_issues() {
        let mut doc = DiagramDocument::default();
        let node = create_test_node("n1");
        doc.document
            .nodes
            .insert(NodeId::new("n1".to_string()), node);

        let issues = collect_validation_issues(&doc);
        assert!(
            issues.is_empty(),
            "Document with valid node should not have validation issues"
        );
    }
}
