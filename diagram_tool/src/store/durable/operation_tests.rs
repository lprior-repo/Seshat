#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use crate::store::durable::error::DurableError;
use crate::store::durable::operation::*;
use crate::store::durable::test_fixtures::create_test_pool;
use crate::store::types::OperationState;

#[tokio::test]
async fn test_start_operation_creates_record_with_started_state() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    let timestamp = 1_700_000_000_i64;

    let result = start_operation(
        &pool,
        "op1".to_string(),
        3,
        "user1".to_string(),
        "Process data".to_string(),
        timestamp,
    )
    .await;

    let record = result.expect("start_operation should succeed");
    assert_eq!(record.operation_id, "op1");
    assert_eq!(record.state, OperationState::Started);
    assert_eq!(record.current_step, 0);
    assert_eq!(record.total_steps, 3);
    assert_eq!(record.author_id, "user1");
    assert_eq!(record.description, "Process data");
    assert_eq!(record.started_at, timestamp);
    assert!(record.completed_at.is_none());

    Ok(())
}

#[tokio::test]
async fn test_get_operation_returns_stored_record() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    let timestamp = 1_700_000_000_i64;

    let created = start_operation(
        &pool,
        "op1".to_string(),
        2,
        "user1".to_string(),
        "Test description".to_string(),
        timestamp,
    )
    .await
    .expect("start_operation should succeed");

    let retrieved = get_operation(&pool, "op1").await;
    let record = retrieved.expect("get_operation should succeed");
    assert_eq!(record.operation_id, created.operation_id);
    assert_eq!(record.state, created.state);
    assert_eq!(record.current_step, created.current_step);
    assert_eq!(record.total_steps, created.total_steps);
    assert_eq!(record.author_id, created.author_id);
    assert_eq!(record.description, created.description);

    Ok(())
}

#[tokio::test]
async fn test_update_operation_state_to_in_progress() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    let timestamp = 1_700_000_000_i64;

    start_operation(
        &pool,
        "op1".to_string(),
        3,
        "user1".to_string(),
        "Process".to_string(),
        timestamp,
    )
    .await
    .expect("start_operation should succeed");

    let result =
        update_operation_state(&pool, "op1", OperationState::InProgress, None, None, None).await;
    let record = result.expect("update_operation_state should succeed");
    assert_eq!(record.state, OperationState::InProgress);

    Ok(())
}

#[tokio::test]
async fn test_update_operation_state_to_completed_sets_completed_at() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    let timestamp = 1_700_000_000_i64;

    start_operation(
        &pool,
        "op1".to_string(),
        3,
        "user1".to_string(),
        "Process".to_string(),
        timestamp,
    )
    .await
    .expect("start_operation should succeed");

    update_operation_state(&pool, "op1", OperationState::InProgress, None, None, None)
        .await
        .expect("update to InProgress should succeed");

    let result = update_operation_state(
        &pool,
        "op1",
        OperationState::Completed,
        Some(3),
        Some(10),
        None,
    )
    .await;
    let record = result.expect("update to Completed should succeed");
    assert_eq!(record.state, OperationState::Completed);
    assert!(record.completed_at.is_some());
    assert_eq!(record.final_revision, Some(10));
    assert_eq!(record.current_step, 3);

    Ok(())
}

#[tokio::test]
async fn test_get_operations_by_state_returns_matching_operations() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    let timestamp = 1_700_000_000_i64;

    start_operation(
        &pool,
        "op1".to_string(),
        2,
        "u1".to_string(),
        "d1".to_string(),
        timestamp,
    )
    .await
    .expect("start op1");
    start_operation(
        &pool,
        "op2".to_string(),
        2,
        "u2".to_string(),
        "d2".to_string(),
        timestamp,
    )
    .await
    .expect("start op2");
    start_operation(
        &pool,
        "op3".to_string(),
        2,
        "u3".to_string(),
        "d3".to_string(),
        timestamp,
    )
    .await
    .expect("start op3");

    update_operation_state(&pool, "op1", OperationState::InProgress, None, None, None)
        .await
        .expect("update op1 to InProgress");

    let result = get_operations_by_state(&pool, OperationState::InProgress).await;
    let operations = result.expect("get_operations_by_state should succeed");
    assert_eq!(operations.len(), 1);
    assert_eq!(operations[0].operation_id, "op1");

    Ok(())
}
