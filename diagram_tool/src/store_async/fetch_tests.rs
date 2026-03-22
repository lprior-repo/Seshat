use crate::store_async::{
    bootstrap::bootstrap_async_store,
    fetch::{fetch_all_events, fetch_events_since},
};
use tempfile::TempDir;

#[tokio::test]
async fn test_fetch_events_happy_path() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let bootstrap = bootstrap_async_store(&db_path).await.unwrap();
    let pool = &bootstrap.pool;

    // Insert some valid events
    sqlx::query(
        "INSERT INTO events (operation_id, revision, timestamp, payload) VALUES (?1, ?2, ?3, ?4)"
    )
    .bind("op1")
    .bind(1)
    .bind("1600000000") // String timestamp
    .bind(r#"{"some":"json"}"#)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO events (operation_id, revision, timestamp, payload) VALUES (?1, ?2, ?3, ?4)"
    )
    .bind("op2")
    .bind(2)
    .bind("1600000001")
    .bind(r#"{"some":"other_json"}"#)
    .execute(pool)
    .await
    .unwrap();

    let all_events = fetch_all_events(pool).await.unwrap();
    assert_eq!(all_events.len(), 2);
    assert_eq!(all_events[0].op_id, "op1");
    assert_eq!(all_events[0].revision, 1);
    assert_eq!(all_events[0].timestamp, 1600000000);
    assert_eq!(all_events[0].payload, r#"{"some":"json"}"#);

    let since_events = fetch_events_since(pool, 1).await.unwrap();
    assert_eq!(since_events.len(), 1);
    assert_eq!(since_events[0].op_id, "op2");
    assert_eq!(since_events[0].revision, 2);
}

#[tokio::test]
async fn test_fetch_events_invalid_timestamp() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_invalid_ts.db");
    
    let bootstrap = bootstrap_async_store(&db_path).await.unwrap();
    let pool = &bootstrap.pool;

    // Insert an event with an invalid timestamp string
    sqlx::query(
        "INSERT INTO events (operation_id, revision, timestamp, payload) VALUES (?1, ?2, ?3, ?4)"
    )
    .bind("op_invalid")
    .bind(1)
    .bind("not-a-number")
    .bind(r#"{"some":"json"}"#)
    .execute(pool)
    .await
    .unwrap();

    let result = fetch_all_events(pool).await;
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    match err {
        crate::store_async::AsyncStoreError::Serialization(msg) => {
            assert_eq!(msg, "Invalid timestamp format");
        },
        _ => panic!("Expected Serialization error"),
    }
}
