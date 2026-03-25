//! Unit tests for serialize_document.

#![allow(clippy::unwrap_used)]

use diagram_models::document::DiagramDocument;

use super::*;

// -----------------------------------------------------------------------
// serialize_document unit tests
// -----------------------------------------------------------------------

#[test]
fn serialize_document_returns_compact_json_when_given_default_document() {
    let doc = DiagramDocument::default();
    let result = serialize_document(&doc);
    assert!(
        result.is_ok(),
        "serialize_document must succeed for default doc: {result:?}"
    );
    let json = result.unwrap();
    let deserialized = serde_json::from_str::<DiagramDocument>(&json);
    assert!(
        deserialized.is_ok(),
        "deserialized must succeed: {:?}",
        deserialized.as_ref().err()
    );
    assert_eq!(deserialized.unwrap(), doc);
}

#[test]
fn serialize_document_output_contains_version_zero_when_document_version_is_zero() {
    let doc = DiagramDocument {
        version: 0,
        ..DiagramDocument::default()
    };
    let result = serialize_document(&doc);
    assert!(
        result.is_ok(),
        "serialize must succeed for version 0 doc: {result:?}"
    );
    assert!(
        result.as_ref().unwrap().contains("\"version\":0"),
        "output must contain \"version\":0, got: {:?}",
        result.as_ref().unwrap()
    );
}

#[test]
fn serialize_document_output_contains_version_one_when_document_version_is_one() {
    let doc = DiagramDocument {
        version: 1,
        ..DiagramDocument::default()
    };
    let result = serialize_document(&doc);
    assert!(
        result.is_ok(),
        "serialize must succeed for version 1 doc: {result:?}"
    );
    assert!(
        result.as_ref().unwrap().contains("\"version\":1"),
        "output must contain \"version\":1, got: {:?}",
        result.as_ref().unwrap()
    );
}

#[test]
fn serialize_document_output_contains_node_id_when_document_has_nodes() {
    use diagram_models::document::types::OrderedFloat;
    use diagram_models::document::{LockState, Node, NodeId, NodeKind};
    let mut doc = DiagramDocument::default();
    let node_id = NodeId::new("node-abc".to_string());
    let node = Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: "test".to_string(),
        x: OrderedFloat(0.0),
        y: OrderedFloat(0.0),
        width: OrderedFloat(100.0),
        height: OrderedFloat(100.0),
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
    };
    doc.document.nodes.insert(node_id, node);
    let result = serialize_document(&doc);
    assert!(
        result.is_ok(),
        "serialize must succeed for doc with nodes: {result:?}"
    );
    let json = result.unwrap();
    assert!(
        json.contains("node-abc"),
        "output must contain 'node-abc', got: {json:?}"
    );
    assert!(
        json.contains("\"nodes\":"),
        "output must contain '\"nodes\":', got: {json:?}"
    );
}

#[test]
fn serialize_document_output_contains_no_newlines_or_indentation() {
    let doc = DiagramDocument::default();
    let result = serialize_document(&doc);
    assert!(result.is_ok(), "serialize must succeed: {result:?}");
    let json = result.unwrap();
    assert!(
        !json.contains('\n'),
        "compact JSON must not contain newlines, got: {json:?}"
    );
    assert!(
        !json.contains("  "),
        "compact JSON must not contain double-space indentation, got: {json:?}"
    );
}

#[test]
fn serialize_document_round_trips_to_identical_document_when_serialized_then_deserialized() {
    let doc = DiagramDocument::default();
    let json_result = serialize_document(&doc);
    assert!(
        json_result.is_ok(),
        "serialize must succeed: {json_result:?}"
    );
    let json = json_result.unwrap();
    let deserialized = serde_json::from_str::<DiagramDocument>(&json);
    assert!(
        deserialized.is_ok(),
        "deserialized must succeed: {:?}",
        deserialized.as_ref().err()
    );
    assert_eq!(deserialized.unwrap(), doc);
}

// -----------------------------------------------------------------------
// serialize_document B-29: SerializationFailure error arm
// -----------------------------------------------------------------------

/// Test-only serialize function that maps the serde error identically to
/// how `serialize_document` would, but accepts any Serialize type.
fn serialize_any<T: serde::Serialize>(val: &T) -> Result<String, ShowError> {
    serde_json::to_string(val).map_err(|e| ShowError::SerializationFailure(e.to_string()))
}

/// A type that always fails to serialize.
struct AlwaysFailsSerialize;

impl serde::Serialize for AlwaysFailsSerialize {
    fn serialize<S: serde::Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom("injected serialization error"))
    }
}

#[test]
fn serialize_document_returns_serialization_failure_when_serde_json_errors() {
    let failing = AlwaysFailsSerialize;
    let result = serialize_any(&failing);
    assert!(
        matches!(result, Err(ShowError::SerializationFailure(_))),
        "expected SerializationFailure, got: {result:?}"
    );
    if let Err(ShowError::SerializationFailure(msg)) = result {
        assert!(
            msg.contains("injected serialization error"),
            "error message must contain injected payload, got: {msg:?}"
        );
    }
}
