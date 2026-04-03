//! Async CRUD tests for ai_documents table.
//!
//! These tests cover the async CRUD operations for the ai_documents table:
//! - insert_ai_document
//! - fetch_ai_document
//! - fetch_ai_documents_by_key
//! - update_ai_document
//! - delete_ai_document
//!
//! RED PHASE: These tests are written BEFORE implementation.
//! They compile but fail because the underlying functions return `todo!()`.

#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

use diagram_models::schema_ai_documents::{AiDocument, LocationType};
use sqlx::SqlitePool;
use tempfile::TempDir;

use crate::store_async::AsyncStoreError;

/// Creates a test pool with the ai_documents table bootstrapped.
async fn create_test_pool() -> Result<(TempDir, SqlitePool), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_ai_docs.db");

    // Create pool directly using sqlx
    let pool = SqlitePool::connect(&format!("sqlite:{}?mode=rwc", db_path.display())).await?;

    // Create the ai_documents table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS ai_documents (
            id TEXT PRIMARY KEY,
            key TEXT NOT NULL,
            json_payload TEXT NOT NULL,
            location_type TEXT NOT NULL,
            location_data TEXT NOT NULL,
            created_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    Ok((temp_dir, pool))
}

/// Helper to create a valid AiDocument for testing.
fn create_test_document(
    id: &str,
    key: &str,
    json_payload: &str,
) -> Result<AiDocument, diagram_models::schema_ai_documents::AiDocumentError> {
    AiDocument::new(
        id.to_string(),
        key.to_string(),
        json_payload.to_string(),
        LocationType::Gps,
        "37.7749,-122.4194".to_string(),
        1700000000,
    )
}

// =============================================================================
// insert_ai_document tests
// =============================================================================

/// Behavior 1: AiDocument insert succeeds when all fields are valid
/// Returns Ok(id) where id matches the document's id field
#[tokio::test]
async fn insert_ai_document_returns_ok_with_id_when_document_is_valid(
) -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, pool) = create_test_pool().await?;
    let doc = create_test_document(
        "doc-123",
        "user-session-abc",
        r#"{"query": "what is rust?"}"#,
    )
    .expect("create_test_document should succeed for valid input");

    let result = super::ai_documents::insert_ai_document(&pool, &doc).await;

    // Assert the result is Ok and id matches
    let id = result?;
    assert_eq!(id, "doc-123", "inserted document id should match original");

    Ok(())
}

/// Behavior 2: AiDocument insert fails when id already exists
/// Returns Err(AsyncStoreError::ValidationFailed("Duplicate id")) with PRIMARY KEY constraint error
#[tokio::test]
async fn insert_ai_document_returns_validation_error_when_id_exists(
) -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, pool) = create_test_pool().await?;
    let doc = create_test_document("existing-doc", "key-1", "{}")
        .expect("create_test_document should succeed for valid input");

    // First insert should succeed
    super::ai_documents::insert_ai_document(&pool, &doc).await?;

    // Second insert with same id should fail
    let result = super::ai_documents::insert_ai_document(&pool, &doc).await;

    assert!(
        result.is_err(),
        "inserting duplicate id should return error"
    );
    let err = result.unwrap_err();
    match err {
        AsyncStoreError::ValidationFailed(msg) => {
            assert!(
                msg.contains("Duplicate id"),
                "Expected 'Duplicate id' message, got: {}",
                msg
            );
        }
        other => return Err(format!("Expected ValidationFailed error, got: {:?}", other).into()),
    }

    Ok(())
}

/// Behavior 3: AiDocument insert fails when database is unavailable
/// Returns Err(AsyncStoreError::Sqlx)
#[tokio::test]
async fn insert_ai_document_returns_sqlx_error_when_pool_is_invalid(
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a pool and then close it, but keep the reference valid
    let (temp_dir, pool) = create_test_pool().await?;
    let pool_ref = &pool;
    // Close the pool to make it invalid
    pool.close().await;
    // temp_dir will be dropped when function exits

    let doc = create_test_document("doc-123", "key-1", "{}")
        .expect("create_test_document should succeed for valid input");
    let result = super::ai_documents::insert_ai_document(pool_ref, &doc).await;

    assert!(
        result.is_err(),
        "insert with closed pool should return error"
    );
    match result.unwrap_err() {
        AsyncStoreError::Sqlx(_) => {}
        other => return Err(format!("Expected Sqlx error, got: {:?}", other).into()),
    }

    // Prevent temp_dir from dropping pool prematurely
    let _ = temp_dir;

    Ok(())
}

/// Behavior 4: AiDocument insert succeeds with empty json_payload
/// Returns Ok(id) — empty payload is valid per schema
#[tokio::test]
async fn insert_ai_document_succeeds_with_empty_json_payload(
) -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, pool) = create_test_pool().await?;
    let doc = create_test_document("empty-payload-doc", "key-1", "")
        .expect("create_test_document should succeed for valid input");

    let result = super::ai_documents::insert_ai_document(&pool, &doc).await;

    let id = result?;
    assert_eq!(id, "empty-payload-doc");

    Ok(())
}

// =============================================================================
// fetch_ai_document tests
// =============================================================================

/// Behavior 5: fetch_ai_document returns Some when document exists
/// Returns Ok(Some(doc)) where doc.id() == "fetch-test-doc"
/// All other fields match the inserted document
#[tokio::test]
async fn fetch_ai_document_returns_some_when_document_exists(
) -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, pool) = create_test_pool().await?;
    let doc = create_test_document("fetch-test-doc", "shared-key", r#"{"result": "test"}"#)
        .expect("create_test_document should succeed for valid input");

    // Insert the document
    super::ai_documents::insert_ai_document(&pool, &doc).await?;

    // Fetch it back
    let result = super::ai_documents::fetch_ai_document(&pool, "fetch-test-doc").await?;

    assert!(
        result.is_some(),
        "fetch should return Some for existing document"
    );
    let fetched = result.unwrap();

    // Assert all fields match
    assert_eq!(fetched.id(), "fetch-test-doc");
    assert_eq!(fetched.key(), "shared-key");
    assert_eq!(fetched.json_payload(), r#"{"result": "test"}"#);
    assert_eq!(fetched.location_type(), &LocationType::Gps);
    assert_eq!(fetched.location_data(), "37.7749,-122.4194");
    assert_eq!(fetched.created_at(), &1700000000);

    Ok(())
}

/// Behavior 6: fetch_ai_document returns None when document does not exist
/// Returns Ok(None)
#[tokio::test]
async fn fetch_ai_document_returns_none_when_document_not_found(
) -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, pool) = create_test_pool().await?;

    let result = super::ai_documents::fetch_ai_document(&pool, "nonexistent-id").await?;

    assert!(
        result.is_none(),
        "fetch should return None for non-existent document"
    );

    Ok(())
}

/// Behavior 7: fetch_ai_document fails when database is unavailable
/// Returns Err(AsyncStoreError::Sqlx)
#[tokio::test]
async fn fetch_ai_document_returns_sqlx_error_when_pool_is_invalid(
) -> Result<(), Box<dyn std::error::Error>> {
    let (temp_dir, pool) = create_test_pool().await?;
    let pool_ref = &pool;
    pool.close().await;

    let result = super::ai_documents::fetch_ai_document(pool_ref, "any-id").await;

    assert!(
        result.is_err(),
        "fetch with closed pool should return error"
    );
    match result.unwrap_err() {
        AsyncStoreError::Sqlx(_) => {}
        other => return Err(format!("Expected Sqlx error, got: {:?}", other).into()),
    }

    let _ = temp_dir;

    Ok(())
}

// =============================================================================
// fetch_ai_documents_by_key tests
// =============================================================================

/// Behavior 8: fetch_ai_documents_by_key returns matching documents
/// Returns Ok(vec) with exactly 2 documents
/// All returned documents have key() == "shared-key"
#[tokio::test]
async fn fetch_ai_documents_by_key_returns_all_matching_documents(
) -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, pool) = create_test_pool().await?;

    // Insert three documents: two with shared-key, one with other-key
    let doc1 = create_test_document("doc-1", "shared-key", r#"{"n": 1}"#)
        .expect("create_test_document should succeed for valid input");
    let doc2 = create_test_document("doc-2", "shared-key", r#"{"n": 2}"#)
        .expect("create_test_document should succeed for valid input");
    let doc3 = create_test_document("doc-3", "other-key", r#"{"n": 3}"#)
        .expect("create_test_document should succeed for valid input");

    super::ai_documents::insert_ai_document(&pool, &doc1).await?;
    super::ai_documents::insert_ai_document(&pool, &doc2).await?;
    super::ai_documents::insert_ai_document(&pool, &doc3).await?;

    // Fetch by shared-key
    let result = super::ai_documents::fetch_ai_documents_by_key(&pool, "shared-key").await?;

    assert_eq!(
        result.len(),
        2,
        "should return exactly 2 documents with shared-key"
    );
    for doc in &result {
        assert_eq!(
            doc.key(),
            "shared-key",
            "all returned docs should have shared-key"
        );
    }

    Ok(())
}

/// Behavior 9: fetch_ai_documents_by_key returns empty vec when no matches
/// Returns Ok(Vec::new())
#[tokio::test]
async fn fetch_ai_documents_by_key_returns_empty_vec_when_no_matches(
) -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, pool) = create_test_pool().await?;

    // Insert some documents but none with "nonexistent-key"
    let doc = create_test_document("doc-1", "other-key", "{}")
        .expect("create_test_document should succeed for valid input");
    super::ai_documents::insert_ai_document(&pool, &doc).await?;

    let result = super::ai_documents::fetch_ai_documents_by_key(&pool, "nonexistent-key").await?;

    assert!(result.is_empty(), "should return empty vec when no matches");

    Ok(())
}

/// Behavior 10: fetch_ai_documents_by_key fails when database is unavailable
/// Returns Err(AsyncStoreError::Sqlx)
#[tokio::test]
async fn fetch_ai_documents_by_key_returns_sqlx_error_when_pool_is_invalid(
) -> Result<(), Box<dyn std::error::Error>> {
    let (temp_dir, pool) = create_test_pool().await?;
    let pool_ref = &pool;
    pool.close().await;

    let result = super::ai_documents::fetch_ai_documents_by_key(pool_ref, "any-key").await;

    assert!(
        result.is_err(),
        "fetch with closed pool should return error"
    );
    match result.unwrap_err() {
        AsyncStoreError::Sqlx(_) => {}
        other => return Err(format!("Expected Sqlx error, got: {:?}", other).into()),
    }

    let _ = temp_dir;
    Ok(())
}

// =============================================================================
// update_ai_document tests
// =============================================================================

/// Behavior 11: update_ai_document succeeds when document exists
/// Returns Ok(()) and fetch returns document with updated fields
#[tokio::test]
async fn update_ai_document_succeeds_and_persists_changes() -> Result<(), Box<dyn std::error::Error>>
{
    let (_temp_dir, pool) = create_test_pool().await?;

    // Insert original document
    let original_doc = create_test_document("update-test-doc", "key-1", r#"{"original": true}"#)
        .expect("create_test_document should succeed for valid input");
    super::ai_documents::insert_ai_document(&pool, &original_doc).await?;

    // Create updated document with same id but different payload
    let updated_doc = create_test_document("update-test-doc", "key-1", r#"{"updated": true}"#)
        .expect("create_test_document should succeed for valid input");

    // Update it - ? propagates errors, so if we reach here, update succeeded
    super::ai_documents::update_ai_document(&pool, &updated_doc).await?;

    // Verify fetch returns updated fields
    let fetched = super::ai_documents::fetch_ai_document(&pool, "update-test-doc").await?;
    assert!(
        fetched.is_some(),
        "document should still exist after update"
    );
    assert_eq!(fetched.unwrap().json_payload(), r#"{"updated": true}"#);

    Ok(())
}

/// Behavior 12: update_ai_document fails when document does not exist
/// Returns Err(AsyncStoreError::ValidationFailed) with "Document not found"
#[tokio::test]
async fn update_ai_document_returns_error_when_document_not_found(
) -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, pool) = create_test_pool().await?;

    // Try to update non-existent document
    let doc = create_test_document("nonexistent-update-doc", "key-1", "{}")
        .expect("create_test_document should succeed for valid input");
    let result = super::ai_documents::update_ai_document(&pool, &doc).await;

    assert!(
        result.is_err(),
        "update should fail for non-existent document"
    );
    match result.unwrap_err() {
        AsyncStoreError::ValidationFailed(msg) => {
            assert!(
                msg.contains("not found"),
                "Expected 'not found' message, got: {msg}"
            );
        }
        other => return Err(format!("Expected ValidationFailed error, got: {:?}", other).into()),
    }

    Ok(())
}

/// Behavior 13: update_ai_document fails when database is unavailable
/// Returns Err(AsyncStoreError::Sqlx)
#[tokio::test]
async fn update_ai_document_returns_sqlx_error_when_pool_is_invalid(
) -> Result<(), Box<dyn std::error::Error>> {
    let (temp_dir, pool) = create_test_pool().await?;
    let pool_ref = &pool;
    pool.close().await;

    let doc = create_test_document("any-doc", "key-1", "{}")
        .expect("create_test_document should succeed for valid input");
    let result = super::ai_documents::update_ai_document(pool_ref, &doc).await;

    assert!(
        result.is_err(),
        "update with closed pool should return error"
    );
    match result.unwrap_err() {
        AsyncStoreError::Sqlx(_) => {}
        other => return Err(format!("Expected Sqlx error, got: {:?}", other).into()),
    }

    Ok(())
}

// =============================================================================
// delete_ai_document tests
// =============================================================================

/// Behavior 14: delete_ai_document succeeds when document exists
/// Returns Ok(1) and fetch returns Ok(None)
#[tokio::test]
async fn delete_ai_document_returns_ok_with_count_one_and_document_is_gone(
) -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, pool) = create_test_pool().await?;

    // Insert document
    let doc = create_test_document("delete-test-doc", "key-1", "{}")
        .expect("create_test_document should succeed for valid input");
    super::ai_documents::insert_ai_document(&pool, &doc).await?;

    // Delete it
    let result = super::ai_documents::delete_ai_document(&pool, "delete-test-doc").await?;

    assert_eq!(result, 1, "delete should return 1 when document existed");

    // Verify it's gone
    let fetched = super::ai_documents::fetch_ai_document(&pool, "delete-test-doc").await?;
    assert!(fetched.is_none(), "document should be gone after delete");

    Ok(())
}

/// Behavior 15: delete_ai_document succeeds (no-op) when document does not exist
/// Returns Ok(0)
#[tokio::test]
async fn delete_ai_document_returns_ok_with_count_zero_when_not_found(
) -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, pool) = create_test_pool().await?;

    let result = super::ai_documents::delete_ai_document(&pool, "nonexistent-delete-doc").await?;

    assert_eq!(
        result, 0,
        "delete should return 0 when document did not exist"
    );

    Ok(())
}

/// Behavior 16: delete_ai_document fails when database is unavailable
/// Returns Err(AsyncStoreError::Sqlx)
#[tokio::test]
async fn delete_ai_document_returns_sqlx_error_when_pool_is_invalid(
) -> Result<(), Box<dyn std::error::Error>> {
    let (temp_dir, pool) = create_test_pool().await?;
    let pool_ref = &pool;
    pool.close().await;

    let result = super::ai_documents::delete_ai_document(pool_ref, "any-id").await;

    assert!(
        result.is_err(),
        "delete with closed pool should return error"
    );
    match result.unwrap_err() {
        AsyncStoreError::Sqlx(_) => {}
        other => return Err(format!("Expected Sqlx error, got: {:?}", other).into()),
    }

    let _ = temp_dir;
    Ok(())
}

// =============================================================================
// Round-trip invariant test
// =============================================================================

/// Round-trip insert → fetch preserves all fields
/// For any valid AiDocument, insert returns Ok(id) AND fetch returns Ok(Some(doc))
/// with identical field values to original doc
#[tokio::test]
async fn roundtrip_insert_fetch_preserves_all_fields() -> Result<(), Box<dyn std::error::Error>> {
    // Test with various document configurations
    let test_cases = vec![
        (
            "roundtrip-1",
            "key-1",
            r#"{"a":1}"#,
            LocationType::Gps,
            "37.7749,-122.4194",
        ),
        (
            "roundtrip-2",
            "key-2",
            r#"{}"#,
            LocationType::FilePath,
            "/home/user/doc.md",
        ),
        (
            "roundtrip-3",
            "key-3",
            "[]",
            LocationType::Url,
            "https://example.com/path",
        ),
        (
            "roundtrip-4",
            "key-4",
            r#""text""#,
            LocationType::DocumentPosition,
            "42:10",
        ),
    ];

    for (id, key, payload, loc_type, loc_data) in test_cases {
        let (_temp_dir, pool) = create_test_pool().await?;

        let doc = AiDocument::new(
            id.to_string(),
            key.to_string(),
            payload.to_string(),
            loc_type.clone(),
            loc_data.to_string(),
            1700000000,
        )
        .expect("AiDocument::new should succeed for valid inputs");

        // Insert
        let result_id = super::ai_documents::insert_ai_document(&pool, &doc).await?;
        assert_eq!(result_id, id);

        // Fetch
        let fetched = super::ai_documents::fetch_ai_document(&pool, id).await?;
        assert!(fetched.is_some(), "fetch should return Some after insert");

        let fetched = fetched.unwrap();
        assert_eq!(fetched.id(), id);
        assert_eq!(fetched.key(), key);
        assert_eq!(fetched.json_payload(), payload);
        assert_eq!(fetched.location_type(), &loc_type);
        assert_eq!(fetched.location_data(), loc_data);
        assert_eq!(fetched.created_at(), &1700000000);
    }

    Ok(())
}
