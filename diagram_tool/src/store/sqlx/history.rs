use crate::store::sqlx::error::*;
use crate::store::sqlx::models::*;
use sqlx::SqlitePool;
use diagram_models::envelope::{encode_event_envelope, parse_event_envelope, EventEnvelope};

/// Fetches the latest revision number from the store
///
/// # Errors
///
/// Returns a `StoreError` if the query fails.
pub async fn fetch_latest_revision(pool: &SqlitePool) -> Result<i64, StoreError> {
    sqlx::query_scalar("SELECT COALESCE(MAX(revision), 0) FROM events")
        .fetch_one(pool)
        .await
        .map_err(StoreError::Sqlx)
}

/// Gets the current revision
///
/// # Errors
///
/// Returns a `StoreError` if the query fails.
pub async fn current_revision(pool: &SqlitePool) -> Result<i64, StoreError> {
    fetch_latest_revision(pool).await
}

/// Gets the next revision number
///
/// # Errors
///
/// Returns a `StoreError` if the query fails.
pub async fn next_revision(pool: &SqlitePool) -> Result<i64, StoreError> {
    let current = current_revision(pool).await?;
    Ok(current + 1)
}

/// Appends a single event to the store
///
/// # Errors
///
/// Returns a `StoreError` if the append fails.
pub async fn append_event(
    pool: &SqlitePool,
    envelope: EventEnvelope,
    expected_revision: Option<i64>,
) -> Result<AppendResult, StoreError> {
    let mut tx = pool.begin().await.map_err(StoreError::Sqlx)?;

    let current_revision: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(revision), 0) FROM events")
        .fetch_one(&mut *tx)
        .await
        .map_err(StoreError::Sqlx)?;

    if let Some(expected) = expected_revision {
        if current_revision != expected {
            return Err(StoreError::RevisionMismatch {
                expected,
                found: current_revision,
            });
        }
    }

    let payload =
        encode_event_envelope(&envelope).map_err(|e| StoreError::Serialization(e.to_string()))?;

    let new_revision = current_revision + 1;
    let insert_result = sqlx::query_scalar::<_, i64>(
        "INSERT INTO events (operation_id, revision, payload, timestamp) 
         VALUES (?1, ?2, ?3, ?4)
         RETURNING revision",
    )
    .bind(&envelope.op_id)
    .bind(new_revision)
    .bind(&payload)
    .bind(envelope.timestamp.to_string())
    .fetch_one(&mut *tx)
    .await;

    let new_revision = match insert_result {
        Ok(rev) => rev,
        Err(e) => {
            if e.to_string().contains("events.revision") {
                return Err(StoreError::RevisionMismatch {
                    expected: current_revision,
                    found: current_revision + 1,
                });
            }
            return Err(StoreError::Sqlx(e));
        }
    };

    tx.commit().await.map_err(StoreError::Sqlx)?;

    Ok(AppendResult {
        revision: new_revision,
        op_id: envelope.op_id,
        timestamp: envelope.timestamp,
    })
}

/// Appends a batch of events atomically
/// Fetches all events since a given revision
///
/// # Errors
///
/// Returns a `StoreError` if the query fails.
pub async fn fetch_events_since(
    pool: &SqlitePool,
    revision: i64,
) -> Result<Vec<EventRecord>, StoreError> {
    let rows = sqlx::query_as::<_, (String, i64, String, String)>(
        "SELECT operation_id, revision, timestamp, payload FROM events WHERE revision > ?1 ORDER BY revision ASC"
    )
    .bind(revision)
    .fetch_all(pool)
    .await
    .map_err(StoreError::Sqlx)?;

    let mut events = Vec::with_capacity(rows.len());
    for (op_id, revision, timestamp_str, payload) in rows {
        let timestamp: i64 = timestamp_str
            .parse()
            .map_err(|_| StoreError::Serialization("Invalid timestamp format".to_string()))?;
        events.push(EventRecord {
            op_id,
            revision,
            timestamp,
            payload,
        });
    }

    Ok(events)
}

/// Fetches all events from the store
///
/// # Errors
///
/// Returns a `StoreError` if the query fails.
pub async fn fetch_all_events(pool: &SqlitePool) -> Result<Vec<EventRecord>, StoreError> {
    let rows = sqlx::query_as::<_, (String, i64, String, String)>(
        "SELECT operation_id, revision, timestamp, payload FROM events ORDER BY revision ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(StoreError::Sqlx)?;

    let mut events = Vec::with_capacity(rows.len());
    for (op_id, revision, timestamp_str, payload) in rows {
        let timestamp: i64 = timestamp_str
            .parse()
            .map_err(|_| StoreError::Serialization("Invalid timestamp format".to_string()))?;
        events.push(EventRecord {
            op_id,
            revision,
            timestamp,
            payload,
        });
    }

    Ok(events)
}
