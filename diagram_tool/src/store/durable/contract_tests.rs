use crate::store::durable::error::DurableError;
use crate::store::durable::operation::*;
use crate::store::durable::outbox::*;
use crate::store::durable::step_journal::*;
use crate::store::durable::test_fixtures::create_test_pool;
use crate::store::types::{OperationState, OutboxStatus, SideEffectType, StepStatus};

#[tokio::test]
async fn test_precondition_operation_id_non_empty() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    let result = start_operation(
        &pool,
        "".to_string(),
        1,
        "author".to_string(),
        "desc".to_string(),
        123,
    )
    .await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_precondition_total_steps_at_least_one() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    let result = start_operation(
        &pool,
        "op1".to_string(),
        0,
        "author".to_string(),
        "desc".to_string(),
        123,
    )
    .await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_precondition_author_id_non_empty() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    let result = start_operation(
        &pool,
        "op1".to_string(),
        1,
        "".to_string(),
        "desc".to_string(),
        123,
    )
    .await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_precondition_description_non_empty() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    let result = start_operation(
        &pool,
        "op1".to_string(),
        1,
        "author".to_string(),
        "".to_string(),
        123,
    )
    .await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_precondition_valid_state_transition() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    start_operation(
        &pool,
        "op1".to_string(),
        2,
        "u1".to_string(),
        "d1".to_string(),
        123,
    )
    .await
    .unwrap();
    let result =
        update_operation_state(&pool, "op1", OperationState::Completed, None, None, None).await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_precondition_step_index_unique() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    start_operation(
        &pool,
        "op1".to_string(),
        2,
        "u1".to_string(),
        "d1".to_string(),
        123,
    )
    .await
    .unwrap();
    record_step(&pool, "op1".to_string(), 0, "step0".to_string(), 123)
        .await
        .unwrap();
    let result = record_step(&pool, "op1".to_string(), 0, "duplicate".to_string(), 123).await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_precondition_get_step_exists() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    start_operation(
        &pool,
        "op1".to_string(),
        2,
        "u1".to_string(),
        "d1".to_string(),
        123,
    )
    .await
    .unwrap();
    let result = get_step(&pool, "op1", 0).await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_precondition_outbox_id_non_empty() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    let result = add_outbox_entry(
        &pool,
        "".to_string(),
        SideEffectType::Webhook,
        "payload".to_string(),
        1,
        3,
        123,
    )
    .await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_precondition_outbox_payload_non_empty() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    let result = add_outbox_entry(
        &pool,
        "outbox1".to_string(),
        SideEffectType::Webhook,
        "".to_string(),
        1,
        3,
        123,
    )
    .await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_postcondition_completed_at_set_on_terminal_state() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    start_operation(
        &pool,
        "op1".to_string(),
        2,
        "u1".to_string(),
        "d1".to_string(),
        123,
    )
    .await
    .unwrap();
    update_operation_state(&pool, "op1", OperationState::InProgress, None, None, None)
        .await
        .unwrap();
    let result =
        update_operation_state(&pool, "op1", OperationState::Completed, None, None, None).await;
    let record = result.expect("update should succeed");
    assert!(record.completed_at.is_some());
    Ok(())
}

#[tokio::test]
async fn test_postcondition_started_at_set_on_running_step() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    start_operation(
        &pool,
        "op1".to_string(),
        2,
        "u1".to_string(),
        "d1".to_string(),
        123,
    )
    .await
    .unwrap();
    record_step(&pool, "op1".to_string(), 0, "step0".to_string(), 123)
        .await
        .unwrap();
    let result = update_step_status(&pool, "op1", 0, StepStatus::Running, None, None).await;
    let record = result.expect("update should succeed");
    assert!(record.started_at.is_some());
    Ok(())
}

#[tokio::test]
async fn test_invariant_operation_state_valid() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    let record = start_operation(
        &pool,
        "op1".to_string(),
        2,
        "u1".to_string(),
        "d1".to_string(),
        123,
    )
    .await
    .unwrap();
    match record.state {
        OperationState::Started
        | OperationState::InProgress
        | OperationState::Completed
        | OperationState::Failed => {}
    }
    Ok(())
}

#[tokio::test]
async fn test_invariant_step_status_valid() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    start_operation(
        &pool,
        "op1".to_string(),
        2,
        "u1".to_string(),
        "d1".to_string(),
        123,
    )
    .await
    .unwrap();
    let record = record_step(&pool, "op1".to_string(), 0, "step0".to_string(), 123)
        .await
        .unwrap();
    match record.status {
        StepStatus::Pending
        | StepStatus::Running
        | StepStatus::Completed
        | StepStatus::Failed
        | StepStatus::Skipped => {}
    }
    Ok(())
}

#[tokio::test]
async fn test_invariant_outbox_status_valid() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    let record = add_outbox_entry(
        &pool,
        "outbox1".to_string(),
        SideEffectType::Webhook,
        "payload".to_string(),
        1,
        3,
        123,
    )
    .await
    .unwrap();
    match record.status {
        OutboxStatus::Pending
        | OutboxStatus::Dispatched
        | OutboxStatus::Acknowledged
        | OutboxStatus::Failed => {}
    }
    Ok(())
}

#[tokio::test]
async fn test_invariant_current_step_lte_total_steps() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    let record = start_operation(
        &pool,
        "op1".to_string(),
        3,
        "u1".to_string(),
        "d1".to_string(),
        123,
    )
    .await
    .unwrap();
    assert!(record.current_step <= record.total_steps);
    Ok(())
}
