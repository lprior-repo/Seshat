use crate::test_utils::{TestCategory, TestHarnessError};
use diagram_models::document::{DiagramDocument, LockState, Node, NodeId, NodeKind, OrderedFloat};

#[cfg(kani)]
#[kani::proof]
fn test_verify_invariants_passes_for_valid_document() {
    let mut doc = DiagramDocument::default();

    let node = Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: "Test".to_string(),
        x: OrderedFloat(100.0),
        y: OrderedFloat(200.0),
        width: OrderedFloat(80.0),
        height: OrderedFloat(40.0),
        font_size: None,
        font_weight: None,
        lock_state: LockState::Unlocked,
        parent: None,
        dag_rank: None,
        tags: im::vector![],
        metadata: im::HashMap::new(),
        z_index: 0,
        style: None,
        collapsed: None,
    };

    doc.document
        .nodes
        .insert(NodeId::new("node-1".to_string()), node);

    let result = crate::test_utils::verify_invariants(&doc);
    assert!(result.is_ok());
}

#[cfg(kani)]
#[kani::proof]
fn test_verify_invariants_fails_for_nan_coordinates() {
    let mut doc = DiagramDocument::default();

    let node = Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: "Bad Node".to_string(),
        x: OrderedFloat(f64::NAN),
        y: OrderedFloat(200.0),
        width: OrderedFloat(80.0),
        height: OrderedFloat(40.0),
        font_size: None,
        font_weight: None,
        lock_state: LockState::Unlocked,
        parent: None,
        dag_rank: None,
        tags: im::vector![],
        metadata: im::HashMap::new(),
        z_index: 0,
        style: None,
        collapsed: None,
    };

    doc.document
        .nodes
        .insert(NodeId::new("bad-node".to_string()), node);

    let result = crate::test_utils::verify_invariants(&doc);
    assert!(result.is_err());

    if let Err(TestHarnessError::InvariantViolation { invariant, .. }) = result {
        assert_eq!(invariant, "no_nan_in_coordinates");
    } else {
        panic!("Expected InvariantViolation");
    }
}

#[cfg(kani)]
#[kani::proof]
fn test_verify_invariants_fails_for_negative_dimensions() {
    let mut doc = DiagramDocument::default();

    let node = Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: "Negative Node".to_string(),
        x: OrderedFloat(100.0),
        y: OrderedFloat(200.0),
        width: OrderedFloat(-10.0),
        height: OrderedFloat(40.0),
        font_size: None,
        font_weight: None,
        lock_state: LockState::Unlocked,
        parent: None,
        dag_rank: None,
        tags: im::vector![],
        metadata: im::HashMap::new(),
        z_index: 0,
        style: None,
        collapsed: None,
    };

    doc.document
        .nodes
        .insert(NodeId::new("negative-node".to_string()), node);

    let result = crate::test_utils::verify_invariants(&doc);
    assert!(result.is_err());

    if let Err(TestHarnessError::InvariantViolation { invariant, .. }) = result {
        assert_eq!(invariant, "positive_dimensions");
    } else {
        panic!("Expected InvariantViolation");
    }
}

#[cfg(kani)]
#[kani::proof]
fn test_compute_document_hash_is_stable() {
    let doc = DiagramDocument::default();
    let hash1 = crate::test_utils::compute_document_hash(&doc);
    let hash2 = crate::test_utils::compute_document_hash(&doc);

    assert_eq!(hash1, hash2);
}

#[cfg(kani)]
#[kani::proof]
fn test_test_db_path_is_unique_per_test() {
    let path1 = crate::test_utils::test_db_path("test_a");
    let path2 = crate::test_utils::test_db_path("test_b");

    assert_ne!(path1, path2);
}

#[cfg(kani)]
#[kani::proof]
fn test_run_all_tests_aggregates_categories() {
    let categories = &[TestCategory::Sel, TestCategory::Clp];
    let report = crate::test_utils::run_all_tests(categories).unwrap();

    assert_eq!(report.total_tests, 35); // 25 + 10
    assert_eq!(report.categories.len(), 2);
}
