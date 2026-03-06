#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::clipboard::{copy_selection, ClipboardError};
    use crate::models::document::{DiagramDocument, Node, NodeId, NodeKind, OrderedFloat};

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

    #[test]
    fn test_copy_with_empty_selection_returns_error() {
        let doc = DiagramDocument::default();
        let result = copy_selection(&doc);
        assert_eq!(result.unwrap_err(), ClipboardError::EmptySelection);
    }

    #[test]
    fn test_copy_single_node_populates_clipboard() {
        let mut doc = DiagramDocument::default();
        let n1 = NodeId::new("n1".to_string());
        doc.document.nodes.insert(n1.clone(), test_node());
        doc.editor_state
            .selected_items
            .insert(n1.as_str().to_string());

        let clipboard = copy_selection(&doc).unwrap();
        assert_eq!(clipboard.nodes.len(), 1);
        assert_eq!(clipboard.nodes[0].0, n1);
        assert_eq!(clipboard.edges.len(), 0);
    }
}
