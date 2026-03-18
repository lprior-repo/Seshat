#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use crate::store::durable::error::DurableError;
use crate::store::durable::operation::*;
use crate::store::durable::step_journal::*;
use crate::store::durable::test_fixtures::create_test_pool;
use crate::store::durable::workflow::*;
use crate::store::types::{OperationState, StepStatus};

#[tokio::test]
async fn test_can_resume_operation_true_when_in_progress_with_pending_steps(
) -> Result<(), DurableError> {
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
    record_step(&pool, "op1".to_string(), 0, "step0".to_string(), timestamp)
        .await
        .unwrap();
    record_step(&pool, "op1".to_string(), 1, "step1".to_string(), timestamp)
        .await
        .unwrap();

    let result = can_resume_operation(&pool, "op1").await;
    let can_resume = result.expect("can_resume_operation should succeed");
    assert!(can_resume);

    Ok(())
}

#[tokio::test]
async fn test_can_resume_operation_false_when_completed() -> Result<(), DurableError> {
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

    let result = can_resume_operation(&pool, "op1").await;
    let can_resume = result.expect("can_resume_operation should succeed");
    assert!(!can_resume);

    Ok(())
}

#[tokio::test]
async fn test_get_next_step_returns_first_pending_step() -> Result<(), DurableError> {
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

    let result = get_next_step(&pool, "op1").await;
    let step = result.expect("get_next_step should succeed");
    assert!(step.is_some());
    assert_eq!(step.unwrap().step_index, 1);

    Ok(())
}
