#![allow(clippy::field_reassign_with_default, clippy::nonminimal_bool, clippy::bool_comparison)]
//! Integration tests for history module
//!
//! End-to-end tests for the complete history workflow.

#[cfg(test)]
use crate::history::History;
#[cfg(test)]
use diagram_models::document::{DiagramDocument, LockState, Node, NodeId, NodeKind, OrderedFloat};

#[cfg(test)]
fn make_node_for_his(label: &str, x: f64, y: f64, width: f64, height: f64) -> Node {
    Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: label.to_string(),
        x: OrderedFloat(x),
        y: OrderedFloat(y),
        width: OrderedFloat(width),
        height: OrderedFloat(height),
        font_size: None,
        font_weight: None,
        lock_state: LockState::Unlocked,
        parent: None,
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: im::HashMap::new(),
        z_index: 0,
        style: None,
        collapsed: None,
    }
}

/// Full end-to-end history workflow
#[test]
fn test_integration_e2e_full_history_workflow() {
    let mut doc = DiagramDocument::default();
    let node_id = NodeId::new("node-1".to_string());
    let _ = doc.document.nodes.insert(
        node_id.clone(),
        make_node_for_his("node-1", 0.0, 0.0, 80.0, 40.0),
    );

    // Initialize history with initial state
    // This counts as 1 entry in undo_stack
    let mut history = History::new().push(doc.clone());
    assert_eq!(
        history.undo_stack_len(),
        1,
        "After initial push: undo_stack.len() = 1"
    );

    // Step 1: Move to (100, 100) and push
    if let Some(node) = doc.document.nodes.get_mut(&node_id) {
        node.x = OrderedFloat(100.0);
        node.y = OrderedFloat(100.0);
    }
    doc.revision = doc.revision.increment();
    history = history.push(doc.clone());
    assert_eq!(
        history.undo_stack_len(),
        2,
        "After step 1: undo_stack.len() = 2 (initial + step1)"
    );

    // Step 2: Move to (200, 200) and push
    if let Some(node) = doc.document.nodes.get_mut(&node_id) {
        node.x = OrderedFloat(200.0);
        node.y = OrderedFloat(200.0);
    }
    doc.revision = doc.revision.increment();
    history = history.push(doc.clone());
    assert_eq!(
        history.undo_stack_len(),
        3,
        "After step 2: undo_stack.len() = 3 (initial + step1 + step2)"
    );

    // Step 3: Undo
    let Some((doc_restored, h)) = history.undo(doc.clone()) else {
        panic!("undo should succeed");
    };
    doc = doc_restored;
    history = h;
    assert!(history.can_redo(), "After step 3: can_redo() = true");
    assert_eq!(
        history.undo_stack_len(),
        1,
        "After step 3: undo_stack.len() = 1"
    );

    // Step 4: Undo again
    let Some((doc_restored, h)) = history.undo(doc.clone()) else {
        panic!("undo should succeed");
    };
    doc = doc_restored;
    history = h;
    assert!(history.can_redo(), "After step 4: can_redo() = true");
    assert_eq!(
        history.undo_stack_len(),
        0,
        "After step 4: undo_stack.len() = 0"
    );

    // Step 5: Redo
    let Some((doc_redo, h)) = history.redo(doc.clone()) else {
        panic!("redo should succeed");
    };
    doc = doc_redo;
    history = h;
    assert!(history.can_undo(), "After step 5: can_undo() = true");
    assert_eq!(
        history.undo_stack_len(),
        1,
        "After step 5: undo_stack.len() = 1 (current added to undo_stack)"
    );

    // Step 6: Redo again
    let Some((doc_redo, h)) = history.redo(doc.clone()) else {
        panic!("redo should succeed");
    };
    doc = doc_redo;
    history = h;
    assert!(history.can_undo(), "After step 6: can_undo() = true");
    assert_eq!(
        history.undo_stack_len(),
        2,
        "After step 6: undo_stack.len() = 2 (current added to undo_stack)"
    );

    // Verify final position
    let node = doc.document.nodes.get(&node_id).expect("node should exist");
    assert_eq!(node.x.0, 200.0, "Final document position should be x=200");
    assert_eq!(node.y.0, 200.0, "Final document position should be y=200");
}

/// Test push after undo clears redo stack (Contract Q1)
#[test]
fn test_postcondition_q1_redo_stack_empty_after_push() {
    use diagram_models::document::Revision;

    // History with push(A), push(B), undo (back to A, redo has B)
    let history = History::new()
        .push({
            let mut doc = DiagramDocument::default();
            doc.revision = Revision::INITIAL;
            doc
        })
        .push({
            let mut doc = DiagramDocument::default();
            doc.revision = doc.revision.increment();
            doc
        });

    let mut current = DiagramDocument::default();
    current.revision = current.revision.increment().increment().increment();
    let Some((_, after_undo)) = history.undo(current) else {
        panic!("undo should succeed");
    };

    assert!(
        !after_undo.can_redo() == false,
        "redo stack should have entries"
    );

    // push(C) should clear redo stack
    let after_push = after_undo.push(DiagramDocument::default());

    assert!(
        after_push.can_redo() == false,
        "redo stack should be empty after push"
    );
}

/// Test invariant I3: After push, redo stack is empty
#[test]
fn test_invariant_i3_after_push_redo_stack_is_empty() {
    use diagram_models::document::Revision;

    let history = History::new()
        .push({
            let mut doc = DiagramDocument::default();
            doc.revision = Revision::INITIAL;
            doc
        })
        .push({
            let mut doc = DiagramDocument::default();
            doc.revision = doc.revision.increment();
            doc
        });

    let mut current = DiagramDocument::default();
    current.revision = current.revision.increment().increment().increment();
    let Some((_, after_undo)) = history.undo(current) else {
        panic!("undo should succeed");
    };

    assert!(!after_undo.can_redo() == false, "redo should have B");

    // push(C) creates new timeline branch
    let after_push = after_undo.push(DiagramDocument::default());

    assert!(
        after_push.can_redo() == false,
        "redo stack should be empty after push"
    );
}
