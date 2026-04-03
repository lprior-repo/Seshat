#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

use std::path::Path;
use std::sync::{Arc, Mutex};

use thiserror::Error;
use tokio::runtime::Runtime;

use crate::store_async::ai_documents::{
    delete_ai_document as delete_ai_document_async, fetch_ai_document, fetch_ai_documents_by_key,
    insert_ai_document, update_ai_document,
};
use crate::store_async::{
    append_batch_async, append_event_async, append_idempotent_async, bootstrap_async_store,
    envelope_to_valid_event, fetch_events_since, parse_revision, reset_store_async,
    AsyncAppendResult, AsyncBatchAppendResult, AsyncStoreBootstrap, AsyncStoreError, EventRecord,
};
use diagram_models::envelope::EventEnvelope;
use diagram_models::schema_ai_documents::AiDocument;

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("Async store error: {0}")]
    AsyncStore(#[from] AsyncStoreError),
    #[error("Failed to spawn tokio runtime: {0}")]
    RuntimeSpawn(String),
    #[error("Runtime not running")]
    RuntimeNotRunning,
    #[error("Pool not initialized")]
    PoolNotInitialized,
    #[error("Failed to acquire pool lock")]
    PoolLockError,
}

pub struct StoreBridge {
    pool: Mutex<Option<sqlx::SqlitePool>>,
    runtime: Arc<Runtime>,
}

impl StoreBridge {
    /// Spawns an async pool.
    ///
    /// # Errors
    /// Returns an error if the runtime or store cannot be created.
    pub fn spawn_async_pool(db_path: &Path) -> Result<Self, BridgeError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| BridgeError::RuntimeSpawn(e.to_string()))?;

        let bootstrap: AsyncStoreBootstrap = runtime
            .block_on(bootstrap_async_store(db_path))
            .map_err(BridgeError::AsyncStore)?;

        Ok(Self {
            pool: Mutex::new(Some(bootstrap.pool)),
            runtime: Arc::new(runtime),
        })
    }

    /// Helper to run an async store operation synchronously.
    ///
    /// This is public to allow server functions to use the bridge for sync operations.
    pub fn run_async<F, Fut, R>(&self, f: F) -> Result<R, BridgeError>
    where
        F: FnOnce(sqlx::SqlitePool) -> Fut,
        Fut: std::future::Future<Output = Result<R, AsyncStoreError>>,
    {
        let pool_guard = self.pool.lock().map_err(|_| BridgeError::PoolLockError)?;
        let pool = pool_guard.as_ref().ok_or(BridgeError::PoolNotInitialized)?;
        let pool_clone = pool.clone();
        drop(pool_guard);
        self.runtime
            .block_on(async { f(pool_clone).await.map_err(BridgeError::AsyncStore) })
    }

    /// Appends an event synchronously.
    ///
    /// # Errors
    /// Returns an error if the store fails to append the event.
    pub fn append_event_sync(
        &self,
        envelope: &EventEnvelope,
        expected_revision: Option<i64>,
    ) -> Result<AsyncAppendResult, BridgeError> {
        // Parse at boundary
        let event = envelope_to_valid_event(envelope).map_err(BridgeError::AsyncStore)?;
        let expected = expected_revision
            .map(|rev| parse_revision(rev).map_err(BridgeError::AsyncStore))
            .transpose()?;

        self.run_async(|pool| async move { append_event_async(&pool, event, expected).await })
    }

    /// Appends a batch of events synchronously.
    ///
    /// # Errors
    /// Returns an error if the store is not initialized or the append fails.
    pub fn append_batch_sync(
        &self,
        ops: &[EventEnvelope],
        expected_revision: Option<i64>,
    ) -> Result<AsyncBatchAppendResult, BridgeError> {
        // Parse at boundary
        let events = ops
            .iter()
            .map(envelope_to_valid_event)
            .collect::<Result<Vec<_>, _>>()
            .map_err(BridgeError::AsyncStore)?;
        let batch = crate::store_async::parse_bounded_batch::<1, 1000>(events)
            .map_err(BridgeError::AsyncStore)?;
        let expected = expected_revision
            .map(|rev| parse_revision(rev).map_err(BridgeError::AsyncStore))
            .transpose()?;

        self.run_async(|pool| async move { append_batch_async(&pool, batch, expected).await })
    }

    /// Appends a single event idempotently, synchronously.
    ///
    /// # Errors
    /// Returns an error if the store is not initialized or the append fails.
    pub fn append_idempotent_sync(
        &self,
        envelope: EventEnvelope,
    ) -> Result<AsyncAppendResult, BridgeError> {
        self.run_async(|pool| async move { append_idempotent_async(&pool, envelope).await })
    }

    /// Fetches events since a given revision synchronously.
    ///
    /// # Errors
    /// Returns an error if the store is not initialized or the fetch fails.
    pub fn fetch_events_since_sync(&self, revision: i64) -> Result<Vec<EventRecord>, BridgeError> {
        self.run_async(|pool| async move { fetch_events_since(&pool, revision).await })
    }

    /// Resets the store by deleting all events synchronously.
    ///
    /// This is used when opening a new document to clear any existing event history.
    ///
    /// # Errors
    /// Returns an error if the store is not initialized or the reset fails.
    pub fn reset_store_sync(&self) -> Result<(), BridgeError> {
        self.run_async(|pool| async move { reset_store_async(&pool).await })
    }

    /// Inserts an AI document synchronously.
    ///
    /// # Errors
    /// Returns an error if the store is not initialized or the insert fails.
    pub fn insert_ai_document_sync(&self, doc: &AiDocument) -> Result<String, BridgeError> {
        self.run_async({
            let doc = doc.clone();
            move |pool| async move { insert_ai_document(&pool, &doc).await }
        })
    }

    /// Fetches an AI document by ID synchronously.
    ///
    /// # Errors
    /// Returns an error if the store is not initialized or the fetch fails.
    pub fn fetch_ai_document_sync(&self, id: &str) -> Result<Option<AiDocument>, BridgeError> {
        self.run_async(|pool| async move { fetch_ai_document(&pool, id).await })
    }

    /// Fetches all AI documents with a given key synchronously.
    ///
    /// # Errors
    /// Returns an error if the store is not initialized or the fetch fails.
    pub fn fetch_ai_documents_by_key_sync(
        &self,
        key: &str,
    ) -> Result<Vec<AiDocument>, BridgeError> {
        self.run_async(|pool| async move { fetch_ai_documents_by_key(&pool, key).await })
    }

    /// Updates an existing AI document synchronously.
    ///
    /// # Errors
    /// Returns an error if the store is not initialized or the update fails.
    pub fn update_ai_document_sync(&self, doc: &AiDocument) -> Result<(), BridgeError> {
        self.run_async({
            let doc = doc.clone();
            move |pool| async move { update_ai_document(&pool, &doc).await }
        })
    }

    /// Deletes an AI document by ID synchronously.
    ///
    /// # Errors
    /// Returns an error if the store is not initialized or the delete fails.
    pub fn delete_ai_document_sync(&self, id: &str) -> Result<u64, BridgeError> {
        self.run_async(|pool| async move { delete_ai_document_async(&pool, id).await })
    }

    /// Shuts down the store bridge.
    ///
    /// # Errors
    /// Never returns an error currently, but signature is kept for symmetry.
    pub fn shutdown(&self) -> Result<(), BridgeError> {
        // Take the pool out of self.pool so subsequent calls get PoolNotInitialized
        let pool = self
            .pool
            .lock()
            .map_err(|_| BridgeError::PoolLockError)?
            .take();
        if let Some(pool) = pool {
            let runtime = Arc::clone(&self.runtime);
            runtime.block_on(async {
                pool.close().await;
            });
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_test_envelope(op_id: &str, node_id: &str, timestamp: i64) -> EventEnvelope {
        use diagram_models::document::NodeId;
        EventEnvelope {
            op_id: op_id.to_string(),
            operation: diagram_models::envelope::DomainOp::NodeAdd {
                id: NodeId::new(node_id.to_string()),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Test Node".to_string(),
            },
            author: diagram_models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp,
        }
    }

    #[test]
    fn test_spawn_and_shutdown() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bridge = StoreBridge::spawn_async_pool(&db_path).expect("Failed to spawn bridge");
        bridge.shutdown().expect("Failed to shutdown");
    }

    #[test]
    fn test_append_event_sync() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bridge = StoreBridge::spawn_async_pool(&db_path).expect("Failed to spawn bridge");
        let envelope = make_test_envelope("test-op-1", "node-1", 1_700_000_000);

        let result = bridge
            .append_event_sync(&envelope, None)
            .expect("Failed to append");
        assert_eq!(result.revision, 1);

        bridge.shutdown().expect("Failed to shutdown");
    }

    #[test]
    fn test_fetch_events_since_sync() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bridge = StoreBridge::spawn_async_pool(&db_path).expect("Failed to spawn bridge");
        let envelope = make_test_envelope("test-op-1", "node-1", 1_700_000_000);

        bridge
            .append_event_sync(&envelope, None)
            .expect("Failed to append");

        let events = bridge.fetch_events_since_sync(0).expect("Failed to fetch");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].op_id, "test-op-1");

        let events = bridge.fetch_events_since_sync(1).expect("Failed to fetch");
        assert!(events.is_empty());

        bridge.shutdown().expect("Failed to shutdown");
    }

    #[test]
    fn test_append_batch_sync() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bridge = StoreBridge::spawn_async_pool(&db_path).expect("Failed to spawn bridge");

        let envelope1 = make_test_envelope("test-op-1", "node-1", 1_700_000_001);
        let envelope2 = make_test_envelope("test-op-2", "node-2", 1_700_000_002);

        let result = bridge
            .append_batch_sync(&[envelope1, envelope2], None)
            .expect("Failed to append batch");

        assert_eq!(result.start_revision, 1);
        assert_eq!(result.end_revision, 2);
        assert_eq!(result.count, 2);

        bridge.shutdown().expect("Failed to shutdown");
    }

    #[test]
    fn test_append_idempotent_sync() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bridge = StoreBridge::spawn_async_pool(&db_path).expect("Failed to spawn bridge");

        let envelope = make_test_envelope("test-op-1", "node-1", 1_700_000_000);

        let result1 = bridge
            .append_idempotent_sync(envelope.clone())
            .expect("Failed to append first");

        let result2 = bridge
            .append_idempotent_sync(envelope)
            .expect("Failed to append second (should be exact duplicate)");

        assert_eq!(result1.revision, result2.revision);

        bridge.shutdown().expect("Failed to shutdown");
    }

    #[test]
    fn test_reset_store_sync() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let bridge = StoreBridge::spawn_async_pool(&db_path).expect("Failed to spawn bridge");

        let envelope = make_test_envelope("test-op-1", "node-1", 1_700_000_000);
        bridge
            .append_event_sync(&envelope, None)
            .expect("Failed to append");

        let events = bridge.fetch_events_since_sync(0).expect("Failed to fetch");
        assert_eq!(events.len(), 1);

        bridge.reset_store_sync().expect("Failed to reset store");

        let events = bridge
            .fetch_events_since_sync(0)
            .expect("Failed to fetch after reset");
        assert!(events.is_empty());

        bridge.shutdown().expect("Failed to shutdown");
    }
}
