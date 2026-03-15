//! Tests for the document types module.

use crate::models::document::types::{AuthorId, EdgeId, NodeId, OrderedFloat, Revision, Timestamp};

#[test]
fn node_id_try_new_with_empty_string_then_it_returns_error() {
    let result = NodeId::try_new(String::new());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "NodeId cannot be empty");
}

#[test]
fn edge_id_try_new_with_empty_string_then_it_returns_error() {
    let result = EdgeId::try_new(String::new());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "EdgeId cannot be empty");
}

#[test]
fn node_id_try_new_with_valid_string_then_it_succeeds() {
    let result = NodeId::try_new(String::from("valid-id"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_str(), "valid-id");
}

#[test]
fn edge_id_try_new_with_valid_string_then_it_succeeds() {
    let result = EdgeId::try_new(String::from("valid-id"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_str(), "valid-id");
}

#[test]
fn author_id_try_new_with_empty_string_then_it_returns_error() {
    let result = AuthorId::try_new(String::new());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "AuthorId cannot be empty");
}

#[test]
fn timestamp_try_new_with_negative_value_then_it_returns_error() {
    let result = Timestamp::try_new(-1);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Timestamp cannot be negative");
}

#[test]
fn revision_when_incremented_then_it_increases_exactly_once() {
    let initial = Revision::INITIAL;
    let next = initial.increment();

    assert_eq!(
        serde_json::to_string(&initial).ok(),
        Some(String::from("0"))
    );
    assert_eq!(serde_json::to_string(&next).ok(), Some(String::from("1")));
}

#[test]
fn ordered_float_operations_when_applied_then_arithmetic_is_exact() {
    let a = OrderedFloat(8.0);
    let b = OrderedFloat(2.0);

    assert_eq!((a + b).0, 10.0);
    assert_eq!((a - b).0, 6.0);
    assert_eq!((a - 3.0).0, 5.0);
    assert_eq!((a * 2.5).0, 20.0);
    assert_eq!((a / 2.0).0, 4.0);
    assert_eq!(a.to_string(), "8");
}

#[test]
fn node_and_edge_ids_when_stringified_then_values_are_preserved() {
    let node = NodeId::new(String::from("node-1"));
    let edge = EdgeId::new(String::from("edge-1"));

    assert_eq!(node.as_str(), "node-1");
    assert_eq!(edge.as_str(), "edge-1");
    assert_eq!(node.to_string(), "node-1");
    assert_eq!(edge.to_string(), "edge-1");
}
