use sqlx::SqlitePool;

use crate::store::durable::error::DurableError;
use crate::store::types::{EventCursor, EventPage, EventRecord};

/// Fetches events using cursor-based pagination
///
/// # Errors
/// Returns an error if database query fails or timestamp parsing fails.
pub async fn fetch_events_cursor(
    pool: &SqlitePool,
    cursor: EventCursor,
) -> Result<EventPage, DurableError> {
    let limit = cursor.limit.min(1000);

    let rows = sqlx::query_as::<_, (String, i64, String, String)>(
        "SELECT operation_id, revision, timestamp, payload FROM events
         WHERE revision > ?1 ORDER BY revision ASC LIMIT ?2",
    )
    .bind(cursor.revision)
    .bind(i64::from(limit + 1))
    .fetch_all(pool)
    .await
    .map_err(DurableError::Sqlx)?;

    let has_more = rows.len() > limit as usize;
    let events: Vec<_> = rows.into_iter().take(limit as usize).collect();

    let mut event_records = Vec::with_capacity(events.len());
    let mut last_revision = cursor.revision;

    for (op_id, revision, timestamp_str, payload) in events {
        let timestamp: i64 = timestamp_str
            .parse()
            .map_err(|_| DurableError::Serialization("Invalid timestamp format".to_string()))?;

        last_revision = revision;
        event_records.push(EventRecord {
            op_id,
            revision,
            timestamp,
            payload,
        });
    }

    let next_cursor = if has_more {
        Some(EventCursor::new(last_revision, cursor.limit))
    } else {
        None
    };

    Ok(EventPage {
        events: event_records,
        next_cursor,
        has_more,
    })
}

/// Parses a cursor from a string
///
/// # Errors
/// Returns an error if cursor string format is invalid.
pub fn parse_cursor(cursor_str: &str) -> Result<EventCursor, DurableError> {
    let parts: Vec<&str> = cursor_str.split(':').collect();
    if parts.len() != 2 {
        return Err(DurableError::CursorParseError(
            "Expected format: revision:limit".to_string(),
        ));
    }

    let revision = parts[0]
        .parse()
        .map_err(|_| DurableError::CursorParseError("Invalid revision".to_string()))?;
    let limit = parts[1]
        .parse()
        .map_err(|_| DurableError::CursorParseError("Invalid limit".to_string()))?;

    Ok(EventCursor::new(revision, limit))
}

/// Serializes a cursor to a string
#[must_use]
pub fn serialize_cursor(cursor: &EventCursor) -> String {
    format!("{}:{}", cursor.revision, cursor.limit)
}
