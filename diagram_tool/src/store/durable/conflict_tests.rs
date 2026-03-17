use crate::store::durable::conflict::generate_conflict_diff;
use crate::store::durable::error::DurableError;
use crate::store::durable::test_fixtures::create_test_pool_with_events;

#[tokio::test]
async fn test_generate_conflict_diff_returns_current_revision() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool_with_events().await?;

    sqlx::query("INSERT INTO events (operation_id, revision, timestamp, payload) VALUES ('op1', 1, '1700000000', '{}')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO events (operation_id, revision, timestamp, payload) VALUES ('op1', 2, '1700000001', '{}')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO events (operation_id, revision, timestamp, payload) VALUES ('op1', 3, '1700000002', '{}')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO events (operation_id, revision, timestamp, payload) VALUES ('op1', 4, '1700000003', '{}')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO events (operation_id, revision, timestamp, payload) VALUES ('op1', 5, '1700000004', '{}')").execute(&pool).await.unwrap();

    let result = generate_conflict_diff(&pool, 3).await;
    let diff = result.expect("generate_conflict_diff should succeed");
    assert_eq!(diff.assumed_revision, 3);
    assert_eq!(diff.actual_revision, 5);
    assert!(!diff.changes.is_empty());

    Ok(())
}
