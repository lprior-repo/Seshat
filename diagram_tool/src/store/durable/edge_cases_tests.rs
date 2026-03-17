use crate::store::durable::conflict::generate_conflict_diff;
use crate::store::durable::cursor::fetch_events_cursor;
use crate::store::durable::error::DurableError;
use crate::store::durable::operation::*;
use crate::store::durable::outbox::*;
use crate::store::durable::step_journal::*;
use crate::store::durable::test_fixtures::*;
use crate::store::types::{EventCursor, OperationState, SideEffectType, StepStatus};

#[tokio::test]
async fn test_start_operation_with_single_step() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    let timestamp = 1_700_000_000_i64;

    let result = start_operation(
        &pool,
        "op1".to_string(),
        1,
        "user1".to_string(),
        "Single step".to_string(),
        timestamp,
    )
    .await;
    let record = result.expect("start_operation should succeed");
    assert_eq!(record.total_steps, 1);

    Ok(())
}

#[tokio::test]
async fn test_update_operation_state_idempotent_completed() -> Result<(), DurableError> {
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
    .unwrap();
    update_operation_state(&pool, "op1", OperationState::InProgress, None, None, None)
        .await
        .unwrap();
    update_operation_state(&pool, "op1", OperationState::Completed, None, None, None)
        .await
        .unwrap();

    let result =
        update_operation_state(&pool, "op1", OperationState::Completed, None, None, None).await;
    let record = result.expect("update_operation_state should succeed");
    assert_eq!(record.state, OperationState::Completed);

    Ok(())
}

#[tokio::test]
async fn test_update_step_status_idempotent_running() -> Result<(), DurableError> {
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
    .unwrap();
    record_step(&pool, "op1".to_string(), 0, "step0".to_string(), timestamp)
        .await
        .unwrap();
    update_step_status(&pool, "op1", 0, StepStatus::Running, None, None)
        .await
        .unwrap();

    let result = update_step_status(&pool, "op1", 0, StepStatus::Running, None, None).await;
    let record = result.expect("update_step_status should succeed");
    assert_eq!(record.status, StepStatus::Running);

    Ok(())
}

#[tokio::test]
async fn test_mark_outbox_failed_multiple_times_increments_retry() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    let timestamp = 1_700_000_000_i64;

    add_outbox_entry(
        &pool,
        "outbox1".to_string(),
        SideEffectType::Webhook,
        "payload".to_string(),
        1,
        3,
        timestamp,
    )
    .await
    .unwrap();

    mark_outbox_failed(&pool, "outbox1", "error1".to_string())
        .await
        .unwrap();
    mark_outbox_failed(&pool, "outbox1", "error2".to_string())
        .await
        .unwrap();
    let result = mark_outbox_failed(&pool, "outbox1", "error3".to_string()).await;

    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_fetch_events_cursor_with_zero_events() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool_with_events().await?;
    let cursor = EventCursor::first(10);
    let result = fetch_events_cursor(&pool, cursor).await;
    let page = result.expect("fetch_events_cursor should succeed");
    assert!(page.events.is_empty());
    assert!(!page.has_more);

    Ok(())
}

#[tokio::test]
async fn test_get_pending_outbox_with_zero_limit() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    let timestamp = 1_700_000_000_i64;

    add_outbox_entry(
        &pool,
        "out1".to_string(),
        SideEffectType::Notify,
        "p1".to_string(),
        1,
        3,
        timestamp,
    )
    .await
    .unwrap();
    let result = get_pending_outbox(&pool, 0).await;
    let entries = result.expect("get_pending_outbox should succeed");
    assert!(entries.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_conflict_diff_with_no_new_events() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool_with_events().await?;

    for i in 1..=5 {
        sqlx::query("INSERT INTO events (operation_id, revision, timestamp, payload) VALUES ('op1', ?1, '1700000000', '{}')")
            .bind(i)
            .execute(&pool).await.unwrap();
    }

    let result = generate_conflict_diff(&pool, 5).await;
    let diff = result.expect("generate_conflict_diff should succeed");
    assert!(diff.changes.is_empty());
    assert_eq!(diff.actual_revision, 5);

    Ok(())
}
