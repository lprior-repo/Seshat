//! Server functions for AI document CRUD operations.
//!
//! These functions are only available on non-WASM targets (server/desktop).
//! They provide a JSON-over-RPC interface for AI document operations.

#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

#[cfg(not(target_arch = "wasm32"))]
use crate::store_async::ai_documents::{
    delete_ai_document as store_delete, fetch_ai_document, fetch_ai_documents_by_key,
    insert_ai_document,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::store_async::error::AsyncStoreError;
#[cfg(not(target_arch = "wasm32"))]
use crate::store_bridge::{BridgeError, StoreBridge};
#[cfg(not(target_arch = "wasm32"))]
use diagram_models::schema_ai_documents::{AiDocument, AiDocumentError, LocationType};

/// JSON representation of an AI document.
#[cfg(not(target_arch = "wasm32"))]
#[derive(serde::Serialize)]
struct DocumentJson {
    id: String,
    key: String,
    json_payload: String,
    location_type: String,
    location_data: String,
    created_at: i64,
}

/// Parameters for storing an AI document.
///
/// Bundled into a struct to stay within the 5-parameter limit.
#[cfg(not(target_arch = "wasm32"))]
pub struct StoreAiDocumentParams {
    pub id: String,
    pub key: String,
    pub json_payload: String,
    pub location_type: String,
    pub location_data: String,
    pub created_at: i64,
}

/// Server-side error type for AI document operations.
///
/// This wraps a JSON error string that can be inspected by tests.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct ServerError(pub String);

#[cfg(not(target_arch = "wasm32"))]
impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl std::error::Error for ServerError {}

#[cfg(not(target_arch = "wasm32"))]
impl std::ops::Deref for ServerError {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

// -------------------------------------------------------------------------
// Server function stubs - these will fail until implemented
// -------------------------------------------------------------------------

/// Store an AI document using bridge context.
///
/// Given valid document fields, persists the document and returns `{"id": "<id>"}`.
/// Given invalid fields, returns `{"error": "<error_type>"}`.
#[cfg(not(target_arch = "wasm32"))]
pub fn store_ai_document(
    bridge: &StoreBridge,
    params: StoreAiDocumentParams,
) -> Result<String, ServerError> {
    // Parse location_type string into LocationType enum
    let location_type = LocationType::from_str(&params.location_type)
        .map_err(|_| ServerError(r#"{"error": "InvalidLocationType"}"#.to_string()))?;

    // Create AiDocument via AiDocument::new()
    let doc = AiDocument::new(
        params.id,
        params.key,
        params.json_payload,
        location_type,
        params.location_data,
        params.created_at,
    )
    .map_err(|e| ServerError(ai_document_error_to_json(e)))?;

    // Capture id before moving doc into closure
    let doc_id = doc.id().to_string();

    // Call bridge to persist - use run_async to execute the async insert
    bridge
        .run_async(|pool| async move { insert_ai_document(&pool, &doc).await })
        .map_err(|e| ServerError(bridge_error_to_json(&e)))?;

    // Return success JSON
    Ok(format!(r#"{{"id": "{doc_id}"}}"#))
}

/// Get an AI document by ID using bridge context.
///
/// Returns `{"document": {...}}` if found, `{"document": null}` if not found.
#[cfg(not(target_arch = "wasm32"))]
pub fn get_ai_document(bridge: &StoreBridge, id: String) -> Result<String, ServerError> {
    match bridge.run_async(|pool| async move { fetch_ai_document(&pool, &id).await }) {
        Ok(Some(doc)) => {
            let json_doc = document_to_json(&doc);
            let json_str = serde_json::to_string_pretty(&json_doc)
                .map_err(|_| ServerError(r#"{"error": "Serialization failed"}"#.to_string()))?;
            Ok(format!(r#"{{"document": {json_str}}}"#))
        }
        Ok(None) => Ok(r#"{"document": null}"#.to_string()),
        Err(e) => Err(ServerError(bridge_error_to_json(&e))),
    }
}

/// List AI documents by key using bridge context.
///
/// Returns `{"documents": [...]}` with all matching documents.
#[cfg(not(target_arch = "wasm32"))]
pub fn list_ai_documents(bridge: &StoreBridge, key: String) -> Result<String, ServerError> {
    match bridge.run_async(|pool| async move { fetch_ai_documents_by_key(&pool, &key).await }) {
        Ok(docs) => {
            let json_docs: Vec<DocumentJson> = docs.iter().map(document_to_json).collect();
            let json_str = serde_json::to_string_pretty(&json_docs)
                .map_err(|_| ServerError(r#"{"error": "Serialization failed"}"#.to_string()))?;
            Ok(format!(r#"{{"documents": {json_str}}}"#))
        }
        Err(e) => Err(ServerError(bridge_error_to_json(&e))),
    }
}

/// Delete an AI document by ID using bridge context.
///
/// Returns `{"deleted": true, "count": N}` on success, `{"deleted": false, "error": "..."}` on failure.
#[cfg(not(target_arch = "wasm32"))]
pub fn delete_ai_document(bridge: &StoreBridge, id: String) -> Result<String, ServerError> {
    match bridge.run_async(|pool| async move { store_delete(&pool, &id).await }) {
        Ok(count) => Ok(format!(r#"{{"deleted": true, "count": {count}}}"#)),
        Err(e) => Err(ServerError(bridge_error_to_json(&e))),
    }
}

// -------------------------------------------------------------------------
// Helper functions for error serialization
// -------------------------------------------------------------------------

/// Converts an `AiDocumentError` to a JSON error string.
#[cfg(not(target_arch = "wasm32"))]
fn ai_document_error_to_json(e: AiDocumentError) -> String {
    match e {
        AiDocumentError::EmptyId => r#"{"error": "EmptyId"}"#.to_string(),
        AiDocumentError::EmptyKey => r#"{"error": "EmptyKey"}"#.to_string(),
        AiDocumentError::InvalidJson => r#"{"error": "InvalidJson"}"#.to_string(),
        AiDocumentError::InvalidLocationDataFormat => {
            r#"{"error": "InvalidLocationDataFormat"}"#.to_string()
        }
        AiDocumentError::EmptyLocationData => r#"{"error": "EmptyLocationData"}"#.to_string(),
    }
}

/// Converts an `AsyncStoreError` to a JSON error string.
#[cfg(not(target_arch = "wasm32"))]
fn async_store_error_to_json(e: &AsyncStoreError) -> String {
    match e {
        AsyncStoreError::Sqlx(sqlx::Error::Database(db_err)) if matches!(db_err.code(), Some(c) if c == "1555") => {
            r#"{"error": "Duplicate id"}"#.to_string()
        }
        AsyncStoreError::Sqlx(_) => r#"{"error": "Database unavailable"}"#.to_string(),
        AsyncStoreError::Io(_) => r#"{"error": "Database unavailable"}"#.to_string(),
        AsyncStoreError::ValidationFailed(msg) => {
            format!(r#"{{"error": "{}"}}"#, msg)
        }
        _ => r#"{"error": "Database unavailable"}"#.to_string(),
    }
}

/// Converts a `BridgeError` to a JSON error string.
#[cfg(not(target_arch = "wasm32"))]
fn bridge_error_to_json(e: &BridgeError) -> String {
    match e {
        BridgeError::AsyncStore(async_err) => async_store_error_to_json(async_err),
        BridgeError::PoolNotInitialized => r#"{"error": "Database unavailable"}"#.to_string(),
        _ => r#"{"error": "Database unavailable"}"#.to_string(),
    }
}

/// Converts an `AiDocument` to its JSON representation.
#[cfg(not(target_arch = "wasm32"))]
fn document_to_json(doc: &AiDocument) -> DocumentJson {
    DocumentJson {
        id: doc.id().to_string(),
        key: doc.key().to_string(),
        json_payload: doc.json_payload().to_string(),
        location_type: doc.location_type().to_string(),
        location_data: doc.location_data().to_string(),
        created_at: *doc.created_at(),
    }
}
