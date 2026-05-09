#![allow(clippy::unwrap_used, clippy::expect_used)]
use crate::history::History;
use crate::ui::toolbar::persistence::common::{
    apply_import_contents, prepare_import_transition, ImportTransitionError,
};
use diagram_models::document::{
    DiagramDocument, LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
};
use im::HashMap;
use proptest::prelude::*;
use std::collections::HashSet;

#[allow(clippy::unwrap_used, clippy::expect_used)]
fn sample_doc_with_node(id: &str, x: f64) -> DiagramDocument {
    let mut doc = DiagramDocument::default();
    let _ = doc.document.nodes.insert(
        NodeId::new(id.to_string()),
        Node {
            kind: NodeKind::Text,
            icon: String::new(),
            label: String::from("Text"),
            x: OrderedFloat(x),
            y: OrderedFloat(120.0),
            width: OrderedFloat(100.0),
            height: OrderedFloat(24.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Locked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        },
    );
    doc
}

#[cfg(kani)]
#[kani::proof]
fn given_malformed_import_when_preparing_transition_then_returns_parse_error() {
    let current = sample_doc_with_node("n-current", 40.0);
    let result = prepare_import_transition(&current, "{this-is-not-json");
    assert!(matches!(result, Err(ImportTransitionError::Parse(_))));
}

#[cfg(kani)]
#[kani::proof]
fn given_semantically_invalid_import_when_preparing_transition_then_returns_validation_error() {
    let current = sample_doc_with_node("n-current", 40.0);
    let invalid = r#"{
        "version": 2,
        "revision": 0,
        "document": {
            "nodes": {},
            "edges": {
                "e1": {
                    "source": "missing-a",
                    "target": "missing-b"
                }
            }
        }
    }"#;

    let result = prepare_import_transition(&current, invalid);
    assert!(matches!(result, Err(ImportTransitionError::Validation(_))));
}

#[cfg(kani)]
#[kani::proof]
#[allow(clippy::unwrap_used, clippy::expect_used)]
fn given_valid_import_when_preparing_transition_then_new_doc_and_history_are_atomic() {
    let current = sample_doc_with_node("n-current", 40.0);
    let valid = serde_json::to_string_pretty(&sample_doc_with_node("n-import", 260.0)).unwrap();

    let (next_doc, next_history) = prepare_import_transition(&current, &valid)
        .expect("valid import should produce a transition");
    assert!(next_doc
        .document
        .nodes
        .contains_key(&NodeId::new(String::from("n-import"))));
    assert!(!next_doc
        .document
        .nodes
        .contains_key(&NodeId::new(String::from("n-current"))));

    let undone = next_history.undo(next_doc.clone());
    assert!(
        undone.is_some(),
        "history should include pre-import snapshot"
    );
    let (restored, _) = undone.expect("undo should restore prior state");
    assert!(restored
        .document
        .nodes
        .contains_key(&NodeId::new(String::from("n-current"))));
    assert!(!restored
        .document
        .nodes
        .contains_key(&NodeId::new(String::from("n-import"))));

    let fresh_history = History::new();
    assert!(fresh_history.undo(current).is_none());
}

#[cfg(kani)]
#[kani::proof]
fn given_import_error_when_applying_contents_then_doc_and_history_remain_unchanged() {
    let mut doc = sample_doc_with_node("n-current", 40.0);
    let previous = sample_doc_with_node("n-prev", 12.0);
    let mut history = History::new().push(previous.clone());

    let doc_before = doc.clone();
    let undo_before = history
        .clone()
        .undo(doc.clone())
        .map(|(snapshot, _)| snapshot);

    let result = apply_import_contents(&mut doc, &mut history, "{not-valid-json");
    assert!(matches!(result, Err(ImportTransitionError::Parse(_))));
    assert_eq!(doc, doc_before);

    let undo_after = history.undo(doc.clone()).map(|(snapshot, _)| snapshot);
    assert_eq!(undo_after, undo_before);
    assert_eq!(undo_after, Some(previous));
}

#[cfg(kani)]
#[kani::proof]
fn given_validation_error_when_applying_contents_then_doc_and_history_remain_unchanged() {
    let mut doc = sample_doc_with_node("n-current", 40.0);
    let previous = sample_doc_with_node("n-prev", 12.0);
    let mut history = History::new().push(previous.clone());

    let invalid = r#"{
        "version": 2,
        "revision": 0,
        "document": {
            "nodes": {},
            "edges": {
                "e1": {
                    "source": "missing-a",
                    "target": "missing-b"
                }
            }
        }
    }"#;

    let doc_before = doc.clone();
    let undo_before = history
        .clone()
        .undo(doc.clone())
        .map(|(snapshot, _)| snapshot);

    let result = apply_import_contents(&mut doc, &mut history, invalid);
    assert!(matches!(result, Err(ImportTransitionError::Validation(_))));
    assert_eq!(doc, doc_before);

    let undo_after = history.undo(doc.clone()).map(|(snapshot, _)| snapshot);
    assert_eq!(undo_after, undo_before);
    assert_eq!(undo_after, Some(previous));
}

#[cfg(kani)]
#[kani::proof]
fn given_import_error_when_selection_exists_then_selection_is_preserved() {
    let mut doc = sample_doc_with_node("n-current", 40.0);
    doc.editor_state.selected_items = HashSet::new().update(String::from("n-current"));
    let mut history = History::new();

    let selected_before = doc.editor_state.selected_items.clone();
    let result = apply_import_contents(&mut doc, &mut history, "{not-valid-json");

    assert!(matches!(result, Err(ImportTransitionError::Parse(_))));
    assert_eq!(doc.editor_state.selected_items, selected_before);
}

// =====================================================================
// Proptest invariants
// =====================================================================

proptest! {
    #![proptest_config(proptest::prelude::ProptestConfig::with_cases(256))]

    #[test]
    fn apply_import_contents_atomicity_invariant(
        _seed in 0u64..1000u64,
    ) {
        // Use a default document as the current state
        let current = DiagramDocument::default();
        let invalid_contents = "{this is not valid json at all!!!";

        let mut doc = current;
        let mut history = History::new();

        // Record state before
        let doc_before = doc.clone();

        // Attempt import - should fail
        let result = apply_import_contents(&mut doc, &mut history, invalid_contents);

        // Invariant: On error, doc must be unchanged (atomicity)
        // Note: We cannot check history equality as History doesn't implement PartialEq
        if result.is_err() {
            prop_assert_eq!(
                doc, doc_before,
                "On error, document must be unchanged (atomicity)"
            );
        }
    }

    #[test]
    fn prepare_import_transition_idempotency_invariant(
        _seed in 0u64..1000u64,
    ) {
        // Use a default document as the current state
        let current = DiagramDocument::default();
        // Create a valid import document
        let import_doc = DiagramDocument::default();

        // Serialize the import document to JSON
        let contents = serde_json::to_string(&import_doc).unwrap();

        // Run migration twice
        let result1 = prepare_import_transition(&current, &contents);
        let result2 = prepare_import_transition(&current, &contents);

        // Invariant: Both should succeed or both should fail (deterministic)
        prop_assert_eq!(
            result1.is_ok(),
            result2.is_ok(),
            "prepare_import_transition should be deterministic"
        );

        // Invariant: If successful, documents should be identical (idempotency)
        // Note: We check only the document since History doesn't implement PartialEq
        if let (Ok((doc1, _history1)), Ok((doc2, _history2))) = (result1, result2) {
            prop_assert_eq!(
                doc1, doc2,
                "Running migration twice should produce identical documents (idempotency)"
            );
        }
    }
}
