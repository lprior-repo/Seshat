use sqlx::SqlitePool;

use crate::store::durable::error::DurableError;
use crate::store::types::{ConflictDiff, DiffDomainOp};
use crate::store_async::{fetch_events_since, fetch_latest_revision};

/// Generates a rich diff when a conditional append is rejected
///
/// # Errors
/// Returns an error if database query fails.
pub async fn generate_conflict_diff(
    pool: &SqlitePool,
    assumed_revision: i64,
) -> Result<ConflictDiff, DurableError> {
    let current_revision = fetch_latest_revision(pool).await?;

    if current_revision <= assumed_revision {
        return Ok(ConflictDiff {
            assumed_revision,
            actual_revision: current_revision,
            changes: vec![],
            first_change_timestamp: 0,
            first_change_author: String::new(),
        });
    }

    let events = fetch_events_since(pool, assumed_revision).await?;

    let mut changes = Vec::with_capacity(events.len());
    let mut first_timestamp: i64 = 0;
    let mut first_author = String::new();

    for event in &events {
        if first_timestamp == 0 {
            first_timestamp = event.timestamp;
            first_author = "unknown".to_string();
        }
        changes.push(DiffDomainOp::Other);
    }

    Ok(ConflictDiff {
        assumed_revision,
        actual_revision: current_revision,
        changes,
        first_change_timestamp: first_timestamp,
        first_change_author: first_author,
    })
}
