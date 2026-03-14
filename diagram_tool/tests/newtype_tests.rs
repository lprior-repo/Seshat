//! Tests for AuthorId and Timestamp newtypes

use diagram_tool::models::document::{AuthorId, Timestamp};

#[test]
fn test_author_id_try_new_valid() {
    let author_id = AuthorId::try_new("author-123".to_string());
    assert!(author_id.is_ok());
    assert_eq!(author_id.unwrap().as_str(), "author-123");
}
