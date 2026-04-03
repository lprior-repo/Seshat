//! Async CRUD operations for the `ai_documents` table.
//!
//! This module provides async functions for inserting, fetching, updating, and deleting
//! AI documents in the `SQLite` database.

use sqlx::SqlitePool;

use crate::store_async::error::AsyncStoreError;
use diagram_models::schema_ai_documents::{AiDocument, AiDocumentError, LocationType};

/// SQL query for inserting a new AI document.
const INSERT_AI_DOCUMENT_QUERY: &str =
    "INSERT INTO ai_documents (id, key, json_payload, location_type, location_data, created_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6)";

/// SQL query for fetching a single AI document by id.
const FETCH_AI_DOCUMENT_QUERY: &str =
    "SELECT id, key, json_payload, location_type, location_data, created_at
     FROM ai_documents
     WHERE id = ?1";

/// SQL query for fetching AI documents by key.
const FETCH_AI_DOCUMENTS_BY_KEY_QUERY: &str =
    "SELECT id, key, json_payload, location_type, location_data, created_at
     FROM ai_documents
     WHERE key = ?1";

/// Bundles the raw fields from a database row before parsing into `AiDocument`.
struct AiDocumentRow {
    id: String,
    key: String,
    json_payload: String,
    location_type_str: String,
    location_data: String,
    created_at: i64,
}

/// Parses a database row into an `AiDocument`.
///
/// # Arguments
///
/// * `row` - The bundled row fields
///
/// # Returns
///
/// * `Ok(AiDocument)` - The parsed document
/// * `Err(AsyncStoreError::ValidationFailed(_))` - If parsing fails
fn parse_ai_document_row(row: AiDocumentRow) -> Result<AiDocument, AsyncStoreError> {
    let location_type = LocationType::from_str(&row.location_type_str)
        .map_err(|e| AsyncStoreError::ValidationFailed(format!("Invalid location_type: {e:?}")))?;
    AiDocument::new(
        row.id,
        row.key,
        row.json_payload,
        location_type,
        row.location_data,
        row.created_at,
    )
    .map_err(|e: AiDocumentError| {
        AsyncStoreError::ValidationFailed(format!("Invalid document: {e:?}"))
    })
}

/// Maps a database row tuple to a parsed `AiDocument`.
fn map_row_to_document(
    id: String,
    key: String,
    json_payload: String,
    location_type_str: String,
    location_data: String,
    created_at: i64,
) -> Result<AiDocument, AsyncStoreError> {
    parse_ai_document_row(AiDocumentRow {
        id,
        key,
        json_payload,
        location_type_str,
        location_data,
        created_at,
    })
}

/// Inserts a new AI document into the database.
///
/// # Arguments
///
/// * `pool` - The `SqlitePool` connection pool
/// * `doc` - The AI document to insert
///
/// # Returns
///
/// * `Ok(id)` - The id of the inserted document
/// * `Err(AsyncStoreError::ValidationFailed(_))` - If the insert fails due to duplicate id
/// * `Err(AsyncStoreError::Sqlx(_))` - If the insert fails for other reasons
pub async fn insert_ai_document(
    pool: &SqlitePool,
    doc: &AiDocument,
) -> Result<String, AsyncStoreError> {
    sqlx::query(INSERT_AI_DOCUMENT_QUERY)
        .bind(doc.id())
        .bind(doc.key())
        .bind(doc.json_payload())
        .bind(doc.location_type().to_string())
        .bind(doc.location_data())
        .bind(doc.created_at())
        .execute(pool)
        .await
        .map_err(handle_insert_error)
        .map(|_| doc.id().to_string())
}

fn handle_insert_error(e: sqlx::Error) -> AsyncStoreError {
    if let sqlx::Error::Database(db_err) = &e {
        if let Some(c) = db_err.code() {
            if c == "1555" {
                return AsyncStoreError::ValidationFailed("Duplicate id".to_string());
            }
        }
    }
    AsyncStoreError::Sqlx(e)
}

/// Fetches a single AI document by id.
///
/// # Arguments
///
/// * `pool` - The `SqlitePool` connection pool
/// * `id` - The document id to fetch
///
/// # Returns
///
/// * `Ok(Some(doc))` - If the document exists
/// * `Ok(None)` - If the document does not exist
/// * `Err(AsyncStoreError::Sqlx(_))` - If the query fails
pub async fn fetch_ai_document(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<AiDocument>, AsyncStoreError> {
    let row = sqlx::query_as::<_, (String, String, String, String, String, i64)>(
        FETCH_AI_DOCUMENT_QUERY,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AsyncStoreError::Sqlx)?;

    row.map(|(id, key, json_payload, location_type_str, location_data, created_at)| {
        map_row_to_document(id, key, json_payload, location_type_str, location_data, created_at)
    })
    .transpose()
}

/// Fetches all AI documents with a given key.
///
/// # Arguments
///
/// * `pool` - The `SqlitePool` connection pool
/// * `key` - The document key to search for
///
/// # Returns
///
/// * `Ok(Vec<AiDocument>)` - All documents matching the key (may be empty)
/// * `Err(AsyncStoreError::Sqlx(_))` - If the query fails
pub async fn fetch_ai_documents_by_key(
    pool: &SqlitePool,
    key: &str,
) -> Result<Vec<AiDocument>, AsyncStoreError> {
    let rows = sqlx::query_as::<_, (String, String, String, String, String, i64)>(
        FETCH_AI_DOCUMENTS_BY_KEY_QUERY,
    )
    .bind(key)
    .fetch_all(pool)
    .await
    .map_err(AsyncStoreError::Sqlx)?;

    rows.into_iter()
        .map(|(id, key, json_payload, location_type_str, location_data, created_at)| {
            map_row_to_document(id, key, json_payload, location_type_str, location_data, created_at)
        })
        .collect()
}

/// Updates an existing AI document.
///
/// # Arguments
///
/// * `pool` - The `SqlitePool` connection pool
/// * `doc` - The document with updated fields (id must match existing document)
///
/// # Returns
///
/// * `Ok(())` - If the update succeeds
/// * `Err(AsyncStoreError::Sqlx(_))` - If the update fails (e.g., document not found)
pub async fn update_ai_document(
    pool: &SqlitePool,
    doc: &AiDocument,
) -> Result<(), AsyncStoreError> {
    sqlx::query(
        "UPDATE ai_documents SET key = ?2, json_payload = ?3, location_type = ?4, location_data = ?5, created_at = ?6 WHERE id = ?1",
    )
    .bind(doc.id())
    .bind(doc.key())
    .bind(doc.json_payload())
    .bind(doc.location_type().to_string())
    .bind(doc.location_data())
    .bind(doc.created_at())
    .execute(pool)
    .await
    .map_err(AsyncStoreError::Sqlx)
    .and_then(|r| if r.rows_affected() == 0 {
        Err(AsyncStoreError::ValidationFailed("Document not found".to_string()))
    } else {
        Ok(())
    })
}

/// Deletes an AI document by id.
///
/// # Arguments
///
/// * `pool` - The `SqlitePool` connection pool
/// * `id` - The document id to delete
///
/// # Returns
///
/// * `Ok(1)` - If the document existed and was deleted
/// * `Ok(0)` - If the document did not exist
/// * `Err(AsyncStoreError::Sqlx(_))` - If the delete fails
pub async fn delete_ai_document(pool: &SqlitePool, id: &str) -> Result<u64, AsyncStoreError> {
    let result = sqlx::query("DELETE FROM ai_documents WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(AsyncStoreError::Sqlx)?;

    Ok(result.rows_affected())
}
