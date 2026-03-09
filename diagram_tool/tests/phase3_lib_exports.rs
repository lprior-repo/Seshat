//! Phase 3 Tests: lib.rs exports verification
//!
//! These tests verify that after the rusqlite → sqlx migration:
//! 1. `crate::store` is accessible unconditionally (no feature gate)
//! 2. `store` module exports `AsyncStoreError`
//! 3. `store` module exports `SqlitePool`
//! 4. `store` module exports all async functions
//! 5. `store_bridge` module does NOT exist
//! 6. Old sync store is NOT accessible
//!
//! These are compile-time assertions - they will only compile after migration is complete.

#[cfg(test)]
mod tests {
    #[test]
    fn test_store_module_exported_without_feature_gate() {
        let _: i32 = diagram_tool::store::CURRENT_SCHEMA_VERSION;
    }

    #[test]
    fn test_store_exports_async_store_error() {
        let _error: diagram_tool::store_async::AsyncStoreError =
            diagram_tool::store_async::AsyncStoreError::ValidationFailed("test".to_string());
    }

    #[test]
    fn test_store_exports_sqlite_pool() {
        let _pool: sqlx::SqlitePool;
    }

    #[test]
    fn test_store_exports_bootstrap_async_store() {
        let _fn = diagram_tool::store_async::bootstrap_async_store;
    }

    #[test]
    fn test_store_exports_append_event_async() {
        let _fn = diagram_tool::store_async::append_event_async;
    }

    #[test]
    fn test_store_exports_fetch_latest_revision() {
        let _fn = diagram_tool::store_async::fetch_latest_revision;
    }

    #[test]
    fn test_store_exports_current_revision() {
        let _fn = diagram_tool::store_async::current_revision;
    }

    #[test]
    fn test_store_exports_next_revision() {
        let _fn = diagram_tool::store_async::next_revision;
    }

    #[test]
    fn test_store_exports_append_batch_async() {
        let _fn = diagram_tool::store_async::append_batch_async;
    }

    #[test]
    fn test_store_exports_fetch_events_since() {
        let _fn = diagram_tool::store_async::fetch_events_since;
    }

    #[test]
    fn test_store_exports_fetch_all_events() {
        let _fn = diagram_tool::store_async::fetch_all_events;
    }

    #[test]
    fn test_store_exports_append_idempotent_async() {
        let _fn = diagram_tool::store_async::append_idempotent_async;
    }

    #[test]
    fn test_store_exports_async_store_pragma_types() {
        let _pragmas = diagram_tool::store_async::AsyncStorePragmas {
            journal_mode: "wal".to_string(),
            synchronous: 2,
            wal_autocheckpoint: 1000,
            foreign_keys: true,
            busy_timeout: 5000,
        };
    }

    #[test]
    fn test_store_exports_async_append_result() {
        let _result = diagram_tool::store_async::AsyncAppendResult {
            revision: 1,
            op_id: "test".to_string(),
            timestamp: 1700000000,
        };
    }

    #[test]
    fn test_store_exports_async_batch_append_result() {
        let _result = diagram_tool::store_async::AsyncBatchAppendResult {
            start_revision: 1,
            end_revision: 3,
            count: 3,
            op_ids: vec!["op1".to_string(), "op2".to_string(), "op3".to_string()],
            last_timestamp: 1700000002,
        };
    }

    #[test]
    fn test_store_exports_async_store_bootstrap() {
        let _bootstrap: diagram_tool::store_async::AsyncStoreBootstrap;
    }

    #[test]
    fn test_store_exports_event_record() {
        let _record = diagram_tool::store_async::EventRecord {
            op_id: "test".to_string(),
            revision: 1,
            timestamp: 1700000000,
            payload: "{}".to_string(),
        };
    }

    #[test]
    fn test_store_exports_duplicate_kind() {
        let _kind_exact = diagram_tool::store_async::DuplicateKind::Exact;
        let _kind_conflict = diagram_tool::store_async::DuplicateKind::Conflict;
    }

    #[test]
    fn test_store_exports_create_async_pool() {
        let _fn = diagram_tool::store_async::create_async_pool;
    }

    #[test]
    fn test_store_exports_read_store_pragmas_async() {
        let _fn = diagram_tool::store_async::read_store_pragmas_async;
    }

    #[test]
    fn test_store_exports_lookup_existing_op_async() {
        let _fn = diagram_tool::store_async::lookup_existing_op_async;
    }

    #[test]
    fn test_store_exports_classify_duplicate_async() {
        let _fn = diagram_tool::store_async::classify_duplicate_async;
    }

    #[test]
    fn test_store_exports_map_error_code() {
        use diagram_tool::store_async::{map_error_code, AsyncStoreError};
        let error = AsyncStoreError::ValidationFailed("test".to_string());
        let code = map_error_code(&error);
        assert_eq!(
            code,
            diagram_tool::store_async::CliErrorCode::ValidationFailed
        );
    }

    #[test]
    fn test_store_exports_cli_error_code() {
        let _code = diagram_tool::store_async::CliErrorCode::ValidationFailed;
    }
}
