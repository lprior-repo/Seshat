#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use std::sync::{Arc, Mutex};
use thiserror::Error;
use tokio::runtime::Runtime;

use crate::models::envelope::EventEnvelope;
use crate::store_async::{
    AsyncAppendResult, AsyncBatchAppendResult, AsyncStoreError, EventRecord,
    append_event_async, append_batch_async, append_idempotent_async,
    bootstrap_async_store, fetch_events_since, lookup_existing_op_async,
};

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("Runtime error: {0}")]
    Runtime(#[from] tokio::runtime::Error),
    #[error("Async store error: {0}")]
    AsyncStore(#[from] AsyncStoreError),
    #[error("Bridge is not initialized")]
    NotInitialized,
}

pub type BridgeResult<T> = Result<T, BridgeError>;

/// Sync wrapper for async database operations
///
/// This struct provides synchronous interfaces to async database operations
/// by spawning a tokio runtime internally. It allows gradual migration from
/// sync to async operations in UI code.
pub struct StoreBridge {
    runtime: Runtime,
    pool: Arc<Mutex<Option<sqlx::SqlitePool>>>,
}

impl StoreBridge {
    /// Spawn a new tokio runtime and create an async database connection pool
    ///
    /// This initializes the bridge with a background tokio runtime that will
    /// handle all async database operations.
    pub fn spawn_async_pool(db_path: &std::path::Path) -> BridgeResult<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        let pool = runtime.block_on(bootstrap_async_store(db_path))?.pool;

        Ok(Self {
            runtime,
            pool: Arc::new(Mutex::new(Some(pool))),
        })
    }

    /// Append an event synchronously (wraps async operation)
    pub fn append_event_sync(
        &self,
        envelope: EventEnvelope,
        expected_revision: Option<i64>,
    ) -> BridgeResult<AsyncAppendResult> {
        let pool = self.get_pool()?;
        let envelope_clone = envelope.clone();

        self.runtime.block_on(async move {
            append_event_async(&pool, envelope_clone, expected_revision).await
        })
        .map_err(Into::into)
    }

    /// Append a batch of events synchronously
    pub fn append_batch_sync(
        &self,
        ops: Vec<EventEnvelope>,
        expected_revision: Option<i64>,
    ) -> BridgeResult<AsyncBatchAppendResult> {
        let pool = self.get_pool()?;
        let ops_clone = ops.clone();

        self.runtime.block_on(async move {
            append_batch_async(&pool, ops_clone, expected_revision).await
        })
        .map_err(Into::into)
    }

    /// Append event with idempotent behavior (handles duplicates)
    pub fn append_idempotent_sync(
        &self,
        envelope: EventEnvelope,
    ) -> BridgeResult<AsyncAppendResult> {
        let pool = self.get_pool()?;
        let envelope_clone = envelope.clone();

        self.runtime.block_on(async move {
            append_idempotent_async(&pool, envelope_clone).await
        })
        .map_err(Into::into)
    }

    /// Fetch events since a given revision
    pub fn fetch_events_since_sync(
        &self,
        revision: i64,
    ) -> BridgeResult<Vec<EventRecord>> {
        let pool = self.get_pool()?;

        self.runtime.block_on(async move {
            fetch_events_since(&pool, revision).await
        })
        .map_err(Into::into)
    }

    /// Lookup existing operation by op_id
    pub fn lookup_existing_op_sync(
        &self,
        op_id: &str,
    ) -> BridgeResult<Option<EventRecord>> {
        let pool = self.get_pool()?;
        let op_id_clone = op_id.to_string();

        self.runtime.block_on(async move {
            lookup_existing_op_async(&pool, &op_id_clone).await
        })
        .map_err(Into::into)
    }

    /// Shutdown the runtime and close connections
    pub fn shutdown(self) -> BridgeResult<()> {
        // The runtime will be dropped automatically
        // The pool will be closed when dropped
        Ok(())
    }

    /// Get reference to the pool (internal use)
    fn get_pool(&self) -> BridgeResult<sqlx::SqlitePool> {
        self.pool
            .lock()
            .map_err(|_| BridgeError::NotInitialized)?
            .clone()
            .ok_or(BridgeError::NotInitialized)
    }
}

impl Drop for StoreBridge {
    fn drop(&mut self) {
        // Ensure the pool is properly closed
        if let Ok(mut pool_opt) = self.pool.lock() {
            *pool_opt = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_bridge_initialization() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bridge = StoreBridge::spawn_async_pool(&db_path);
        assert!(bridge.is_ok(), "Bridge should initialize successfully");

        let bridge = bridge.unwrap();
        let result = bridge.shutdown();
        assert!(result.is_ok(), "Bridge should shutdown cleanly");
    }

    #[test]
    fn test_append_event_sync() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let mut bridge = StoreBridge::spawn_async_pool(&db_path).unwrap();

        let envelope = EventEnvelope {
            op_id: "test-op-1".to_string(),
            operation: crate::models::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
            author: crate::models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        let result = bridge.append_event_sync(envelope, None);
        assert!(result.is_ok(), "Append should succeed");

        let result = result.unwrap();
        assert_eq!(result.revision, 1);
        assert_eq!(result.op_id, "test-op-1");
    }

    #[test]
    fn test_idempotent_append_sync() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let mut bridge = StoreBridge::spawn_async_pool(&db_path).unwrap();

        let envelope = EventEnvelope {
            op_id: "idempotent-test".to_string(),
            operation: crate::models::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
            author: crate::models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        // First append
        let result1 = bridge.append_idempotent_sync(envelope.clone());
        assert!(result1.is_ok());
        let result1 = result1.unwrap();
        assert_eq!(result1.revision, 1);

        // Second append with same op_id should return existing result
        let result2 = bridge.append_idempotent_sync(envelope);
        assert!(result2.is_ok());
        let result2 = result2.unwrap();
        assert_eq!(result2.revision, 1); // Same revision
        assert_eq!(result2.op_id, "idempotent-test");
    }
}