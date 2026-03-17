use crate::store::durable::conflict::generate_conflict_diff;
use crate::store::durable::cursor::fetch_events_cursor;
use crate::store::durable::error::DurableError;
use crate::store::durable::operation::*;
use crate::store::durable::outbox::*;
use crate::store::durable::step_journal::*;
use crate::store::durable::test_fixtures::*;
use crate::store::durable::workflow::*;
use crate::store::types::{EventCursor, OperationState, OutboxStatus, SideEffectType, StepStatus};

#[tokio::test]
async fn test_complete_multi_step_operation_workflow() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    let timestamp = 1_700_000_000_i64;

    let op = start_operation(
        &pool,
        "op1".to_string(),
        3,
        "user1".to_string(),
        "Process data".to_string(),
        timestamp,
    )
    .await
    .unwrap();
    record_step(&pool, "op1".to_string(), 0, "fetch".to_string(), timestamp)
        .await
        .unwrap();
    record_step(
        &pool,
        "op1".to_string(),
        1,
        "transform".to_string(),
        timestamp,
    )
    .await
    .unwrap();
    record_step(&pool, "op1".to_string(), 2, "save".to_string(), timestamp)
        .await
        .unwrap();

    update_step_status(&pool, "op1", 0, StepStatus::Running, None, None)
        .await
        .unwrap();
    update_step_status(&pool, "op1", 0, StepStatus::Completed, Some(1), None)
        .await
        .unwrap();

    update_step_status(&pool, "op1", 1, StepStatus::Running, None, None)
        .await
        .unwrap();
    update_step_status(&pool, "op1", 1, StepStatus::Completed, Some(5), None)
        .await
        .unwrap();

    update_step_status(&pool, "op1", 2, StepStatus::Running, None, None)
        .await
        .unwrap();
    update_step_status(&pool, "op1", 2, StepStatus::Completed, Some(10), None)
        .await
        .unwrap();

    update_operation_state(&pool, "op1", OperationState::InProgress, None, None, None)
        .await
        .unwrap();
    let final_op = update_operation_state(
        &pool,
        "op1",
        OperationState::Completed,
        Some(3),
        Some(10),
        None,
    )
    .await
    .unwrap();

    let step0 = get_step(&pool, "op1", 0).await.unwrap();
    let step1 = get_step(&pool, "op1", 1).await.unwrap();
    let step2 = get_step(&pool, "op1", 2).await.unwrap();

    assert_eq!(step0.status, StepStatus::Completed);
    assert_eq!(step1.status, StepStatus::Completed);
    assert_eq!(step2.status, StepStatus::Completed);
    assert_eq!(final_op.state, OperationState::Completed);
    assert!(final_op.completed_at.is_some());
    assert_eq!(final_op.final_revision, Some(10));

    Ok(())
}

#[tokio::test]
async fn test_failed_operation_with_retry() -> Result<(), DurableError> {
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
    let failed_step = update_step_status(
        &pool,
        "op1",
        0,
        StepStatus::Failed,
        None,
        Some("Step 0 failed".to_string()),
    )
    .await
    .unwrap();
    let failed_op = update_operation_state(
        &pool,
        "op1",
        OperationState::Failed,
        None,
        None,
        Some("Step 0 failed".to_string()),
    )
    .await
    .unwrap();

    assert_eq!(failed_step.status, StepStatus::Failed);
    assert_eq!(failed_step.error_message, Some("Step 0 failed".to_string()));
    assert_eq!(failed_op.state, OperationState::Failed);
    assert_eq!(failed_op.error_message, Some("Step 0 failed".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_outbox_processing_flow() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    let timestamp = 1_700_000_000_i64;

    add_outbox_entry(
        &pool,
        "out1".to_string(),
        SideEffectType::Webhook,
        r#"{"url":"http://example.com"}"#.to_string(),
        1,
        3,
        timestamp,
    )
    .await
    .unwrap();

    let dispatched = mark_outbox_dispatched(&pool, "out1").await.unwrap();
    let acknowledged = acknowledge_outbox(&pool, "out1").await.unwrap();

    assert_eq!(dispatched.status, OutboxStatus::Dispatched);
    assert!(dispatched.dispatched_at.is_some());
    assert_eq!(acknowledged.status, OutboxStatus::Acknowledged);
    assert!(acknowledged.acknowledged_at.is_some());

    Ok(())
}

#[tokio::test]
async fn test_cursor_pagination_through_events() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool_with_events().await?;

    for i in 1..=25 {
        sqlx::query("INSERT INTO events (operation_id, revision, timestamp, payload) VALUES ('op1', ?1, '1700000000', ?2)")
            .bind(i)
            .bind(format!(r#"{{"id":{}}}"#, i))
            .execute(&pool).await.unwrap();
    }

    let page1 = fetch_events_cursor(&pool, EventCursor::first(10))
        .await
        .unwrap();
    let next = page1.next_cursor.expect("next cursor");
    let page2 = fetch_events_cursor(&pool, next).await.unwrap();
    let next2 = page2.next_cursor.expect("next cursor 2");
    let page3 = fetch_events_cursor(&pool, next2).await.unwrap();

    assert_eq!(page1.events.len(), 10);
    assert!(page1.has_more);
    assert_eq!(page2.events.len(), 10);
    assert!(page2.has_more);
    assert_eq!(page3.events.len(), 5);
    assert!(!page3.has_more);

    Ok(())
}

#[tokio::test]
async fn test_conflict_detection() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool_with_events().await?;

    for i in 1..=15 {
        sqlx::query("INSERT INTO events (operation_id, revision, timestamp, payload) VALUES ('op1', ?1, '1700000000', '{}')")
            .bind(i)
            .execute(&pool).await.unwrap();
    }

    let diff = generate_conflict_diff(&pool, 10).await.unwrap();
    assert_eq!(diff.assumed_revision, 10);
    assert_eq!(diff.actual_revision, 15);
    assert!(!diff.changes.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_resume_interrupted_operation() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    let timestamp = 1_700_000_000_i64;

    start_operation(
        &pool,
        "op1".to_string(),
        3,
        "u1".to_string(),
        "d1".to_string(),
        timestamp,
    )
    .await
    .unwrap();
    update_operation_state(&pool, "op1", OperationState::InProgress, None, None, None)
        .await
        .unwrap();

    record_step(&pool, "op1".to_string(), 0, "step0".to_string(), timestamp)
        .await
        .unwrap();
    record_step(&pool, "op1".to_string(), 1, "step1".to_string(), timestamp)
        .await
        .unwrap();
    record_step(&pool, "op1".to_string(), 2, "step2".to_string(), timestamp)
        .await
        .unwrap();

    update_step_status(&pool, "op1", 0, StepStatus::Running, None, None)
        .await
        .unwrap();
    update_step_status(&pool, "op1", 0, StepStatus::Completed, None, None)
        .await
        .unwrap();

    update_step_status(&pool, "op1", 1, StepStatus::Running, None, None)
        .await
        .unwrap();
    update_step_status(
        &pool,
        "op1",
        1,
        StepStatus::Failed,
        None,
        Some("error".to_string()),
    )
    .await
    .unwrap();

    let can_resume = can_resume_operation(&pool, "op1").await.unwrap();
    let next_step = get_next_step(&pool, "op1").await.unwrap();

    assert!(can_resume);
    assert!(next_step.is_some());
    assert_eq!(next_step.unwrap().step_index, 1);

    Ok(())
}
