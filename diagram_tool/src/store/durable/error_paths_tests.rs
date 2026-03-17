use crate::store::durable::error::DurableError;
use crate::store::durable::operation::*;
use crate::store::durable::outbox::*;
use crate::store::durable::step_journal::*;
use crate::store::durable::test_fixtures::create_test_pool;
use crate::store::types::{OperationState, SideEffectType, StepStatus};

#[tokio::test]
async fn test_get_operation_not_found_returns_error() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;

    let result = get_operation(&pool, "nonexistent").await;

    assert!(result.is_err());
    if let Err(DurableError::OperationNotFound(id)) = result {
        assert_eq!(id, "nonexistent");
    } else {
        panic!("Expected OperationNotFound error");
    }

    Ok(())
}

#[tokio::test]
async fn test_update_operation_state_invalid_transition_returns_error() -> Result<(), DurableError>
{
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

    let result =
        update_operation_state(&pool, "op1", OperationState::Completed, None, None, None).await;

    assert!(result.is_err());
    if let Err(DurableError::OperationStateInvalid { expected, found }) = result {
        assert_eq!(expected, OperationState::Completed);
        assert_eq!(found, OperationState::Started);
    } else {
        panic!("Expected OperationStateInvalid error");
    }

    Ok(())
}

#[tokio::test]
async fn test_get_step_not_found_returns_error() -> Result<(), DurableError> {
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

    let result = get_step(&pool, "op1", 99).await;

    assert!(result.is_err());
    if let Err(DurableError::StepNotFound {
        operation_id,
        step_index,
    }) = result
    {
        assert_eq!(operation_id, "op1");
        assert_eq!(step_index, 99);
    } else {
        panic!("Expected StepNotFound error");
    }

    Ok(())
}

#[tokio::test]
async fn test_update_step_status_already_completed_returns_error() -> Result<(), DurableError> {
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
    update_step_status(&pool, "op1", 0, StepStatus::Completed, None, None)
        .await
        .unwrap();

    let result = update_step_status(
        &pool,
        "op1",
        0,
        StepStatus::Failed,
        None,
        Some("error".to_string()),
    )
    .await;

    assert!(result.is_err());
    if let Err(DurableError::StepAlreadyCompleted {
        operation_id,
        step_index,
    }) = result
    {
        assert_eq!(operation_id, "op1");
        assert_eq!(step_index, 0);
    } else {
        panic!("Expected StepAlreadyCompleted error");
    }

    Ok(())
}

#[tokio::test]
async fn test_mark_outbox_failed_max_retries_exceeded_returns_error() -> Result<(), DurableError> {
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

    for _ in 0..3 {
        mark_outbox_failed(&pool, "outbox1", "error".to_string())
            .await
            .unwrap();
    }

    let result = mark_outbox_failed(&pool, "outbox1", "error".to_string()).await;

    assert!(result.is_err());
    if let Err(DurableError::OutboxMaxRetriesExceeded(id)) = result {
        assert_eq!(id, "outbox1");
    } else {
        panic!("Expected OutboxMaxRetriesExceeded error");
    }

    Ok(())
}
