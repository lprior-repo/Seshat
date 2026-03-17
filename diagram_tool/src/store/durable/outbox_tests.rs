use crate::store::durable::error::DurableError;
use crate::store::durable::outbox::*;
use crate::store::durable::test_fixtures::create_test_pool;
use crate::store::types::{OutboxStatus, SideEffectType};

#[tokio::test]
async fn test_add_outbox_entry_creates_pending_entry() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    let timestamp = 1_700_000_000_i64;

    let result = add_outbox_entry(
        &pool,
        "outbox1".to_string(),
        SideEffectType::Webhook,
        r#"{"url":"http://example.com"}"#.to_string(),
        1,
        3,
        timestamp,
    )
    .await;
    let record = result.expect("add_outbox_entry should succeed");
    assert_eq!(record.id, "outbox1");
    assert_eq!(record.side_effect_type, SideEffectType::Webhook);
    assert_eq!(record.status, OutboxStatus::Pending);
    assert_eq!(record.retry_count, 0);
    assert_eq!(record.max_retries, 3);

    Ok(())
}

#[tokio::test]
async fn test_get_outbox_entry_returns_stored_entry() -> Result<(), DurableError> {
    let (pool, _temp_dir) = create_test_pool().await?;
    let timestamp = 1_700_000_000_i64;

    let created = add_outbox_entry(
        &pool,
        "outbox1".to_string(),
        SideEffectType::Notify,
        "payload".to_string(),
        1,
        3,
        timestamp,
    )
    .await
    .unwrap();
    let retrieved = get_outbox_entry(&pool, "outbox1").await;
    let record = retrieved.expect("get_outbox_entry should succeed");
    assert_eq!(record.id, created.id);
    assert_eq!(record.side_effect_type, created.side_effect_type);
    assert_eq!(record.status, created.status);
    assert_eq!(record.retry_count, created.retry_count);

    Ok(())
}

#[tokio::test]
async fn test_mark_outbox_dispatched_sets_dispatched_at() -> Result<(), DurableError> {
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
    let result = mark_outbox_dispatched(&pool, "outbox1").await;
    let record = result.expect("mark_outbox_dispatched should succeed");
    assert_eq!(record.status, OutboxStatus::Dispatched);
    assert!(record.dispatched_at.is_some());

    Ok(())
}

#[tokio::test]
async fn test_acknowledge_outbox_sets_acknowledged_at() -> Result<(), DurableError> {
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
    mark_outbox_dispatched(&pool, "outbox1").await.unwrap();

    let result = acknowledge_outbox(&pool, "outbox1").await;
    let record = result.expect("acknowledge_outbox should succeed");
    assert_eq!(record.status, OutboxStatus::Acknowledged);
    assert!(record.acknowledged_at.is_some());

    Ok(())
}

#[tokio::test]
async fn test_mark_outbox_failed_increments_retry_count() -> Result<(), DurableError> {
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
    let result = mark_outbox_failed(&pool, "outbox1", "timeout".to_string()).await;
    let record = result.expect("mark_outbox_failed should succeed");
    assert_eq!(record.status, OutboxStatus::Failed);
    assert_eq!(record.retry_count, 1);
    assert_eq!(record.last_error, Some("timeout".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_get_pending_outbox_returns_pending_and_failed_entries() -> Result<(), DurableError> {
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
    add_outbox_entry(
        &pool,
        "out2".to_string(),
        SideEffectType::Notify,
        "p2".to_string(),
        1,
        3,
        timestamp,
    )
    .await
    .unwrap();
    add_outbox_entry(
        &pool,
        "out3".to_string(),
        SideEffectType::Notify,
        "p3".to_string(),
        1,
        3,
        timestamp,
    )
    .await
    .unwrap();

    mark_outbox_dispatched(&pool, "out1").await.unwrap();
    acknowledge_outbox(&pool, "out1").await.unwrap();
    mark_outbox_dispatched(&pool, "out2").await.unwrap();

    let result = get_pending_outbox(&pool, 10).await;
    let entries = result.expect("get_pending_outbox should succeed");
    assert_eq!(entries.len(), 2);

    Ok(())
}
