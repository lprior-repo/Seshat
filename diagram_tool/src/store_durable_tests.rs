//! Durable Store Tests (lewis-w9d)
//!
//! This module provides comprehensive tests for the durable store implementation,
//! following the Martin Fowler Given-When-Then test plan from martin-fowler-tests.md.
//!
//! Tests cover:
//! - Operation tracking (start, get, update, list by state)
//! - Step journal (record, get, update status, pending, skip)
//! - Outbox (add, get, dispatch, acknowledge, fail, pending)
//! - Conflict diff generation
//! - Cursor-based pagination
//! - Workflow helpers (can_resume, get_next_step)
//! - Error paths and edge cases
//! - Contract verification

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use sqlx::SqlitePool;
use tempfile::TempDir;

use crate::store::types::{
    ConflictDiff, DiffDomainOp, EventCursor, EventPage, EventRecord, OperationRecord,
    OperationState, OutboxRecord, OutboxStatus, SideEffectType, StepRecord, StepStatus,
};
use crate::store_durable::{
    add_outbox_entry, acknowledge_outbox, can_resume_operation, fetch_events_cursor,
    generate_conflict_diff, get_next_step, get_operation, get_operations_by_state,
    get_outbox_entry, get_pending_outbox, get_pending_steps, get_step, mark_outbox_dispatched,
    mark_outbox_failed, parse_cursor, record_step, serialize_cursor, skip_step, start_operation,
    update_operation_state, update_step_status, bootstrap_durable_store, run_durable_migration,
    DurableConfig, DurableError,
};

// =============================================================================
// Test Fixtures and Helpers
// =============================================================================

/// Creates a test database pool with migrations run
async fn create_test_pool() -> Result<(SqlitePool, TempDir), DurableError> {
    let temp_dir = TempDir::new()
        .map_err(|e| DurableError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
    let db_path = temp_dir.path().join("test.db");

    let bootstrap = bootstrap_durable_store(&db_path, DurableConfig::default())
        .await?;

    Ok((bootstrap.pool, temp_dir))
}

/// Creates a test pool and ensures the events table exists for cursor tests
async fn create_test_pool_with_events() -> Result<(SqlitePool, TempDir), DurableError> {
    let (pool, temp_dir) = create_test_pool().await?;

    // Create events table if it doesn't exist (for cursor tests)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS events (
            operation_id TEXT NOT NULL,
            revision INTEGER NOT NULL,
            timestamp TEXT NOT NULL,
            payload TEXT NOT NULL,
            PRIMARY KEY (operation_id, revision)
        )",
    )
    .execute(&pool)
    .await
    .map_err(DurableError::Sqlx)?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_revision ON events(revision)")
        .execute(&pool)
        .await
        .map_err(DurableError::Sqlx)?;

    Ok((pool, temp_dir))
}

// =============================================================================
// Happy Path Tests - Operation Tracking
// =============================================================================

mod operation_tracking {
    use super::*;

    #[tokio::test]
    async fn test_start_operation_creates_record_with_started_state() -> Result<(), DurableError> {
        // Given: Valid operation_id, total_steps=3, author_id, description, timestamp
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        // When: start_operation is called
        let result = start_operation(
            &pool,
            "op1".to_string(),
            3,
            "user1".to_string(),
            "Process data".to_string(),
            timestamp,
        )
        .await;

        // Then: Returns OperationRecord with state=Started, current_step=0, total_steps=3
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
        // Given: An operation exists in database with known values
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

        // When: get_operation is called with that operation_id
        let retrieved = get_operation(&pool, "op1").await;

        // Then: Returns exact OperationRecord matching stored data
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
        // Given: An operation in Started state exists
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

        // When: update_operation_state is called with new_state=InProgress
        let result = update_operation_state(&pool, "op1", OperationState::InProgress, None, None, None)
            .await;

        // Then: Returns OperationRecord with state=InProgress
        let record = result.expect("update_operation_state should succeed");
        assert_eq!(record.state, OperationState::InProgress);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_operation_state_to_completed_sets_completed_at() -> Result<(), DurableError> {
        // Given: An operation in InProgress state exists
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

        // When: update_operation_state is called with new_state=Completed, final_revision=10
        let result = update_operation_state(
            &pool,
            "op1",
            OperationState::Completed,
            Some(3),
            Some(10),
            None,
        )
        .await;

        // Then: Returns OperationRecord with state=Completed and completed_at is Some(timestamp)
        let record = result.expect("update to Completed should succeed");
        assert_eq!(record.state, OperationState::Completed);
        assert!(record.completed_at.is_some());
        assert_eq!(record.final_revision, Some(10));
        assert_eq!(record.current_step, 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_operations_by_state_returns_matching_operations() -> Result<(), DurableError> {
        // Given: Multiple operations in different states exist
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 2, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start op1");
        start_operation(&pool, "op2".to_string(), 2, "u2".to_string(), "d2".to_string(), timestamp)
            .await
            .expect("start op2");
        start_operation(&pool, "op3".to_string(), 2, "u3".to_string(), "d3".to_string(), timestamp)
            .await
            .expect("start op3");

        update_operation_state(&pool, "op1", OperationState::InProgress, None, None, None)
            .await
            .expect("update op1 to InProgress");

        // When: get_operations_by_state is called with state=InProgress
        let result = get_operations_by_state(&pool, OperationState::InProgress).await;

        // Then: Returns only operations with state=InProgress
        let operations = result.expect("get_operations_by_state should succeed");
        assert_eq!(operations.len(), 1);
        assert_eq!(operations[0].operation_id, "op1");

        Ok(())
    }
}

// =============================================================================
// Happy Path Tests - Step Journal
// =============================================================================

mod step_journal {
    use super::*;

    #[tokio::test]
    async fn test_record_step_creates_pending_step() -> Result<(), DurableError> {
        // Given: An operation exists
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

        // When: record_step is called with step_index=0, step_name="analyze"
        let result = record_step(&pool, "op1".to_string(), 0, "analyze".to_string(), timestamp).await;

        // Then: Returns StepRecord with status=Pending, step_index=0
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
        // Given: A step exists in the journal
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 2, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start operation");

        let created = record_step(&pool, "op1".to_string(), 0, "step1".to_string(), timestamp)
            .await
            .expect("record step");

        // When: get_step is called with operation_id and step_index
        let retrieved = get_step(&pool, "op1", 0).await;

        // Then: Returns exact StepRecord
        let record = retrieved.expect("get_step should succeed");
        assert_eq!(record.operation_id, created.operation_id);
        assert_eq!(record.step_index, created.step_index);
        assert_eq!(record.step_name, created.step_name);
        assert_eq!(record.status, created.status);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_step_status_to_running_sets_started_at() -> Result<(), DurableError> {
        // Given: A step with status=Pending exists
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 2, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start operation");
        record_step(&pool, "op1".to_string(), 0, "step1".to_string(), timestamp)
            .await
            .expect("record step");

        // When: update_step_status is called with new_status=Running
        let result = update_step_status(&pool, "op1", 0, StepStatus::Running, None, None).await;

        // Then: Returns StepRecord with status=Running and started_at is Some(timestamp)
        let record = result.expect("update_step_status should succeed");
        assert_eq!(record.status, StepStatus::Running);
        assert!(record.started_at.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_update_step_status_to_completed_sets_completed_at() -> Result<(), DurableError> {
        // Given: A step with status=Running exists
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 2, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start operation");
        record_step(&pool, "op1".to_string(), 0, "step1".to_string(), timestamp)
            .await
            .expect("record step");
        update_step_status(&pool, "op1", 0, StepStatus::Running, None, None)
            .await
            .expect("set to running");

        // When: update_step_status is called with new_status=Completed, event_revision=5
        let result = update_step_status(&pool, "op1", 0, StepStatus::Completed, Some(5), None).await;

        // Then: Returns StepRecord with status=Completed, completed_at is Some(timestamp), event_revision=5
        let record = result.expect("update_step_status should succeed");
        assert_eq!(record.status, StepStatus::Completed);
        assert!(record.completed_at.is_some());
        assert_eq!(record.event_revision, Some(5));

        Ok(())
    }

    #[tokio::test]
    async fn test_get_pending_steps_returns_pending_and_failed_steps() -> Result<(), DurableError> {
        // Given: An operation with steps in various statuses
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 3, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start");

        record_step(&pool, "op1".to_string(), 0, "step0".to_string(), timestamp)
            .await
            .expect("step 0");
        record_step(&pool, "op1".to_string(), 1, "step1".to_string(), timestamp)
            .await
            .expect("step 1");
        record_step(&pool, "op1".to_string(), 2, "step2".to_string(), timestamp)
            .await
            .expect("step 2");

        // Set step 0 to completed
        update_step_status(&pool, "op1", 0, StepStatus::Running, None, None)
            .await
            .expect("running");
        update_step_status(&pool, "op1", 0, StepStatus::Completed, None, None)
            .await
            .expect("completed");

        // Set step 1 to failed
        update_step_status(&pool, "op1", 1, StepStatus::Running, None, None)
            .await
            .expect("running");
        update_step_status(&pool, "op1", 1, StepStatus::Failed, Some(1), Some("error".to_string()))
            .await
            .expect("failed");

        // When: get_pending_steps is called
        let result = get_pending_steps(&pool, "op1").await;

        // Then: Returns only steps with status in (pending, running, failed), ordered by step_index
        let steps = result.expect("get_pending_steps should succeed");
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].step_index, 1); // failed
        assert_eq!(steps[1].step_index, 2); // pending

        Ok(())
    }

    #[tokio::test]
    async fn test_skip_step_marks_step_as_skipped() -> Result<(), DurableError> {
        // Given: A step with status=Pending exists
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 2, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start");
        record_step(&pool, "op1".to_string(), 0, "step1".to_string(), timestamp)
            .await
            .expect("record step");

        // When: skip_step is called
        let result = skip_step(&pool, "op1", 0).await;

        // Then: Returns StepRecord with status=Skipped
        let record = result.expect("skip_step should succeed");
        assert_eq!(record.status, StepStatus::Skipped);
        assert!(record.completed_at.is_some());

        Ok(())
    }
}

// =============================================================================
// Happy Path Tests - Outbox
// =============================================================================

mod outbox {
    use super::*;

    #[tokio::test]
    async fn test_add_outbox_entry_creates_pending_entry() -> Result<(), DurableError> {
        // Given: Valid outbox parameters
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        // When: add_outbox_entry is called
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

        // Then: Returns OutboxRecord with status=Pending, retry_count=0
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
        // Given: An outbox entry exists
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
        .expect("add outbox entry");

        // When: get_outbox_entry is called
        let retrieved = get_outbox_entry(&pool, "outbox1").await;

        // Then: Returns exact OutboxRecord
        let record = retrieved.expect("get_outbox_entry should succeed");
        assert_eq!(record.id, created.id);
        assert_eq!(record.side_effect_type, created.side_effect_type);
        assert_eq!(record.status, created.status);
        assert_eq!(record.retry_count, created.retry_count);

        Ok(())
    }

    #[tokio::test]
    async fn test_mark_outbox_dispatched_sets_dispatched_at() -> Result<(), DurableError> {
        // Given: An outbox entry with status=Pending exists
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
        .expect("add entry");

        // When: mark_outbox_dispatched is called
        let result = mark_outbox_dispatched(&pool, "outbox1").await;

        // Then: Returns OutboxRecord with status=Dispatched and dispatched_at is Some(timestamp)
        let record = result.expect("mark_outbox_dispatched should succeed");
        assert_eq!(record.status, OutboxStatus::Dispatched);
        assert!(record.dispatched_at.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_acknowledge_outbox_sets_acknowledged_at() -> Result<(), DurableError> {
        // Given: An outbox entry with status=Dispatched exists
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
        .expect("add");
        mark_outbox_dispatched(&pool, "outbox1").await.expect("dispatch");

        // When: acknowledge_outbox is called
        let result = acknowledge_outbox(&pool, "outbox1").await;

        // Then: Returns OutboxRecord with status=Acknowledged and acknowledged_at is Some(timestamp)
        let record = result.expect("acknowledge_outbox should succeed");
        assert_eq!(record.status, OutboxStatus::Acknowledged);
        assert!(record.acknowledged_at.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_mark_outbox_failed_increments_retry_count() -> Result<(), DurableError> {
        // Given: An outbox entry with status=Pending, retry_count=0, max_retries=3
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
        .expect("add");

        // When: mark_outbox_failed is called with error_message="timeout"
        let result = mark_outbox_failed(&pool, "outbox1", "timeout".to_string()).await;

        // Then: Returns OutboxRecord with status=Failed, retry_count=1, last_error="timeout"
        let record = result.expect("mark_outbox_failed should succeed");
        assert_eq!(record.status, OutboxStatus::Failed);
        assert_eq!(record.retry_count, 1);
        assert_eq!(record.last_error, Some("timeout".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_get_pending_outbox_returns_pending_and_failed_entries() -> Result<(), DurableError> {
        // Given: Multiple outbox entries in various states
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        add_outbox_entry(&pool, "out1".to_string(), SideEffectType::Notify, "p1".to_string(), 1, 3, timestamp)
            .await
            .expect("add 1");
        add_outbox_entry(&pool, "out2".to_string(), SideEffectType::Notify, "p2".to_string(), 1, 3, timestamp)
            .await
            .expect("add 2");
        add_outbox_entry(&pool, "out3".to_string(), SideEffectType::Notify, "p3".to_string(), 1, 3, timestamp)
            .await
            .expect("add 3");

        mark_outbox_dispatched(&pool, "out1").await.expect("dispatch");
        acknowledge_outbox(&pool, "out1").await.expect("ack");
        mark_outbox_dispatched(&pool, "out2").await.expect("dispatch 2");

        // When: get_pending_outbox is called with limit=10
        let result = get_pending_outbox(&pool, 10).await;

        // Then: Returns entries with status in (Pending, Failed), ordered by created_at, max 10
        let entries = result.expect("get_pending_outbox should succeed");
        assert_eq!(entries.len(), 2);

        Ok(())
    }
}

// =============================================================================
// Happy Path Tests - Conflict Diff
// =============================================================================

mod conflict_diff {
    use super::*;

    #[tokio::test]
    async fn test_generate_conflict_diff_returns_current_revision() -> Result<(), DurableError> {
        // Given: Database with events up to revision 5
        let (pool, _temp_dir) = create_test_pool_with_events().await?;

        // Insert events directly
        sqlx::query("INSERT INTO events (operation_id, revision, timestamp, payload) VALUES ('op1', 1, '1700000000', '{}')")
            .execute(&pool).await.map_err(DurableError::Sqlx)?;
        sqlx::query("INSERT INTO events (operation_id, revision, timestamp, payload) VALUES ('op1', 2, '1700000001', '{}')")
            .execute(&pool).await.map_err(DurableError::Sqlx)?;
        sqlx::query("INSERT INTO events (operation_id, revision, timestamp, payload) VALUES ('op1', 3, '1700000002', '{}')")
            .execute(&pool).await.map_err(DurableError::Sqlx)?;
        sqlx::query("INSERT INTO events (operation_id, revision, timestamp, payload) VALUES ('op1', 4, '1700000003', '{}')")
            .execute(&pool).await.map_err(DurableError::Sqlx)?;
        sqlx::query("INSERT INTO events (operation_id, revision, timestamp, payload) VALUES ('op1', 5, '1700000004', '{}')")
            .execute(&pool).await.map_err(DurableError::Sqlx)?;

        // When: generate_conflict_diff is called with assumed_revision=3
        let result = generate_conflict_diff(&pool, 3).await;

        // Then: Returns ConflictDiff with assumed_revision=3, actual_revision=5
        let diff = result.expect("generate_conflict_diff should succeed");
        assert_eq!(diff.assumed_revision, 3);
        assert_eq!(diff.actual_revision, 5);
        assert!(!diff.changes.is_empty());

        Ok(())
    }
}

// =============================================================================
// Happy Path Tests - Cursor Pagination
// =============================================================================

mod cursor_pagination {
    use super::*;

    #[tokio::test]
    async fn test_fetch_events_cursor_returns_first_page() -> Result<(), DurableError> {
        // Given: Database with 50 events
        let (pool, _temp_dir) = create_test_pool_with_events().await?;

        for i in 1..=50 {
            sqlx::query("INSERT INTO events (operation_id, revision, timestamp, payload) VALUES ('op1', ?1, '1700000000', ?2)")
                .bind(i)
                .bind(format!(r#"{{"event":{}}}"#, i))
                .execute(&pool).await.map_err(DurableError::Sqlx)?;
        }

        // When: fetch_events_cursor is called with cursor=EventCursor::first(10)
        let cursor = EventCursor::first(10);
        let result = fetch_events_cursor(&pool, cursor).await;

        // Then: Returns EventPage with 10 events, has_more=true, next_cursor is Some
        let page = result.expect("fetch_events_cursor should succeed");
        assert_eq!(page.events.len(), 10);
        assert!(page.has_more);
        assert!(page.next_cursor.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_fetch_events_cursor_respects_limit_cap() -> Result<(), DurableError> {
        // Given: Database with events
        let (pool, _temp_dir) = create_test_pool_with_events().await?;

        for i in 1..=10 {
            sqlx::query("INSERT INTO events (operation_id, revision, timestamp, payload) VALUES ('op1', ?1, '1700000000', '{}')")
                .bind(i)
                .execute(&pool).await.map_err(DurableError::Sqlx)?;
        }

        // When: fetch_events_cursor is called with limit=5000
        let cursor = EventCursor::new(0, 5000);
        let result = fetch_events_cursor(&pool, cursor).await;

        // Then: Returns at most 1000 events (capped)
        let page = result.expect("fetch_events_cursor should succeed");
        assert!(page.events.len() <= 1000);

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_cursor_valid_format() {
        // Given: Cursor string "10:50"
        let cursor_str = "10:50";

        // When: parse_cursor is called
        let result = parse_cursor(cursor_str);

        // Then: Returns EventCursor with revision=10, limit=50
        let cursor = result.expect("parse_cursor should succeed");
        assert_eq!(cursor.revision, 10);
        assert_eq!(cursor.limit, 50);
    }

    #[tokio::test]
    async fn test_serialize_cursor_produces_correct_format() {
        // Given: EventCursor with revision=10, limit=50
        let cursor = EventCursor::new(10, 50);

        // When: serialize_cursor is called
        let result = serialize_cursor(&cursor);

        // Then: Returns "10:50"
        assert_eq!(result, "10:50");
    }

    #[tokio::test]
    async fn test_fetch_events_cursor_at_end_returns_no_next_cursor() -> Result<(), DurableError> {
        // Given: Database with fewer events than limit
        let (pool, _temp_dir) = create_test_pool_with_events().await?;

        for i in 1..=5 {
            sqlx::query("INSERT INTO events (operation_id, revision, timestamp, payload) VALUES ('op1', ?1, '1700000000', '{}')")
                .bind(i)
                .execute(&pool).await.map_err(DurableError::Sqlx)?;
        }

        // When: fetch_events_cursor is called with cursor covering all events
        let cursor = EventCursor::new(0, 10);
        let result = fetch_events_cursor(&pool, cursor).await;

        // Then: Returns EventPage with has_more=false, next_cursor=None
        let page = result.expect("fetch_events_cursor should succeed");
        assert!(!page.has_more);
        assert!(page.next_cursor.is_none());

        Ok(())
    }
}

// =============================================================================
// Happy Path Tests - Workflow Helpers
// =============================================================================

mod workflow_helpers {
    use super::*;

    #[tokio::test]
    async fn test_can_resume_operation_true_when_in_progress_with_pending_steps(
    ) -> Result<(), DurableError> {
        // Given: Operation in InProgress state with pending steps
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 2, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start");
        update_operation_state(&pool, "op1", OperationState::InProgress, None, None, None)
            .await
            .expect("in progress");
        record_step(&pool, "op1".to_string(), 0, "step0".to_string(), timestamp)
            .await
            .expect("step 0");
        record_step(&pool, "op1".to_string(), 1, "step1".to_string(), timestamp)
            .await
            .expect("step 1");

        // When: can_resume_operation is called
        let result = can_resume_operation(&pool, "op1").await;

        // Then: Returns true
        let can_resume = result.expect("can_resume_operation should succeed");
        assert!(can_resume);

        Ok(())
    }

    #[tokio::test]
    async fn test_can_resume_operation_false_when_completed() -> Result<(), DurableError> {
        // Given: Operation in Completed state
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 2, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start");
        update_operation_state(&pool, "op1", OperationState::InProgress, None, None, None)
            .await
            .expect("in progress");
        update_operation_state(&pool, "op1", OperationState::Completed, None, None, None)
            .await
            .expect("completed");

        // When: can_resume_operation is called
        let result = can_resume_operation(&pool, "op1").await;

        // Then: Returns false
        let can_resume = result.expect("can_resume_operation should succeed");
        assert!(!can_resume);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_next_step_returns_first_pending_step() -> Result<(), DurableError> {
        // Given: Operation with steps at indices 0=completed, 1=pending, 2=pending
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 3, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start");
        record_step(&pool, "op1".to_string(), 0, "step0".to_string(), timestamp)
            .await
            .expect("step 0");
        record_step(&pool, "op1".to_string(), 1, "step1".to_string(), timestamp)
            .await
            .expect("step 1");
        record_step(&pool, "op1".to_string(), 2, "step2".to_string(), timestamp)
            .await
            .expect("step 2");

        // Set step 0 to completed
        update_step_status(&pool, "op1", 0, StepStatus::Running, None, None)
            .await
            .expect("running");
        update_step_status(&pool, "op1", 0, StepStatus::Completed, None, None)
            .await
            .expect("completed");

        // When: get_next_step is called
        let result = get_next_step(&pool, "op1").await;

        // Then: Returns Some(StepRecord) with step_index=1
        let step = result.expect("get_next_step should succeed");
        assert!(step.is_some());
        assert_eq!(step.unwrap().step_index, 1);

        Ok(())
    }
}

// =============================================================================
// Error Path Tests
// =============================================================================

mod error_paths {
    use super::*;

    #[tokio::test]
    async fn test_get_operation_not_found_returns_error() -> Result<(), DurableError> {
        // Given: No operation with id "nonexistent"
        let (pool, _temp_dir) = create_test_pool().await?;

        // When: get_operation(pool, "nonexistent") is called
        let result = get_operation(&pool, "nonexistent").await;

        // Then: Returns Err(DurableError::OperationNotFound("nonexistent".to_string()))
        assert!(result.is_err());
        if let Err(DurableError::OperationNotFound(id)) = result {
            assert_eq!(id, "nonexistent");
        } else {
            panic!("Expected OperationNotFound error");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_update_operation_state_invalid_transition_returns_error() -> Result<(), DurableError> {
        // Given: Operation in Started state
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 2, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start");

        // When: update_operation_state is called with new_state=Completed (invalid from Started)
        let result = update_operation_state(&pool, "op1", OperationState::Completed, None, None, None)
            .await;

        // Then: Returns Err(DurableError::OperationStateInvalid { expected: Completed, found: Started })
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
        // Given: No step with operation_id="op1", step_index=99
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 2, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start");

        // When: get_step(pool, "op1", 99) is called
        let result = get_step(&pool, "op1", 99).await;

        // Then: Returns Err(DurableError::StepNotFound { operation_id: "op1", step_index: 99 })
        assert!(result.is_err());
        if let Err(DurableError::StepNotFound { operation_id, step_index }) = result {
            assert_eq!(operation_id, "op1");
            assert_eq!(step_index, 99);
        } else {
            panic!("Expected StepNotFound error");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_update_step_status_already_completed_returns_error() -> Result<(), DurableError> {
        // Given: Step with status=Completed
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 2, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start");
        record_step(&pool, "op1".to_string(), 0, "step0".to_string(), timestamp)
            .await
            .expect("record");
        update_step_status(&pool, "op1", 0, StepStatus::Running, None, None)
            .await
            .expect("running");
        update_step_status(&pool, "op1", 0, StepStatus::Completed, None, None)
            .await
            .expect("completed");

        // When: update_step_status is called with new_status=Failed
        let result = update_step_status(&pool, "op1", 0, StepStatus::Failed, None, Some("error".to_string()))
            .await;

        // Then: Returns Err(DurableError::StepAlreadyCompleted { operation_id, step_index })
        assert!(result.is_err());
        if let Err(DurableError::StepAlreadyCompleted { operation_id, step_index }) = result {
            assert_eq!(operation_id, "op1");
            assert_eq!(step_index, 0);
        } else {
            panic!("Expected StepAlreadyCompleted error");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_mark_outbox_failed_max_retries_exceeded_returns_error() -> Result<(), DurableError> {
        // Given: Outbox entry with retry_count=3, max_retries=3
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
        .expect("add");

        // Fail 3 times
        for _ in 0..3 {
            mark_outbox_failed(&pool, "outbox1", "error".to_string()).await?;
        }

        // When: mark_outbox_failed is called
        let result = mark_outbox_failed(&pool, "outbox1", "error".to_string()).await;

        // Then: Returns Err(DurableError::OutboxMaxRetriesExceeded(id))
        assert!(result.is_err());
        if let Err(DurableError::OutboxMaxRetriesExceeded(id)) = result {
            assert_eq!(id, "outbox1");
        } else {
            panic!("Expected OutboxMaxRetriesExceeded error");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_cursor_invalid_format_returns_error() {
        // Given: Cursor string "invalid"
        let cursor_str = "invalid";

        // When: parse_cursor("invalid") is called
        let result = parse_cursor(cursor_str);

        // Then: Returns Err(DurableError::CursorParseError("Expected format: revision:limit".to_string()))
        assert!(result.is_err());
        if let Err(DurableError::CursorParseError(msg)) = result {
            assert!(msg.contains("Expected format"));
        } else {
            panic!("Expected CursorParseError");
        }
    }

    #[tokio::test]
    async fn test_parse_cursor_invalid_revision_returns_error() {
        // Given: Cursor string "abc:10"
        let cursor_str = "abc:10";

        // When: parse_cursor("abc:10") is called
        let result = parse_cursor(cursor_str);

        // Then: Returns Err(DurableError::CursorParseError("Invalid revision".to_string()))
        assert!(result.is_err());
        if let Err(DurableError::CursorParseError(msg)) = result {
            assert!(msg.contains("revision"));
        } else {
            panic!("Expected CursorParseError");
        }
    }
}

// =============================================================================
// Edge Case Tests
// =============================================================================

mod edge_cases {
    use super::*;

    #[tokio::test]
    async fn test_start_operation_with_single_step() -> Result<(), DurableError> {
        // Given: total_steps=1
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        // When: start_operation is called
        let result = start_operation(
            &pool,
            "op1".to_string(),
            1,
            "user1".to_string(),
            "Single step".to_string(),
            timestamp,
        )
        .await;

        // Then: OperationRecord created with total_steps=1
        let record = result.expect("start_operation should succeed");
        assert_eq!(record.total_steps, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_operation_state_idempotent_completed() -> Result<(), DurableError> {
        // Given: Operation already in Completed state
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 2, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start");
        update_operation_state(&pool, "op1", OperationState::InProgress, None, None, None)
            .await
            .expect("in progress");
        update_operation_state(&pool, "op1", OperationState::Completed, None, None, None)
            .await
            .expect("completed");

        // When: update_operation_state is called with new_state=Completed
        let result = update_operation_state(&pool, "op1", OperationState::Completed, None, None, None)
            .await;

        // Then: Returns OperationRecord with state=Completed (idempotent)
        let record = result.expect("update_operation_state should succeed");
        assert_eq!(record.state, OperationState::Completed);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_step_status_idempotent_running() -> Result<(), DurableError> {
        // Given: Step already in Running state
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 2, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start");
        record_step(&pool, "op1".to_string(), 0, "step0".to_string(), timestamp)
            .await
            .expect("record");
        update_step_status(&pool, "op1", 0, StepStatus::Running, None, None)
            .await
            .expect("running");

        // When: update_step_status is called with new_status=Running
        let result = update_step_status(&pool, "op1", 0, StepStatus::Running, None, None).await;

        // Then: Returns StepRecord with status=Running (idempotent)
        let record = result.expect("update_step_status should succeed");
        assert_eq!(record.status, StepStatus::Running);

        Ok(())
    }

    #[tokio::test]
    async fn test_mark_outbox_failed_multiple_times_increments_retry() -> Result<(), DurableError> {
        // Given: Outbox entry with retry_count=0, max_retries=3
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
        .expect("add");

        // When: mark_outbox_failed called 3 times
        mark_outbox_failed(&pool, "outbox1", "error1".to_string()).await?;
        mark_outbox_failed(&pool, "outbox1", "error2".to_string()).await?;
        let result = mark_outbox_failed(&pool, "outbox1", "error3".to_string()).await;

        // Then: On 3rd call returns Err(OutboxMaxRetriesExceeded)
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_fetch_events_cursor_with_zero_events() -> Result<(), DurableError> {
        // Given: Empty events table
        let (pool, _temp_dir) = create_test_pool_with_events().await?;

        // When: fetch_events_cursor is called
        let cursor = EventCursor::first(10);
        let result = fetch_events_cursor(&pool, cursor).await;

        // Then: Returns EventPage with empty events, has_more=false
        let page = result.expect("fetch_events_cursor should succeed");
        assert!(page.events.is_empty());
        assert!(!page.has_more);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_pending_outbox_with_zero_limit() -> Result<(), DurableError> {
        // Given: Database with outbox entries
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        add_outbox_entry(&pool, "out1".to_string(), SideEffectType::Notify, "p1".to_string(), 1, 3, timestamp)
            .await
            .expect("add");

        // When: get_pending_outbox is called with limit=0
        let result = get_pending_outbox(&pool, 0).await;

        // Then: Returns empty Vec (limit=0 means no results)
        let entries = result.expect("get_pending_outbox should succeed");
        assert!(entries.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_conflict_diff_with_no_new_events() -> Result<(), DurableError> {
        // Given: Database at revision 5, caller assumes revision 5
        let (pool, _temp_dir) = create_test_pool_with_events().await?;

        for i in 1..=5 {
            sqlx::query("INSERT INTO events (operation_id, revision, timestamp, payload) VALUES ('op1', ?1, '1700000000', '{}')")
                .bind(i)
                .execute(&pool).await.map_err(DurableError::Sqlx)?;
        }

        // When: generate_conflict_diff with assumed_revision=5
        let result = generate_conflict_diff(&pool, 5).await;

        // Then: Returns ConflictDiff with empty changes list
        let diff = result.expect("generate_conflict_diff should succeed");
        assert!(diff.changes.is_empty());
        assert_eq!(diff.actual_revision, 5);

        Ok(())
    }
}

// =============================================================================
// Contract Verification Tests
// =============================================================================

mod contract_verification {
    use super::*;

    #[tokio::test]
    async fn test_precondition_operation_id_non_empty() -> Result<(), DurableError> {
        // Given: Empty operation_id
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        // When: start_operation is called
        let result = start_operation(
            &pool,
            "".to_string(),
            1,
            "author".to_string(),
            "desc".to_string(),
            timestamp,
        )
        .await;

        // Then: Returns Err(ValidationFailed) - precondition enforced
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_precondition_total_steps_at_least_one() -> Result<(), DurableError> {
        // Given: total_steps=0
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        // When: start_operation is called
        let result = start_operation(
            &pool,
            "op1".to_string(),
            0,
            "author".to_string(),
            "desc".to_string(),
            timestamp,
        )
        .await;

        // Then: Returns Err(ValidationFailed) - precondition enforced
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_precondition_author_id_non_empty() -> Result<(), DurableError> {
        // Given: author_id=""
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        // When: start_operation is called
        let result = start_operation(
            &pool,
            "op1".to_string(),
            1,
            "".to_string(),
            "description".to_string(),
            timestamp,
        )
        .await;

        // Then: Returns Err(ValidationFailed) - precondition enforced
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_precondition_description_non_empty() -> Result<(), DurableError> {
        // Given: description=""
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        // When: start_operation is called
        let result = start_operation(
            &pool,
            "op1".to_string(),
            1,
            "author".to_string(),
            "".to_string(),
            timestamp,
        )
        .await;

        // Then: Returns Err(ValidationFailed) - precondition enforced
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_precondition_valid_state_transition() -> Result<(), DurableError> {
        // Given: Operation in Started state
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 2, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start");

        // When: update_operation_state with Completed
        let result = update_operation_state(&pool, "op1", OperationState::Completed, None, None, None)
            .await;

        // Then: Returns Err(OperationStateInvalid) - precondition enforced
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_precondition_step_index_unique() -> Result<(), DurableError> {
        // Given: Step with step_index=0 already exists
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 2, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start");
        record_step(&pool, "op1".to_string(), 0, "step0".to_string(), timestamp)
            .await
            .expect("record step 0");

        // When: record_step with step_index=0
        let result = record_step(&pool, "op1".to_string(), 0, "duplicate".to_string(), timestamp).await;

        // Then: Returns Err - duplicate key constraint violation
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_precondition_get_step_exists() -> Result<(), DurableError> {
        // Given: No step with operation_id and step_index
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 2, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start");

        // When: get_step is called
        let result = get_step(&pool, "op1", 0).await;

        // Then: Returns Err(StepNotFound) - precondition enforced
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_precondition_outbox_id_non_empty() -> Result<(), DurableError> {
        // Given: outbox id=""
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        // When: add_outbox_entry is called
        let result = add_outbox_entry(
            &pool,
            "".to_string(),
            SideEffectType::Webhook,
            "payload".to_string(),
            1,
            3,
            timestamp,
        )
        .await;

        // Then: Returns Err(ValidationFailed) - precondition enforced
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_precondition_outbox_payload_non_empty() -> Result<(), DurableError> {
        // Given: payload=""
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        // When: add_outbox_entry is called
        let result = add_outbox_entry(
            &pool,
            "outbox1".to_string(),
            SideEffectType::Webhook,
            "".to_string(),
            1,
            3,
            timestamp,
        )
        .await;

        // Then: Returns Err(ValidationFailed) - precondition enforced
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_postcondition_completed_at_set_on_terminal_state() -> Result<(), DurableError> {
        // Given: Operation in InProgress state
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 2, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start");
        update_operation_state(&pool, "op1", OperationState::InProgress, None, None, None)
            .await
            .expect("in progress");

        // When: update_operation_state with Completed
        let result = update_operation_state(&pool, "op1", OperationState::Completed, None, None, None)
            .await;

        // Then: Returned record has completed_at=Some(_) - postcondition verified
        let record = result.expect("update should succeed");
        assert!(record.completed_at.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_postcondition_started_at_set_on_running_step() -> Result<(), DurableError> {
        // Given: Step in Pending state
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 2, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start");
        record_step(&pool, "op1".to_string(), 0, "step0".to_string(), timestamp)
            .await
            .expect("record");

        // When: update_step_status with Running
        let result = update_step_status(&pool, "op1", 0, StepStatus::Running, None, None).await;

        // Then: Returned record has started_at=Some(_) - postcondition verified
        let record = result.expect("update should succeed");
        assert!(record.started_at.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_invariant_operation_state_valid() -> Result<(), DurableError> {
        // Given: Any OperationRecord
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        // When: Operation is created and states change
        let record = start_operation(
            &pool,
            "op1".to_string(),
            2,
            "u1".to_string(),
            "d1".to_string(),
            timestamp,
        )
        .await
        .expect("start");

        // Then: state is always one of Started, InProgress, Completed, Failed
        match record.state {
            OperationState::Started | OperationState::InProgress | OperationState::Completed | OperationState::Failed => {}
            _ => panic!("Invalid operation state"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_invariant_step_status_valid() -> Result<(), DurableError> {
        // Given: Any StepRecord
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 2, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start");

        // When: Step is recorded
        let record = record_step(&pool, "op1".to_string(), 0, "step0".to_string(), timestamp)
            .await
            .expect("record");

        // Then: status is always one of Pending, Running, Completed, Failed, Skipped
        match record.status {
            StepStatus::Pending | StepStatus::Running | StepStatus::Completed | StepStatus::Failed | StepStatus::Skipped => {}
            _ => panic!("Invalid step status"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_invariant_outbox_status_valid() -> Result<(), DurableError> {
        // Given: Any OutboxRecord
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        // When: Outbox entry is added
        let record = add_outbox_entry(
            &pool,
            "outbox1".to_string(),
            SideEffectType::Webhook,
            "payload".to_string(),
            1,
            3,
            timestamp,
        )
        .await
        .expect("add");

        // Then: status is always one of Pending, Dispatched, Acknowledged, Failed
        match record.status {
            OutboxStatus::Pending | OutboxStatus::Dispatched | OutboxStatus::Acknowledged | OutboxStatus::Failed => {}
            _ => panic!("Invalid outbox status"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_invariant_current_step_lte_total_steps() -> Result<(), DurableError> {
        // Given: Any OperationRecord
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        // When: Operation with total_steps=3 is created
        let record = start_operation(
            &pool,
            "op1".to_string(),
            3,
            "u1".to_string(),
            "d1".to_string(),
            timestamp,
        )
        .await
        .expect("start");

        // Then: current_step <= total_steps
        assert!(record.current_step <= record.total_steps);

        Ok(())
    }
}

// =============================================================================
// Contract Violation Tests
// =============================================================================

mod contract_violations {
    use super::*;

    #[tokio::test]
    async fn test_p1_violation_empty_operation_id_returns_validation_error() -> Result<(), DurableError> {
        // Given: operation_id=""
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        // When: start_operation(pool, "".to_string(), 1, "author".to_string(), "desc".to_string(), 123)
        let result = start_operation(
            &pool,
            "".to_string(),
            1,
            "author".to_string(),
            "desc".to_string(),
            timestamp,
        )
        .await;

        // Then: Returns Err(DurableError::ValidationFailed(_))
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_p1_violation_zero_total_steps_returns_validation_error() -> Result<(), DurableError> {
        // Given: total_steps=0
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        // When: start_operation(pool, "op1".to_string(), 0, "author".to_string(), "desc".to_string(), 123)
        let result = start_operation(
            &pool,
            "op1".to_string(),
            0,
            "author".to_string(),
            "desc".to_string(),
            timestamp,
        )
        .await;

        // Then: Returns Err(DurableError::ValidationFailed(_))
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_p2_violation_nonexistent_operation_returns_not_found() -> Result<(), DurableError> {
        // Given: operation_id does not exist
        let (pool, _temp_dir) = create_test_pool().await?;

        // When: get_operation(pool, "nonexistent")
        let result = get_operation(&pool, "nonexistent").await;

        // Then: Returns Err(DurableError::OperationNotFound("nonexistent".to_string()))
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_p3_violation_invalid_state_transition_returns_error() -> Result<(), DurableError> {
        // Given: Operation in Started state
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 2, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start");

        // When: update_operation_state(pool, "op1", OperationState::Completed, None, None, None)
        let result = update_operation_state(&pool, "op1", OperationState::Completed, None, None, None)
            .await;

        // Then: Returns Err(DurableError::OperationStateInvalid { expected: Completed, found: Started })
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_p6_violation_update_completed_step_returns_error() -> Result<(), DurableError> {
        // Given: Step with status=Completed
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 2, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start");
        record_step(&pool, "op1".to_string(), 0, "step0".to_string(), timestamp)
            .await
            .expect("record");
        update_step_status(&pool, "op1", 0, StepStatus::Running, None, None)
            .await
            .expect("running");
        update_step_status(&pool, "op1", 0, StepStatus::Completed, None, None)
            .await
            .expect("completed");

        // When: update_step_status(pool, "op1", 0, StepStatus::Failed, None, None)
        let result = update_step_status(&pool, "op1", 0, StepStatus::Failed, None, None).await;

        // Then: Returns Err(DurableError::StepAlreadyCompleted { operation_id: "op1", step_index: 0 })
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_p9_violation_max_retries_exceeded_returns_error() -> Result<(), DurableError> {
        // Given: Outbox entry with retry_count >= max_retries
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        add_outbox_entry(
            &pool,
            "outbox1".to_string(),
            SideEffectType::Webhook,
            "payload".to_string(),
            1,
            2,
            timestamp,
        )
        .await
        .expect("add");

        mark_outbox_failed(&pool, "outbox1", "error".to_string()).await?;
        mark_outbox_failed(&pool, "outbox1", "error".to_string()).await?;

        // When: mark_outbox_failed(pool, "id", "error".to_string())
        let result = mark_outbox_failed(&pool, "outbox1", "error".to_string()).await;

        // Then: Returns Err(DurableError::OutboxMaxRetriesExceeded("id".to_string()))
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_p11_violation_invalid_cursor_format_returns_error() {
        // Given: cursor_str="invalid"
        let cursor_str = "invalid";

        // When: parse_cursor("invalid")
        let result = parse_cursor(cursor_str);

        // Then: Returns Err(DurableError::CursorParseError("Expected format: revision:limit".to_string()))
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_q3_violation_completed_operation_missing_completed_at() -> Result<(), DurableError> {
        // Given: Operation updated to Completed state
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 2, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start");
        update_operation_state(&pool, "op1", OperationState::InProgress, None, None, None)
            .await
            .expect("in progress");

        // When: update_operation_state returns
        let result = update_operation_state(&pool, "op1", OperationState::Completed, None, None, None)
            .await;

        // Then: Returned record's completed_at is Some(_) - verifies postcondition
        let record = result.expect("update should succeed");
        assert!(record.completed_at.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_q6_violation_running_step_missing_started_at() -> Result<(), DurableError> {
        // Given: Step updated to Running status
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 2, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start");
        record_step(&pool, "op1".to_string(), 0, "step0".to_string(), timestamp)
            .await
            .expect("record");

        // When: update_step_status returns
        let result = update_step_status(&pool, "op1", 0, StepStatus::Running, None, None).await;

        // Then: Returned record's started_at is Some(_) - verifies postcondition
        let record = result.expect("update should succeed");
        assert!(record.started_at.is_some());

        Ok(())
    }
}

// =============================================================================
// Integration / End-to-End Scenarios
// =============================================================================

mod scenarios {
    use super::*;

    /// Scenario 1: Complete Multi-Step Operation Workflow
    #[tokio::test]
    async fn test_complete_multi_step_operation_workflow() -> Result<(), DurableError> {
        // Given: Database with bootstrap_durable_store initialized
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        // When:
        // 1. start_operation with operation_id="op1", total_steps=3, author_id="user1", description="Process data"
        let op = start_operation(
            &pool,
            "op1".to_string(),
            3,
            "user1".to_string(),
            "Process data".to_string(),
            timestamp,
        )
        .await
        .expect("start_operation");

        // 2. record_step for step_index=0, step_name="fetch"
        record_step(&pool, "op1".to_string(), 0, "fetch".to_string(), timestamp).await.expect("step 0");

        // 3. record_step for step_index=1, step_name="transform"
        record_step(&pool, "op1".to_string(), 1, "transform".to_string(), timestamp).await.expect("step 1");

        // 4. record_step for step_index=2, step_name="save"
        record_step(&pool, "op1".to_string(), 2, "save".to_string(), timestamp).await.expect("step 2");

        // 5. update_step_status for step 0 to Running, then Completed
        update_step_status(&pool, "op1", 0, StepStatus::Running, None, None).await.expect("running 0");
        update_step_status(&pool, "op1", 0, StepStatus::Completed, Some(1), None).await.expect("completed 0");

        // 6. update_step_status for step 1 to Running, then Completed
        update_step_status(&pool, "op1", 1, StepStatus::Running, None, None).await.expect("running 1");
        update_step_status(&pool, "op1", 1, StepStatus::Completed, Some(5), None).await.expect("completed 1");

        // 7. update_step_status for step 2 to Running, then Completed
        update_step_status(&pool, "op1", 2, StepStatus::Running, None, None).await.expect("running 2");
        update_step_status(&pool, "op1", 2, StepStatus::Completed, Some(10), None).await.expect("completed 2");

        // 8. update_operation_state to InProgress
        update_operation_state(&pool, "op1", OperationState::InProgress, None, None, None)
            .await
            .expect("to in_progress");

        // 9. update_operation_state to Completed with final_revision=10
        let final_op = update_operation_state(&pool, "op1", OperationState::Completed, Some(3), Some(10), None)
            .await
            .expect("to completed");

        // Then:
        // - All steps have status=Completed
        // - Operation has state=Completed, completed_at is Some, final_revision=10
        let step0 = get_step(&pool, "op1", 0).await.expect("get step 0");
        let step1 = get_step(&pool, "op1", 1).await.expect("get step 1");
        let step2 = get_step(&pool, "op1", 2).await.expect("get step 2");

        assert_eq!(step0.status, StepStatus::Completed);
        assert_eq!(step1.status, StepStatus::Completed);
        assert_eq!(step2.status, StepStatus::Completed);
        assert_eq!(final_op.state, OperationState::Completed);
        assert!(final_op.completed_at.is_some());
        assert_eq!(final_op.final_revision, Some(10));

        Ok(())
    }

    /// Scenario 2: Failed Operation with Retry
    #[tokio::test]
    async fn test_failed_operation_with_retry() -> Result<(), DurableError> {
        // Given: Operation started and steps recorded
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 2, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start");
        record_step(&pool, "op1".to_string(), 0, "step0".to_string(), timestamp)
            .await
            .expect("step 0");

        // When:
        // 1. First step fails: update_step_status to Running, then Failed
        update_step_status(&pool, "op1", 0, StepStatus::Running, None, None)
            .await
            .expect("running");
        let failed_step = update_step_status(
            &pool,
            "op1",
            0,
            StepStatus::Failed,
            None,
            Some("Step 0 failed".to_string()),
        )
        .await
        .expect("failed");

        // 2. Operation marked as Failed with error_message="Step 0 failed"
        let failed_op = update_operation_state(
            &pool,
            "op1",
            OperationState::Failed,
            None,
            None,
            Some("Step 0 failed".to_string()),
        )
        .await
        .expect("operation failed");

        // Then:
        // - Step has status=Failed, error_message="Step 0 failed"
        // - Operation has state=Failed, error_message="Step 0 failed"
        assert_eq!(failed_step.status, StepStatus::Failed);
        assert_eq!(failed_step.error_message, Some("Step 0 failed".to_string()));
        assert_eq!(failed_op.state, OperationState::Failed);
        assert_eq!(failed_op.error_message, Some("Step 0 failed".to_string()));

        Ok(())
    }

    /// Scenario 3: Outbox Processing Flow
    #[tokio::test]
    async fn test_outbox_processing_flow() -> Result<(), DurableError> {
        // Given: add_outbox_entry with side_effect_type=Webhook, payload="{\"url\":\"http://example.com\"}"
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
        .expect("add");

        // When:
        // 1. mark_outbox_dispatched
        let dispatched = mark_outbox_dispatched(&pool, "out1").await.expect("dispatched");

        // 2. acknowledge_outbox
        let acknowledged = acknowledge_outbox(&pool, "out1").await.expect("acknowledged");

        // Then:
        // - Outbox status transitions: Pending -> Dispatched -> Acknowledged
        // - dispatched_at and acknowledged_at are set
        assert_eq!(dispatched.status, OutboxStatus::Dispatched);
        assert!(dispatched.dispatched_at.is_some());

        assert_eq!(acknowledged.status, OutboxStatus::Acknowledged);
        assert!(acknowledged.acknowledged_at.is_some());

        Ok(())
    }

    /// Scenario 4: Cursor Pagination Through Events
    #[tokio::test]
    async fn test_cursor_pagination_through_events() -> Result<(), DurableError> {
        // Given: 25 events in database
        let (pool, _temp_dir) = create_test_pool_with_events().await?;

        for i in 1..=25 {
            sqlx::query("INSERT INTO events (operation_id, revision, timestamp, payload) VALUES ('op1', ?1, '1700000000', ?2)")
                .bind(i)
                .bind(format!(r#"{{"id":{}}}"#, i))
                .execute(&pool).await.map_err(DurableError::Sqlx)?;
        }

        // When:
        // 1. fetch_events_cursor with EventCursor::first(10)
        let page1 = fetch_events_cursor(&pool, EventCursor::first(10))
            .await
            .expect("page 1");

        // 2. fetch_events_cursor with next_cursor
        let next = page1.next_cursor.expect("next cursor");
        let page2 = fetch_events_cursor(&pool, next).await.expect("page 2");

        // 3. Continue until has_more=false
        let next2 = page2.next_cursor.expect("next cursor 2");
        let page3 = fetch_events_cursor(&pool, next2).await.expect("page 3");

        // Then:
        // - First page: 10 events, has_more=true
        // - Second page: 10 events, has_more=true
        // - Third page: 5 events, has_more=false
        assert_eq!(page1.events.len(), 10);
        assert!(page1.has_more);

        assert_eq!(page2.events.len(), 10);
        assert!(page2.has_more);

        assert_eq!(page3.events.len(), 5);
        assert!(!page3.has_more);

        Ok(())
    }

    /// Scenario 5: Conflict Detection
    #[tokio::test]
    async fn test_conflict_detection() -> Result<(), DurableError> {
        // Given: Another client added events, current revision is 15
        let (pool, _temp_dir) = create_test_pool_with_events().await?;

        for i in 1..=15 {
            sqlx::query("INSERT INTO events (operation_id, revision, timestamp, payload) VALUES ('op1', ?1, '1700000000', '{}')")
                .bind(i)
                .execute(&pool).await.map_err(DurableError::Sqlx)?;
        }

        // When: generate_conflict_diff with assumed_revision=10
        let diff = generate_conflict_diff(&pool, 10).await.expect("generate diff");

        // Then:
        // - Returns ConflictDiff with assumed_revision=10, actual_revision=15
        // - Changes list contains the 5 new events
        assert_eq!(diff.assumed_revision, 10);
        assert_eq!(diff.actual_revision, 15);
        assert!(!diff.changes.is_empty());

        Ok(())
    }

    /// Scenario 6: Resume Interrupted Operation
    #[tokio::test]
    async fn test_resume_interrupted_operation() -> Result<(), DurableError> {
        // Given: Operation in InProgress state with steps: 0=Completed, 1=Failed, 2=Pending
        let (pool, _temp_dir) = create_test_pool().await?;
        let timestamp = 1_700_000_000_i64;

        start_operation(&pool, "op1".to_string(), 3, "u1".to_string(), "d1".to_string(), timestamp)
            .await
            .expect("start");
        update_operation_state(&pool, "op1", OperationState::InProgress, None, None, None)
            .await
            .expect("in progress");

        record_step(&pool, "op1".to_string(), 0, "step0".to_string(), timestamp)
            .await
            .expect("step 0");
        record_step(&pool, "op1".to_string(), 1, "step1".to_string(), timestamp)
            .await
            .expect("step 1");
        record_step(&pool, "op1".to_string(), 2, "step2".to_string(), timestamp)
            .await
            .expect("step 2");

        // Step 0: Completed
        update_step_status(&pool, "op1", 0, StepStatus::Running, None, None)
            .await
            .expect("running 0");
        update_step_status(&pool, "op1", 0, StepStatus::Completed, None, None)
            .await
            .expect("completed 0");

        // Step 1: Failed
        update_step_status(&pool, "op1", 1, StepStatus::Running, None, None)
            .await
            .expect("running 1");
        update_step_status(&pool, "op1", 1, StepStatus::Failed, None, Some("error".to_string()))
            .await
            .expect("failed 1");

        // When:
        // 1. can_resume_operation
        let can_resume = can_resume_operation(&pool, "op1").await.expect("can_resume");

        // 2. get_next_step
        let next_step = get_next_step(&pool, "op1").await.expect("get_next_step");

        // Then:
        // - can_resume_operation returns true
        // - get_next_step returns step with step_index=1 (first failed/pending)
        assert!(can_resume);
        assert!(next_step.is_some());
        assert_eq!(next_step.unwrap().step_index, 1);

        Ok(())
    }
}
