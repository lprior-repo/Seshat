#[cfg(test)]
mod tests {
    use crate::export::svg::nodes::render_nodes;
    use diagram_models::document::{
        DiagramDocument, DocumentData, LockState, Node, NodeId, NodeKind, OrderedFloat,
    };
    use im::HashMap;

    fn create_test_node(id: &str, x: f64, y: f64) -> (NodeId, Node) {
        (
            NodeId::new(id.to_string()),
            Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: format!("Node {}", id),
                x: OrderedFloat(x),
                y: OrderedFloat(y),
                width: OrderedFloat(100.0),
                height: OrderedFloat(60.0),
                font_size: None,
                font_weight: None,
                lock_state: LockState::Unlocked,
                parent: None,
                dag_rank: None,
                tags: im::Vector::new(),
                metadata: HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            },
        )
    }

    #[test]
    fn test_render_nodes_generates_valid_svg_elements() {
        let mut nodes = im::HashMap::new();
        let (id, node) = create_test_node("1", 50.0, 75.0);
        nodes.insert(id, node);

        let doc = DiagramDocument {
            version: 2,
            revision: diagram_models::document::Revision::INITIAL,
            document: DocumentData {
                nodes,
                edges: im::HashMap::new(),
            },
            editor_state: Default::default(),
        };

        let mut svg = String::new();

        render_nodes(&doc, &mut svg);

        assert!(
            svg.contains("<g transform='translate(50, 75)'>"),
            "Missing group transform"
        );
        assert!(
            svg.contains(
                "<rect width='100' height='60' fill='white' stroke='black' rx='4' ry='4'/>"
            ),
            "Missing rect bounds"
        );
        assert!(svg.contains("Node 1"), "Missing node label");
        assert!(svg.contains("</g>"), "Missing closing group tag");
    }
}
