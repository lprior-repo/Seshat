#![allow(clippy::unwrap_used, clippy::expect_used)]
use crate::history::History;
use crate::ui::toolbar::persistence::common::{
    apply_import_contents, prepare_import_transition, ImportTransitionError,
};
use diagram_models::document::{
    DiagramDocument, LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
};
use im::HashMap;
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
#[test]
fn given_malformed_import_when_preparing_transition_then_returns_parse_error() {
    let current = sample_doc_with_node("n-current", 40.0);
    let result = prepare_import_transition(&current, "{this-is-not-json");
    assert!(matches!(result, Err(ImportTransitionError::Parse(_))));
}

#[cfg(kani)]
#[kani::proof]
#[test]
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
#[test]
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
#[test]
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
#[test]
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
#[test]
fn given_import_error_when_selection_exists_then_selection_is_preserved() {
    let mut doc = sample_doc_with_node("n-current", 40.0);
    doc.editor_state.selected_items = HashSet::new().update(String::from("n-current"));
    let mut history = History::new();

    let selected_before = doc.editor_state.selected_items.clone();
    let result = apply_import_contents(&mut doc, &mut history, "{not-valid-json");

    assert!(matches!(result, Err(ImportTransitionError::Parse(_))));
    assert_eq!(doc.editor_state.selected_items, selected_before);
}
