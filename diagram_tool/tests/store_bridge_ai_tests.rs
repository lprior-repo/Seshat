//! Integration tests for StoreBridge AI document sync methods.
//!
//! These tests verify that StoreBridge provides synchronous wrapper methods
//! for AI document CRUD operations, and that server functions can use bridge
//! context instead of direct SqlitePool parameter.
//!
//! Tests are written FIRST (TDD) - they will FAIL until the sync methods
//! and bridge-context server functions are implemented.

#![cfg(not(target_arch = "wasm32"))]

use diagram_models::schema_ai_documents::{AiDocument, LocationType};
use tempfile::TempDir;

use diagram_tool::store_async::error::AsyncStoreError;
use diagram_tool::store_bridge::BridgeError;
use diagram_tool::store_bridge::StoreBridge;

/// Helper to create a valid test AiDocument.
fn make_test_ai_document(id: &str, key: &str) -> AiDocument {
    AiDocument::new(
        id.to_string(),
        key.to_string(),
        r#"{"test": true}"#.to_string(),
        LocationType::Gps,
        "37.7749,-122.4194".to_string(),
        1_700_000_000i64,
    )
    .expect("Failed to create test AiDocument")
}

// ============================================================================
// StoreBridge AI Document Sync Methods - Insert
// ============================================================================

/// Behavior 1: StoreBridge inserts AI document and returns id when input is valid.
///
/// Given: A StoreBridge with initialized pool
/// When: insert_ai_document_sync is called with valid AiDocument
/// Then: Returns Ok("<id>") with the document's id
#[test]
fn store_bridge_inserts_ai_document_returns_id_when_input_is_valid() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bridge = StoreBridge::spawn_async_pool(&db_path).expect("Failed to spawn bridge");
    let doc = make_test_ai_document("valid-insert-test", "test-key");

    let result = bridge.insert_ai_document_sync(&doc);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    assert_eq!(result.unwrap(), "valid-insert-test");

    bridge.shutdown().expect("Failed to shutdown");
}

/// Behavior 6: StoreBridge returns error when inserting AI document with duplicate id.
///
/// Given: A StoreBridge with initialized pool; a document with id "dup-id" already exists
/// When: insert_ai_document_sync is called with AiDocument having id "dup-id"
/// Then: Returns Err(BridgeError::AsyncStore(AsyncStoreError::Sqlx(...))) with SQLite constraint error
#[test]
fn store_bridge_returns_sqlx_error_when_inserting_ai_document_with_duplicate_id() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bridge = StoreBridge::spawn_async_pool(&db_path).expect("Failed to spawn bridge");
    let doc = make_test_ai_document("dup-id", "test-key");

    // First insert should succeed
    bridge
        .insert_ai_document_sync(&doc)
        .expect("First insert should succeed");

    // Second insert with same id should fail
    let result = bridge.insert_ai_document_sync(&doc);
    assert!(
        matches!(
            result,
            Err(BridgeError::AsyncStore(AsyncStoreError::ValidationFailed(
                _
            )))
        ),
        "Expected Sqlx error for duplicate id, got {:?}",
        result
    );

    bridge.shutdown().expect("Failed to shutdown");
}

// ============================================================================
// StoreBridge AI Document Sync Methods - Fetch
// ============================================================================

/// Behavior 7: StoreBridge fetches AI document and returns Some when id exists.
///
/// Given: A StoreBridge with initialized pool; a document with id "fetch-test" exists
/// When: fetch_ai_document_sync("fetch-test") is called
/// Then: Returns Ok(Some(AiDocument)) with id="fetch-test"
#[test]
fn store_bridge_fetches_ai_document_returns_some_when_exists() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bridge = StoreBridge::spawn_async_pool(&db_path).expect("Failed to spawn bridge");
    let doc = make_test_ai_document("fetch-test", "test-key");
    bridge
        .insert_ai_document_sync(&doc)
        .expect("Failed to insert");

    let result = bridge.fetch_ai_document_sync("fetch-test");
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    let opt_doc = result.unwrap();
    assert!(opt_doc.is_some(), "Expected Some, got None");
    assert_eq!(opt_doc.unwrap().id(), "fetch-test");

    bridge.shutdown().expect("Failed to shutdown");
}

/// Behavior 8: StoreBridge fetches AI document and returns None when id does not exist.
///
/// Given: A StoreBridge with initialized pool; no document with id "nonexistent" exists
/// When: fetch_ai_document_sync("nonexistent") is called
/// Then: Returns Ok(None)
#[test]
fn store_bridge_fetches_ai_document_returns_none_when_not_found() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bridge = StoreBridge::spawn_async_pool(&db_path).expect("Failed to spawn bridge");

    let result = bridge.fetch_ai_document_sync("nonexistent");
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    assert_eq!(result.unwrap(), None);

    bridge.shutdown().expect("Failed to shutdown");
}

/// Behavior 9: StoreBridge returns error when fetching AI document and pool is not initialized.
///
/// Given: A StoreBridge with closed pool
/// When: fetch_ai_document_sync("any-id") is called
/// Then: Returns Err(BridgeError::PoolNotInitialized) or Err(BridgeError::AsyncStore(...))
#[test]
fn store_bridge_returns_error_when_fetching_ai_document_and_pool_is_not_initialized() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bridge = StoreBridge::spawn_async_pool(&db_path).expect("Failed to spawn bridge");
    bridge.shutdown().expect("Failed to shutdown");

    let result = bridge.fetch_ai_document_sync("any-id");
    assert!(
        matches!(
            result,
            Err(diagram_tool::store_bridge::BridgeError::PoolNotInitialized)
        ),
        "Expected PoolNotInitialized error, got {:?}",
        result
    );
}

// ============================================================================
// StoreBridge AI Document Sync Methods - List
// ============================================================================

/// Behavior 10: StoreBridge lists AI documents and returns Vec when key matches.
///
/// Given: A StoreBridge with initialized pool; documents with key "shared-key" exist (doc-1, doc-2)
/// When: fetch_ai_documents_by_key_sync("shared-key") is called
/// Then: Returns Ok(Vec<AiDocument>) containing 2 documents
#[test]
fn store_bridge_lists_ai_documents_returns_vec_when_key_matches() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bridge = StoreBridge::spawn_async_pool(&db_path).expect("Failed to spawn bridge");

    let doc1 = make_test_ai_document("list-doc-1", "shared-key");
    let doc2 = make_test_ai_document("list-doc-2", "shared-key");
    bridge
        .insert_ai_document_sync(&doc1)
        .expect("Failed to insert doc1");
    bridge
        .insert_ai_document_sync(&doc2)
        .expect("Failed to insert doc2");

    let result = bridge.fetch_ai_documents_by_key_sync("shared-key");
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    let docs = result.unwrap();
    assert_eq!(docs.len(), 2, "Expected 2 documents, got {}", docs.len());

    bridge.shutdown().expect("Failed to shutdown");
}

/// Behavior 11: StoreBridge lists AI documents and returns empty Vec when no matches.
///
/// Given: A StoreBridge with initialized pool; no documents with key "nonexistent-key" exist
/// When: fetch_ai_documents_by_key_sync("nonexistent-key") is called
/// Then: Returns Ok(Vec::new())
#[test]
fn store_bridge_lists_ai_documents_returns_empty_vec_when_no_matches() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bridge = StoreBridge::spawn_async_pool(&db_path).expect("Failed to spawn bridge");

    let result = bridge.fetch_ai_documents_by_key_sync("nonexistent-key");
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    assert!(result.unwrap().is_empty(), "Expected empty Vec");

    bridge.shutdown().expect("Failed to shutdown");
}

/// Behavior 12: StoreBridge returns error when listing AI documents and pool is not initialized.
///
/// Given: A StoreBridge with closed pool
/// When: fetch_ai_documents_by_key_sync("any-key") is called
/// Then: Returns Err(BridgeError::PoolNotInitialized) or Err(BridgeError::AsyncStore(...))
#[test]
fn store_bridge_returns_error_when_listing_ai_documents_and_pool_is_not_initialized() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bridge = StoreBridge::spawn_async_pool(&db_path).expect("Failed to spawn bridge");
    bridge.shutdown().expect("Failed to shutdown");

    let result = bridge.fetch_ai_documents_by_key_sync("any-key");
    assert!(
        matches!(
            result,
            Err(diagram_tool::store_bridge::BridgeError::PoolNotInitialized)
        ),
        "Expected PoolNotInitialized error, got {:?}",
        result
    );
}

// ============================================================================
// StoreBridge AI Document Sync Methods - Update
// ============================================================================

/// Behavior 13: StoreBridge updates AI document and returns Ok when document exists.
///
/// Given: A StoreBridge with initialized pool; a document with id "update-test" exists
/// When: update_ai_document_sync(updated_doc) is called with a document having id "update-test"
/// Then: Returns Ok(())
#[test]
fn store_bridge_updates_ai_document_returns_ok_when_document_exists() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bridge = StoreBridge::spawn_async_pool(&db_path).expect("Failed to spawn bridge");
    let doc = make_test_ai_document("update-test", "old-key");
    bridge
        .insert_ai_document_sync(&doc)
        .expect("Failed to insert");

    let updated_doc = AiDocument::new(
        "update-test".to_string(),
        "new-key".to_string(),
        r#"{"updated": true}"#.to_string(),
        LocationType::Gps,
        "40.7128,-74.0060".to_string(),
        1_700_000_001i64,
    )
    .expect("Failed to create updated doc");

    let result = bridge.update_ai_document_sync(&updated_doc);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);

    bridge.shutdown().expect("Failed to shutdown");
}

/// Behavior 14: StoreBridge updates AI document and returns error when document not found.
///
/// Given: A StoreBridge with initialized pool; no document with id "nonexistent-update" exists
/// When: update_ai_document_sync(doc_with_nonexistent_id) is called
/// Then: Returns Err(BridgeError::AsyncStore(AsyncStoreError::ValidationFailed(
///     "Document not found"
/// )))
#[test]
fn store_bridge_updates_ai_document_returns_error_when_document_not_found() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bridge = StoreBridge::spawn_async_pool(&db_path).expect("Failed to spawn bridge");
    let doc = make_test_ai_document("nonexistent-update", "test-key");

    let result = bridge.update_ai_document_sync(&doc);
    assert!(
        matches!(result, Err(diagram_tool::store_bridge::BridgeError::AsyncStore(
            diagram_tool::store_async::error::AsyncStoreError::ValidationFailed(ref msg)
        )) if msg.contains("Document not found")),
        "Expected ValidationFailed with 'Document not found', got {:?}",
        result
    );

    bridge.shutdown().expect("Failed to shutdown");
}

/// Behavior 15: StoreBridge returns error when updating AI document and pool is not initialized.
///
/// Given: A StoreBridge with closed pool
/// When: update_ai_document_sync(any_doc) is called
/// Then: Returns Err(BridgeError::PoolNotInitialized) or Err(BridgeError::AsyncStore(...))
#[test]
fn store_bridge_returns_error_when_updating_ai_document_and_pool_is_not_initialized() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bridge = StoreBridge::spawn_async_pool(&db_path).expect("Failed to spawn bridge");
    bridge.shutdown().expect("Failed to shutdown");

    let doc = make_test_ai_document("any-id", "any-key");
    let result = bridge.update_ai_document_sync(&doc);
    assert!(
        matches!(
            result,
            Err(diagram_tool::store_bridge::BridgeError::PoolNotInitialized)
        ),
        "Expected PoolNotInitialized error, got {:?}",
        result
    );
}

// ============================================================================
// StoreBridge AI Document Sync Methods - Delete
// ============================================================================

/// Behavior 16: StoreBridge deletes AI document and returns count=1 when document existed.
///
/// Given: A StoreBridge with initialized pool; a document with id "delete-test" exists
/// When: delete_ai_document_sync("delete-test") is called
/// Then: Returns Ok(1)
#[test]
fn store_bridge_deletes_ai_document_returns_count_one_when_document_existed() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bridge = StoreBridge::spawn_async_pool(&db_path).expect("Failed to spawn bridge");
    let doc = make_test_ai_document("delete-test", "test-key");
    bridge
        .insert_ai_document_sync(&doc)
        .expect("Failed to insert");

    let result = bridge.delete_ai_document_sync("delete-test");
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    assert_eq!(result.unwrap(), 1, "Expected deleted count of 1");

    bridge.shutdown().expect("Failed to shutdown");
}

/// Behavior 17: StoreBridge deletes AI document and returns count=0 when document did not exist.
///
/// Given: A StoreBridge with initialized pool; no document with id "nonexistent-delete" exists
/// When: delete_ai_document_sync("nonexistent-delete") is called
/// Then: Returns Ok(0)
#[test]
fn store_bridge_deletes_ai_document_returns_count_zero_when_document_did_not_exist() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bridge = StoreBridge::spawn_async_pool(&db_path).expect("Failed to spawn bridge");

    let result = bridge.delete_ai_document_sync("nonexistent-delete");
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    assert_eq!(result.unwrap(), 0, "Expected deleted count of 0");

    bridge.shutdown().expect("Failed to shutdown");
}

/// Behavior 18: StoreBridge returns error when deleting AI document and pool is not initialized.
///
/// Given: A StoreBridge with closed pool
/// When: delete_ai_document_sync("any-id") is called
/// Then: Returns Err(BridgeError::PoolNotInitialized) or Err(BridgeError::AsyncStore(...))
#[test]
fn store_bridge_returns_error_when_deleting_ai_document_and_pool_is_not_initialized() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bridge = StoreBridge::spawn_async_pool(&db_path).expect("Failed to spawn bridge");
    bridge.shutdown().expect("Failed to shutdown");

    let result = bridge.delete_ai_document_sync("any-id");
    assert!(
        matches!(
            result,
            Err(diagram_tool::store_bridge::BridgeError::PoolNotInitialized)
        ),
        "Expected PoolNotInitialized error, got {:?}",
        result
    );
}

// ============================================================================
// Server Functions - Bridge Context Tests
// ============================================================================

/// Behavior 19: Server function store_ai_document uses bridge context not pool parameter.
///
/// Given: A StoreBridge with initialized pool; server function configured to use bridge context
/// When: store_ai_document(params) is called with valid params (no pool parameter)
/// Then: Document is stored via bridge context; returns Ok(json_id_string)
///
/// This test uses the bridge-based server function variant that takes &StoreBridge
/// instead of SqlitePool directly.
#[test]
fn server_store_ai_document_uses_bridge_context_not_pool_parameter() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bridge = StoreBridge::spawn_async_pool(&db_path).expect("Failed to spawn bridge");

    let params = diagram_tool::server::ai_documents::StoreAiDocumentParams {
        id: "server-insert-test".to_string(),
        key: "test-key".to_string(),
        json_payload: r#"{"server": true}"#.to_string(),
        location_type: "GPS".to_string(),
        location_data: "37.7749,-122.4194".to_string(),
        created_at: 1_700_000_000i64,
    };

    // This should call the bridge-based variant, not the pool-based one
    let result = diagram_tool::server::ai_documents::store_ai_document(&bridge, params);

    // Assert the result is success with JSON containing the id
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    let json_str = result.unwrap();
    assert!(
        json_str.contains("\"id\": \"server-insert-test\""),
        "Expected JSON with id, got: {}",
        json_str
    );

    bridge.shutdown().expect("Failed to shutdown");
}

/// Behavior 20: Server function get_ai_document uses bridge context not pool parameter.
///
/// Given: A StoreBridge with initialized pool; a document with id "server-fetch-test" exists;
///        server function configured to use bridge context
/// When: get_ai_document("server-fetch-test") is called (no pool parameter)
/// Then: Document is fetched via bridge context; returns Ok(json_document_string)
///
/// This test uses the bridge-based server function variant that takes &StoreBridge
/// instead of SqlitePool directly.
#[test]
fn server_get_ai_document_uses_bridge_context_not_pool_parameter() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bridge = StoreBridge::spawn_async_pool(&db_path).expect("Failed to spawn bridge");

    // First insert a document using the sync method
    let doc = make_test_ai_document("server-fetch-test", "test-key");
    bridge
        .insert_ai_document_sync(&doc)
        .expect("Failed to insert document");

    // Now fetch it using the bridge-based server function
    let result = diagram_tool::server::ai_documents::get_ai_document(
        &bridge,
        "server-fetch-test".to_string(),
    );

    // Assert the result is success with JSON containing the document
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    let json_str = result.unwrap();
    assert!(
        json_str.contains("\"document\":"),
        "Expected JSON with document, got: {}",
        json_str
    );
    assert!(
        json_str.contains("\"id\": \"server-fetch-test\""),
        "Expected JSON with correct id, got: {}",
        json_str
    );

    bridge.shutdown().expect("Failed to shutdown");
}

/// Behavior 21: Server function list_ai_documents uses bridge context not pool parameter.
///
/// Given: A StoreBridge with initialized pool; documents with key "server-list-key" exist;
///        server function configured to use bridge context
/// When: list_ai_documents("server-list-key") is called (no pool parameter)
/// Then: Documents are listed via bridge context; returns Ok(json_documents_array)
#[test]
fn server_list_ai_documents_uses_bridge_context_not_pool_parameter() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bridge = StoreBridge::spawn_async_pool(&db_path).expect("Failed to spawn bridge");

    // Insert two documents with the same key
    let doc1 = make_test_ai_document("server-list-1", "server-list-key");
    let doc2 = make_test_ai_document("server-list-2", "server-list-key");
    bridge
        .insert_ai_document_sync(&doc1)
        .expect("Failed to insert doc1");
    bridge
        .insert_ai_document_sync(&doc2)
        .expect("Failed to insert doc2");

    // Now list them using the bridge-based server function
    let result = diagram_tool::server::ai_documents::list_ai_documents(
        &bridge,
        "server-list-key".to_string(),
    );

    // Assert the result is success with JSON containing an array of documents
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    let json_str = result.unwrap();
    assert!(
        json_str.contains("\"documents\":"),
        "Expected JSON with documents array, got: {}",
        json_str
    );

    bridge.shutdown().expect("Failed to shutdown");
}

/// Behavior 22: Server function delete_ai_document uses bridge context not pool parameter.
///
/// Given: A StoreBridge with initialized pool; a document with id "server-delete-test" exists;
///        server function configured to use bridge context
/// When: delete_ai_document("server-delete-test") is called (no pool parameter)
/// Then: Document is deleted via bridge context; returns Ok(json_with_deleted_true_count_1)
#[test]
fn server_delete_ai_document_uses_bridge_context_not_pool_parameter() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let bridge = StoreBridge::spawn_async_pool(&db_path).expect("Failed to spawn bridge");

    // First insert a document using the sync method
    let doc = make_test_ai_document("server-delete-test", "test-key");
    bridge
        .insert_ai_document_sync(&doc)
        .expect("Failed to insert document");

    // Now delete it using the bridge-based server function
    let result = diagram_tool::server::ai_documents::delete_ai_document(
        &bridge,
        "server-delete-test".to_string(),
    );

    // Assert the result is success with JSON containing deleted:true and count:1
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    let json_str = result.unwrap();
    assert!(
        json_str.contains("\"deleted\": true"),
        "Expected JSON with deleted:true, got: {}",
        json_str
    );
    assert!(
        json_str.contains("\"count\": 1"),
        "Expected JSON with count:1, got: {}",
        json_str
    );

    bridge.shutdown().expect("Failed to shutdown");
}
