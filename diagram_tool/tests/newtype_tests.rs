//! Tests for AuthorId and Timestamp newtypes

use diagram_tool::models::document::{AuthorId, NodeId, Timestamp};

#[test]
fn test_author_id_try_new_valid() {
    let author_id = AuthorId::try_new("author-123".to_string());
    assert!(author_id.is_ok());
    assert_eq!(author_id.unwrap().as_str(), "author-123");
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
    assert_eq!(result.unwrap().as_str(), "node-1");
}
