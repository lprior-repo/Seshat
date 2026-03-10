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
use crate::store_async::{
    append_batch_async, append_event_async, append_idempotent_async, bootstrap_async_store,
    fetch_events_since, AsyncAppendResult, AsyncBatchAppendResult, AsyncStoreError,
    AsyncStoreBootstrap, EventRecord,
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

        let bootstrap: AsyncStoreBootstrap = runtime.block_on(bootstrap_async_store(db_path))
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
        envelope: EventEnvelope,
        expected_revision: Option<i64>,
    ) -> Result<AsyncAppendResult, BridgeError> {
        let pool = self.pool.clone();
        
        self.runtime.block_on(async {
            let p = {
                let pool_guard = pool.lock().await;
                pool_guard.as_ref().ok_or(BridgeError::PoolNotInitialized)?.clone()
            };
            append_event_async(&p, envelope, expected_revision)
                .await
                .map_err(BridgeError::AsyncStore)
        })
    }

<<<<<<< conflict 1 of 8
+++++++ ozopryly fda186e0 "refactor: apply strict DDD types to store.rs (seshat-6sl)" (rebase destination)
    /// Appends a batch of events synchronously.
    ///
    /// # Errors
    /// Returns an error if the store is not initialized or the append fails.
%%%%%%% diff from: kzssywqk cad5af6f "bd: backup 2026-03-10 00:19" (parents of rebased revision)
\\\\\\\        to: xmmqkotr c61a8cb9 "Fix syntax error and test issues" (rebased revision)
+    /// Appends a batch of events synchronously.
+    ///
+    /// # Errors
+    /// Returns an error if the store fails to append the events.
>>>>>>> conflict 1 of 8 ends
    pub fn append_batch_sync(
        &self,
        ops: Vec<EventEnvelope>,
        expected_revision: Option<i64>,
    ) -> Result<AsyncBatchAppendResult, BridgeError> {
        let pool = self.pool.clone();
        
        self.runtime.block_on(async {
<<<<<<< conflict 2 of 8
%%%%%%% diff from: kzssywqk cad5af6f "bd: backup 2026-03-10 00:19" (parents of rebased revision)
\\\\\\\        to: ozopryly fda186e0 "refactor: apply strict DDD types to store.rs (seshat-6sl)" (rebase destination)
             let pool_guard = pool.lock().await;
-            let pool = pool_guard.as_ref().ok_or(BridgeError::PoolNotInitialized)?;
-            append_batch_async(pool, ops, expected_revision)
+            let pool = pool_guard.as_ref().ok_or(BridgeError::PoolNotInitialized)?.clone();
+            drop(pool_guard);
+            append_batch_async(&pool, ops, expected_revision)
+++++++ xmmqkotr c61a8cb9 "Fix syntax error and test issues" (rebased revision)
            let p = {
                let pool_guard = pool.lock().await;
                pool_guard.as_ref().ok_or(BridgeError::PoolNotInitialized)?.clone()
            };
            append_batch_async(&p, ops, expected_revision)
>>>>>>> conflict 2 of 8 ends
                .await
                .map_err(BridgeError::AsyncStore)
        })
    }

<<<<<<< conflict 3 of 8
+++++++ ozopryly fda186e0 "refactor: apply strict DDD types to store.rs (seshat-6sl)" (rebase destination)
    /// Appends an idempotent event synchronously.
    ///
    /// # Errors
    /// Returns an error if the store is not initialized or the append fails.
%%%%%%% diff from: kzssywqk cad5af6f "bd: backup 2026-03-10 00:19" (parents of rebased revision)
\\\\\\\        to: xmmqkotr c61a8cb9 "Fix syntax error and test issues" (rebased revision)
+    /// Appends a single event idempotently, synchronously.
+    ///
+    /// # Errors
+    /// Returns an error if the store fails to append the event.
>>>>>>> conflict 3 of 8 ends
    pub fn append_idempotent_sync(
        &self,
        envelope: EventEnvelope,
    ) -> Result<AsyncAppendResult, BridgeError> {
        let pool = self.pool.clone();
        
        self.runtime.block_on(async {
<<<<<<< conflict 4 of 8
%%%%%%% diff from: kzssywqk cad5af6f "bd: backup 2026-03-10 00:19" (parents of rebased revision)
\\\\\\\        to: ozopryly fda186e0 "refactor: apply strict DDD types to store.rs (seshat-6sl)" (rebase destination)
             let pool_guard = pool.lock().await;
-            let pool = pool_guard.as_ref().ok_or(BridgeError::PoolNotInitialized)?;
-            append_idempotent_async(pool, envelope)
+            let pool = pool_guard.as_ref().ok_or(BridgeError::PoolNotInitialized)?.clone();
+            drop(pool_guard);
+            append_idempotent_async(&pool, envelope)
+++++++ xmmqkotr c61a8cb9 "Fix syntax error and test issues" (rebased revision)
            let p = {
                let pool_guard = pool.lock().await;
                pool_guard.as_ref().ok_or(BridgeError::PoolNotInitialized)?.clone()
            };
            append_idempotent_async(&p, envelope)
>>>>>>> conflict 4 of 8 ends
                .await
                .map_err(BridgeError::AsyncStore)
        })
    }

<<<<<<< conflict 5 of 8
+++++++ ozopryly fda186e0 "refactor: apply strict DDD types to store.rs (seshat-6sl)" (rebase destination)
    /// Fetches events since a given revision synchronously.
    ///
    /// # Errors
    /// Returns an error if the store is not initialized or the fetch fails.
%%%%%%% diff from: kzssywqk cad5af6f "bd: backup 2026-03-10 00:19" (parents of rebased revision)
\\\\\\\        to: xmmqkotr c61a8cb9 "Fix syntax error and test issues" (rebased revision)
+    /// Fetches events since a given revision synchronously.
+    ///
+    /// # Errors
+    /// Returns an error if the store fails to fetch the events.
>>>>>>> conflict 5 of 8 ends
    pub fn fetch_events_since_sync(
        &self,
        revision: i64,
    ) -> Result<Vec<EventRecord>, BridgeError> {
        let pool = self.pool.clone();
        
        self.runtime.block_on(async {
<<<<<<< conflict 6 of 8
%%%%%%% diff from: kzssywqk cad5af6f "bd: backup 2026-03-10 00:19" (parents of rebased revision)
\\\\\\\        to: ozopryly fda186e0 "refactor: apply strict DDD types to store.rs (seshat-6sl)" (rebase destination)
             let pool_guard = pool.lock().await;
-            let pool = pool_guard.as_ref().ok_or(BridgeError::PoolNotInitialized)?;
-            fetch_events_since(pool, revision)
+            let pool = pool_guard.as_ref().ok_or(BridgeError::PoolNotInitialized)?.clone();
+            drop(pool_guard);
+            fetch_events_since(&pool, revision)
+++++++ xmmqkotr c61a8cb9 "Fix syntax error and test issues" (rebased revision)
            let p = {
                let pool_guard = pool.lock().await;
                pool_guard.as_ref().ok_or(BridgeError::PoolNotInitialized)?.clone()
            };
            fetch_events_since(&p, revision)
>>>>>>> conflict 6 of 8 ends
                .await
                .map_err(BridgeError::AsyncStore)
        })
    }

<<<<<<< conflict 7 of 8
%%%%%%% diff from: kzssywqk cad5af6f "bd: backup 2026-03-10 00:19" (parents of rebased revision)
\\\\\\\        to: ozopryly fda186e0 "refactor: apply strict DDD types to store.rs (seshat-6sl)" (rebase destination)
+    /// Shuts down the store bridge.
+    ///
+    /// # Errors
+    /// Currently never returns an error, but kept for future expansion.
+++++++ xmmqkotr c61a8cb9 "Fix syntax error and test issues" (rebased revision)
    /// Shuts down the store bridge.
    ///
    /// # Errors
    /// Never returns an error currently, but signature is kept for symmetry.
>>>>>>> conflict 7 of 8 ends
    pub fn shutdown(self) -> Result<(), BridgeError> {
        self.runtime.block_on(async {
<<<<<<< conflict 8 of 8
%%%%%%% diff from: kzssywqk cad5af6f "bd: backup 2026-03-10 00:19" (parents of rebased revision)
\\\\\\\        to: ozopryly fda186e0 "refactor: apply strict DDD types to store.rs (seshat-6sl)" (rebase destination)
             let mut pool_guard = self.pool.lock().await;
             if let Some(pool) = pool_guard.take() {
+                drop(pool_guard);
+++++++ xmmqkotr c61a8cb9 "Fix syntax error and test issues" (rebased revision)
            let p_opt = {
                let mut pool_guard = self.pool.lock().await;
                pool_guard.take()
            };
            if let Some(pool) = p_opt {
>>>>>>> conflict 8 of 8 ends
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

        let result = bridge.append_event_sync(envelope, None).expect("Failed to append");
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

        bridge.append_event_sync(envelope, None).expect("Failed to append");

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
            .append_batch_sync(vec![envelope1, envelope2], None)
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
