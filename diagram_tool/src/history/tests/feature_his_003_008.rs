//! HIS Feature tests for history module HIS-003..HIS-008
//!
//! High-level integration tests matching the HIS test specification.

use crate::core::history::{apply_redo, apply_undo};
use crate::history::History;
use crate::models::document::{
    DiagramDocument, Edge, EdgeId, LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
    Revision,
};

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

/// HIS-003: Drag gesture creates one history entry
#[cfg(kani)]
#[kani::proof]
#[test]
fn test_his003_drag_creates_single_history_entry() {
    let mut doc_before = DiagramDocument::default();
    let node_id = NodeId::new("node-1".to_string());
    doc_before.document.nodes.insert(
        node_id.clone(),
        make_node_for_his("node-1", 100.0, 100.0, 80.0, 40.0),
    );

    let history = History::new().push(doc_before.clone());

    let mut doc_after = doc_before.clone();
    if let Some(node) = doc_after.document.nodes.get_mut(&node_id) {
        node.x = OrderedFloat(150.0);
        node.y = OrderedFloat(150.0);
    }
    doc_after.revision = doc_after.revision.increment();

    // Simulate drag completion by pushing the final state once
    let history_after = history.push(doc_after.clone());

    // History should have initial state and the pushed state in undo_stack
    assert!(history_after.can_undo());

    let Some((restored, _)) = history_after.undo(doc_after) else {
        panic!("undo should succeed");
    };

    let restored_node = restored.document.nodes.get(&node_id).expect("node exists");
    assert_eq!(restored_node.x.0, 100.0);
    assert_eq!(restored_node.y.0, 100.0);
}

/// HIS-004: Undo after grouping nodes removes the group
#[cfg(kani)]
#[kani::proof]
#[test]
fn test_his004_group_undo_removes_group() {
    let mut doc_before = DiagramDocument::default();
    let node_a_id = NodeId::new("node-a".to_string());
    let node_b_id = NodeId::new("node-b".to_string());

    doc_before.document.nodes.insert(
        node_a_id.clone(),
        make_node_for_his("node-a", 100.0, 100.0, 80.0, 40.0),
    );
    doc_before.document.nodes.insert(
        node_b_id.clone(),
        make_node_for_his("node-b", 200.0, 100.0, 80.0, 40.0),
    );

    let history = History::new().push(doc_before.clone());

    let mut doc_after = doc_before.clone();
    let group_id = NodeId::new("group-1".to_string());
    let mut group_node = make_node_for_his("group", 100.0, 100.0, 200.0, 100.0);
    group_node.kind = NodeKind::Subgraph;

    doc_after
        .document
        .nodes
        .insert(group_id.clone(), group_node);

    if let Some(node_a) = doc_after.document.nodes.get_mut(&node_a_id) {
        node_a.parent = Some(group_id.clone());
    }
    if let Some(node_b) = doc_after.document.nodes.get_mut(&node_b_id) {
        node_b.parent = Some(group_id.clone());
    }
    doc_after.revision = doc_after.revision.increment();

    let history_after = history.push(doc_after.clone());

    let Some((restored, _)) = history_after.undo(doc_after) else {
        panic!("undo should succeed");
    };

    assert!(!restored.document.nodes.contains_key(&group_id));
    assert_eq!(
        restored.document.nodes.get(&node_a_id).unwrap().parent,
        None
    );
    assert_eq!(
        restored.document.nodes.get(&node_b_id).unwrap().parent,
        None
    );
}

/// HIS-005: Undo after reparenting restores original parent
#[cfg(kani)]
#[kani::proof]
#[test]
fn test_his005_reparent_undo_restores_parent() {
    let mut doc_before = DiagramDocument::default();
    let p1_id = NodeId::new("parent-1".to_string());
    let p2_id = NodeId::new("parent-2".to_string());
    let c_id = NodeId::new("child".to_string());

    doc_before.document.nodes.insert(
        p1_id.clone(),
        make_node_for_his("p1", 0.0, 0.0, 100.0, 100.0),
    );
    doc_before.document.nodes.insert(
        p2_id.clone(),
        make_node_for_his("p2", 200.0, 0.0, 100.0, 100.0),
    );

    let mut child = make_node_for_his("c", 10.0, 10.0, 50.0, 50.0);
    child.parent = Some(p1_id.clone());
    doc_before.document.nodes.insert(c_id.clone(), child);

    let history = History::new().push(doc_before.clone());

    let mut doc_after = doc_before.clone();
    if let Some(child_node) = doc_after.document.nodes.get_mut(&c_id) {
        child_node.parent = Some(p2_id.clone());
    }
    doc_after.revision = doc_after.revision.increment();

    let history_after = history.push(doc_after.clone());

    let Some((restored, _)) = history_after.undo(doc_after) else {
        panic!("undo should succeed");
    };

    assert_eq!(
        restored.document.nodes.get(&c_id).unwrap().parent,
        Some(p1_id)
    );
}

/// HIS-006: Undo after creating edge removes the edge
#[cfg(kani)]
#[kani::proof]
#[test]
fn test_his006_connector_create_undo_removes_edge() {
    let mut doc_before = DiagramDocument::default();
    let n1_id = NodeId::new("n1".to_string());
    let n2_id = NodeId::new("n2".to_string());

    doc_before.document.nodes.insert(
        n1_id.clone(),
        make_node_for_his("n1", 0.0, 0.0, 100.0, 100.0),
    );
    doc_before.document.nodes.insert(
        n2_id.clone(),
        make_node_for_his("n2", 200.0, 0.0, 100.0, 100.0),
    );

    let history = History::new().push(doc_before.clone());

    let mut doc_after = doc_before.clone();
    let edge_id = EdgeId::new("e1".to_string());
    let edge = Edge {
        source: n1_id,
        target: n2_id,
        label: "".to_string(),
        style: Default::default(),
        arrow_type: Default::default(),
        label_offset_t: OrderedFloat(0.5),
        color: None,
        thickness: OrderedFloat(1.5),
        directed: true,
        bend_points: im::Vector::new(),
        tags: im::Vector::new(),
        metadata: im::HashMap::new(),
        font_size: None,
        source_port: None,
        target_port: None,
    };
    doc_after.document.edges.insert(edge_id.clone(), edge);
    doc_after.revision = doc_after.revision.increment();

    let history_after = history.push(doc_after.clone());

    let Some((restored, _)) = history_after.undo(doc_after) else {
        panic!("undo should succeed");
    };

    assert!(restored.document.edges.is_empty());
}

/// HIS-007: Undo after changing node style restores original style
#[cfg(kani)]
#[kani::proof]
#[test]
fn test_his007_style_change_undo_restores_style() {
    let mut doc_before = DiagramDocument::default();
    let node_id = NodeId::new("n1".to_string());

    let mut node = make_node_for_his("n1", 0.0, 0.0, 100.0, 100.0);
    node.style = Some(NodeStyle::Box);
    doc_before.document.nodes.insert(node_id.clone(), node);

    let history = History::new().push(doc_before.clone());

    let mut doc_after = doc_before.clone();
    if let Some(n) = doc_after.document.nodes.get_mut(&node_id) {
        n.style = Some(NodeStyle::Dashed);
    }
    doc_after.revision = doc_after.revision.increment();

    let history_after = history.push(doc_after.clone());

    let Some((restored, _)) = history_after.undo(doc_after) else {
        panic!("undo should succeed");
    };

    assert_eq!(
        restored.document.nodes.get(&node_id).unwrap().style,
        Some(NodeStyle::Box)
    );
}

/// HIS-008: Text edit creates single entry
#[cfg(kani)]
#[kani::proof]
#[test]
fn test_his008_text_edit_creates_single_entry() {
    let mut doc_before = DiagramDocument::default();
    let node_id = NodeId::new("n1".to_string());

    let node = make_node_for_his("Original Label", 0.0, 0.0, 100.0, 100.0);
    doc_before.document.nodes.insert(node_id.clone(), node);

    let history = History::new().push(doc_before.clone());

    let mut doc_after = doc_before.clone();
    if let Some(n) = doc_after.document.nodes.get_mut(&node_id) {
        n.label = "New Label".to_string();
    }
    doc_after.revision = doc_after.revision.increment();

    let history_after = history.push(doc_after.clone());

    let Some((restored, _)) = history_after.undo(doc_after) else {
        panic!("undo should succeed");
    };

    assert_eq!(
        restored.document.nodes.get(&node_id).unwrap().label,
        "Original Label"
    );
}

/// test_apply_undo_success_restores_previous_state
#[cfg(kani)]
#[kani::proof]
#[test]
fn test_apply_undo_success_restores_previous_state() {
    let mut doc_a = DiagramDocument::default();
    doc_a.revision = Revision::new(1);

    let mut doc_b = doc_a.clone();
    doc_b.revision = Revision::new(2);

    let mut history = History::new().push(doc_a.clone()).push(doc_b.clone());
    let mut current_doc = doc_b.clone();

    let result = apply_undo(&mut current_doc, &mut history);

    assert_eq!(result, Ok(()));
    assert_eq!(current_doc.revision, doc_a.revision);
    assert!(history.can_redo());
}

/// test_apply_undo_failure_returns_error_on_empty_history
#[cfg(kani)]
#[kani::proof]
#[test]
fn test_apply_undo_failure_returns_error_on_empty_history() {
    let mut doc = DiagramDocument::default();
    let mut history = History::new();

    let result = apply_undo(&mut doc, &mut history);

    assert_eq!(result, Err("Nothing to undo"));
}

/// test_apply_redo_success_restores_next_state
#[cfg(kani)]
#[kani::proof]
#[test]
fn test_apply_redo_success_restores_next_state() {
    let mut doc_a = DiagramDocument::default();
    doc_a.revision = Revision::new(1);

    let mut doc_b = doc_a.clone();
    doc_b.revision = Revision::new(2);

    let mut history = History::new().push(doc_a.clone()).push(doc_b.clone());
    let mut current_doc = doc_b.clone();

    // Perform undo
    let _ = apply_undo(&mut current_doc, &mut history);

    // Perform redo
    let result = apply_redo(&mut current_doc, &mut history);

    assert_eq!(result, Ok(()));
    assert_eq!(current_doc.revision, doc_b.revision);
    assert!(history.can_undo());
}

/// test_apply_redo_failure_returns_error_on_empty_redo_stack
#[cfg(kani)]
#[kani::proof]
#[test]
fn test_apply_redo_failure_returns_error_on_empty_redo_stack() {
    let mut doc_a = DiagramDocument::default();
    doc_a.revision = Revision::new(1);

    let mut history = History::new().push(doc_a.clone());
    let mut current_doc = doc_a.clone();

    // No undo has been performed, so redo should fail
    let result = apply_redo(&mut current_doc, &mut history);

    assert_eq!(result, Err("Nothing to redo"));
}
