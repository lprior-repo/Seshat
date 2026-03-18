#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use crate::store::durable::cursor::*;
use crate::store::durable::error::DurableError;
use crate::store::durable::test_fixtures::create_test_pool_with_events;
use crate::store::types::EventCursor;

#[tokio::test]
async fn test_fetch_events_cursor_returns_first_page() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool_with_events().await?;

    for i in 1..=50 {
        sqlx::query("INSERT INTO events (operation_id, revision, timestamp, payload) VALUES ('op1', ?1, '1700000000', ?2)")
            .bind(i)
            .bind(format!(r#"{{"event":{}}}"#, i))
            .execute(&pool).await.unwrap();
    }

    let cursor = EventCursor::first(10);
    let result = fetch_events_cursor(&pool, cursor).await;
    let page = result.expect("fetch_events_cursor should succeed");
    assert_eq!(page.events.len(), 10);
    assert!(page.has_more);
    assert!(page.next_cursor.is_some());

    Ok(())
}

#[tokio::test]
async fn test_fetch_events_cursor_respects_limit_cap() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool_with_events().await?;

    for i in 1..=10 {
        sqlx::query("INSERT INTO events (operation_id, revision, timestamp, payload) VALUES ('op1', ?1, '1700000000', '{}')")
            .bind(i)
            .execute(&pool).await.unwrap();
    }

    let cursor = EventCursor::new(0, 5000);
    let result = fetch_events_cursor(&pool, cursor).await;
    let page = result.expect("fetch_events_cursor should succeed");
    assert!(page.events.len() <= 1000);

    Ok(())
}

#[tokio::test]
async fn test_parse_cursor_valid_format() {
    let cursor_str = "10:50";
    let result = parse_cursor(cursor_str);
    let cursor = result.expect("parse_cursor should succeed");
    assert_eq!(cursor.revision, 10);
    assert_eq!(cursor.limit, 50);
}

#[tokio::test]
async fn test_serialize_cursor_produces_correct_format() {
    let cursor = EventCursor::new(10, 50);
    let result = serialize_cursor(&cursor);
    assert_eq!(result, "10:50");
}

#[tokio::test]
async fn test_fetch_events_cursor_at_end_returns_no_next_cursor() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool_with_events().await?;

    for i in 1..=5 {
        sqlx::query("INSERT INTO events (operation_id, revision, timestamp, payload) VALUES ('op1', ?1, '1700000000', '{}')")
            .bind(i)
            .execute(&pool).await.unwrap();
    }

    let cursor = EventCursor::new(0, 10);
    let result = fetch_events_cursor(&pool, cursor).await;
    let page = result.expect("fetch_events_cursor should succeed");
    assert!(!page.has_more);
    assert!(page.next_cursor.is_none());

    Ok(())
}
