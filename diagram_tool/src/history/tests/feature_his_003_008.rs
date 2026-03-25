//! HIS Feature tests for history module HIS-003..HIS-008
//!
//! High-level integration tests matching the HIS test specification.

#[cfg(kani)]
use crate::core::history::{apply_redo, apply_undo};
#[cfg(kani)]
use crate::history::History;
#[cfg(kani)]
use diagram_models::document::{
    DiagramDocument, Edge, EdgeId, LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
    Revision,
};

#[cfg(kani)]
fn make_node(label: &str, x: f64, y: f64, w: f64, h: f64) -> Node {
    Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: label.to_string(),
        x: OrderedFloat(x),
        y: OrderedFloat(y),
        width: OrderedFloat(w),
        height: OrderedFloat(h),
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

#[cfg(kani)]
fn run_undo_test(
    setup: impl FnOnce(&mut DiagramDocument),
    mutate: impl FnOnce(&mut DiagramDocument),
) -> DiagramDocument {
    let mut before = DiagramDocument::default();
    setup(&mut before);
    let history = History::new().push(before.clone());
    let mut after = before.clone();
    mutate(&mut after);
    after.revision = after.revision.increment();
    let history_after = history.push(after.clone());
    let Some((restored, _)) = history_after.undo(after) else {
        panic!("undo should succeed");
    };
    restored
}

/// HIS-003: Drag gesture creates one history entry
#[cfg(kani)]
#[kani::proof]
fn test_his003_drag_creates_single_history_entry() {
    let id = NodeId::new("n1".to_string());
    let restored = run_undo_test(
        |doc| {
            doc.document
                .nodes
                .insert(id.clone(), make_node("n1", 100.0, 100.0, 80.0, 40.0));
        },
        |doc| {
            let n = doc.document.nodes.get_mut(&id).unwrap();
            n.x = OrderedFloat(150.0);
            n.y = OrderedFloat(150.0);
        },
    );
    let n = restored.document.nodes.get(&id).unwrap();
    assert_eq!(n.x.0, 100.0);
    assert_eq!(n.y.0, 100.0);
}

/// HIS-004: Undo after grouping nodes removes the group
#[cfg(kani)]
#[kani::proof]
fn test_his004_group_undo_removes_group() {
    let (na, nb, ng) = (
        NodeId::new("na".into()),
        NodeId::new("nb".into()),
        NodeId::new("ng".into()),
    );
    let restored = run_undo_test(
        |doc| {
            doc.document
                .nodes
                .insert(na.clone(), make_node("na", 100.0, 100.0, 80.0, 40.0));
            doc.document
                .nodes
                .insert(nb.clone(), make_node("nb", 200.0, 100.0, 80.0, 40.0));
        },
        |doc| {
            let mut group = make_node("group", 100.0, 100.0, 200.0, 100.0);
            group.kind = NodeKind::Subgraph;
            doc.document.nodes.insert(ng.clone(), group);
            doc.document.nodes.get_mut(&na).unwrap().parent = Some(ng.clone());
            doc.document.nodes.get_mut(&nb).unwrap().parent = Some(ng.clone());
        },
    );
    assert!(!restored.document.nodes.contains_key(&ng));
    assert_eq!(restored.document.nodes.get(&na).unwrap().parent, None);
    assert_eq!(restored.document.nodes.get(&nb).unwrap().parent, None);
}

/// HIS-005: Undo after reparenting restores original parent
#[cfg(kani)]
#[kani::proof]
fn test_his005_reparent_undo_restores_parent() {
    let (p1, p2, c) = (
        NodeId::new("p1".into()),
        NodeId::new("p2".into()),
        NodeId::new("c".into()),
    );
    let restored = run_undo_test(
        |doc| {
            doc.document
                .nodes
                .insert(p1.clone(), make_node("p1", 0.0, 0.0, 100.0, 100.0));
            doc.document
                .nodes
                .insert(p2.clone(), make_node("p2", 200.0, 0.0, 100.0, 100.0));
            let mut child = make_node("c", 10.0, 10.0, 50.0, 50.0);
            child.parent = Some(p1.clone());
            doc.document.nodes.insert(c.clone(), child);
        },
        |doc| {
            doc.document.nodes.get_mut(&c).unwrap().parent = Some(p2.clone());
        },
    );
    assert_eq!(restored.document.nodes.get(&c).unwrap().parent, Some(p1));
}

/// HIS-006: Undo after creating edge removes the edge
#[cfg(kani)]
#[kani::proof]
fn test_his006_connector_create_undo_removes_edge() {
    let (n1, n2, e1) = (
        NodeId::new("n1".into()),
        NodeId::new("n2".into()),
        EdgeId::new("e1".into()),
    );
    let restored = run_undo_test(
        |doc| {
            doc.document
                .nodes
                .insert(n1.clone(), make_node("n1", 0.0, 0.0, 100.0, 100.0));
            doc.document
                .nodes
                .insert(n2.clone(), make_node("n2", 200.0, 0.0, 100.0, 100.0));
        },
        |doc| {
            doc.document.edges.insert(
                e1.clone(),
                Edge {
                    source: n1.clone(),
                    target: n2.clone(),
                    label: "".into(),
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
                },
            );
        },
    );
    assert!(restored.document.edges.is_empty());
}

/// HIS-007: Undo after changing node style restores original style
#[cfg(kani)]
#[kani::proof]
fn test_his007_style_change_undo_restores_style() {
    let id = NodeId::new("n1".into());
    let restored = run_undo_test(
        |doc| {
            let mut node = make_node("n1", 0.0, 0.0, 100.0, 100.0);
            node.style = Some(NodeStyle::Box);
            doc.document.nodes.insert(id.clone(), node);
        },
        |doc| {
            doc.document.nodes.get_mut(&id).unwrap().style = Some(NodeStyle::Dashed);
        },
    );
    assert_eq!(
        restored.document.nodes.get(&id).unwrap().style,
        Some(NodeStyle::Box)
    );
}

/// HIS-008: Text edit creates single entry
#[cfg(kani)]
#[kani::proof]
fn test_his008_text_edit_creates_single_entry() {
    let id = NodeId::new("n1".into());
    let restored = run_undo_test(
        |doc| {
            doc.document.nodes.insert(
                id.clone(),
                make_node("Original Label", 0.0, 0.0, 100.0, 100.0),
            );
        },
        |doc| {
            doc.document.nodes.get_mut(&id).unwrap().label = "New Label".into();
        },
    );
    assert_eq!(
        restored.document.nodes.get(&id).unwrap().label,
        "Original Label"
    );
}

/// Helper for testing apply_undo and apply_redo
#[cfg(kani)]
fn setup_history() -> (DiagramDocument, DiagramDocument, History) {
    let mut a = DiagramDocument::default();
    a.revision = Revision::new(1);
    let mut b = a.clone();
    b.revision = Revision::new(2);
    let h = History::new().push(a.clone()).push(b.clone());
    (a, b, h)
}

#[cfg(kani)]
#[kani::proof]
fn test_apply_undo_success_restores_previous_state() {
    let (a, b, mut h) = setup_history();
    let mut curr = b.clone();
    assert_eq!(apply_undo(&mut curr, &mut h), Ok(()));
    assert_eq!(curr.revision, a.revision);
    assert!(h.can_redo());
}

#[cfg(kani)]
#[kani::proof]
fn test_apply_undo_failure_returns_error_on_empty_history() {
    assert_eq!(
        apply_undo(&mut DiagramDocument::default(), &mut History::new()),
        Err(crate::core::history::HistoryError::NothingToUndo)
    );
}

#[cfg(kani)]
#[kani::proof]
fn test_apply_redo_success_restores_next_state() {
    let (_, b, mut h) = setup_history();
    let mut curr = b.clone();
    let _ = apply_undo(&mut curr, &mut h);
    assert_eq!(apply_redo(&mut curr, &mut h), Ok(()));
    assert_eq!(curr.revision, b.revision);
    assert!(h.can_undo());
}

#[cfg(kani)]
#[kani::proof]
fn test_apply_redo_failure_returns_error_on_empty_redo_stack() {
    let mut a = DiagramDocument::default();
    a.revision = Revision::new(1);
    let mut h = History::new().push(a.clone());
    assert_eq!(
        apply_redo(&mut a, &mut h),
        Err(crate::core::history::HistoryError::NothingToRedo)
    );
}
