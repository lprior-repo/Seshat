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
    use crate::core::history::{apply_redo, apply_undo, HistoryError};
    use crate::history::History;
    use diagram_models::document::{DiagramDocument, Node, NodeId, NodeKind, OrderedFloat};
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
            lock_state: Default::default(),
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
    fn given_empty_history_when_undo_applied_then_returns_nothing_to_undo_error() {
        // GIVEN
        let mut doc = DiagramDocument::default();
        let mut history = History::default();

        // WHEN
        let result = apply_undo(&mut doc, &mut history);

        // THEN
        assert_eq!(result, Err(HistoryError::NothingToUndo));
    }

    #[test]
    fn given_empty_history_when_redo_applied_then_returns_nothing_to_redo_error() {
        // GIVEN
        let mut doc = DiagramDocument::default();
        let mut history = History::default();

        // WHEN
        let result = apply_redo(&mut doc, &mut history);

        // THEN
        assert_eq!(result, Err(HistoryError::NothingToRedo));
    }

    #[test]
    fn given_history_with_one_change_when_undo_applied_then_document_is_reverted() {
        // GIVEN
        let initial_doc = DiagramDocument::default();
        let history = History::default();

        // Push initial state
        let history = history.push(initial_doc.clone());

        // Make a change
        let mut doc_after_add = initial_doc.clone();
        doc_after_add.revision = doc_after_add.revision.increment();
        let n1 = NodeId::new("n1".to_string());
        doc_after_add
            .document
            .nodes
            .insert(n1.clone(), create_test_node("n1"));

        // Push changed state
        let mut current_history = history.push(doc_after_add.clone());
        let mut current_doc = doc_after_add;

        // WHEN
        let result = apply_undo(&mut current_doc, &mut current_history);

        // THEN
        assert_eq!(result, Ok(()));
        assert!(
            !current_doc.document.nodes.contains_key(&n1),
            "Document should be reverted to state before node was added"
        );
        assert_eq!(current_doc.revision, initial_doc.revision);
    }

    #[test]
    fn given_undone_state_when_redo_applied_then_change_is_restored() {
        // GIVEN
        let initial_doc = DiagramDocument::default();
        let history = History::default();
        let history = history.push(initial_doc.clone());

        let mut doc_after_add = initial_doc.clone();
        doc_after_add.revision = doc_after_add.revision.increment();
        let n1 = NodeId::new("n1".to_string());
        doc_after_add
            .document
            .nodes
            .insert(n1.clone(), create_test_node("n1"));

        let mut current_history = history.push(doc_after_add.clone());
        let mut current_doc = doc_after_add;

        apply_undo(&mut current_doc, &mut current_history).unwrap();

        // WHEN
        let result = apply_redo(&mut current_doc, &mut current_history);

        // THEN
        assert_eq!(result, Ok(()));
        assert!(
            current_doc.document.nodes.contains_key(&n1),
            "Document should have the node restored"
        );
    }

    #[test]
    fn given_multiple_changes_when_undone_and_redone_then_states_match_exactly() {
        // GIVEN
        let doc0 = DiagramDocument::default();
        let history = History::default();
        let h0 = history.push(doc0.clone());

        let mut doc1 = doc0.clone();
        doc1.revision = doc1.revision.increment();
        let n1 = NodeId::new("n1".to_string());
        doc1.document
            .nodes
            .insert(n1.clone(), create_test_node("n1"));
        let h1 = h0.push(doc1.clone());

        let mut doc2 = doc1.clone();
        doc2.revision = doc2.revision.increment();
        let n2 = NodeId::new("n2".to_string());
        doc2.document
            .nodes
            .insert(n2.clone(), create_test_node("n2"));
        let mut h_final = h1.push(doc2.clone());

        let mut active_doc = doc2.clone();

        // WHEN undo twice
        apply_undo(&mut active_doc, &mut h_final).unwrap();
        assert_eq!(active_doc.document.nodes.len(), 1); // Only n1

        apply_undo(&mut active_doc, &mut h_final).unwrap();
        assert_eq!(active_doc.document.nodes.len(), 0); // Empty

        // THEN redo twice
        apply_redo(&mut active_doc, &mut h_final).unwrap();
        assert_eq!(active_doc.document.nodes.len(), 1);
        assert!(active_doc.document.nodes.contains_key(&n1));

        apply_redo(&mut active_doc, &mut h_final).unwrap();
        assert_eq!(active_doc.document.nodes.len(), 2);
        assert!(active_doc.document.nodes.contains_key(&n1));
        assert!(active_doc.document.nodes.contains_key(&n2));
    }
}
