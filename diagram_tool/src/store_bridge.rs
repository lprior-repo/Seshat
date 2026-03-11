#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use std::path::Path;
use std::sync::Arc;

use thiserror::Error;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use crate::models::envelope::EventEnvelope;
use crate::store::types::ValidEvent;
use crate::store_async::{
    append_batch_async, append_event_async, append_idempotent_async, bootstrap_async_store,
    envelope_to_valid_event, fetch_events_since, parse_revision, AsyncAppendResult,
    AsyncBatchAppendResult, AsyncStoreBootstrap, AsyncStoreError, EventRecord,
};

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
    pool: Arc<Mutex<Option<sqlx::SqlitePool>>>,
    runtime: Runtime,
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
            pool: Arc::new(Mutex::new(Some(bootstrap.pool))),
            runtime,
        })
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
        let pool = self.pool.clone();

        self.runtime.block_on(async {
            let p = {
                let pool_guard = pool.lock().await;
                pool_guard
                    .as_ref()
                    .ok_or(BridgeError::PoolNotInitialized)?
                    .clone()
            };

            // Parse at boundary: convert envelope to ValidEvent
            let event = envelope_to_valid_event(envelope).map_err(BridgeError::AsyncStore)?;

            // Parse at boundary: convert expected_revision to Option<Revision>
            let expected = match expected_revision {
                Some(rev) => Some(parse_revision(rev).map_err(BridgeError::AsyncStore)?),
                None => None,
            };

            append_event_async(&p, event, expected)
                .await
                .map_err(BridgeError::AsyncStore)
        })
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
        let pool = self.pool.clone();

        self.runtime.block_on(async {
            let p = {
                let pool_guard = pool.lock().await;
                pool_guard
                    .as_ref()
                    .ok_or(BridgeError::PoolNotInitialized)?
                    .clone()
            };

            // Parse at boundary: convert envelopes to ValidEvents
            let events: Result<Vec<ValidEvent>, _> =
                ops.iter().map(envelope_to_valid_event).collect();
            let events = events.map_err(BridgeError::AsyncStore)?;

            // Parse at boundary: convert to BoundedBatch (MIN=1, MAX=1000)
            let batch = crate::store_async::parse_bounded_batch::<1, 1000>(events)
                .map_err(BridgeError::AsyncStore)?;

            // Parse at boundary: convert expected_revision to Option<Revision>
            let expected = match expected_revision {
                Some(rev) => Some(parse_revision(rev).map_err(BridgeError::AsyncStore)?),
                None => None,
            };

            append_batch_async(&p, batch, expected)
                .await
                .map_err(BridgeError::AsyncStore)
        })
    }

    /// Appends a single event idempotently, synchronously.
    ///
    /// # Errors
    /// Returns an error if the store is not initialized or the append fails.
    pub fn append_idempotent_sync(
        &self,
        envelope: EventEnvelope,
    ) -> Result<AsyncAppendResult, BridgeError> {
        let pool = self.pool.clone();

        self.runtime.block_on(async {
            let p = {
                let pool_guard = pool.lock().await;
                pool_guard
                    .as_ref()
                    .ok_or(BridgeError::PoolNotInitialized)?
                    .clone()
            };
            append_idempotent_async(&p, envelope)
                .await
                .map_err(BridgeError::AsyncStore)
        })
    }

    /// Fetches events since a given revision synchronously.
    ///
    /// # Errors
    /// Returns an error if the store is not initialized or the fetch fails.
    pub fn fetch_events_since_sync(&self, revision: i64) -> Result<Vec<EventRecord>, BridgeError> {
        let pool = self.pool.clone();

        self.runtime.block_on(async {
            let p = {
                let pool_guard = pool.lock().await;
                pool_guard
                    .as_ref()
                    .ok_or(BridgeError::PoolNotInitialized)?
                    .clone()
            };
            fetch_events_since(&p, revision)
                .await
                .map_err(BridgeError::AsyncStore)
        })
    }

    /// Shuts down the store bridge.
    ///
    /// # Errors
    /// Never returns an error currently, but signature is kept for symmetry.
    pub fn shutdown(self) -> Result<(), BridgeError> {
        self.runtime.block_on(async {
            let p_opt = {
                let mut pool_guard = self.pool.lock().await;
                pool_guard.take()
            };
            if let Some(pool) = p_opt {
                pool.close().await;
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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

        let envelope1 = EventEnvelope {
            op_id: "test-op-1".to_string(),
            operation: crate::models::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: "Node 1".to_string(),
            },
            author: crate::models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000001,
        };

        let envelope2 = EventEnvelope {
            op_id: "test-op-2".to_string(),
            operation: crate::models::envelope::DomainOp::NodeAdd {
                id: "node-2".to_string(),
                x: 30.0,
                y: 40.0,
                width: 100.0,
                height: 50.0,
                label: "Node 2".to_string(),
            },
            author: crate::models::envelope::Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000002,
        };

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

        let result1 = bridge
            .append_idempotent_sync(envelope.clone())
            .expect("Failed to append first");

        let result2 = bridge
            .append_idempotent_sync(envelope)
            .expect("Failed to append second (should be exact duplicate)");

        assert_eq!(result1.revision, result2.revision);

        bridge.shutdown().expect("Failed to shutdown");
    }
}
