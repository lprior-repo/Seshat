use crate::store::sqlx::error::*;
use crate::store::sqlx::models::*;
use sqlx::SqlitePool;
use diagram_models::envelope::{encode_event_envelope, parse_event_envelope, EventEnvelope};

/// Save a snapshot of the current projection state
///
/// This function:
/// 1. Validates the projection revision matches current latest revision
/// 2. Serializes the projection to JSON
/// 3. Stores in the snapshots table
///
/// # Errors
/// Returns `StoreError::SnapshotStale` if projection revision doesn't match
/// Returns `StoreError::Serialization` if encoding fails
/// Returns `StoreError::Sqlx` if database operations fail
pub async fn save_snapshot(
    pool: &SqlitePool,
    projection: &diagram_models::projection::DiagramProjection,
) -> Result<SnapshotMeta, StoreError> {
    let current_revision: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(revision), 0) FROM events")
        .fetch_one(pool)
        .await?;

    let projection_revision = i64::try_from(projection.revision)
        .map_err(|_| StoreError::Serialization("Revision too large for i64".to_string()))?;

    if projection_revision != current_revision {
        return Err(StoreError::SnapshotStale {
            expected: current_revision,
            found: projection_revision,
        });
    }

    let payload =
        serde_json::to_string(projection).map_err(|e| StoreError::Serialization(e.to_string()))?;

    let mut tx = pool.begin().await?;

    let now_ts: i64 = sqlx::query_scalar("SELECT CAST(strftime('%s', 'now') AS INTEGER)")
        .fetch_one(&mut *tx)
        .await?;

    sqlx::query(
        "INSERT OR REPLACE INTO snapshots (revision, payload, created_at) VALUES (?1, ?2, ?3)",
    )
    .bind(projection_revision)
    .bind(&payload)
    .bind(now_ts)
    .execute(&mut *tx)
    .await?;

    let id: i64 = sqlx::query_scalar("SELECT id FROM snapshots WHERE revision = ?1")
        .bind(projection_revision)
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(SnapshotMeta {
        id,
        revision: projection_revision,
        created_at: now_ts,
    })
}

/// Load projection from latest snapshot with tail replay
///
/// This function:
/// 1. Loads the latest snapshot from the database
/// 2. Fetches all events with revision greater than snapshot revision
/// 3. Replays events on top of the snapshot to produce the final projection
///
/// If no snapshot exists, falls back to full replay from revision 0.
///
/// # Errors
/// Returns `StoreError::NotFound` if no snapshot exists
/// Returns `StoreError::Serialization` if deserialization fails
/// Returns `StoreError::Sqlx` if database operations fail
pub async fn load_projection_from_snapshot(
    pool: &SqlitePool,
) -> Result<diagram_models::projection::DiagramProjection, StoreError> {
    let snapshot_result = sqlx::query_as::<_, (i64, i64, String, i64)>(
        "SELECT id, revision, payload, created_at FROM snapshots ORDER BY revision DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;

    // If no snapshot exists, start from revision 0 and replay all events
    let base_projection: diagram_models::projection::DiagramProjection = match snapshot_result {
        Some((_snapshot_id, _revision, payload, _created_at)) => {
            serde_json::from_str(&payload).map_err(|e| StoreError::Serialization(e.to_string()))?
        }
        None => diagram_models::projection::DiagramProjection::default(),
    };

    let rows = sqlx::query_as::<_, (String, i64, String, String)>(
        "SELECT operation_id, revision, payload, timestamp FROM events WHERE revision > ?1 ORDER BY revision ASC",
    )
    .bind(base_projection.revision.cast_signed())
    .fetch_all(pool)
    .await?;

    // Use fold to replay events functionally (no mutability)
    let final_projection = rows.into_iter().try_fold(base_projection, |acc, row| {
        let (op_id, _revision, event_payload, timestamp_str) = row;
        let envelope = parse_event_envelope(&event_payload)
            .map_err(|e| StoreError::Serialization(e.to_string()))?;

        let timestamp: i64 = timestamp_str
            .parse()
            .map_err(|_| StoreError::Serialization("Invalid timestamp format".to_string()))?;

        let event = diagram_models::projection::EventRecord {
            op_id,
            revision: acc.revision,
            operation: envelope.operation,
            author: envelope.author,
            timestamp,
        };

    use diagram_models::projection::apply_event(acc, &event)
            .map_err(|e| StoreError::Serialization(format!("Replay error: {e}")))
    })?;

    Ok(final_projection)
}
/// Get metadata for the latest snapshot
///
/// Returns `Ok(Some(meta))` if a snapshot exists, `Ok(None)` if no snapshots exist.
///
/// # Errors
/// Returns `StoreError::Sqlx` if database operations fail
pub async fn get_latest_snapshot_meta(
    pool: &SqlitePool,
) -> Result<Option<SnapshotMeta>, StoreError> {
    let result = sqlx::query_as::<_, (i64, i64, i64)>(
        "SELECT id, revision, created_at FROM snapshots ORDER BY revision DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;

    match result {
        Some((id, revision, created_at)) => Ok(Some(SnapshotMeta {
            id,
            revision,
            created_at,
        })),
        None => Ok(None),
    }
}

/// Delete a snapshot by revision
///
/// # Errors
/// Returns `StoreError::InvalidInput` if revision is negative
/// Returns `StoreError::NotFound` if no snapshot exists at the given revision
/// Returns `StoreError::Sqlx` if database operations fail
pub async fn delete_snapshot(pool: &SqlitePool, revision: i64) -> Result<(), StoreError> {
    if revision < 0 {
        return Err(StoreError::InvalidInput(
            "revision must be non-negative".to_string(),
        ));
    }

    let result = sqlx::query("DELETE FROM snapshots WHERE revision = ?1")
        .bind(revision)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(StoreError::NotFound(format!(
            "no snapshot at revision {revision}"
        )));
    }

    Ok(())
}

/// List all snapshot metadata, ordered by revision descending
///
/// # Errors
/// Returns `StoreError::Sqlx` if database operations fail
pub async fn list_snapshots(pool: &SqlitePool) -> Result<Vec<SnapshotMeta>, StoreError> {
    let rows = sqlx::query_as::<_, (i64, i64, i64)>(
        "SELECT id, revision, created_at FROM snapshots ORDER BY revision DESC",
    )
    .fetch_all(pool)
    .await?;

    let snapshots = rows
        .into_iter()
        .map(|(id, revision, created_at)| SnapshotMeta {
            id,
            revision,
            created_at,
        })
        .collect();

    Ok(snapshots)
}
