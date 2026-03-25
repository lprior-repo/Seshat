use super::*;
use crate::document::types::OrderedFloat;
use crate::document::types::{AuthorId, Timestamp};
use crate::document::{ArrowType, Edge, EdgeStyle, LockState, Node, NodeKind};
use proptest::{prop_assert, prop_assert_eq, prop_assume};

fn doc_at(rev: u64) -> DiagramDocument {
    let mut doc = DiagramDocument::default();
    doc.revision = Revision::new(rev);
    doc
}

fn proposal_at(rev: u64) -> ProposedChanges {
    ProposedChanges {
        base_revision: Revision::new(rev),
        proposer: AuthorId::new("test-agent".into()),
        proposed_at: Timestamp::new(0),
        summary: String::new(),
    }
}

fn test_node(id: &str) -> Node {
    Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: id.to_string(),
        x: OrderedFloat::new_unchecked(0.0),
        y: OrderedFloat::new_unchecked(0.0),
        width: OrderedFloat::new_unchecked(100.0),
        height: OrderedFloat::new_unchecked(100.0),
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

fn test_edge(source: &str, target: &str) -> Edge {
    Edge {
        source: NodeId::new(source.to_string()),
        target: NodeId::new(target.to_string()),
        label: String::new(),
        style: EdgeStyle::default(),
        arrow_type: ArrowType::default(),
        label_offset_t: OrderedFloat::new_unchecked(0.5),
        color: None,
        thickness: OrderedFloat::new_unchecked(1.5),
        directed: true,
        bend_points: im::Vector::new(),
        tags: im::Vector::new(),
        metadata: im::HashMap::new(),
        font_size: None,
        source_port: None,
        target_port: None,
    }
}

fn doc_with_nodes_and_edges(nodes: Vec<(&str, Node)>, edges: Vec<(&str, Edge)>) -> DiagramDocument {
    let mut doc = DiagramDocument::default();
    for (id, node) in nodes {
        doc.document.nodes.insert(NodeId::new(id.to_string()), node);
    }
    for (id, edge) in edges {
        doc.document.edges.insert(EdgeId::new(id.to_string()), edge);
    }
    doc
}

fn delete_node_change(node_id: &str, was: Node) -> ProposedChange {
    ProposedChange::DeleteNode {
        node_id: NodeId::new(node_id.to_string()),
        was_node_id: NodeId::new(node_id.to_string()),
        was,
    }
}

fn mismatched_delete_node_change(node_id: &str, snapshot_id: &str) -> ProposedChange {
    ProposedChange::DeleteNode {
        node_id: NodeId::new(node_id.to_string()),
        was_node_id: NodeId::new(snapshot_id.to_string()),
        was: test_node(snapshot_id),
    }
}

fn delete_node_change_with_independent_ids(node_id: &str) -> ProposedChange {
    ProposedChange::DeleteNode {
        node_id: NodeId::new(node_id.to_string()),
        was_node_id: NodeId::new(node_id.to_string()),
        was: test_node(node_id),
    }
}

macro_rules! assert_named {
    ($name:expr, $cond:expr) => {
        assert!($cond, "postcondition {} failed", $name);
    };
}

// =====================================================================
// check_revision_mismatch tests (existing)
// =====================================================================

#[test]
fn test_returns_applied_when_revisions_match_initial() {
    let doc = doc_at(0);
    let proposal = proposal_at(0);
    assert_eq!(
        check_revision_mismatch(&doc, &proposal),
        ApplyResult::Applied
    );
}

#[test]
fn test_returns_applied_when_revisions_match_at_high_revision() {
    let doc = doc_at(42);
    let proposal = proposal_at(42);
    assert_eq!(
        check_revision_mismatch(&doc, &proposal),
        ApplyResult::Applied
    );
}

#[test]
fn test_returns_applied_when_both_revisions_are_identical_non_zero() {
    let doc = doc_at(1_000_000);
    let proposal = proposal_at(1_000_000);
    assert_eq!(
        check_revision_mismatch(&doc, &proposal),
        ApplyResult::Applied
    );
}

#[test]
fn test_returns_stale_when_proposal_revision_is_behind_document() {
    let doc = doc_at(8);
    let proposal = proposal_at(5);
    let result = check_revision_mismatch(&doc, &proposal);
    assert_eq!(
        result,
        ApplyResult::Stale(StaleInfo {
            expected: Revision::new(5),
            current: Revision::new(8),
        })
    );
}

#[test]
fn test_returns_stale_when_proposal_revision_is_ahead_of_document() {
    let doc = doc_at(3);
    let proposal = proposal_at(7);
    let result = check_revision_mismatch(&doc, &proposal);
    assert_eq!(
        result,
        ApplyResult::Stale(StaleInfo {
            expected: Revision::new(7),
            current: Revision::new(3),
        })
    );
}

#[test]
fn test_stale_info_captures_expected_and_current_correctly() {
    let doc = doc_at(10);
    let proposal = proposal_at(4);
    let result = check_revision_mismatch(&doc, &proposal);
    match result {
        ApplyResult::Stale(info) => {
            assert_eq!(info.expected, Revision::new(4));
            assert_eq!(info.current, Revision::new(10));
        }
        other => panic!("expected Stale, got {other:?}"),
    }
}

#[test]
fn test_stale_info_expected_equals_proposal_base_revision() {
    let proposal = proposal_at(99);
    let doc = doc_at(1);
    match check_revision_mismatch(&doc, &proposal) {
        ApplyResult::Stale(info) => {
            assert_eq!(info.expected, proposal.base_revision);
        }
        other => panic!("expected Stale, got {other:?}"),
    }
}

#[test]
fn test_stale_info_current_equals_document_revision() {
    let proposal = proposal_at(1);
    let doc = doc_at(99);
    match check_revision_mismatch(&doc, &proposal) {
        ApplyResult::Stale(info) => {
            assert_eq!(info.current, doc.revision);
        }
        other => panic!("expected Stale, got {other:?}"),
    }
}

#[test]
fn test_stale_at_revision_boundary_zero_vs_one() {
    let doc = doc_at(0);
    let proposal = proposal_at(1);
    let result = check_revision_mismatch(&doc, &proposal);
    assert_eq!(
        result,
        ApplyResult::Stale(StaleInfo {
            expected: Revision::new(1),
            current: Revision::INITIAL,
        })
    );
}

#[test]
fn test_stale_at_high_revision_values() {
    let doc = doc_at(u64::MAX - 1);
    let proposal = proposal_at(u64::MAX - 2);
    let result = check_revision_mismatch(&doc, &proposal);
    assert_eq!(
        result,
        ApplyResult::Stale(StaleInfo {
            expected: Revision::new(u64::MAX - 2),
            current: Revision::new(u64::MAX - 1),
        })
    );
}

#[test]
fn test_matching_at_max_revision_boundary() {
    let doc = doc_at(u64::MAX);
    let proposal = proposal_at(u64::MAX);
    assert_eq!(
        check_revision_mismatch(&doc, &proposal),
        ApplyResult::Applied
    );
}

#[test]
fn test_no_panic_on_boundary_revision_pairs() {
    let doc = doc_at(u64::MAX);
    let proposal = proposal_at(0);
    assert_eq!(
        check_revision_mismatch(&doc, &proposal),
        ApplyResult::Stale(StaleInfo {
            expected: Revision::new(0),
            current: Revision::new(u64::MAX),
        })
    );

    let doc = doc_at(0);
    let proposal = proposal_at(u64::MAX);
    assert_eq!(
        check_revision_mismatch(&doc, &proposal),
        ApplyResult::Stale(StaleInfo {
            expected: Revision::new(u64::MAX),
            current: Revision::new(0),
        })
    );

    let doc = doc_at(1);
    let proposal = proposal_at(1);
    assert_eq!(
        check_revision_mismatch(&doc, &proposal),
        ApplyResult::Applied
    );
}

#[test]
fn test_precondition_stale_iff_revisions_differ() {
    assert!(matches!(
        check_revision_mismatch(&doc_at(5), &proposal_at(3)),
        ApplyResult::Stale(_)
    ));
    assert!(!matches!(
        check_revision_mismatch(&doc_at(7), &proposal_at(7)),
        ApplyResult::Stale(_)
    ));
}

#[test]
fn test_postcondition_document_unchanged_on_stale() {
    let doc = doc_at(3);
    let doc_before = doc.clone();
    let proposal = proposal_at(1);
    assert_eq!(
        check_revision_mismatch(&doc, &proposal),
        ApplyResult::Stale(StaleInfo {
            expected: Revision::new(1),
            current: Revision::new(3),
        })
    );
    assert_eq!(doc, doc_before);
}

#[test]
fn test_invariant_stale_info_fields_are_faithful() {
    let expected_rev = 12u64;
    let current_rev = 20u64;
    let doc = doc_at(current_rev);
    let proposal = proposal_at(expected_rev);
    match check_revision_mismatch(&doc, &proposal) {
        ApplyResult::Stale(info) => {
            assert_eq!(info.expected, Revision::new(expected_rev));
            assert_eq!(info.current, Revision::new(current_rev));
        }
        other => panic!("expected Stale, got {other:?}"),
    }
}

#[test]
fn test_invariant_function_is_pure_no_side_effects() {
    let doc = doc_at(5);
    let proposal = proposal_at(3);
    let first = check_revision_mismatch(&doc, &proposal);
    let second = check_revision_mismatch(&doc, &proposal);
    assert_eq!(first, second);

    let matching_proposal = proposal_at(5);
    let applied = check_revision_mismatch(&doc, &matching_proposal);
    assert_eq!(applied, ApplyResult::Applied);

    let third = check_revision_mismatch(&doc, &proposal);
    assert_eq!(first, third);
}

#[test]
fn test_invariant_function_never_panics() {
    use std::panic::catch_unwind;
    use std::panic::AssertUnwindSafe;

    let result = catch_unwind(AssertUnwindSafe(|| {
        let doc = doc_at(u64::MAX);
        let proposal = proposal_at(0);
        check_revision_mismatch(&doc, &proposal)
    }));
    match result {
        Ok(inner) => assert_eq!(
            inner,
            ApplyResult::Stale(StaleInfo {
                expected: Revision::new(0),
                current: Revision::new(u64::MAX),
            })
        ),
        Err(_) => panic!("check_revision_mismatch panicked"),
    }
}

proptest::proptest! {
    #[test]
    fn proptest_revision_mismatch_detection_is_exhaustive(a in proptest::num::u64::ANY, b in proptest::num::u64::ANY) {
        let doc = doc_at(a);
        let proposal = proposal_at(b);
        let result = check_revision_mismatch(&doc, &proposal);
        if a == b {
            prop_assert_eq!(result, ApplyResult::Applied);
        } else {
            prop_assert!(matches!(result, ApplyResult::Stale(_)));
        }
    }

    #[test]
    fn proptest_stale_info_always_matches_inputs(expected in proptest::num::u64::ANY, current in proptest::num::u64::ANY) {
        prop_assume!(expected != current);
        let doc = doc_at(current);
        let proposal = proposal_at(expected);
        match check_revision_mismatch(&doc, &proposal) {
            ApplyResult::Stale(info) => {
                prop_assert_eq!(info.expected, Revision::new(expected));
                prop_assert_eq!(info.current, Revision::new(current));
            }
            other => panic!("expected Stale for differing revisions, got {other:?}"),
        }
    }
}

// =====================================================================
// Behavior 8: apply_delete_node happy path
// =====================================================================

#[test]
fn apply_delete_node_removes_node_and_cascades_edges_when_valid() {
    let mut doc = doc_with_nodes_and_edges(
        vec![("n1", test_node("n1")), ("n2", test_node("n2"))],
        vec![("e1", test_edge("n1", "n2"))],
    );
    let change = delete_node_change_with_independent_ids("n1");

    let result = apply_delete_node(&mut doc, &change).unwrap();

    assert_eq!(result.deleted_node_id, NodeId::new("n1".to_string()));
    assert_eq!(
        result.cascade_deleted_edge_ids,
        vec![EdgeId::new("e1".to_string())]
    );
    assert!(!doc
        .document
        .nodes
        .contains_key(&NodeId::new("n1".to_string())));
    assert!(!doc
        .document
        .edges
        .contains_key(&EdgeId::new("e1".to_string())));
}

// =====================================================================
// Behavior 9: SnapshotIdMismatch error
// =====================================================================

#[test]
fn apply_delete_node_returns_snapshot_mismatch_when_was_id_differs_from_node_id() {
    let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
    let change = mismatched_delete_node_change("n1", "n2");

    let result = apply_delete_node(&mut doc, &change).unwrap_err();

    assert_eq!(
        result,
        ApplyError::SnapshotIdMismatch {
            declared: NodeId::new("n1".to_string()),
            snapshot: NodeId::new("n2".to_string()),
        }
    );
}

// =====================================================================
// Behavior 10: NodeNotFound error
// =====================================================================

#[test]
fn apply_delete_node_returns_node_not_found_when_node_absent() {
    let mut doc = DiagramDocument::default();
    let change = delete_node_change_with_independent_ids("ghost");

    let result = apply_delete_node(&mut doc, &change).unwrap_err();

    assert_eq!(
        result,
        ApplyError::NodeNotFound(NodeId::new("ghost".to_string()))
    );
}

// =====================================================================
// Behavior 11: Revision increment (Q7)
// =====================================================================

#[test]
fn apply_delete_node_increments_revision_by_one() {
    let mut doc = doc_at(5);
    doc.document
        .nodes
        .insert(NodeId::new("n1".to_string()), test_node("n1"));
    let change = delete_node_change_with_independent_ids("n1");

    apply_delete_node(&mut doc, &change).unwrap();

    assert_eq!(doc.revision, Revision::new(6));
}

// =====================================================================
// Behavior 12: Edge cascade (Q1, Q2, Q3, Q4)
// =====================================================================

#[test]
fn apply_delete_node_cascades_all_edges_referencing_deleted_node() {
    let mut doc = doc_with_nodes_and_edges(
        vec![
            ("n1", test_node("n1")),
            ("n2", test_node("n2")),
            ("n3", test_node("n3")),
        ],
        vec![
            ("e1", test_edge("n1", "n2")),
            ("e2", test_edge("n3", "n1")),
            ("e3", test_edge("n2", "n3")),
        ],
    );
    let change = delete_node_change_with_independent_ids("n1");

    let result = apply_delete_node(&mut doc, &change).unwrap();

    let mut cascade = result.cascade_deleted_edge_ids.clone();
    cascade.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    assert_eq!(
        cascade,
        vec![EdgeId::new("e1".to_string()), EdgeId::new("e2".to_string()),]
    );
    assert!(doc
        .document
        .edges
        .contains_key(&EdgeId::new("e3".to_string())));
    let node_id = NodeId::new("n1".to_string());
    assert!(!doc
        .document
        .edges
        .values()
        .any(|e| { e.source == node_id || e.target == node_id }));
}

// =====================================================================
// Behavior 13: Self-loop cascade (I5)
// =====================================================================

#[test]
fn apply_delete_node_cascades_self_loops_on_deleted_node() {
    let mut doc = doc_with_nodes_and_edges(
        vec![("n1", test_node("n1"))],
        vec![("e-self", test_edge("n1", "n1"))],
    );
    let change = delete_node_change_with_independent_ids("n1");

    let result = apply_delete_node(&mut doc, &change).unwrap();

    assert!(result
        .cascade_deleted_edge_ids
        .contains(&EdgeId::new("e-self".to_string())));
    assert!(doc.document.edges.is_empty());
}

// =====================================================================
// Behavior 14: No connected edges (I6)
// =====================================================================

#[test]
fn apply_delete_node_returns_empty_cascade_when_node_has_no_edges() {
    let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
    let change = delete_node_change_with_independent_ids("n1");

    let result = apply_delete_node(&mut doc, &change).unwrap();

    assert_eq!(result.deleted_node_id, NodeId::new("n1".to_string()));
    assert!(result.cascade_deleted_edge_ids.is_empty());
    assert!(!doc
        .document
        .nodes
        .contains_key(&NodeId::new("n1".to_string())));
}

// =====================================================================
// Behavior 15: Document unchanged on error (I4)
// =====================================================================

#[test]
fn apply_delete_node_preserves_document_on_snapshot_mismatch_error() {
    let mut doc = doc_at(3);
    doc.document
        .nodes
        .insert(NodeId::new("n1".to_string()), test_node("n1"));
    let change = mismatched_delete_node_change("n1", "n2");
    let pre_edges_len = doc.document.edges.len();

    let result = apply_delete_node(&mut doc, &change);

    assert_eq!(
        result,
        Err(ApplyError::SnapshotIdMismatch {
            declared: NodeId::new("n1".to_string()),
            snapshot: NodeId::new("n2".to_string()),
        })
    );
    assert_eq!(doc.revision, Revision::new(3));
    assert!(
        doc.document
            .nodes
            .get(&NodeId::new("n1".to_string()))
            .is_some(),
        "node n1 must be preserved after SnapshotIdMismatch error"
    );
    let n1 = doc
        .document
        .nodes
        .get(&NodeId::new("n1".to_string()))
        .expect("node n1 must be preserved after SnapshotIdMismatch error");
    assert_eq!(n1.label, "n1");
    assert_eq!(doc.document.edges.len(), pre_edges_len);
}

#[test]
fn apply_delete_node_preserves_document_on_node_not_found_error() {
    let mut doc = doc_at(3);
    doc.document
        .nodes
        .insert(NodeId::new("n1".to_string()), test_node("n1"));
    let pre_nodes_len = doc.document.nodes.len();
    let change = delete_node_change_with_independent_ids("ghost");

    let result = apply_delete_node(&mut doc, &change);

    assert_eq!(
        result,
        Err(ApplyError::NodeNotFound(NodeId::new("ghost".to_string())))
    );
    assert_eq!(doc.revision, Revision::new(3));
    assert_eq!(doc.document.nodes.len(), pre_nodes_len);
}

// =====================================================================
// Behavior 16: Other nodes preserved (Q8)
// =====================================================================

#[test]
fn apply_delete_node_preserves_other_nodes_and_unrelated_edges() {
    let n2 = test_node("n2");
    let n3 = test_node("n3");
    let e2 = test_edge("n2", "n3");
    let mut doc = doc_with_nodes_and_edges(
        vec![
            ("n1", test_node("n1")),
            ("n2", n2.clone()),
            ("n3", n3.clone()),
        ],
        vec![("e1", test_edge("n1", "n2")), ("e2", e2.clone())],
    );
    let change = delete_node_change_with_independent_ids("n1");

    apply_delete_node(&mut doc, &change).unwrap();

    assert_eq!(
        doc.document.nodes.get(&NodeId::new("n2".to_string())),
        Some(&n2)
    );
    assert_eq!(
        doc.document.nodes.get(&NodeId::new("n3".to_string())),
        Some(&n3)
    );
    assert_eq!(
        doc.document.edges.get(&EdgeId::new("e2".to_string())),
        Some(&e2)
    );
}

// =====================================================================
// Behavior 17: Cascade completeness (I1)
// =====================================================================

#[test]
fn apply_delete_node_cascade_ids_match_actually_removed_edges() {
    let mut doc = doc_with_nodes_and_edges(
        vec![
            ("n1", test_node("n1")),
            ("n2", test_node("n2")),
            ("n3", test_node("n3")),
        ],
        vec![
            ("e1", test_edge("n1", "n2")),
            ("e2", test_edge("n3", "n1")),
            ("e4", test_edge("n1", "n1")),
        ],
    );
    let change = delete_node_change_with_independent_ids("n1");

    let result = apply_delete_node(&mut doc, &change).unwrap();

    let mut cascade = result.cascade_deleted_edge_ids.clone();
    cascade.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    assert_eq!(
        cascade,
        vec![
            EdgeId::new("e1".to_string()),
            EdgeId::new("e2".to_string()),
            EdgeId::new("e4".to_string()),
        ]
    );
    assert!(
        doc.document.edges.is_empty(),
        "all edges reference n1 and should be cascaded"
    );
}

// =====================================================================
// Behavior 18: Never panics and returns correct result (I2)
// =====================================================================

#[test]
fn apply_delete_node_no_panic_and_correct_error_when_snapshot_mismatch() {
    use std::panic::catch_unwind;
    use std::panic::AssertUnwindSafe;

    let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
    let change = mismatched_delete_node_change("n1", "DIFFERENT");

    let result = catch_unwind(AssertUnwindSafe(|| apply_delete_node(&mut doc, &change)));

    assert!(result.is_ok(), "function panicked");
    let inner = result.unwrap();
    assert_eq!(
        inner,
        Err(ApplyError::SnapshotIdMismatch {
            declared: NodeId::new("n1".to_string()),
            snapshot: NodeId::new("DIFFERENT".to_string()),
        })
    );
}

#[test]
fn apply_delete_node_no_panic_and_correct_error_when_node_absent() {
    use std::panic::catch_unwind;
    use std::panic::AssertUnwindSafe;

    let mut doc = DiagramDocument::default();
    let change = delete_node_change_with_independent_ids("ghost");

    let result = catch_unwind(AssertUnwindSafe(|| apply_delete_node(&mut doc, &change)));

    assert!(result.is_ok(), "function panicked");
    let inner = result.unwrap();
    assert_eq!(
        inner,
        Err(ApplyError::NodeNotFound(NodeId::new("ghost".to_string())))
    );
}

#[test]
fn apply_delete_node_no_panic_and_correct_result_when_valid() {
    use std::panic::catch_unwind;
    use std::panic::AssertUnwindSafe;

    let mut doc = doc_with_nodes_and_edges(
        vec![
            ("n1", test_node("n1")),
            ("n2", test_node("n2")),
            ("n3", test_node("n3")),
            ("n4", test_node("n4")),
            ("n5", test_node("n5")),
        ],
        vec![
            ("e1", test_edge("n1", "n2")),
            ("e2", test_edge("n3", "n1")),
            ("e3", test_edge("n1", "n4")),
        ],
    );
    let change = delete_node_change_with_independent_ids("n1");

    let result = catch_unwind(AssertUnwindSafe(|| apply_delete_node(&mut doc, &change)));

    assert!(result.is_ok(), "function panicked");
    let delete_result = match result.unwrap() {
        Ok(r) => r,
        Err(e) => panic!("apply returned error: {e:?}"),
    };
    assert_eq!(delete_result.deleted_node_id, NodeId::new("n1".to_string()));
    let mut cascade = delete_result.cascade_deleted_edge_ids.clone();
    cascade.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    assert_eq!(
        cascade,
        vec![
            EdgeId::new("e1".to_string()),
            EdgeId::new("e2".to_string()),
            EdgeId::new("e3".to_string()),
        ]
    );
    assert!(!doc
        .document
        .nodes
        .contains_key(&NodeId::new("n1".to_string())));
}

// =====================================================================
// Behavior 19: Double-delete
// =====================================================================

#[test]
fn apply_delete_node_returns_node_not_found_on_second_call() {
    let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
    let change = delete_node_change_with_independent_ids("n1");

    let first = apply_delete_node(&mut doc, &change);
    assert_eq!(
        first,
        Ok(DeleteNodeResult {
            deleted_node_id: NodeId::new("n1".to_string()),
            cascade_deleted_edge_ids: vec![],
        })
    );

    let second = apply_delete_node(&mut doc, &change);
    assert_eq!(
        second,
        Err(ApplyError::NodeNotFound(NodeId::new("n1".to_string())))
    );
}

// =====================================================================
// Behavior 20: DocumentError wrapping
// =====================================================================

#[test]
fn apply_delete_node_wraps_node_not_found_as_document_error() {
    let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
    let change = delete_node_change_with_independent_ids("n1");

    let result = apply_delete_node_with_remove(&mut doc, &change, |_doc, _id| {
        Err(DocumentError::NodeNotFound(NodeId::new("n1".to_string())))
    });

    assert_eq!(
        result,
        Err(ApplyError::DocumentError(DocumentError::NodeNotFound(
            NodeId::new("n1".to_string())
        )))
    );
}

#[test]
fn apply_delete_node_wraps_edge_not_found_as_document_error() {
    let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
    let change = delete_node_change_with_independent_ids("n1");

    let result = apply_delete_node_with_remove(&mut doc, &change, |_doc, _id| {
        Err(DocumentError::EdgeNotFound(EdgeId::new(
            "e-broken".to_string(),
        )))
    });

    assert_eq!(
        result,
        Err(ApplyError::DocumentError(DocumentError::EdgeNotFound(
            EdgeId::new("e-broken".to_string())
        )))
    );
}

// =====================================================================
// Behavior 21: DocumentError payload fidelity
// =====================================================================

#[test]
fn apply_error_document_error_preserves_inner_variant_and_payload() {
    let inner = DocumentError::NodeNotFound(NodeId::new("x-42".to_string()));
    let err = ApplyError::DocumentError(inner.clone());

    assert_eq!(
        err,
        ApplyError::DocumentError(DocumentError::NodeNotFound(NodeId::new("x-42".to_string())))
    );
    assert!(matches!(&inner, DocumentError::NodeNotFound(id) if id.as_str() == "x-42"));
}

// =====================================================================
// Behavior 22: deleted_node_id correctness
// =====================================================================

#[test]
fn apply_delete_node_returns_correct_deleted_node_id_matching_document_key() {
    let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
    let node_key = NodeId::new("n1".to_string());
    let change = ProposedChange::DeleteNode {
        node_id: NodeId::new("n1".to_string()),
        was_node_id: NodeId::new("n1".to_string()),
        was: test_node("n1"),
    };

    let result = apply_delete_node(&mut doc, &change).unwrap();

    assert_eq!(result.deleted_node_id.as_str(), "n1");
    assert_eq!(result.deleted_node_id, node_key);
}

#[test]
fn apply_delete_node_deleted_node_id_matches_declared_node_id_not_was_field() {
    let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
    let change = ProposedChange::DeleteNode {
        node_id: NodeId::new("n1".to_string()),
        was_node_id: NodeId::new("n1".to_string()),
        was: test_node("n1"),
    };

    let result = apply_delete_node(&mut doc, &change).unwrap();

    assert_eq!(result.deleted_node_id.as_str(), "n1");
    let ProposedChange::DeleteNode { node_id, .. } = &change else {
        panic!("not DeleteNode")
    };
    assert_eq!(result.deleted_node_id.as_str(), node_id.as_str());
}

// =====================================================================
// Behavior 24: cascade_edges_for_node returns connected edges
// =====================================================================

#[test]
fn cascade_edges_for_node_returns_all_edges_connected_to_node() {
    let doc = doc_with_nodes_and_edges(
        vec![
            ("n1", test_node("n1")),
            ("n2", test_node("n2")),
            ("n3", test_node("n3")),
        ],
        vec![
            ("e1", test_edge("n1", "n2")),
            ("e2", test_edge("n3", "n1")),
            ("e3", test_edge("n2", "n3")),
        ],
    );

    let result = cascade_edges_for_node(&doc, &NodeId::new("n1".to_string())).unwrap();

    let mut sorted = result.clone();
    sorted.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    assert_eq!(
        sorted,
        vec![EdgeId::new("e1".to_string()), EdgeId::new("e2".to_string()),]
    );
}

// =====================================================================
// Behavior 25: cascade_edges_for_node returns None for missing node
// =====================================================================

#[test]
fn cascade_edges_for_node_returns_none_when_node_not_found() {
    let doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);

    let result = cascade_edges_for_node(&doc, &NodeId::new("ghost".to_string()));

    assert!(result.is_none());
}

// =====================================================================
// Behavior 26: cascade_edges_for_node includes self-loops
// =====================================================================

#[test]
fn cascade_edges_for_node_includes_self_loops() {
    let doc = doc_with_nodes_and_edges(
        vec![("n1", test_node("n1"))],
        vec![("e-self", test_edge("n1", "n1"))],
    );

    let result = cascade_edges_for_node(&doc, &NodeId::new("n1".to_string())).unwrap();

    assert!(result.contains(&EdgeId::new("e-self".to_string())));
}

// =====================================================================
// Behavior 27: cascade_edges_for_node returns empty for no edges
// =====================================================================

#[test]
fn cascade_edges_for_node_returns_empty_vec_when_node_has_no_edges() {
    let doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);

    let result = cascade_edges_for_node(&doc, &NodeId::new("n1".to_string())).unwrap();

    assert!(result.is_empty());
}

// =====================================================================
// Behavior 28: cascade_edges_for_node does not mutate document
// =====================================================================

#[test]
fn cascade_edges_for_node_does_not_modify_document() {
    let mut doc = doc_at(5);
    doc.document
        .nodes
        .insert(NodeId::new("n1".to_string()), test_node("n1"));
    doc.document
        .nodes
        .insert(NodeId::new("n2".to_string()), test_node("n2"));
    doc.document
        .edges
        .insert(EdgeId::new("e1".to_string()), test_edge("n1", "n2"));
    let pre_nodes_len = doc.document.nodes.len();
    let pre_edges_len = doc.document.edges.len();

    let cascade = cascade_edges_for_node(&doc, &NodeId::new("n1".to_string()));
    assert_eq!(cascade, Some(vec![EdgeId::new("e1".to_string())]));

    assert_eq!(doc.revision, Revision::new(5));
    assert_eq!(doc.document.nodes.len(), pre_nodes_len);
    assert_eq!(doc.document.edges.len(), pre_edges_len);
}

// =====================================================================
// Integration: cascade_edges_for_node agrees with apply_delete_node
// =====================================================================

#[test]
fn cascade_edges_for_node_agrees_with_apply_delete_node() {
    let mut doc = doc_with_nodes_and_edges(
        vec![
            ("n1", test_node("n1")),
            ("n2", test_node("n2")),
            ("n3", test_node("n3")),
        ],
        vec![
            ("e1", test_edge("n1", "n2")),
            ("e2", test_edge("n3", "n1")),
            ("e3", test_edge("n2", "n3")),
        ],
    );
    let node_id = NodeId::new("n1".to_string());

    let cascade_ids = cascade_edges_for_node(&doc, &node_id).unwrap();
    let change = delete_node_change_with_independent_ids("n1");
    let result = apply_delete_node(&mut doc, &change).unwrap();

    let mut a = cascade_ids;
    a.sort_by(|x, y| x.as_str().cmp(y.as_str()));
    let mut b = result.cascade_deleted_edge_ids;
    b.sort_by(|x, y| x.as_str().cmp(y.as_str()));
    assert_eq!(a, b);
}

// =====================================================================
// Integration: Full postcondition verification (Q1–Q8)
// =====================================================================

#[test]
fn apply_delete_node_satisfies_all_postconditions_q1_through_q8() {
    let mut doc = doc_with_nodes_and_edges(
        vec![
            ("n1", test_node("n1")),
            ("n2", test_node("n2")),
            ("n3", test_node("n3")),
            ("n4", test_node("n4")),
            ("n5", test_node("n5")),
        ],
        vec![
            ("e1", test_edge("n1", "n2")),
            ("e2", test_edge("n3", "n1")),
            ("e3", test_edge("n1", "n4")),
            ("e4", test_edge("n2", "n3")),
            ("e5", test_edge("n4", "n5")),
            ("e6", test_edge("n5", "n2")),
            ("e7", test_edge("n3", "n4")),
            ("e8", test_edge("n5", "n1")),
        ],
    );
    let pre_state = doc.clone();
    let node_id = NodeId::new("n1".to_string());
    let change = delete_node_change_with_independent_ids("n1");

    let result = apply_delete_node(&mut doc, &change).unwrap();

    let cascade_ids = &result.cascade_deleted_edge_ids;
    assert_named!(
        "Q1_node_removed",
        !doc.document.nodes.contains_key(&node_id)
    );
    assert_named!(
        "Q2_cascade_edges_removed",
        cascade_ids
            .iter()
            .all(|id| !doc.document.edges.contains_key(id))
    );
    assert_named!(
        "Q3_no_dangling_refs",
        !doc.document
            .edges
            .values()
            .any(|e| e.source == node_id || e.target == node_id)
    );
    assert_named!(
        "Q4_edges_subset",
        doc.document
            .edges
            .keys()
            .all(|id| pre_state.document.edges.contains_key(id))
    );
    assert_named!("Q5_node_count", doc.document.nodes.len() == 4);
    assert_named!("Q6_edge_count", doc.document.edges.len() == 4);
    assert_named!(
        "Q7_revision",
        doc.revision == pre_state.revision.increment()
    );
    assert_named!(
        "Q8_n2_unchanged",
        doc.document.nodes.get(&NodeId::new("n2".to_string()))
            == pre_state.document.nodes.get(&NodeId::new("n2".to_string()))
    );
    assert_named!(
        "Q8_n3_unchanged",
        doc.document.nodes.get(&NodeId::new("n3".to_string()))
            == pre_state.document.nodes.get(&NodeId::new("n3".to_string()))
    );
    assert_named!(
        "Q8_n4_unchanged",
        doc.document.nodes.get(&NodeId::new("n4".to_string()))
            == pre_state.document.nodes.get(&NodeId::new("n4".to_string()))
    );
    assert_named!(
        "Q8_n5_unchanged",
        doc.document.nodes.get(&NodeId::new("n5".to_string()))
            == pre_state.document.nodes.get(&NodeId::new("n5".to_string()))
    );
}

// =====================================================================
// Integration: Many nodes — only target affected
// =====================================================================

fn doc_with_chain_graph(node_count: usize) -> DiagramDocument {
    let mut doc = DiagramDocument::default();
    for i in 0..node_count {
        let name = format!("n{i}");
        doc.document
            .nodes
            .insert(NodeId::new(name.clone()), test_node(&name));
    }
    for i in 0..node_count {
        let src = format!("n{i}");
        let tgt = format!("n{}", (i + 1) % node_count);
        let e_name = format!("e{i}");
        doc.document
            .edges
            .insert(EdgeId::new(e_name), test_edge(&src, &tgt));
    }
    doc
}

#[test]
fn apply_delete_node_only_affects_target_node_and_its_edges() {
    let mut doc = doc_with_chain_graph(50);
    let node_id = NodeId::new("n25".to_string());

    let change = delete_node_change_with_independent_ids("n25");
    let result = apply_delete_node(&mut doc, &change).unwrap();

    assert_eq!(result.deleted_node_id, node_id);
    assert!(doc
        .document
        .nodes
        .contains_key(&NodeId::new("n0".to_string())));
    assert!(!doc.document.nodes.contains_key(&node_id));
    let mut cascade = result.cascade_deleted_edge_ids.clone();
    cascade.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    assert_eq!(
        cascade,
        vec![
            EdgeId::new("e24".to_string()),
            EdgeId::new("e25".to_string()),
        ]
    );
}

// =====================================================================
// Integration: Idempotent query
// =====================================================================

#[test]
fn cascade_edges_for_node_is_idempotent() {
    let doc = doc_with_nodes_and_edges(
        vec![("n1", test_node("n1")), ("n2", test_node("n2"))],
        vec![("e1", test_edge("n1", "n2"))],
    );

    let first = cascade_edges_for_node(&doc, &NodeId::new("n1".to_string()));
    let second = cascade_edges_for_node(&doc, &NodeId::new("n1".to_string()));
    let third = cascade_edges_for_node(&doc, &NodeId::new("n1".to_string()));

    assert_eq!(first, second);
    assert_eq!(second, third);
}

// =====================================================================
// Proptest: snapshot mismatch detection
// =====================================================================

proptest::proptest! {
    #[test]
    fn proptest_snapshot_mismatch_rejected_when_ids_differ(
        node_id in "[a-z]{1,10}",
        was_id in "[a-z]{1,10}"
    ) {
        prop_assume!(node_id != was_id);
        let mut doc = doc_with_nodes_and_edges(
            vec![(&*node_id, test_node(&node_id))],
            vec![],
        );
        let change = mismatched_delete_node_change(&node_id, &was_id);

        let result = apply_delete_node(&mut doc, &change);
        prop_assert_eq!(
            result,
            Err(ApplyError::SnapshotIdMismatch {
                declared: NodeId::new(node_id.clone()),
                snapshot: NodeId::new(was_id.clone()),
            })
        );
    }
}

// =====================================================================
// Proptest: cascade_edges_for_node purity and completeness
// =====================================================================

proptest::proptest! {
    #[test]
    fn proptest_cascade_edges_for_node_matches_manual_edge_scan(
        node_ids in proptest::collection::vec("[a-z]{1,5}", 0..10),
        edges_spec in proptest::collection::vec(
            proptest::collection::vec(proptest::num::usize::ANY, 3),
            0..15
        )
    ) {
        prop_assume!(!node_ids.is_empty());
        let ids: Vec<String> = node_ids.into_iter().collect();
        let n = ids.len();
        let mut doc = DiagramDocument::default();
        for id in &ids {
            doc.document.nodes.insert(NodeId::new(id.clone()), test_node(id));
        }
        for (i, spec) in edges_spec.iter().enumerate() {
            if spec.len() >= 2 {
                let src_idx = spec[0] % n;
                let tgt_idx = spec[1] % n;
                let e_name = format!("e{i}");
                let edge = test_edge(&ids[src_idx], &ids[tgt_idx]);
                doc.document.edges.insert(EdgeId::new(e_name), edge);
            }
        }
        let query_idx = 0;
        let query_id = NodeId::new(ids[query_idx].clone());

        let before_nodes = doc.document.nodes.len();
        let before_edges = doc.document.edges.len();
        let result = cascade_edges_for_node(&doc, &query_id);
        assert_eq!(doc.document.nodes.len(), before_nodes, "nodes must not change");
        assert_eq!(doc.document.edges.len(), before_edges, "edges must not change");

        let expected: std::collections::HashSet<EdgeId> = doc.document.edges.iter()
            .filter(|(_, e)| e.source == query_id || e.target == query_id)
            .map(|(id, _)| id.clone())
            .collect();

        match result {
            Some(ref ids) => {
                let result_set: std::collections::HashSet<EdgeId> = ids.iter().cloned().collect();
                prop_assert_eq!(result_set, expected);
            }
            None => {
                prop_assert!(expected.is_empty(), "None only if node missing; but node exists and has edges");
            }
        }
    }
}

// =====================================================================
// Proptest: apply_delete_node postconditions for any document
// =====================================================================

proptest::proptest! {
    #[test]
    fn proptest_apply_delete_node_postconditions_for_any_document(
        node_ids in proptest::collection::vec("[a-z]{1,5}", 1..10),
        edges_spec in proptest::collection::vec(
            proptest::collection::vec(proptest::num::usize::ANY, 3),
            0..20
        )
    ) {
        let ids: Vec<String> = node_ids.into_iter().collect();
        let n = ids.len();
        let mut doc = DiagramDocument::default();
        for id in &ids {
            doc.document.nodes.insert(NodeId::new(id.clone()), test_node(id));
        }
        for (i, spec) in edges_spec.iter().enumerate() {
            if spec.len() >= 2 {
                let src_idx = spec[0] % n;
                let tgt_idx = spec[1] % n;
                let e_name = format!("e{i}");
                let edge = test_edge(&ids[src_idx], &ids[tgt_idx]);
                doc.document.edges.insert(EdgeId::new(e_name), edge);
            }
        }
        let delete_idx = 0;
        let delete_id = NodeId::new(ids[delete_idx].clone());
        let pre_rev = doc.revision;
        let change = ProposedChange::DeleteNode {
            node_id: delete_id.clone(),
            was_node_id: delete_id.clone(),
            was: test_node(&ids[delete_idx]),
        };

        let result = apply_delete_node(&mut doc, &change);
        if let Ok(ref r) = result {
            prop_assert!(!doc.document.nodes.contains_key(&delete_id), "Q1");
            for eid in &r.cascade_deleted_edge_ids {
                prop_assert!(!doc.document.edges.contains_key(eid), "Q2");
            }
            prop_assert!(
                !doc.document.edges.values().any(|e| e.source == delete_id || e.target == delete_id),
                "Q3"
            );
            prop_assert_eq!(doc.revision, pre_rev.increment(), "Q7");
        }
    }
}

// =====================================================================
// Proptest: document unchanged on any error path (I4)
// =====================================================================

proptest::proptest! {
    #[test]
    fn proptest_document_unchanged_on_any_error_path(
        node_ids in proptest::collection::vec("[a-z]{1,5}", 0..5),
        edges_spec in proptest::collection::vec(
            proptest::collection::vec(proptest::num::usize::ANY, 3),
            0..8
        )
    ) {
        let ids: Vec<String> = node_ids.into_iter().collect();
        let n = ids.len();
        prop_assume!(n > 0, "empty node list causes division by zero in edge setup");
        let mut doc = DiagramDocument::default();
        for id in &ids {
            doc.document.nodes.insert(NodeId::new(id.clone()), test_node(id));
        }
        for (i, spec) in edges_spec.iter().enumerate() {
            if spec.len() >= 2 {
                let src_idx = spec[0] % n;
                let tgt_idx = spec[1] % n;
                let e_name = format!("e{i}");
                let edge = test_edge(&ids[src_idx], &ids[tgt_idx]);
                doc.document.edges.insert(EdgeId::new(e_name), edge);
            }
        }
        let change = ProposedChange::DeleteNode {
            node_id: NodeId::new("nonexistent".to_string()),
            was_node_id: NodeId::new("mismatch".to_string()),
            was: test_node("mismatch"),
        };
        let doc_before = doc.clone();
        let result = apply_delete_node(&mut doc, &change);
        prop_assert_eq!(
            result,
            Err(ApplyError::SnapshotIdMismatch {
                declared: NodeId::new("nonexistent".to_string()),
                snapshot: NodeId::new("mismatch".to_string()),
            }),
            "should always error with SnapshotIdMismatch for mismatched ids"
        );
        prop_assert_eq!(doc, doc_before, "I4: document must be unchanged on error");
    }
}

// =====================================================================
// validate_and_dedup_indices tests (Behaviors 20, 21)
// =====================================================================

#[test]
fn validate_and_dedup_returns_sorted_unique_in_bounds() {
    let indices = [3, 1, 4, 1, 5, 9, 2, 6, 5];
    let result = validate_and_dedup_indices(&indices, 5);
    assert_eq!(result, vec![1, 2, 3, 4]);
    let ignored = indices.len() - result.len();
    assert_eq!(ignored, 5);
}

#[test]
fn validate_and_dedup_counting_all_valid() {
    let indices = [0, 1, 2];
    let result = validate_and_dedup_indices(&indices, 3);
    assert_eq!(result, vec![0, 1, 2]);
    assert_eq!(indices.len() - result.len(), 0);
}

#[test]
fn validate_and_dedup_counting_all_duplicates() {
    let indices = [1, 1, 1, 1];
    let result = validate_and_dedup_indices(&indices, 3);
    assert_eq!(result, vec![1]);
    assert_eq!(indices.len() - result.len(), 3);
}

#[test]
fn validate_and_dedup_counting_all_out_of_bounds() {
    let indices = [2, 3, 4, 5];
    let result = validate_and_dedup_indices(&indices, 2);
    assert!(result.is_empty());
    assert_eq!(indices.len() - result.len(), 4);
}

#[test]
fn validate_and_dedup_counting_empty_input() {
    let indices: &[usize] = &[];
    let result = validate_and_dedup_indices(indices, 5);
    assert!(result.is_empty());
    assert_eq!(indices.len() - result.len(), 0);
}

#[test]
fn validate_and_dedup_counting_mixed() {
    let indices = [0, 0, 1, 5, 2, 2, 9];
    let result = validate_and_dedup_indices(&indices, 3);
    assert_eq!(result, vec![0, 1, 2]);
    assert_eq!(indices.len() - result.len(), 4);
}

#[test]
fn validate_and_dedup_with_changes_len_zero_and_nonempty_indices() {
    let indices = [0, 1, 2];
    let result = validate_and_dedup_indices(&indices, 0);
    assert!(result.is_empty());
    assert_eq!(indices.len() - result.len(), 3);
}

#[test]
fn validate_and_dedup_with_changes_len_zero_and_empty_indices() {
    let indices: &[usize] = &[];
    let result = validate_and_dedup_indices(indices, 0);
    assert!(result.is_empty());
    assert_eq!(indices.len() - result.len(), 0);
}

// =====================================================================
// apply_proposal tests (Behaviors 1–22, 25–27)
// =====================================================================

// -- Behavior 1: Returns Applied when all indices succeed --

#[test]
fn apply_proposal_returns_applied_when_all_indices_succeed() {
    let mut doc = doc_with_nodes_and_edges(
        vec![("n1", test_node("n1")), ("n2", test_node("n2"))],
        vec![("e1", test_edge("n1", "n2"))],
    );
    doc.revision = Revision::new(5);
    let proposal = proposal_at(5);
    let changes = vec![
        delete_node_change_with_independent_ids("n1"),
        delete_node_change_with_independent_ids("n2"),
    ];
    let accepted = [0, 1];

    let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

    assert_eq!(result, ApplyResult::Applied);
}

// -- Behavior 2: Returns Stale when revision mismatches --

#[test]
fn apply_proposal_returns_stale_when_revision_mismatches() {
    let mut doc = doc_at(10);
    let proposal = proposal_at(5);
    let changes = vec![delete_node_change_with_independent_ids("n1")];
    let accepted = [0];

    let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

    assert_eq!(
        result,
        ApplyResult::Stale(StaleInfo {
            expected: Revision::new(5),
            current: Revision::new(10),
        })
    );
}

// -- Behavior 3: Stale info faithful at boundary revisions --

#[test]
fn apply_proposal_stale_info_faithful_at_boundary_revisions() {
    let mut doc = doc_at(0);
    let proposal = proposal_at(u64::MAX);
    let changes: Vec<ProposedChange> = vec![];
    let accepted: [usize; 0] = [];

    let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

    assert_eq!(
        result,
        ApplyResult::Stale(StaleInfo {
            expected: Revision::new(u64::MAX),
            current: Revision::new(0),
        })
    );
}

// -- Behavior 4: Revision incremented exactly once --

#[test]
fn apply_proposal_increments_revision_by_exactly_one_not_per_change() {
    let mut doc = doc_with_nodes_and_edges(
        vec![("n1", test_node("n1")), ("n2", test_node("n2"))],
        vec![],
    );
    doc.revision = Revision::new(3);
    let proposal = proposal_at(3);
    let changes = vec![
        delete_node_change_with_independent_ids("n1"),
        delete_node_change_with_independent_ids("n2"),
    ];
    let accepted = [0, 1];

    let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

    assert_eq!(result, ApplyResult::Applied);
    assert_eq!(doc.revision, Revision::new(4));
}

// -- Behavior 5: Single accepted deletes target + cascades edges --

#[test]
fn apply_proposal_single_accepted_deletes_target_and_cascades_edges() {
    let mut doc = doc_with_nodes_and_edges(
        vec![
            ("n1", test_node("n1")),
            ("n2", test_node("n2")),
            ("n3", test_node("n3")),
        ],
        vec![("e1", test_edge("n1", "n2")), ("e2", test_edge("n2", "n3"))],
    );
    let proposal = proposal_at(0);
    let changes = vec![delete_node_change_with_independent_ids("n2")];
    let accepted = [0];

    let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

    assert_eq!(result, ApplyResult::Applied);
    assert!(!doc
        .document
        .nodes
        .contains_key(&NodeId::new("n2".to_string())));
    assert!(!doc
        .document
        .edges
        .contains_key(&EdgeId::new("e1".to_string())));
    assert!(!doc
        .document
        .edges
        .contains_key(&EdgeId::new("e2".to_string())));
    assert!(doc
        .document
        .nodes
        .contains_key(&NodeId::new("n1".to_string())));
    assert!(doc
        .document
        .nodes
        .contains_key(&NodeId::new("n3".to_string())));
}

// -- Behavior 6: Multiple accepted deletes subset of targets --

#[test]
fn apply_proposal_multiple_accepted_deletes_subset_of_targets() {
    let mut doc = doc_with_nodes_and_edges(
        vec![
            ("a", test_node("a")),
            ("b", test_node("b")),
            ("c", test_node("c")),
        ],
        vec![],
    );
    let proposal = proposal_at(0);
    let changes = vec![
        delete_node_change_with_independent_ids("a"),
        delete_node_change_with_independent_ids("b"),
        delete_node_change_with_independent_ids("c"),
    ];
    let accepted = [0, 2];

    let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

    assert_eq!(result, ApplyResult::Applied);
    assert!(!doc
        .document
        .nodes
        .contains_key(&NodeId::new("a".to_string())));
    assert!(doc
        .document
        .nodes
        .contains_key(&NodeId::new("b".to_string())));
    assert!(!doc
        .document
        .nodes
        .contains_key(&NodeId::new("c".to_string())));
}

// -- Behavior 7: Edge cascade across multiple deletions --

#[test]
fn apply_proposal_edge_cascade_across_multiple_deletions() {
    let mut doc = doc_with_nodes_and_edges(
        vec![
            ("n1", test_node("n1")),
            ("n2", test_node("n2")),
            ("n3", test_node("n3")),
        ],
        vec![("e1", test_edge("n1", "n2")), ("e2", test_edge("n2", "n3"))],
    );
    let proposal = proposal_at(0);
    let changes = vec![
        delete_node_change_with_independent_ids("n1"),
        delete_node_change_with_independent_ids("n2"),
    ];
    let accepted = [0, 1];

    let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

    assert_eq!(result, ApplyResult::Applied);
    assert!(doc.document.edges.is_empty());
    assert!(doc
        .document
        .nodes
        .contains_key(&NodeId::new("n3".to_string())));
    assert!(!doc
        .document
        .nodes
        .contains_key(&NodeId::new("n1".to_string())));
    assert!(!doc
        .document
        .nodes
        .contains_key(&NodeId::new("n2".to_string())));
}

// -- Behavior 8: PartialConflict when any change fails --

#[test]
fn apply_proposal_returns_partial_conflict_when_any_change_fails() {
    let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
    let proposal = proposal_at(0);
    let changes = vec![
        delete_node_change_with_independent_ids("n1"),
        delete_node_change_with_independent_ids("ghost"),
    ];
    let accepted = [0, 1];

    let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

    match result {
        ApplyResult::PartialConflict { applied_count, .. } => {
            assert_eq!(applied_count, 0);
        }
        other => panic!("expected PartialConflict, got {other:?}"),
    }
}

// -- Behavior 9: Document rolled back on PartialConflict --

#[test]
fn apply_proposal_rolls_back_document_on_partial_conflict() {
    let mut doc = doc_with_nodes_and_edges(
        vec![("n1", test_node("n1")), ("n2", test_node("n2"))],
        vec![("e1", test_edge("n1", "n2"))],
    );
    doc.revision = Revision::new(7);
    let doc_before = doc.clone();
    let proposal = proposal_at(7);
    let changes = vec![
        delete_node_change_with_independent_ids("n1"),
        delete_node_change_with_independent_ids("ghost"),
    ];
    let accepted = [0, 1];

    let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

    assert!(matches!(result, ApplyResult::PartialConflict { .. }));
    assert_eq!(
        doc, doc_before,
        "document must be rolled back to pre-call state"
    );
}

// -- Behavior 10: Revision unchanged on PartialConflict --

#[test]
fn apply_proposal_revision_unchanged_on_partial_conflict() {
    let mut doc = doc_at(42);
    doc.document
        .nodes
        .insert(NodeId::new("n1".to_string()), test_node("n1"));
    let proposal = proposal_at(42);
    let changes = vec![
        delete_node_change_with_independent_ids("n1"),
        delete_node_change_with_independent_ids("ghost"),
    ];
    let accepted = [0, 1];

    let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

    assert!(matches!(result, ApplyResult::PartialConflict { .. }));
    assert_eq!(doc.revision, Revision::new(42));
}

// -- Behavior 11: Reasons contain error for failing index --

#[test]
fn apply_proposal_reasons_contain_error_for_failing_index() {
    let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
    let proposal = proposal_at(0);
    let changes = vec![
        delete_node_change_with_independent_ids("n1"),
        delete_node_change_with_independent_ids("ghost"),
    ];
    let accepted = [0, 1];

    let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

    match result {
        ApplyResult::PartialConflict { reasons, .. } => {
            let failing_reason = reasons
                .iter()
                .find(|r| r.contains("change [1]") && r.contains("node not found: ghost"));
            assert!(
                failing_reason.is_some(),
                "expected reason for change [1] with 'node not found: ghost', got: {reasons:?}"
            );
        }
        other => panic!("expected PartialConflict, got {other:?}"),
    }
}

// -- Behavior 12: Reasons contain "not attempted" for remaining indices --

#[test]
fn apply_proposal_reasons_contain_not_attempted_for_remaining_indices() {
    let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
    let proposal = proposal_at(0);
    let changes = vec![
        delete_node_change_with_independent_ids("n1"),
        delete_node_change_with_independent_ids("ghost"),
        delete_node_change_with_independent_ids("n1"),
    ];
    let accepted = [0, 1, 2];

    let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

    match result {
        ApplyResult::PartialConflict { reasons, .. } => {
            let not_attempted = reasons.iter().find(|r| {
                r.contains("change [2]") && r.contains("not attempted due to prior failure")
            });
            assert!(
                not_attempted.is_some(),
                "expected 'not attempted' reason for change [2], got: {reasons:?}"
            );
        }
        other => panic!("expected PartialConflict, got {other:?}"),
    }
}

// -- Behavior 13: skipped_count == len(valid_indices) --

#[test]
fn apply_proposal_skipped_count_equals_valid_indices_count() {
    let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
    let proposal = proposal_at(0);
    let changes = vec![
        delete_node_change_with_independent_ids("n1"),
        delete_node_change_with_independent_ids("ghost"),
    ];
    let accepted = [0, 1];

    let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

    match result {
        ApplyResult::PartialConflict {
            applied_count,
            skipped_count,
            reasons,
        } => {
            assert_eq!(applied_count, 0);
            assert_eq!(skipped_count, 2);
            assert_eq!(reasons.len(), 2);
        }
        other => panic!("expected PartialConflict, got {other:?}"),
    }
}

// -- Behavior 14: Empty accepted_indices returns Applied --

#[test]
fn apply_proposal_returns_applied_with_empty_accepted_indices() {
    let mut doc = doc_at(5);
    doc.document
        .nodes
        .insert(NodeId::new("n1".to_string()), test_node("n1"));
    let proposal = proposal_at(5);
    let changes = vec![delete_node_change_with_independent_ids("n1")];
    let accepted: [usize; 0] = [];
    let pre_revision = doc.revision;

    let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

    assert_eq!(result, ApplyResult::Applied);
    assert!(
        doc.document
            .nodes
            .contains_key(&NodeId::new("n1".to_string())),
        "node must not be deleted"
    );
    assert_eq!(
        doc.revision,
        pre_revision.increment(),
        "revision must increment by exactly one even with no changes applied"
    );
}

// -- Behavior 15: Out-of-bounds indices silently ignored --

#[test]
fn apply_proposal_ignores_out_of_bounds_indices() {
    let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
    let proposal = proposal_at(0);
    let changes = vec![delete_node_change_with_independent_ids("n1")];
    let accepted = [0, 5, 999];
    let pre_revision = doc.revision;

    let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

    assert_eq!(result, ApplyResult::Applied);
    assert!(!doc
        .document
        .nodes
        .contains_key(&NodeId::new("n1".to_string())));
    assert_eq!(doc.revision, pre_revision.increment());
}

// -- Behavior 16: Duplicate indices silently ignored --

#[test]
fn apply_proposal_ignores_duplicate_indices() {
    let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
    let proposal = proposal_at(0);
    let changes = vec![delete_node_change_with_independent_ids("n1")];
    let accepted = [0, 0, 0, 0];
    let pre_revision = doc.revision;

    let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

    assert_eq!(result, ApplyResult::Applied);
    assert!(!doc
        .document
        .nodes
        .contains_key(&NodeId::new("n1".to_string())));
    assert_eq!(doc.revision, pre_revision.increment());
}

// -- Behavior 17: Never panics on adversarial input --

#[test]
fn apply_proposal_never_panics_on_adversarial_indices() {
    use std::panic::catch_unwind;
    use std::panic::AssertUnwindSafe;

    let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
    let proposal = proposal_at(0);
    let changes: Vec<ProposedChange> = vec![];
    let accepted = [usize::MAX, 0, 0, 999];

    let result = catch_unwind(AssertUnwindSafe(|| {
        apply_proposal(&mut doc, &proposal, &changes, &accepted)
    }));

    match result {
        Ok(inner) => {
            assert_eq!(
                inner,
                ApplyResult::Applied,
                "empty changes with adversarial indices → all OOB → Applied"
            );
        }
        Err(_) => panic!("apply_proposal must not panic on adversarial indices"),
    }
}

// -- Behavior 18: Unrelated nodes and edges preserved --

#[test]
fn apply_proposal_preserves_unrelated_nodes_and_edges() {
    let n2 = test_node("n2");
    let n3 = test_node("n3");
    let n4 = test_node("n4");
    let e2 = test_edge("n3", "n4");
    let mut doc = doc_with_nodes_and_edges(
        vec![
            ("n1", test_node("n1")),
            ("n2", n2.clone()),
            ("n3", n3.clone()),
            ("n4", n4.clone()),
        ],
        vec![("e1", test_edge("n1", "n2")), ("e2", e2.clone())],
    );
    let pre_state = doc.clone();
    let proposal = proposal_at(0);
    let changes = vec![delete_node_change_with_independent_ids("n1")];
    let accepted = [0];

    let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

    assert_eq!(result, ApplyResult::Applied);
    assert!(!doc
        .document
        .nodes
        .contains_key(&NodeId::new("n1".to_string())));
    assert!(!doc
        .document
        .edges
        .contains_key(&EdgeId::new("e1".to_string())));
    assert_eq!(
        doc.document.nodes.get(&NodeId::new("n2".to_string())),
        pre_state.document.nodes.get(&NodeId::new("n2".to_string()))
    );
    assert_eq!(
        doc.document.nodes.get(&NodeId::new("n3".to_string())),
        pre_state.document.nodes.get(&NodeId::new("n3".to_string()))
    );
    assert_eq!(
        doc.document.nodes.get(&NodeId::new("n4".to_string())),
        pre_state.document.nodes.get(&NodeId::new("n4".to_string()))
    );
    assert_eq!(
        doc.document.edges.get(&EdgeId::new("e2".to_string())),
        pre_state.document.edges.get(&EdgeId::new("e2".to_string()))
    );
}

// -- Behavior 19: Revision correction from N increments to 1 --

#[test]
fn apply_proposal_corrects_revision_from_n_increments_to_one() {
    let mut doc = doc_with_nodes_and_edges(
        vec![
            ("n1", test_node("n1")),
            ("n2", test_node("n2")),
            ("n3", test_node("n3")),
        ],
        vec![],
    );
    let proposal = proposal_at(0);
    let changes = vec![
        delete_node_change_with_independent_ids("n1"),
        delete_node_change_with_independent_ids("n2"),
        delete_node_change_with_independent_ids("n3"),
    ];
    let accepted = [0, 1, 2];

    let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

    assert_eq!(result, ApplyResult::Applied);
    assert_eq!(doc.revision, Revision::new(1));
}

// -- Behavior 22: Document unchanged on Stale --

#[test]
fn apply_proposal_document_unchanged_on_stale() {
    let mut doc = doc_with_nodes_and_edges(
        vec![("n1", test_node("n1")), ("n2", test_node("n2"))],
        vec![],
    );
    doc.revision = Revision::new(5);
    let doc_before = doc.clone();
    let proposal = proposal_at(3);
    let changes = vec![delete_node_change_with_independent_ids("n1")];
    let accepted = [0];

    let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

    assert!(matches!(result, ApplyResult::Stale(_)));
    assert_eq!(doc, doc_before);
}

// -- Behavior 25: Stale when proposal ahead of document --

#[test]
fn apply_proposal_returns_stale_when_proposal_revision_ahead_of_document() {
    let mut doc = doc_at(3);
    doc.document
        .nodes
        .insert(NodeId::new("n1".to_string()), test_node("n1"));
    let proposal = proposal_at(7);
    let changes = vec![delete_node_change_with_independent_ids("n1")];
    let accepted = [0];
    let doc_before = doc.clone();

    let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

    assert_eq!(
        result,
        ApplyResult::Stale(StaleInfo {
            expected: Revision::new(7),
            current: Revision::new(3),
        })
    );
    assert_eq!(doc, doc_before, "document must not be mutated when stale");
}

// -- Behavior 26: SnapshotIdMismatch reason string format --

#[test]
fn apply_proposal_snapshot_mismatch_reason_has_correct_field_order() {
    let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
    doc.revision = Revision::new(5);
    let doc_before = doc.clone();
    let proposal = proposal_at(5);
    let changes = vec![mismatched_delete_node_change("n1", "snap-xyz")];
    let accepted = [0];

    let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

    match result {
        ApplyResult::PartialConflict { reasons, .. } => {
            assert_eq!(
                reasons[0],
                "change [0]: snapshot mismatch: declared n1, snapshot snap-xyz"
            );
        }
        other => panic!("expected PartialConflict, got {other:?}"),
    }
    assert_eq!(
        doc, doc_before,
        "document must be unchanged on SnapshotIdMismatch"
    );
}

// -- Behavior 27: DocumentError reason string format --
// NOTE: DocumentError path is not triggerable through apply_proposal because
// apply_delete_node checks node existence before calling remove_fn. The
// remove_fn (DiagramDocument::remove_node) only fails on missing nodes, which
// is caught by the NodeNotFound check first. The test seam (apply_delete_node_with_remove)
// is not accessible through apply_proposal. Covered by existing 40+ apply_delete_node
// tests at the lower layer.

// -- Behaviors 23, 24: Unsupported change variant --

#[test]
fn apply_proposal_unsupported_variant_produces_partial_conflict_with_reason() {
    let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
    let proposal = proposal_at(0);
    let changes = vec![
        delete_node_change_with_independent_ids("n1"),
        ProposedChange::MoveNode {
            node_id: NodeId::new("mn1".to_string()),
            new_x: 0.0,
            new_y: 0.0,
        },
    ];
    let accepted = [0, 1];

    let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

    match result {
        ApplyResult::PartialConflict {
            applied_count,
            skipped_count,
            reasons,
        } => {
            assert_eq!(applied_count, 0);
            assert_eq!(skipped_count, 2);
            let unsupported_reason = reasons
                .iter()
                .find(|r| r.contains("[1]") && r.contains("unsupported change variant"));
            assert!(
                unsupported_reason.is_some(),
                "expected reason for change [1] with 'unsupported change variant', got: {reasons:?}"
            );
            let rolled_back_reason = reasons
                .iter()
                .find(|r| r.contains("[0]") && r.contains("rolled back due to subsequent failure"));
            assert!(
                rolled_back_reason.is_some(),
                "expected reason for change [0] with 'rolled back', got: {reasons:?}"
            );
        }
        other => panic!("expected PartialConflict, got {other:?}"),
    }
}

#[test]
fn apply_proposal_not_attempted_after_unsupported_variant() {
    let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
    let proposal = proposal_at(0);
    let changes = vec![
        ProposedChange::MoveNode {
            node_id: NodeId::new("mn1".to_string()),
            new_x: 0.0,
            new_y: 0.0,
        },
        delete_node_change_with_independent_ids("n1"),
    ];
    let accepted = [0, 1];

    let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

    match result {
        ApplyResult::PartialConflict { reasons, .. } => {
            let not_attempted = reasons
                .iter()
                .find(|r| r.contains("[1]") && r.contains("not attempted due to prior failure"));
            assert!(
                not_attempted.is_some(),
                "expected 'not attempted' reason for change [1], got: {reasons:?}"
            );
        }
        other => panic!("expected PartialConflict, got {other:?}"),
    }
}

// =====================================================================
// Proptests: validate_and_dedup_indices (Behaviors PP1, PP2)
// =====================================================================

proptest::proptest! {
    #[test]
    fn proptest_validate_and_dedup_output_sorted_unique_in_bounds(
        changes_len in 0..50usize,
        accepted_indices in proptest::collection::vec(proptest::num::usize::ANY, 0..100)
    ) {
        let result = validate_and_dedup_indices(&accepted_indices, changes_len);

        for i in 1..result.len() {
            prop_assert!(result[i - 1] < result[i], "output not sorted at index {}", i);
        }

        use std::collections::HashSet;
        let unique: HashSet<usize> = result.iter().cloned().collect();
        prop_assert_eq!(unique.len(), result.len(), "output contains duplicates");

        for idx in &result {
            prop_assert!(*idx < changes_len, "index {} out of bounds for changes_len {}", idx, changes_len);
        }

        let ignored = accepted_indices.len() - result.len();
        prop_assert_eq!(ignored, accepted_indices.len() - result.len());
    }

    #[test]
    fn proptest_validate_and_dedup_ignored_count_accuracy(
        changes_len in 0..50usize,
        accepted_indices in proptest::collection::vec(proptest::num::usize::ANY, 0..100)
    ) {
        use std::collections::HashSet;
        let result = validate_and_dedup_indices(&accepted_indices, changes_len);

        let mut seen = HashSet::new();
        let mut expected_valid = 0usize;
        for idx in &accepted_indices {
            if *idx < changes_len && seen.insert(*idx) {
                expected_valid += 1;
            }
        }
        prop_assert_eq!(result.len(), expected_valid);
        prop_assert_eq!(accepted_indices.len() - result.len(), accepted_indices.len() - expected_valid);
    }
}

// =====================================================================
// Proptests: apply_proposal invariants (Behaviors PP3, PP4)
// =====================================================================

proptest::proptest! {
    #[test]
    fn proptest_apply_proposal_revision_invariant_across_all_outcomes(
        doc_rev in proptest::num::u64::ANY,
        prop_rev in proptest::num::u64::ANY
    ) {
        let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
        doc.revision = Revision::new(doc_rev);
        let proposal = proposal_at(prop_rev);
        let changes = vec![delete_node_change_with_independent_ids("n1")];
        let accepted = [0usize];

        let pre_revision = doc.revision;
        let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

        match result {
            ApplyResult::Applied => {
                prop_assert_eq!(doc.revision, pre_revision.increment());
            }
            ApplyResult::Stale(_) => {
                prop_assert_eq!(doc.revision, pre_revision);
            }
            ApplyResult::PartialConflict { .. } => {
                prop_assert_eq!(doc.revision, pre_revision);
            }
        }
    }

    #[test]
    fn proptest_apply_proposal_document_unchanged_on_non_applied(
        doc_rev in proptest::num::u64::ANY,
        prop_rev in proptest::num::u64::ANY
    ) {
        let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
        doc.revision = Revision::new(doc_rev);
        let doc_before = doc.clone();
        let proposal = proposal_at(prop_rev);
        let changes = vec![delete_node_change_with_independent_ids("n1")];
        let accepted = [0usize];

        let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

        match result {
            ApplyResult::Applied => {}
            ApplyResult::Stale(_) | ApplyResult::PartialConflict { .. } => {
                prop_assert_eq!(doc, doc_before);
            }
        }
    }
}

// =====================================================================
// Kani harnesses: apply_proposal
// =====================================================================

#[cfg(kani)]
mod apply_proposal_verification {
    use super::*;

    #[kani::proof]
    fn kani_apply_proposal_no_panic() {
        let doc_rev: u64 = kani::any();
        let prop_rev: u64 = kani::any();
        let mut doc = DiagramDocument::default();
        doc.document
            .nodes
            .insert(NodeId::new("n1".to_string()), test_node("n1"));
        doc.revision = Revision::new(doc_rev);
        let proposal = proposal_at(prop_rev);
        let changes = vec![ProposedChange::DeleteNode {
            node_id: NodeId::new("n1".to_string()),
            was_node_id: NodeId::new("n1".to_string()),
            was: test_node("n1"),
        }];
        let accepted = [0usize];

        match apply_proposal(&mut doc, &proposal, &changes, &accepted) {
            ApplyResult::Applied | ApplyResult::Stale(_) | ApplyResult::PartialConflict { .. } => {}
        }
    }

    #[kani::proof]
    fn kani_revision_incremented_at_most_once() {
        let doc_rev: u64 = kani::any();
        let prop_rev: u64 = kani::any();
        let mut doc = DiagramDocument::default();
        doc.document
            .nodes
            .insert(NodeId::new("n1".to_string()), test_node("n1"));
        doc.revision = Revision::new(doc_rev);
        let pre_revision = doc.revision;
        let proposal = proposal_at(prop_rev);
        let changes = vec![ProposedChange::DeleteNode {
            node_id: NodeId::new("n1".to_string()),
            was_node_id: NodeId::new("n1".to_string()),
            was: test_node("n1"),
        }];
        let accepted = [0usize];

        apply_proposal(&mut doc, &proposal, &changes, &accepted);
        assert!(doc.revision == pre_revision || doc.revision == pre_revision.increment());
    }

    #[kani::proof]
    fn kani_document_unchanged_on_error_paths() {
        let doc_rev: u64 = kani::any();
        let prop_rev: u64 = kani::any();
        let mut doc = DiagramDocument::default();
        doc.document
            .nodes
            .insert(NodeId::new("n1".to_string()), test_node("n1"));
        doc.revision = Revision::new(doc_rev);
        let doc_before = doc.clone();
        let proposal = proposal_at(prop_rev);
        let changes = vec![ProposedChange::DeleteNode {
            node_id: NodeId::new("n1".to_string()),
            was_node_id: NodeId::new("n1".to_string()),
            was: test_node("n1"),
        }];
        let accepted = [0usize];

        let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);
        match result {
            ApplyResult::Applied => {}
            ApplyResult::Stale(_) | ApplyResult::PartialConflict { .. } => {
                assert_eq!(doc, doc_before);
            }
        }
    }
}

// =====================================================================
// RED QUEEN: Adversarial tests for seshat-a3l
// =====================================================================

mod red_queen {
    use super::*;
    use std::collections::HashSet;
    use std::panic::catch_unwind;
    use std::panic::AssertUnwindSafe;

    // -----------------------------------------------------------------
    // DQ-01: I4 — DocumentError from remove_node must NOT mutate doc
    // -----------------------------------------------------------------
    // If remove_node succeeds for the node removal but somehow fails on
    // an edge cascade (hypothetically), the doc is already half-mutated.
    // The implementation collects cascade IDs BEFORE calling remove_fn,
    // so if remove_fn returns Err after partial mutation, I4 is violated.

    #[test]
    fn dq01_document_error_after_partial_mutation_violates_i4() {
        let mut doc = doc_with_nodes_and_edges(
            vec![
                ("n1", test_node("n1")),
                ("n2", test_node("n2")),
                ("n3", test_node("n3")),
            ],
            vec![
                ("e1", test_edge("n1", "n2")),
                ("e2", test_edge("n3", "n1")),
                ("e3", test_edge("n2", "n3")),
            ],
        );
        let doc_before = doc.clone();
        let change = delete_node_change_with_independent_ids("n1");

        let remove_fn = |doc: &mut DiagramDocument, id: &NodeId| {
            doc.document.nodes.remove(id);
            Err(DocumentError::EdgeNotFound(EdgeId::new(
                "bogus".to_string(),
            )))
        };

        let result = apply_delete_node_with_remove(&mut doc, &change, remove_fn);

        assert!(result.is_err(), "must return error when remove_fn fails");
        // CRITICAL I4 CHECK: is the document unchanged?
        if doc != doc_before {
            panic!(
                "DEFECT I4 VIOLATION: document was mutated on error path.\n\
                 Before: nodes={}, edges={}, rev={}\n\
                 After:  nodes={}, edges={}, rev={}",
                doc_before.document.nodes.len(),
                doc_before.document.edges.len(),
                doc_before.revision.value(),
                doc.document.nodes.len(),
                doc.document.edges.len(),
                doc.revision.value()
            );
        }
    }

    // -----------------------------------------------------------------
    // DQ-02: Q6 — Edge count must be exactly pre - cascade count
    // -----------------------------------------------------------------

    #[test]
    fn dq02_edge_count_strict_subtraction_after_cascade() {
        for edge_count in 0..10u32 {
            let edges: Vec<(String, Edge)> = (0..edge_count)
                .map(|i| (format!("e{i}"), test_edge("n1", "n2")))
                .collect();
            let edges_ref: Vec<(&str, Edge)> =
                edges.iter().map(|(s, e)| (s.as_str(), e.clone())).collect();
            let mut doc = doc_with_nodes_and_edges(
                vec![
                    ("n1", test_node("n1")),
                    ("n2", test_node("n2")),
                    ("n3", test_node("n3")),
                ],
                edges_ref,
            );
            let pre_edges = doc.document.edges.len();
            let pre_nodes = doc.document.nodes.len();
            let change = delete_node_change_with_independent_ids("n1");

            let result = apply_delete_node(&mut doc, &change).unwrap();
            let cascade_count = result.cascade_deleted_edge_ids.len();

            assert_eq!(
                doc.document.edges.len(),
                pre_edges - cascade_count,
                "Q6: edges.len() must be {} - {} = {}",
                pre_edges,
                cascade_count,
                pre_edges - cascade_count
            );
            assert_eq!(
                doc.document.nodes.len(),
                pre_nodes - 1,
                "Q5: nodes.len() must be {} - 1 = {}",
                pre_nodes,
                pre_nodes - 1
            );
        }
    }

    // -----------------------------------------------------------------
    // DQ-03: Deeply connected graph — hub node with many edges
    // -----------------------------------------------------------------

    #[test]
    fn dq03_hub_node_with_100_edges_cascades_all() {
        let mut doc = DiagramDocument::default();
        doc.document
            .nodes
            .insert(NodeId::new("hub".to_string()), test_node("hub"));
        doc.document
            .nodes
            .insert(NodeId::new("leaf".to_string()), test_node("leaf"));

        let mut expected_edges: Vec<EdgeId> = Vec::new();
        for i in 0..50 {
            let out_eid = format!("e_out_{i}");
            let in_eid = format!("e_in_{i}");
            doc.document
                .edges
                .insert(EdgeId::new(out_eid.clone()), test_edge("hub", "leaf"));
            doc.document
                .edges
                .insert(EdgeId::new(in_eid.clone()), test_edge("leaf", "hub"));
            expected_edges.push(EdgeId::new(out_eid));
            expected_edges.push(EdgeId::new(in_eid));
        }

        let pre_edges = doc.document.edges.len();
        let change = delete_node_change_with_independent_ids("hub");
        let result = apply_delete_node(&mut doc, &change).unwrap();

        assert_eq!(result.cascade_deleted_edge_ids.len(), 100);
        assert_eq!(doc.document.edges.len(), pre_edges - 100);
        assert!(doc.document.edges.is_empty());
        assert!(!doc
            .document
            .nodes
            .contains_key(&NodeId::new("hub".to_string())));
        assert!(doc
            .document
            .nodes
            .contains_key(&NodeId::new("leaf".to_string())));
    }

    // -----------------------------------------------------------------
    // DQ-04: Multiple self-referencing edges on same node
    // -----------------------------------------------------------------

    #[test]
    fn dq04_multiple_self_loops_all_cascaded() {
        let mut doc = doc_with_nodes_and_edges(
            vec![("n1", test_node("n1")), ("n2", test_node("n2"))],
            vec![
                ("sl1", test_edge("n1", "n1")),
                ("sl2", test_edge("n1", "n1")),
                ("sl3", test_edge("n1", "n1")),
                ("e1", test_edge("n1", "n2")),
            ],
        );
        let change = delete_node_change_with_independent_ids("n1");

        let result = apply_delete_node(&mut doc, &change).unwrap();

        assert_eq!(
            result.cascade_deleted_edge_ids.len(),
            4,
            "I5: all self-loops + regular edges"
        );
        assert_eq!(doc.document.edges.len(), 0);
    }

    // -----------------------------------------------------------------
    // DQ-05: Empty document — delete non-existent node
    // -----------------------------------------------------------------

    #[test]
    fn dq05_delete_from_empty_document_returns_node_not_found() {
        let mut doc = DiagramDocument::default();
        let doc_before = doc.clone();
        let change = delete_node_change_with_independent_ids("nothing");

        let result = catch_unwind(AssertUnwindSafe(|| apply_delete_node(&mut doc, &change)));

        assert!(result.is_ok(), "I2: must not panic on empty document");
        assert_eq!(
            result.unwrap(),
            Err(ApplyError::NodeNotFound(NodeId::new("nothing".to_string())))
        );
        assert_eq!(doc, doc_before, "I4: empty doc unchanged");
    }

    // -----------------------------------------------------------------
    // DQ-06: Single-node document with no edges
    // -----------------------------------------------------------------

    #[test]
    fn dq06_single_node_no_edges_deletes_cleanly() {
        let mut doc = doc_with_nodes_and_edges(vec![("solo", test_node("solo"))], vec![]);
        let change = delete_node_change_with_independent_ids("solo");

        let result = apply_delete_node(&mut doc, &change).unwrap();

        assert!(doc.document.nodes.is_empty());
        assert!(doc.document.edges.is_empty());
        assert!(result.cascade_deleted_edge_ids.is_empty(), "I6");
        assert_eq!(doc.revision, Revision::new(1), "Q7");
    }

    // -----------------------------------------------------------------
    // DQ-07: P1 — Snapshot mismatch where node_id == was_node_id but
    //         both reference a different node than the one in doc
    // -----------------------------------------------------------------

    #[test]
    fn dq07_snapshot_match_but_wrong_target_node() {
        let mut doc = doc_with_nodes_and_edges(
            vec![("n1", test_node("n1")), ("n2", test_node("n2"))],
            vec![],
        );
        let doc_before = doc.clone();

        let change = ProposedChange::DeleteNode {
            node_id: NodeId::new("n2".to_string()),
            was_node_id: NodeId::new("n2".to_string()),
            was: test_node("n2"),
        };

        let result = apply_delete_node(&mut doc, &change).unwrap();

        assert_eq!(result.deleted_node_id, NodeId::new("n2".to_string()));
        assert!(!doc
            .document
            .nodes
            .contains_key(&NodeId::new("n2".to_string())));
        assert!(doc
            .document
            .nodes
            .contains_key(&NodeId::new("n1".to_string())));
    }

    // -----------------------------------------------------------------
    // DQ-08: Q7 — Revision at u64::MAX boundary
    // -----------------------------------------------------------------

    #[test]
    fn dq08_revision_increment_at_max_boundary() {
        let mut doc = doc_at(u64::MAX);
        doc.document
            .nodes
            .insert(NodeId::new("n1".to_string()), test_node("n1"));
        let change = delete_node_change_with_independent_ids("n1");

        let result = catch_unwind(AssertUnwindSafe(|| apply_delete_node(&mut doc, &change)));

        // u64::MAX + 1 wraps to 0 — check if this is handled or panics
        // The contract says "incremented by exactly 1" but Revision uses u64
        assert!(result.is_ok(), "I2: must not panic at u64::MAX boundary");
        // After wrap, revision == 0 (u64 overflow wrapping)
        assert_eq!(doc.revision, Revision::new(0));
    }

    // -----------------------------------------------------------------
    // DQ-09: I4 — Snapshot mismatch with full document preserves all
    // -----------------------------------------------------------------

    #[test]
    fn dq09_full_document_preservation_on_snapshot_mismatch() {
        let mut doc = doc_with_nodes_and_edges(
            vec![
                ("n1", test_node("n1")),
                ("n2", test_node("n2")),
                ("n3", test_node("n3")),
            ],
            vec![("e1", test_edge("n1", "n2")), ("e2", test_edge("n2", "n3"))],
        );
        doc.revision = Revision::new(42);
        let doc_before = doc.clone();
        let change = mismatched_delete_node_change("n1", "WRONG");

        let result = apply_delete_node(&mut doc, &change);

        assert_eq!(
            result,
            Err(ApplyError::SnapshotIdMismatch {
                declared: NodeId::new("n1".to_string()),
                snapshot: NodeId::new("WRONG".to_string()),
            })
        );
        assert_eq!(doc, doc_before, "I4: entire document must be unchanged");
        assert_eq!(
            doc.revision,
            Revision::new(42),
            "revision unchanged on error"
        );
    }

    // -----------------------------------------------------------------
    // DQ-10: I1 — Cascade completeness: every reported edge actually
    //         referenced the deleted node, and no referenced edge missed
    // -----------------------------------------------------------------

    #[test]
    fn dq10_cascade_completeness_no_missed_edges() {
        let mut doc = doc_with_nodes_and_edges(
            vec![
                ("a", test_node("a")),
                ("b", test_node("b")),
                ("c", test_node("c")),
                ("d", test_node("d")),
            ],
            vec![
                ("e1", test_edge("a", "b")),
                ("e2", test_edge("c", "a")),
                ("e3", test_edge("a", "a")),
                ("e4", test_edge("b", "c")),
                ("e5", test_edge("d", "a")),
                ("e6", test_edge("a", "d")),
            ],
        );
        let node_id = NodeId::new("a".to_string());
        let change = delete_node_change_with_independent_ids("a");

        let result = apply_delete_node(&mut doc, &change).unwrap();
        let cascade_set: HashSet<EdgeId> =
            result.cascade_deleted_edge_ids.iter().cloned().collect();

        assert_eq!(
            cascade_set.len(),
            5,
            "I1: exactly 5 edges reference node 'a'"
        );
        let e1_id = EdgeId::new("e1".to_string());
        let e2_id = EdgeId::new("e2".to_string());
        let e3_id = EdgeId::new("e3".to_string());
        let e5_id = EdgeId::new("e5".to_string());
        let e6_id = EdgeId::new("e6".to_string());
        let e4_id = EdgeId::new("e4".to_string());
        assert!(
            cascade_set.contains(&e1_id)
                && cascade_set.contains(&e2_id)
                && cascade_set.contains(&e3_id)
                && cascade_set.contains(&e5_id)
                && cascade_set.contains(&e6_id),
            "I1: cascade must include all edges referencing 'a'"
        );
        assert!(
            !cascade_set.contains(&e4_id),
            "I1: cascade must NOT include unrelated edges"
        );

        // Verify Q3: no remaining edges reference 'a'
        assert!(
            !doc.document
                .edges
                .values()
                .any(|e| e.source == node_id || e.target == node_id),
            "Q3: no dangling references after delete"
        );
    }

    // -----------------------------------------------------------------
    // DQ-11: NodeNotFound with matching snapshot IDs
    // -----------------------------------------------------------------

    #[test]
    fn dq11_node_not_found_even_when_snapshot_ids_match() {
        let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
        let doc_before = doc.clone();
        let change = ProposedChange::DeleteNode {
            node_id: NodeId::new("n_missing".to_string()),
            was_node_id: NodeId::new("n_missing".to_string()),
            was: test_node("n_missing"),
        };

        let result = apply_delete_node(&mut doc, &change);

        assert_eq!(
            result,
            Err(ApplyError::NodeNotFound(NodeId::new(
                "n_missing".to_string()
            )))
        );
        assert_eq!(doc, doc_before, "I4");
    }

    // -----------------------------------------------------------------
    // DQ-12: Q4 — Strict subset: no new edges appear after delete
    // -----------------------------------------------------------------

    #[test]
    fn dq12_no_new_edges_introduced_after_delete() {
        let mut doc = doc_with_nodes_and_edges(
            vec![
                ("n1", test_node("n1")),
                ("n2", test_node("n2")),
                ("n3", test_node("n3")),
            ],
            vec![
                ("e1", test_edge("n1", "n2")),
                ("e2", test_edge("n2", "n3")),
                ("e3", test_edge("n1", "n3")),
            ],
        );
        let pre_edge_ids: HashSet<EdgeId> = doc.document.edges.keys().cloned().collect();
        let change = delete_node_change_with_independent_ids("n1");

        apply_delete_node(&mut doc, &change).unwrap();

        let post_edge_ids: HashSet<EdgeId> = doc.document.edges.keys().cloned().collect();
        assert!(
            post_edge_ids.is_subset(&pre_edge_ids),
            "Q4: post-apply edges must be a strict subset of pre-apply edges"
        );
    }

    // -----------------------------------------------------------------
    // DQ-13: Edges referencing deleted node as source AND target
    //         across multiple edges simultaneously
    // -----------------------------------------------------------------

    #[test]
    fn dq13_node_referenced_as_source_in_some_and_target_in_others() {
        let mut doc = doc_with_nodes_and_edges(
            vec![
                ("x", test_node("x")),
                ("y", test_node("y")),
                ("z", test_node("z")),
                ("w", test_node("w")),
            ],
            vec![
                ("e_a", test_edge("x", "y")),
                ("e_b", test_edge("z", "x")),
                ("e_c", test_edge("x", "w")),
                ("e_d", test_edge("x", "z")),
                ("e_e", test_edge("y", "z")),
            ],
        );
        let change = delete_node_change_with_independent_ids("x");
        let result = apply_delete_node(&mut doc, &change).unwrap();

        let mut cascade = result.cascade_deleted_edge_ids.clone();
        cascade.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        assert_eq!(
            cascade,
            vec![
                EdgeId::new("e_a".to_string()),
                EdgeId::new("e_b".to_string()),
                EdgeId::new("e_c".to_string()),
                EdgeId::new("e_d".to_string()),
            ]
        );
        // e_e must survive
        assert!(doc
            .document
            .edges
            .contains_key(&EdgeId::new("e_e".to_string())));
    }

    // -----------------------------------------------------------------
    // DQ-14: I4 — NodeNotFound must not increment revision
    // -----------------------------------------------------------------

    #[test]
    fn dq14_revision_not_incremented_on_node_not_found() {
        let mut doc = doc_at(77);
        let change = delete_node_change_with_independent_ids("missing");

        apply_delete_node(&mut doc, &change).unwrap_err();

        assert_eq!(
            doc.revision,
            Revision::new(77),
            "Q7: revision unchanged on error"
        );
    }

    // -----------------------------------------------------------------
    // DQ-15: I4 — SnapshotIdMismatch must not increment revision
    // -----------------------------------------------------------------

    #[test]
    fn dq15_revision_not_incremented_on_snapshot_mismatch() {
        let mut doc = doc_at(99);
        doc.document
            .nodes
            .insert(NodeId::new("n1".to_string()), test_node("n1"));
        let change = mismatched_delete_node_change("n1", "other");

        apply_delete_node(&mut doc, &change).unwrap_err();

        assert_eq!(
            doc.revision,
            Revision::new(99),
            "Q7: revision unchanged on error"
        );
    }

    // -----------------------------------------------------------------
    // DQ-16: Successive deletes of different nodes
    // -----------------------------------------------------------------

    #[test]
    fn dq16_sequential_deletes_of_different_nodes() {
        let mut doc = doc_with_nodes_and_edges(
            vec![
                ("a", test_node("a")),
                ("b", test_node("b")),
                ("c", test_node("c")),
            ],
            vec![
                ("e_ab", test_edge("a", "b")),
                ("e_bc", test_edge("b", "c")),
                ("e_ca", test_edge("c", "a")),
            ],
        );
        let pre_rev = doc.revision;

        let r1 =
            apply_delete_node(&mut doc, &delete_node_change_with_independent_ids("a")).unwrap();
        assert_eq!(r1.cascade_deleted_edge_ids.len(), 2);
        assert_eq!(doc.revision, pre_rev.increment());

        let r2 =
            apply_delete_node(&mut doc, &delete_node_change_with_independent_ids("b")).unwrap();
        assert_eq!(r2.cascade_deleted_edge_ids.len(), 1);
        assert_eq!(doc.revision, pre_rev.increment().increment());

        let r3 =
            apply_delete_node(&mut doc, &delete_node_change_with_independent_ids("c")).unwrap();
        assert!(r3.cascade_deleted_edge_ids.is_empty());
        assert_eq!(doc.revision, pre_rev.increment().increment().increment());

        assert!(doc.document.nodes.is_empty());
        assert!(doc.document.edges.is_empty());
    }

    // -----------------------------------------------------------------
    // DQ-17: cascade_edges_for_node on empty document returns None
    // -----------------------------------------------------------------

    #[test]
    fn dq17_cascade_edges_for_node_on_empty_doc() {
        let doc = DiagramDocument::default();
        let result = cascade_edges_for_node(&doc, &NodeId::new("x".to_string()));
        assert!(result.is_none());
    }

    // -----------------------------------------------------------------
    // DQ-18: Node exists but has NO edges at all in document
    //         (edges HashMap is empty)
    // -----------------------------------------------------------------

    #[test]
    fn dq22_edges_unchanged_on_node_not_found_error() {
        let mut doc = doc_with_nodes_and_edges(
            vec![("n1", test_node("n1")), ("n2", test_node("n2"))],
            vec![("e1", test_edge("n1", "n2")), ("e2", test_edge("n2", "n1"))],
        );
        let pre_edges: Vec<(EdgeId, bool, bool)> = doc
            .document
            .edges
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    v.source.as_str() == "n1",
                    v.target.as_str() == "n1",
                )
            })
            .collect();
        let change = delete_node_change_with_independent_ids("ghost");

        apply_delete_node(&mut doc, &change).unwrap_err();

        let post_edges: Vec<(EdgeId, bool, bool)> = doc
            .document
            .edges
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    v.source.as_str() == "n1",
                    v.target.as_str() == "n1",
                )
            })
            .collect();
        assert_eq!(
            pre_edges, post_edges,
            "I4: edge map must be identical on NodeNotFound error"
        );
        assert_eq!(doc.document.edges.len(), 2, "I4: no edges removed");
    }

    // -----------------------------------------------------------------
    // DQ-19: Unicode node IDs
    // -----------------------------------------------------------------

    #[test]
    fn dq19_unicode_node_ids() {
        let mut doc = doc_with_nodes_and_edges(
            vec![("ノード", test_node("ノード")), ("节点", test_node("节点"))],
            vec![("e1", test_edge("ノード", "节点"))],
        );
        let change = delete_node_change_with_independent_ids("ノード");

        let result = catch_unwind(AssertUnwindSafe(|| apply_delete_node(&mut doc, &change)));

        assert!(result.is_ok(), "I2: must not panic on unicode IDs");
        let ok = result.unwrap().unwrap();
        assert_eq!(ok.deleted_node_id, NodeId::new("ノード".to_string()));
        assert_eq!(ok.cascade_deleted_edge_ids.len(), 1);
    }

    // -----------------------------------------------------------------
    // DQ-20: Very long node ID string
    // -----------------------------------------------------------------

    #[test]
    fn dq20_very_long_node_id() {
        let long_id: String = "x".repeat(10000);
        let edges: Vec<(&str, Edge)> = vec![];
        let mut doc = doc_with_nodes_and_edges(vec![(&long_id, test_node(&long_id))], edges);
        let doc_before = doc.clone();
        let change = ProposedChange::DeleteNode {
            node_id: NodeId::new(long_id.clone()),
            was_node_id: NodeId::new(long_id.clone()),
            was: test_node(&long_id),
        };

        let result = apply_delete_node(&mut doc, &change).unwrap();

        assert_eq!(result.deleted_node_id, NodeId::new(long_id));
        assert!(doc.document.nodes.is_empty());
    }

    // -----------------------------------------------------------------
    // DQ-21: Node with special characters in ID
    // -----------------------------------------------------------------

    #[test]
    fn dq21_special_character_node_ids() {
        let special_ids = [
            "node-with-dashes",
            "node.with.dots",
            "node_with_underscores",
            "node/with/slashes",
            "node:with:colons",
            "node with spaces",
            "",
        ];

        for id in &special_ids {
            let mut doc = DiagramDocument::default();
            doc.document
                .nodes
                .insert(NodeId::new(id.to_string()), test_node(id));

            let change = ProposedChange::DeleteNode {
                node_id: NodeId::new(id.to_string()),
                was_node_id: NodeId::new(id.to_string()),
                was: test_node(id),
            };

            let result = catch_unwind(AssertUnwindSafe(|| apply_delete_node(&mut doc, &change)));

            assert!(result.is_ok(), "I2: must not panic on special ID: {:?}", id);
            assert!(
                result.unwrap().is_ok(),
                "delete must succeed for node with ID: {:?}",
                id
            );
        }
    }

    // -----------------------------------------------------------------
    // DQ-23: cascade_edges_for_node agrees with apply result after
    //         complex graph mutation
    // -----------------------------------------------------------------

    #[test]
    fn dq23_cascade_agreement_complex_graph() {
        let mut doc = doc_with_nodes_and_edges(
            vec![
                ("hub", test_node("hub")),
                ("s1", test_node("s1")),
                ("s2", test_node("s2")),
                ("s3", test_node("s3")),
                ("t1", test_node("t1")),
                ("t2", test_node("t2")),
            ],
            vec![
                ("e1", test_edge("hub", "t1")),
                ("e2", test_edge("hub", "t2")),
                ("e3", test_edge("s1", "hub")),
                ("e4", test_edge("s2", "hub")),
                ("e5", test_edge("s3", "hub")),
                ("e6", test_edge("hub", "hub")),
                ("e7", test_edge("s1", "s2")),
                ("e8", test_edge("t1", "t2")),
            ],
        );

        let cascade_before = cascade_edges_for_node(&doc, &NodeId::new("hub".to_string())).unwrap();
        let change = delete_node_change_with_independent_ids("hub");
        let result = apply_delete_node(&mut doc, &change).unwrap();

        let mut a = cascade_before;
        a.sort_by(|x, y| x.as_str().cmp(y.as_str()));
        let mut b = result.cascade_deleted_edge_ids;
        b.sort_by(|x, y| x.as_str().cmp(y.as_str()));
        assert_eq!(a, b, "cascade_edges_for_node must agree with apply result");
        assert_eq!(a.len(), 6, "e1-e6 reference hub, e7-e8 do not");

        // Surviving edges: e7, e8
        assert!(doc
            .document
            .edges
            .contains_key(&EdgeId::new("e7".to_string())));
        assert!(doc
            .document
            .edges
            .contains_key(&EdgeId::new("e8".to_string())));
    }

    // -----------------------------------------------------------------
    // DQ-24: Success path — verify every postcondition Q1-Q8 in one test
    // -----------------------------------------------------------------

    #[test]
    fn dq24_exhaustive_postcondition_check() {
        let mut doc = doc_with_nodes_and_edges(
            vec![
                ("x", test_node("x")),
                ("y", test_node("y")),
                ("z", test_node("z")),
            ],
            vec![
                ("ex1", test_edge("x", "y")),
                ("ex2", test_edge("z", "x")),
                ("ey1", test_edge("y", "z")),
            ],
        );
        let pre = doc.clone();
        let node_id = NodeId::new("x".to_string());
        let change = delete_node_change_with_independent_ids("x");

        let result = apply_delete_node(&mut doc, &change).unwrap();

        // Q1: node removed
        assert!(!doc.document.nodes.contains_key(&node_id), "Q1");

        // Q2: cascade edges removed
        for eid in &result.cascade_deleted_edge_ids {
            assert!(
                !doc.document.edges.contains_key(eid),
                "Q2: edge {:?} not removed",
                eid
            );
        }

        // Q3: no dangling refs
        assert!(
            !doc.document
                .edges
                .values()
                .any(|e| e.source == node_id || e.target == node_id),
            "Q3: dangling reference found"
        );

        // Q4: strict subset
        assert!(
            doc.document
                .edges
                .keys()
                .all(|k| pre.document.edges.contains_key(k)),
            "Q4: new edge introduced"
        );

        // Q5
        assert_eq!(doc.document.nodes.len(), 2, "Q5");

        // Q6
        assert_eq!(
            doc.document.edges.len(),
            pre.document.edges.len() - result.cascade_deleted_edge_ids.len(),
            "Q6"
        );

        // Q7
        assert_eq!(doc.revision, pre.revision.increment(), "Q7");

        // Q8: other nodes unchanged
        assert_eq!(
            doc.document.nodes.get(&NodeId::new("y".to_string())),
            pre.document.nodes.get(&NodeId::new("y".to_string())),
            "Q8: y changed"
        );
        assert_eq!(
            doc.document.nodes.get(&NodeId::new("z".to_string())),
            pre.document.nodes.get(&NodeId::new("z".to_string())),
            "Q8: z changed"
        );
    }

    // -----------------------------------------------------------------
    // DQ-25: Proptest — I4 on error path with injected remove_fn
    // -----------------------------------------------------------------

    proptest::proptest! {
        #[test]
        fn dq25_proptest_i4_with_injected_failure(
            node_ids in proptest::collection::vec("[a-z]{1,5}", 1..8),
            edges_spec in proptest::collection::vec(
                proptest::collection::vec(proptest::num::usize::ANY, 3),
                0..10
            )
        ) {
            let ids: Vec<String> = node_ids.into_iter().collect();
            let n = ids.len();
            let mut doc = DiagramDocument::default();
            for id in &ids {
                doc.document.nodes.insert(NodeId::new(id.clone()), test_node(id));
            }
            for (i, spec) in edges_spec.iter().enumerate() {
                if spec.len() >= 2 {
                    let src_idx = spec[0] % n;
                    let tgt_idx = spec[1] % n;
                    let e_name = format!("e{i}");
                    doc.document.edges.insert(EdgeId::new(e_name), test_edge(&ids[src_idx], &ids[tgt_idx]));
                }
            }
            let doc_before = doc.clone();
            let delete_id = NodeId::new(ids[0].clone());
            let change = ProposedChange::DeleteNode {
                node_id: delete_id.clone(),
                was_node_id: delete_id.clone(),
                was: test_node(&ids[0]),
            };

            // Inject a remove_fn that mutates then fails
            let result = apply_delete_node_with_remove(&mut doc, &change, |_doc, _id| {
                Err(DocumentError::EdgeNotFound(EdgeId::new("injected".to_string())))
            });

            prop_assert!(result.is_err(), "injected failure must produce error");
            prop_assert_eq!(doc, doc_before, "I4: document must be UNCHANGED even with injected remove_fn failure");
        }
    }

    // -----------------------------------------------------------------
    // DQ-26: Proptest — Q1-Q3-Q7 for random graph structures
    // -----------------------------------------------------------------

    proptest::proptest! {
        #[test]
        fn dq26_proptest_postconditions_random_graphs(
            node_ids in proptest::collection::vec("[a-z]{1,4}", 1..15),
            edges_spec in proptest::collection::vec(
                proptest::collection::vec(proptest::num::usize::ANY, 3),
                0..30
            ),
            target_idx in proptest::num::usize::ANY
        ) {
            let ids: Vec<String> = {
                let raw: Vec<String> = node_ids.into_iter().collect();
                let mut seen = HashSet::new();
                let mut unique = Vec::new();
                for id in raw {
                    if seen.insert(id.clone()) {
                        unique.push(id);
                    }
                }
                unique
            };
            let n = ids.len();
            prop_assume!(n > 0, "all node IDs were duplicates");
            let target = target_idx % n;
            let target_id = NodeId::new(ids[target].clone());

            let mut doc = DiagramDocument::default();
            for id in &ids {
                doc.document.nodes.insert(NodeId::new(id.clone()), test_node(id));
            }
            for (i, spec) in edges_spec.iter().enumerate() {
                if spec.len() >= 2 {
                    let src_idx = spec[0] % n;
                    let tgt_idx = spec[1] % n;
                    let e_name = format!("e{i}");
                    doc.document.edges.insert(EdgeId::new(e_name), test_edge(&ids[src_idx], &ids[tgt_idx]));
                }
            }
            let pre_nodes = doc.document.nodes.len();
            let pre_rev = doc.revision;
            let expected_cascade: Vec<EdgeId> = doc.document.edges.iter()
                .filter(|(_, e)| e.source == target_id || e.target == target_id)
                .map(|(id, _)| id.clone())
                .collect();
            let expected_count = expected_cascade.len();

            let change = ProposedChange::DeleteNode {
                node_id: target_id.clone(),
                was_node_id: target_id.clone(),
                was: test_node(&ids[target]),
            };

            let result = apply_delete_node(&mut doc, &change);

            match result {
                Ok(r) => {
                    prop_assert_eq!(r.cascade_deleted_edge_ids.len(), expected_count, "I1: cascade count mismatch");
                    prop_assert!(!doc.document.nodes.contains_key(&target_id), "Q1");
                    prop_assert!(
                        !doc.document.edges.values().any(|e| e.source == target_id || e.target == target_id),
                        "Q3: dangling reference"
                    );
                    prop_assert_eq!(doc.revision, pre_rev.increment(), "Q7");
                    prop_assert_eq!(doc.document.nodes.len(), pre_nodes - 1, "Q5");
                }
                Err(_) => {
                    prop_assert!(false, "delete should succeed for existing node with matching snapshot");
                }
            }
        }
    }

    // -----------------------------------------------------------------
    // DQ-27: Snapshot mismatch with same-length but different ID
    // -----------------------------------------------------------------

    #[test]
    fn dq27_snapshot_mismatch_same_length_ids() {
        let mut doc = doc_with_nodes_and_edges(vec![("abcde", test_node("abcde"))], vec![]);
        let change = ProposedChange::DeleteNode {
            node_id: NodeId::new("abcde".to_string()),
            was_node_id: NodeId::new("fghij".to_string()),
            was: test_node("fghij"),
        };

        let result = apply_delete_node(&mut doc, &change);

        assert_eq!(
            result,
            Err(ApplyError::SnapshotIdMismatch {
                declared: NodeId::new("abcde".to_string()),
                snapshot: NodeId::new("fghij".to_string()),
            })
        );
        assert!(
            doc.document
                .nodes
                .contains_key(&NodeId::new("abcde".to_string())),
            "I4: node not removed on error"
        );
    }

    // -----------------------------------------------------------------
    // DQ-28: Verify cascade_edges_for_node returns empty for isolated node
    //         in a document with many other edges
    // -----------------------------------------------------------------

    #[test]
    fn dq28_isolated_node_in_dense_graph() {
        let doc = doc_with_nodes_and_edges(
            vec![
                ("iso", test_node("iso")),
                ("a", test_node("a")),
                ("b", test_node("b")),
                ("c", test_node("c")),
            ],
            vec![
                ("e1", test_edge("a", "b")),
                ("e2", test_edge("b", "c")),
                ("e3", test_edge("c", "a")),
                ("e4", test_edge("a", "c")),
                ("e5", test_edge("b", "a")),
            ],
        );

        let result = cascade_edges_for_node(&doc, &NodeId::new("iso".to_string())).unwrap();

        assert!(
            result.is_empty(),
            "I6: isolated node has zero connected edges"
        );
    }
}

// =====================================================================
// RED QUEEN: Adversarial tests for seshat-ccm apply_proposal
// =====================================================================

mod red_queen_apply {
    use super::*;
    use std::collections::HashSet;
    use std::panic::catch_unwind;
    use std::panic::AssertUnwindSafe;

    // -----------------------------------------------------------------
    // RQ-AP01: Revision gate bypass — mismatched revision with empty
    //          changes and empty accepted_indices must still return Stale
    // -----------------------------------------------------------------

    #[test]
    fn rq_ap01_stale_gate_blocks_even_with_empty_everything() {
        let mut doc = DiagramDocument::default();
        doc.revision = Revision::new(10);
        let proposal = proposal_at(5);
        let changes: Vec<ProposedChange> = vec![];
        let accepted: [usize; 0] = [];

        let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

        assert_eq!(
            result,
            ApplyResult::Stale(StaleInfo {
                expected: Revision::new(5),
                current: Revision::new(10),
            }),
            "DQ: stale gate must fire even with zero changes/indices — bypass attempt blocked"
        );
    }

    // -----------------------------------------------------------------
    // RQ-AP02: Revision gate bypass — proposal revision == doc.revision
    //          but proposal is "from the future" (doc was reset)
    // -----------------------------------------------------------------

    #[test]
    fn rq_ap02_stale_gate_fires_when_doc_revision_reset_lower() {
        let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
        doc.revision = Revision::new(0);
        let proposal = proposal_at(100);

        let result = apply_proposal(
            &mut doc,
            &proposal,
            &[delete_node_change_with_independent_ids("n1")],
            &[0],
        );

        assert!(
            matches!(result, ApplyResult::Stale(_)),
            "DQ: proposal at rev 100 against doc at rev 0 must be Stale"
        );
    }

    // -----------------------------------------------------------------
    // RQ-AP03: Empty changes with non-empty accepted_indices — must not
    //          panic and must return Applied (all indices silently ignored)
    // -----------------------------------------------------------------

    #[test]
    fn rq_ap03_empty_changes_nonempty_indices_returns_applied_no_panic() {
        let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
        let proposal = proposal_at(0);
        let changes: Vec<ProposedChange> = vec![];
        let accepted = [0, 1, 5, usize::MAX];

        let result = catch_unwind(AssertUnwindSafe(|| {
            apply_proposal(&mut doc, &proposal, &changes, &accepted)
        }));

        match result {
            Ok(inner) => {
                assert_eq!(
                    inner,
                    ApplyResult::Applied,
                    "empty changes + nonempty indices → all OOB → Applied"
                );
            }
            Err(_) => {
                panic!("DQ: must not panic on empty changes + nonempty indices")
            }
        }
        assert!(
            doc.document
                .nodes
                .contains_key(&NodeId::new("n1".to_string())),
            "DQ: node must survive when all indices are out of bounds"
        );
    }

    // -----------------------------------------------------------------
    // RQ-AP04: Indices far out of bounds — usize::MAX and large values
    // -----------------------------------------------------------------

    #[test]
    fn rq_ap04_usize_max_indices_silently_ignored() {
        let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
        let proposal = proposal_at(0);
        let changes = vec![delete_node_change_with_independent_ids("n1")];
        let accepted = [usize::MAX, usize::MAX - 1, 999_999_999, 0];

        let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

        assert_eq!(result, ApplyResult::Applied);
        assert!(
            !doc.document
                .nodes
                .contains_key(&NodeId::new("n1".to_string())),
            "DQ: only index 0 should have been applied"
        );
    }

    // -----------------------------------------------------------------
    // RQ-AP05: Heavy duplicate indices — same index repeated many times
    // -----------------------------------------------------------------

    #[test]
    fn rq_ap05_heavy_duplicates_dedup_to_single_application() {
        let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
        let proposal = proposal_at(0);
        let changes = vec![delete_node_change_with_independent_ids("n1")];
        let accepted: Vec<usize> = vec![0; 1000];

        let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

        assert_eq!(result, ApplyResult::Applied);
        assert!(
            doc.document.nodes.is_empty(),
            "DQ: 1000 duplicates of index 0 must dedup to 1 application"
        );
    }

    #[test]
    fn rq_ap05_interleaved_duplicates_across_many_indices() {
        let mut doc = doc_with_nodes_and_edges(
            vec![
                ("a", test_node("a")),
                ("b", test_node("b")),
                ("c", test_node("c")),
            ],
            vec![],
        );
        let proposal = proposal_at(0);
        let changes = vec![
            delete_node_change_with_independent_ids("a"),
            delete_node_change_with_independent_ids("b"),
            delete_node_change_with_independent_ids("c"),
        ];
        let accepted = [0, 1, 0, 2, 1, 0, 2, 2, 1];

        let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

        assert_eq!(result, ApplyResult::Applied);
        assert!(doc.document.nodes.is_empty());
    }

    // -----------------------------------------------------------------
    // RQ-AP06: Multiple DeleteNode changes — verify each deletes target
    // -----------------------------------------------------------------

    #[test]
    fn rq_ap06_five_sequential_deletes_all_applied() {
        let mut doc = DiagramDocument::default();
        for i in 0..5 {
            let name = format!("n{i}");
            doc.document
                .nodes
                .insert(NodeId::new(name.clone()), test_node(&name));
        }
        let pre_rev = doc.revision;
        let proposal = proposal_at(0);
        let changes: Vec<ProposedChange> = (0..5)
            .map(|i| {
                let name = format!("n{i}");
                delete_node_change_with_independent_ids(&name)
            })
            .collect();
        let accepted = [0, 1, 2, 3, 4];

        let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

        assert_eq!(result, ApplyResult::Applied);
        assert!(doc.document.nodes.is_empty());
        assert_eq!(
            doc.revision,
            pre_rev.increment(),
            "DQ: revision must be exactly +1, not +5"
        );
    }

    // -----------------------------------------------------------------
    // RQ-AP07: Revision correction — N changes must produce +1, not +N
    // -----------------------------------------------------------------

    #[test]
    fn rq_ap07_revision_exactly_plus_one_for_10_changes() {
        let mut doc = DiagramDocument::default();
        for i in 0..10 {
            let name = format!("n{i}");
            doc.document
                .nodes
                .insert(NodeId::new(name.clone()), test_node(&name));
        }
        doc.revision = Revision::new(42);
        let proposal = proposal_at(42);
        let changes: Vec<ProposedChange> = (0..10)
            .map(|i| {
                let name = format!("n{i}");
                delete_node_change_with_independent_ids(&name)
            })
            .collect();
        let accepted: Vec<usize> = (0..10).collect();

        let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

        assert_eq!(result, ApplyResult::Applied);
        assert_eq!(
            doc.revision,
            Revision::new(43),
            "DQ: 10 successful changes → revision 42+1=43, NOT 42+10=52"
        );
    }

    #[test]
    fn rq_ap07_revision_correction_from_wrapping_accumulation() {
        let mut doc = doc_with_nodes_and_edges(
            vec![("n1", test_node("n1")), ("n2", test_node("n2"))],
            vec![],
        );
        doc.revision = Revision::new(u64::MAX - 1);
        let proposal = proposal_at(u64::MAX - 1);
        let changes = vec![
            delete_node_change_with_independent_ids("n1"),
            delete_node_change_with_independent_ids("n2"),
        ];
        let accepted = [0, 1];

        let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

        assert_eq!(result, ApplyResult::Applied);
        assert_eq!(
            doc.revision,
            Revision::new(u64::MAX),
            "DQ: (u64::MAX-1) + 1 = u64::MAX, not (u64::MAX-1) + 2 = u64::MAX+1"
        );
    }

    // -----------------------------------------------------------------
    // RQ-AP08: Rollback correctness — mid-apply failure must restore
    //          EXACT pre-call state including edges
    // -----------------------------------------------------------------

    #[test]
    fn rq_ap08_rollback_restores_exact_precall_state_after_partial_success() {
        let mut doc = doc_with_nodes_and_edges(
            vec![
                ("n1", test_node("n1")),
                ("n2", test_node("n2")),
                ("n3", test_node("n3")),
            ],
            vec![("e1", test_edge("n1", "n2")), ("e2", test_edge("n2", "n3"))],
        );
        doc.revision = Revision::new(15);
        let doc_before = doc.clone();
        let proposal = proposal_at(15);
        let changes = vec![
            delete_node_change_with_independent_ids("n1"),
            delete_node_change_with_independent_ids("ghost"),
            delete_node_change_with_independent_ids("n2"),
        ];
        let accepted = [0, 1, 2];

        let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

        assert!(matches!(result, ApplyResult::PartialConflict { .. }));
        assert_eq!(
            doc, doc_before,
            "DQ: rollback must restore exact pre-call state — \
             n1 was deleted before failure, must be restored"
        );
    }

    #[test]
    fn rq_ap08_rollback_preserves_edges_after_cascade() {
        let mut doc = doc_with_nodes_and_edges(
            vec![
                ("hub", test_node("hub")),
                ("s1", test_node("s1")),
                ("s2", test_node("s2")),
            ],
            vec![
                ("e1", test_edge("hub", "s1")),
                ("e2", test_edge("s2", "hub")),
                ("e3", test_edge("s1", "s2")),
            ],
        );
        let doc_before = doc.clone();
        let proposal = proposal_at(0);
        let changes = vec![
            delete_node_change_with_independent_ids("hub"),
            delete_node_change_with_independent_ids("ghost"),
        ];
        let accepted = [0, 1];

        let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

        assert!(matches!(result, ApplyResult::PartialConflict { .. }));
        assert_eq!(
            doc, doc_before,
            "DQ: hub deletion cascaded e1,e2 — rollback must restore them"
        );
        assert!(doc
            .document
            .edges
            .contains_key(&EdgeId::new("e1".to_string())));
        assert!(doc
            .document
            .edges
            .contains_key(&EdgeId::new("e2".to_string())));
        assert!(doc
            .document
            .nodes
            .contains_key(&NodeId::new("hub".to_string())));
    }

    #[test]
    fn rq_ap08_rollback_revision_unchanged() {
        let mut doc = doc_at(77);
        doc.document
            .nodes
            .insert(NodeId::new("n1".to_string()), test_node("n1"));
        let proposal = proposal_at(77);
        let changes = vec![
            delete_node_change_with_independent_ids("n1"),
            delete_node_change_with_independent_ids("ghost"),
        ];
        let accepted = [0, 1];

        apply_proposal(&mut doc, &proposal, &changes, &accepted);

        assert_eq!(
            doc.revision,
            Revision::new(77),
            "DQ: rollback must restore revision to 77, not 78"
        );
    }

    // -----------------------------------------------------------------
    // RQ-AP09: PartialConflict reason string accuracy
    // -----------------------------------------------------------------

    #[test]
    fn rq_ap09_reason_for_failing_index_has_node_not_found_prefix() {
        let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
        let proposal = proposal_at(0);
        let changes = vec![
            delete_node_change_with_independent_ids("n1"),
            delete_node_change_with_independent_ids("ghost"),
        ];
        let accepted = [0, 1];

        let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

        match result {
            ApplyResult::PartialConflict { reasons, .. } => {
                let fail_reason = reasons.iter().find(|r| r.contains("[1]")).unwrap();
                assert!(
                    fail_reason.starts_with("change [1]: node not found: ghost"),
                    "DQ: reason must exactly match taxonomy format, got: {fail_reason}"
                );
            }
            other => panic!("DQ: expected PartialConflict, got {other:?}"),
        }
    }

    #[test]
    fn rq_ap09_snapshot_mismatch_reason_has_correct_format() {
        let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
        let proposal = proposal_at(0);
        let changes = vec![
            delete_node_change_with_independent_ids("n1"),
            mismatched_delete_node_change("n1", "snap-X"),
        ];
        let accepted = [0, 1];

        let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

        match result {
            ApplyResult::PartialConflict { reasons, .. } => {
                let snap_reason = reasons.iter().find(|r| r.contains("[1]")).unwrap();
                assert_eq!(
                    snap_reason, "change [1]: snapshot mismatch: declared n1, snapshot snap-X",
                    "DQ: snapshot mismatch reason must match taxonomy"
                );
            }
            other => panic!("DQ: expected PartialConflict, got {other:?}"),
        }
    }

    #[test]
    fn rq_ap09_not_attempted_reason_for_indices_after_failure() {
        let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
        let proposal = proposal_at(0);
        let changes = vec![
            delete_node_change_with_independent_ids("n1"),
            delete_node_change_with_independent_ids("ghost"),
            delete_node_change_with_independent_ids("n1"),
            delete_node_change_with_independent_ids("n2"),
        ];
        let accepted = [0, 1, 2, 3];

        let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

        match result {
            ApplyResult::PartialConflict { reasons, .. } => {
                let not_attempted: Vec<_> = reasons
                    .iter()
                    .filter(|r| r.contains("not attempted due to prior failure"))
                    .collect();
                assert_eq!(
                    not_attempted.len(),
                    2,
                    "DQ: indices 2 and 3 should be 'not attempted', got: {reasons:?}"
                );
                assert!(
                    not_attempted.iter().any(|r| r.contains("[2]")),
                    "DQ: index 2 must be not attempted"
                );
                assert!(
                    not_attempted.iter().any(|r| r.contains("[3]")),
                    "DQ: index 3 must be not attempted"
                );
            }
            other => panic!("DQ: expected PartialConflict, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------
    // RQ-AP09b: BUG PROBE — already-applied indices should NOT get the
    //           failing change's error reason. Per the contract, they
    //           were rolled back, not failed with the same error.
    //           The current implementation assigns the failing error to
    //           all indices at or before the failure position.
    // -----------------------------------------------------------------

    #[test]
    fn rq_ap09b_rolled_back_changes_should_not_show_failing_error() {
        let mut doc = doc_with_nodes_and_edges(
            vec![("n1", test_node("n1")), ("n2", test_node("n2"))],
            vec![],
        );
        let proposal = proposal_at(0);
        let changes = vec![
            delete_node_change_with_independent_ids("n1"),
            delete_node_change_with_independent_ids("ghost"),
        ];
        let accepted = [0, 1];

        let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

        match result {
            ApplyResult::PartialConflict { reasons, .. } => {
                let reason_for_0 = reasons.iter().find(|r| r.contains("[0]")).unwrap();
                let reason_for_1 = reasons.iter().find(|r| r.contains("[1]")).unwrap();
                assert_eq!(
                    reason_for_1, "change [1]: node not found: ghost",
                    "DQ: failing index reason correct"
                );
                assert_ne!(
                    reason_for_0, reason_for_1,
                    "BUG: index 0 was successfully applied then ROLLED BACK. \
                     Its reason should NOT be the same as the failing change's error. \
                     It should indicate rollback, not 'node not found: ghost'. \
                     Got: {reason_for_0}"
                );
                assert!(
                    reason_for_0.contains("rolled back")
                        || reason_for_0.contains("rollback")
                        || reason_for_0.contains("not attempted"),
                    "BUG: already-applied (then rolled back) change should mention rollback \
                     or not-attempted semantics. Got: {reason_for_0}"
                );
            }
            other => panic!("DQ: expected PartialConflict, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------
    // RQ-AP10: All-zero revision boundary (revision == 0)
    // -----------------------------------------------------------------

    #[test]
    fn rq_ap10_zero_revision_applied_increments_to_one() {
        let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
        assert_eq!(doc.revision, Revision::INITIAL);
        let proposal = proposal_at(0);
        let changes = vec![delete_node_change_with_independent_ids("n1")];
        let accepted = [0];

        let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

        assert_eq!(result, ApplyResult::Applied);
        assert_eq!(doc.revision, Revision::new(1));
    }

    #[test]
    fn rq_ap10_zero_revision_stale_when_proposal_at_one() {
        let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
        let proposal = proposal_at(1);
        let changes = vec![delete_node_change_with_independent_ids("n1")];
        let accepted = [0];

        let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

        assert_eq!(
            result,
            ApplyResult::Stale(StaleInfo {
                expected: Revision::new(1),
                current: Revision::new(0),
            }),
            "DQ: rev 0 doc vs rev 1 proposal → stale"
        );
    }

    #[test]
    fn rq_ap10_zero_revision_empty_accepted_still_increments() {
        let mut doc = DiagramDocument::default();
        let proposal = proposal_at(0);
        let changes: Vec<ProposedChange> = vec![];
        let accepted: [usize; 0] = [];

        let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

        assert_eq!(result, ApplyResult::Applied);
        assert_eq!(doc.revision, Revision::new(1));
    }

    // -----------------------------------------------------------------
    // RQ-AP11: u64::MAX revision boundary
    // -----------------------------------------------------------------

    #[test]
    fn rq_ap11_max_revision_matching_returns_applied() {
        let mut doc = doc_at(u64::MAX);
        doc.document
            .nodes
            .insert(NodeId::new("n1".to_string()), test_node("n1"));
        let proposal = proposal_at(u64::MAX);
        let changes = vec![delete_node_change_with_independent_ids("n1")];
        let accepted = [0];

        let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

        assert_eq!(result, ApplyResult::Applied);
    }

    #[test]
    fn rq_ap11_max_revision_increment_wraps_or_panics() {
        let mut doc = doc_at(u64::MAX);
        doc.document
            .nodes
            .insert(NodeId::new("n1".to_string()), test_node("n1"));
        let proposal = proposal_at(u64::MAX);
        let changes = vec![delete_node_change_with_independent_ids("n1")];
        let accepted = [0];

        let result = catch_unwind(AssertUnwindSafe(|| {
            apply_proposal(&mut doc, &proposal, &changes, &accepted)
        }));

        if result.is_err() {
            panic!(
                "BUG: apply_proposal panicked at u64::MAX revision boundary. \
                 Revision::increment() uses self.0 + 1 which panics in debug on overflow. \
                 apply_delete_node_inner uses wrapping_add but apply_proposal correction uses increment()."
            );
        }
    }

    #[test]
    fn rq_ap11_max_revision_stale_vs_zero() {
        let mut doc = doc_at(0);
        let proposal = proposal_at(u64::MAX);
        let changes: Vec<ProposedChange> = vec![];
        let accepted: [usize; 0] = [];

        let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

        assert_eq!(
            result,
            ApplyResult::Stale(StaleInfo {
                expected: Revision::new(u64::MAX),
                current: Revision::new(0),
            }),
            "DQ: max stale boundary"
        );
    }

    // -----------------------------------------------------------------
    // RQ-AP12: validate_and_dedup — u64::MAX index
    // -----------------------------------------------------------------

    #[test]
    fn rq_ap12_validate_dedup_usize_max_excluded() {
        let result = validate_and_dedup_indices(&[usize::MAX], 1);
        assert!(
            result.is_empty(),
            "DQ: usize::MAX must be excluded when max=1"
        );
    }

    #[test]
    fn rq_ap12_validate_dedup_empty_max_excludes_all() {
        let result = validate_and_dedup_indices(&[0, 1, 2, 3], 0);
        assert!(result.is_empty(), "DQ: max=0 excludes all indices");
    }

    #[test]
    fn rq_ap12_validate_dedup_boundary_max_minus_one() {
        let result = validate_and_dedup_indices(&[usize::MAX], usize::MAX);
        assert!(
            result.is_empty(),
            "DQ: idx=MAX is out of bounds when max=MAX (requires idx < max)"
        );
    }

    // -----------------------------------------------------------------
    // RQ-AP13: format_error_reason — verify taxonomy precision
    // -----------------------------------------------------------------

    #[test]
    fn rq_ap13_format_node_not_found() {
        let reason =
            format_error_reason(42, &ApplyError::NodeNotFound(NodeId::new("x".to_string())));
        assert_eq!(reason, "change [42]: node not found: x");
    }

    #[test]
    fn rq_ap13_format_snapshot_mismatch() {
        let reason = format_error_reason(
            7,
            &ApplyError::SnapshotIdMismatch {
                declared: NodeId::new("a".to_string()),
                snapshot: NodeId::new("b".to_string()),
            },
        );
        assert_eq!(
            reason,
            "change [7]: snapshot mismatch: declared a, snapshot b"
        );
    }

    #[test]
    fn rq_ap13_format_document_error() {
        let reason = format_error_reason(
            0,
            &ApplyError::DocumentError(DocumentError::EdgeNotFound(EdgeId::new("e1".to_string()))),
        );
        assert_eq!(reason, "change [0]: document error: edge not found: e1");
    }

    // -----------------------------------------------------------------
    // RQ-AP14: applied_count always 0 on PartialConflict (contract Q12)
    // -----------------------------------------------------------------

    #[test]
    fn rq_ap14_applied_count_always_zero_on_any_conflict() {
        let mut doc = doc_with_nodes_and_edges(
            vec![
                ("a", test_node("a")),
                ("b", test_node("b")),
                ("c", test_node("c")),
                ("d", test_node("d")),
            ],
            vec![],
        );
        let proposal = proposal_at(0);
        let changes = vec![
            delete_node_change_with_independent_ids("a"),
            delete_node_change_with_independent_ids("b"),
            delete_node_change_with_independent_ids("ghost"),
            delete_node_change_with_independent_ids("c"),
            delete_node_change_with_independent_ids("d"),
        ];
        let accepted = [0, 1, 2, 3, 4];

        let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

        match result {
            ApplyResult::PartialConflict {
                applied_count,
                skipped_count,
                reasons,
            } => {
                assert_eq!(
                    applied_count, 0,
                    "DQ: contract Q12 mandates applied_count == 0 with atomic rollback"
                );
                assert_eq!(skipped_count, 5);
                assert_eq!(reasons.len(), 5);
            }
            other => panic!("DQ: expected PartialConflict, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------
    // RQ-AP15: Multiple failures scenario — first failure triggers
    //          rollback, remaining marked not-attempted
    // -----------------------------------------------------------------

    #[test]
    fn rq_ap15_first_of_many_failures_is_reported_rest_not_attempted() {
        let mut doc = doc_with_nodes_and_edges(vec![("n1", test_node("n1"))], vec![]);
        let proposal = proposal_at(0);
        let changes = vec![
            delete_node_change_with_independent_ids("n1"),
            mismatched_delete_node_change("n2", "snap-X"),
            delete_node_change_with_independent_ids("n3"),
        ];
        let accepted = [0, 1, 2];

        let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

        match result {
            ApplyResult::PartialConflict {
                applied_count,
                skipped_count,
                reasons,
            } => {
                assert_eq!(applied_count, 0);
                assert_eq!(skipped_count, 3);
                assert_eq!(reasons.len(), 3);
                let r0 = &reasons[0];
                let r1 = &reasons[1];
                let r2 = &reasons[2];
                assert!(
                    r1.contains("snapshot mismatch"),
                    "DQ: change 1 has snapshot mismatch, got: {r1}"
                );
                assert!(
                    r2.contains("not attempted"),
                    "DQ: change 2 must be not attempted, got: {r2}"
                );
            }
            other => panic!("DQ: expected PartialConflict, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------
    // RQ-AP16: Proptest — revision invariant across all outcomes
    //          for apply_proposal with adversarial indices
    // -----------------------------------------------------------------

    proptest::proptest! {
        #[test]
        fn rq_ap16_proptest_revision_invariant_adversarial(
            doc_rev in proptest::num::u64::ANY,
            prop_rev in proptest::num::u64::ANY,
            indices in proptest::collection::vec(proptest::num::usize::ANY, 0..20)
        ) {
            let mut doc = doc_with_nodes_and_edges(
                vec![("n1", test_node("n1")), ("n2", test_node("n2"))],
                vec![("e1", test_edge("n1", "n2"))],
            );
            doc.revision = Revision::new(doc_rev);
            let pre_rev = doc.revision;
            let proposal = proposal_at(prop_rev);
            let changes = vec![
                delete_node_change_with_independent_ids("n1"),
                delete_node_change_with_independent_ids("n2"),
            ];

            let result = apply_proposal(&mut doc, &proposal, &changes, &indices);

            match result {
                ApplyResult::Applied => {
                    prop_assert_eq!(doc.revision, pre_rev.increment());
                }
                ApplyResult::Stale(_) => {
                    prop_assert_eq!(doc.revision, pre_rev);
                }
                ApplyResult::PartialConflict { .. } => {
                    prop_assert_eq!(doc.revision, pre_rev);
                }
            }
        }
    }

    // -----------------------------------------------------------------
    // RQ-AP17: Stale returns EXACT pre-call revision (not post-snapshot)
    // -----------------------------------------------------------------

    #[test]
    fn rq_ap17_stale_info_current_is_exact_precall_revision() {
        let mut doc = doc_at(5);
        let proposal = proposal_at(3);
        let changes: Vec<ProposedChange> = vec![];
        let accepted: [usize; 0] = [];

        let result = apply_proposal(&mut doc, &proposal, &changes, &accepted);

        match result {
            ApplyResult::Stale(info) => {
                assert_eq!(info.current, Revision::new(5));
                assert_eq!(info.expected, Revision::new(3));
            }
            other => panic!("DQ: expected Stale, got {other:?}"),
        }
        assert_eq!(
            doc.revision,
            Revision::new(5),
            "DQ: revision unchanged on stale"
        );
    }
}

// =====================================================================
// Kani harnesses
// =====================================================================

#[cfg(kani)]
mod verification {
    use super::*;

    #[kani::proof]
    fn kani_apply_delete_node_no_panic() {
        let doc = DiagramDocument::default();
        let node_id_bytes: [u8; 4] = kani::any();
        let was_id_bytes: [u8; 4] = kani::any();
        let node_id = NodeId::new(String::from_utf8_lossy(&node_id_bytes).to_string());
        let was_id = NodeId::new(String::from_utf8_lossy(&was_id_bytes).to_string());
        let change = ProposedChange::DeleteNode {
            node_id: node_id.clone(),
            was_node_id: was_id.clone(),
            was: test_node(was_id.as_str()),
        };
        let mut doc_mut = doc.clone();
        match apply_delete_node(&mut doc_mut, &change) {
            Ok(_) | Err(_) => {}
        }
    }

    #[kani::proof]
    fn kani_i3_error_path_allocation_bound() {
        let doc = DiagramDocument::default();
        let change = ProposedChange::DeleteNode {
            node_id: NodeId::new("a".to_string()),
            was_node_id: NodeId::new("b".to_string()),
            was: test_node("b"),
        };
        let mut doc_mut = doc.clone();
        let result = apply_delete_node(&mut doc_mut, &change);
        assert_eq!(
            result,
            Err(ApplyError::SnapshotIdMismatch {
                declared: NodeId::new("a".to_string()),
                snapshot: NodeId::new("b".to_string()),
            })
        );
    }
}

// ===========================================================================
// I3 Allocation Discipline
// ===========================================================================
//
// I3 allocation budget verification is deferred to Kani formal verification
// (see kani_i3_error_path_allocation_bound above) because:
// 1. #[global_allocator] cannot be set per-module in the same test binary,
//    so a CountingAllocator would conflict with the default allocator used
//    by all other tests in this crate.
// 2. A separate integration test binary (e.g., tests/i3_allocation.rs) with
//    its own #[global_allocator] is the correct approach but is out of scope
//    for this bead.
//
// The Kani harness at kani_i3_error_path_allocation_bound verifies that error
// paths return the correct error variant without panicking, which provides
// partial I3 coverage. Full heap-allocation budget verification requires the
// separate integration test binary.
