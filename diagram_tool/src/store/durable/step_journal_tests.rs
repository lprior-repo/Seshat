use crate::store::durable::error::DurableError;
use crate::store::durable::operation::start_operation;
use crate::store::durable::step_journal::*;
use crate::store::durable::test_fixtures::create_test_pool;
use crate::store::types::StepStatus;

#[tokio::test]
async fn test_record_step_creates_pending_step() -> Result<(), DurableError> {
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
    .unwrap();

    let result = record_step(
        &pool,
        "op1".to_string(),
        0,
        "analyze".to_string(),
        timestamp,
    )
    .await;
    let record = result.expect("record_step should succeed");
    assert_eq!(record.operation_id, "op1");
    assert_eq!(record.step_index, 0);
    assert_eq!(record.step_name, "analyze");
    assert_eq!(record.status, StepStatus::Pending);
    assert!(record.started_at.is_none());
    assert!(record.completed_at.is_none());

    Ok(())
}

#[tokio::test]
async fn test_get_step_returns_stored_step() -> Result<(), DurableError> {
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
    let created = record_step(&pool, "op1".to_string(), 0, "step1".to_string(), timestamp)
        .await
        .unwrap();

    let retrieved = get_step(&pool, "op1", 0).await;
    let record = retrieved.expect("get_step should succeed");
    assert_eq!(record.operation_id, created.operation_id);
    assert_eq!(record.step_index, created.step_index);
    assert_eq!(record.step_name, created.step_name);
    assert_eq!(record.status, created.status);

    Ok(())
}

#[tokio::test]
async fn test_update_step_status_to_running_sets_started_at() -> Result<(), DurableError> {
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
    record_step(&pool, "op1".to_string(), 0, "step1".to_string(), timestamp)
        .await
        .unwrap();

    let result = update_step_status(&pool, "op1", 0, StepStatus::Running, None, None).await;
    let record = result.expect("update_step_status should succeed");
    assert_eq!(record.status, StepStatus::Running);
    assert!(record.started_at.is_some());

    Ok(())
}

#[tokio::test]
async fn test_update_step_status_to_completed_sets_completed_at() -> Result<(), DurableError> {
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
    record_step(&pool, "op1".to_string(), 0, "step1".to_string(), timestamp)
        .await
        .unwrap();
    update_step_status(&pool, "op1", 0, StepStatus::Running, None, None)
        .await
        .unwrap();

    let result = update_step_status(&pool, "op1", 0, StepStatus::Completed, Some(5), None).await;
    let record = result.expect("update_step_status should succeed");
    assert_eq!(record.status, StepStatus::Completed);
    assert!(record.completed_at.is_some());
    assert_eq!(record.event_revision, Some(5));

    Ok(())
}

#[tokio::test]
async fn test_get_pending_steps_returns_pending_and_failed_steps() -> Result<(), DurableError> {
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

    update_step_status(&pool, "op1", 1, StepStatus::Running, None, None)
        .await
        .unwrap();
    update_step_status(
        &pool,
        "op1",
        1,
        StepStatus::Failed,
        Some(1),
        Some("error".to_string()),
    )
    .await
    .unwrap();

    let result = get_pending_steps(&pool, "op1").await;
    let steps = result.expect("get_pending_steps should succeed");
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[0].step_index, 1);
    assert_eq!(steps[1].step_index, 2);

    Ok(())
}

#[tokio::test]
async fn test_skip_step_marks_step_as_skipped() -> Result<(), DurableError> {
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
    record_step(&pool, "op1".to_string(), 0, "step1".to_string(), timestamp)
        .await
        .unwrap();

    let result = skip_step(&pool, "op1", 0).await;
    let record = result.expect("skip_step should succeed");
    assert_eq!(record.status, StepStatus::Skipped);
    assert!(record.completed_at.is_some());

    Ok(())
}
