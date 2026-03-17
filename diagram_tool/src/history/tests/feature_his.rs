//! HIS Feature tests for history module
//!
//! High-level integration tests matching the HIS test specification.

#[cfg(kani)]
use crate::history::History;
#[cfg(kani)]
use diagram_models::document::{DiagramDocument, Node, NodeId, NodeKind, OrderedFloat, Revision};

#[cfg(kani)]
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

/// HIS-001: Move node undo restores original position
#[cfg(kani)]
#[kani::proof]
#[test]
fn given_node_at_position_when_moved_and_undo_then_position_restored() {
    let mut doc_before = DiagramDocument::default();
    let node_id = NodeId::new("node-1".to_string());
    let _ = doc_before.document.nodes.insert(
        node_id.clone(),
        make_node_for_his("node-1", 100.0, 100.0, 80.0, 40.0),
    );

    // Push the initial state (this is what undo will restore to)
    let history = History::new().push(doc_before.clone());

    // Move the node (this is the current state after the operation)
    let mut doc_after = doc_before.clone();
    if let Some(node) = doc_after.document.nodes.get_mut(&node_id) {
        node.x = OrderedFloat(200.0);
        node.y = OrderedFloat(200.0);
    }
    doc_after.revision = doc_after.revision.increment();

    // Undo should restore the initial position
    let Some((restored, _)) = history.undo(doc_after) else {
        panic!("undo should succeed");
    };

    let restored_node = restored
        .document
        .nodes
        .get(&node_id)
        .expect("node should exist");
    assert_eq!(restored_node.x.0, 100.0, "x should be restored to 100.0");
    assert_eq!(restored_node.y.0, 100.0, "y should be restored to 100.0");
}

/// HIS-002: Resize undo restores exact original dimensions
#[cfg(kani)]
#[kani::proof]
#[test]
fn given_node_with_dimensions_when_resized_and_undo_then_dimensions_restored() {
    let mut doc_before = DiagramDocument::default();
    let node_id = NodeId::new("node-1".to_string());
    let _ = doc_before.document.nodes.insert(
        node_id.clone(),
        make_node_for_his("node-1", 100.0, 100.0, 80.0, 40.0),
    );

    // Push the initial state (this is what undo will restore to)
    let history = History::new().push(doc_before.clone());

    // Resize the node (this is the current state after the operation)
    let mut doc_after = doc_before.clone();
    if let Some(node) = doc_after.document.nodes.get_mut(&node_id) {
        node.width = OrderedFloat(160.0);
        node.height = OrderedFloat(80.0);
    }
    doc_after.revision = doc_after.revision.increment();

    // Undo should restore original dimensions
    let Some((restored, _)) = history.undo(doc_after) else {
        panic!("undo should succeed");
    };

    let restored_node = restored
        .document
        .nodes
        .get(&node_id)
        .expect("node should exist");
    assert_eq!(
        restored_node.width.0, 80.0,
        "width should be restored to 80.0"
    );
    assert_eq!(
        restored_node.height.0, 40.0,
        "height should be restored to 40.0"
    );
}

/// HIS-011: Push after undo clears redo stack
#[cfg(kani)]
#[kani::proof]
#[test]
fn given_history_with_redo_entries_when_push_then_redo_stack_cleared() {
    let mut doc1 = DiagramDocument::default();
    let node_id = NodeId::new("node-1".to_string());
    let _ = doc1.document.nodes.insert(
        node_id.clone(),
        make_node_for_his("node-1", 100.0, 100.0, 80.0, 40.0),
    );

    let history = History::new()
        .push(doc1.clone())
        .push({
            let mut d = doc1.clone();
            if let Some(n) = d.document.nodes.get_mut(&node_id) {
                n.x = OrderedFloat(200.0);
            }
            d.revision = d.revision.increment();
            d
        })
        .push({
            let mut d = doc1.clone();
            if let Some(n) = d.document.nodes.get_mut(&node_id) {
                n.x = OrderedFloat(300.0);
            }
            d.revision = d.revision.increment();
            d
        });

    // Undo to create redo entries
    let current = {
        let mut d = doc1.clone();
        if let Some(n) = d.document.nodes.get_mut(&node_id) {
            n.x = OrderedFloat(400.0);
        }
        d.revision = d.revision.increment();
        d
    };

    let Some((_, after_undo)) = history.undo(current.clone()) else {
        panic!("undo should succeed");
    };
    assert!(
        !after_undo.can_redo() == false,
        "redo stack should have entries after undo"
    );

    // Push a new state - redo stack should be cleared
    let new_doc = {
        let mut d = doc1.clone();
        if let Some(n) = d.document.nodes.get_mut(&node_id) {
            n.x = OrderedFloat(500.0);
        }
        d.revision = d.revision.increment();
        d
    };
    let after_push = after_undo.push(new_doc);

    assert!(
        after_push.can_redo() == false,
        "redo stack should be empty after push"
    );
}

/// HIS-012: Multiple undos walk back through history correctly
#[cfg(kani)]
#[kani::proof]
#[test]
fn given_history_with_multiple_states_when_undo_multiple_times_then_walks_back_correctly() {
    let mut doc_a = DiagramDocument::default();
    let node_id = NodeId::new("node-1".to_string());
    let _ = doc_a.document.nodes.insert(
        node_id.clone(),
        make_node_for_his("node-1", 100.0, 100.0, 80.0, 40.0),
    );
    doc_a.revision = Revision::INITIAL;

    let doc_b = {
        let mut d = doc_a.clone();
        if let Some(n) = d.document.nodes.get_mut(&node_id) {
            n.x = OrderedFloat(200.0);
        }
        d.revision = d.revision.increment();
        d
    };

    let doc_c = {
        let mut d = doc_b.clone();
        if let Some(n) = d.document.nodes.get_mut(&node_id) {
            n.x = OrderedFloat(300.0);
        }
        d.revision = d.revision.increment();
        d
    };

    let current = {
        let mut d = doc_c.clone();
        if let Some(n) = d.document.nodes.get_mut(&node_id) {
            n.x = OrderedFloat(400.0);
        }
        d.revision = d.revision.increment();
        d
    };

    let history = History::new()
        .push(doc_a.clone())
        .push(doc_b.clone())
        .push(doc_c.clone());

    // First undo -> C
    let Some((state_c, history_after_1)) = history.undo(current.clone()) else {
        panic!("first undo should succeed");
    };
    let node_c = state_c
        .document
        .nodes
        .get(&node_id)
        .expect("node should exist");
    assert_eq!(
        node_c.x.0, 300.0,
        "first undo should restore state C (x=300)"
    );

    // Second undo -> B
    let Some((state_b, history_after_2)) = history_after_1.undo(state_c.clone()) else {
        panic!("second undo should succeed");
    };
    let node_b = state_b
        .document
        .nodes
        .get(&node_id)
        .expect("node should exist");
    assert_eq!(
        node_b.x.0, 200.0,
        "second undo should restore state B (x=200)"
    );

    // Third undo -> A
    let Some((state_a, _)) = history_after_2.undo(state_b.clone()) else {
        panic!("third undo should succeed");
    };
    let node_a = state_a
        .document
        .nodes
        .get(&node_id)
        .expect("node should exist");
    assert_eq!(
        node_a.x.0, 100.0,
        "third undo should restore state A (x=100)"
    );
}

/// HIS-013: Redo after multiple undos works correctly
#[cfg(kani)]
#[kani::proof]
#[test]
fn given_history_after_multiple_undos_when_redo_then_walks_forward_correctly() {
    let mut doc_a = DiagramDocument::default();
    let node_id = NodeId::new("node-1".to_string());
    let _ = doc_a.document.nodes.insert(
        node_id.clone(),
        make_node_for_his("node-1", 100.0, 100.0, 80.0, 40.0),
    );
    doc_a.revision = Revision::INITIAL;

    let doc_b = {
        let mut d = doc_a.clone();
        if let Some(n) = d.document.nodes.get_mut(&node_id) {
            n.x = OrderedFloat(200.0);
        }
        d.revision = d.revision.increment();
        d
    };

    let doc_c = {
        let mut d = doc_b.clone();
        if let Some(n) = d.document.nodes.get_mut(&node_id) {
            n.x = OrderedFloat(300.0);
        }
        d.revision = d.revision.increment();
        d
    };

    let current = {
        let mut d = doc_c.clone();
        if let Some(n) = d.document.nodes.get_mut(&node_id) {
            n.x = OrderedFloat(400.0);
        }
        d.revision = d.revision.increment();
        d
    };

    let history = History::new()
        .push(doc_a.clone())
        .push(doc_b.clone())
        .push(doc_c.clone());

    // Undo twice (now at B)
    let Some((state_c, history_after_1)) = history.undo(current.clone()) else {
        panic!("first undo should succeed");
    };
    let Some((state_b, history_after_2)) = history_after_1.undo(state_c.clone()) else {
        panic!("second undo should succeed");
    };
    let node_b = state_b
        .document
        .nodes
        .get(&node_id)
        .expect("node should exist");
    assert_eq!(node_b.x.0, 200.0, "should be at state B (x=200)");

    // Redo once -> C
    let Some((state_c_again, history_after_redo1)) = history_after_2.redo(state_b.clone()) else {
        panic!("first redo should succeed");
    };
    let node_c = state_c_again
        .document
        .nodes
        .get(&node_id)
        .expect("node should exist");
    assert_eq!(
        node_c.x.0, 300.0,
        "first redo should restore state C (x=300)"
    );

    // Redo again -> current (400)
    let Some((state_current, _)) = history_after_redo1.redo(state_c_again.clone()) else {
        panic!("second redo should succeed");
    };
    let node_final = state_current
        .document
        .nodes
        .get(&node_id)
        .expect("node should exist");
    assert_eq!(
        node_final.x.0, 400.0,
        "second redo should restore current state (x=400)"
    );
}
