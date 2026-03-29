#![allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use crate::store_async::{
    bootstrap::bootstrap_async_store,
    fetch::{fetch_all_events, fetch_events_since},
};
use tempfile::TempDir;

#[tokio::test]
async fn test_fetch_events_happy_path() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");

    let bootstrap = bootstrap_async_store(&db_path).await?;
    let pool = &bootstrap.pool;

    // Insert some valid events
    sqlx::query(
        "INSERT INTO events (operation_id, revision, timestamp, payload) VALUES (?1, ?2, ?3, ?4)",
    )
    .bind("op1")
    .bind(1)
    .bind("1600000000") // String timestamp
    .bind(r#"{"some":"json"}"#)
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO events (operation_id, revision, timestamp, payload) VALUES (?1, ?2, ?3, ?4)",
    )
    .bind("op2")
    .bind(2)
    .bind("1600000001")
    .bind(r#"{"some":"other_json"}"#)
    .execute(pool)
    .await?;

    let all_events = fetch_all_events(pool).await?;
    assert_eq!(all_events.len(), 2);
    assert_eq!(all_events[0].op_id, "op1");
    assert_eq!(all_events[0].revision, 1);
    assert_eq!(all_events[0].timestamp, 1600000000);
    assert_eq!(all_events[0].payload, r#"{"some":"json"}"#);

    let since_events = fetch_events_since(pool, 1).await?;
    assert_eq!(since_events.len(), 1);
    assert_eq!(since_events[0].op_id, "op2");
    assert_eq!(since_events[0].revision, 2);

    Ok(())
}

#[tokio::test]
async fn test_fetch_events_invalid_timestamp() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_invalid_ts.db");

    let bootstrap = bootstrap_async_store(&db_path).await?;
    let pool = &bootstrap.pool;

    // Insert an event with an invalid timestamp string
    sqlx::query(
        "INSERT INTO events (operation_id, revision, timestamp, payload) VALUES (?1, ?2, ?3, ?4)",
    )
    .bind("op_invalid")
    .bind(1)
    .bind("not-a-number")
    .bind(r#"{"some":"json"}"#)
    .execute(pool)
    .await?;

    let result = fetch_all_events(pool).await;
    assert!(result.is_err());

    let err = result.unwrap_err();
    match err {
        crate::store_async::AsyncStoreError::Serialization(msg) => {
            assert_eq!(msg, "Invalid timestamp format");
        }
        _ => return Err("Expected Serialization error".into()),
    }

    Ok(())
}
