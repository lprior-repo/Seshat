#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
//! Tests for `AuthorId` and Timestamp newtypes

use diagram_models::document::{AuthorId, NodeId};

#[test]
fn test_author_id_try_new_valid() {
    let author_id = AuthorId::try_new("author-123".to_string());
    assert!(author_id.is_ok());
    if let Ok(id) = author_id {
        assert_eq!(id.as_str(), "author-123");
    }
}

#[test]
fn test_node_id_try_new_empty_string_rejected() {
    let result = NodeId::try_new(String::new());
    assert!(result.is_err());
}

#[test]
fn test_node_id_try_new_valid_string_succeeds() {
    let result = NodeId::try_new("node-1".to_string());
    assert!(result.is_ok());
    if let Ok(id) = result {
        assert_eq!(id.as_str(), "node-1");
    }
}
