use anyhow::{anyhow, Result};
use std::path::Path;

#[cfg(feature = "async-db")]
use tokio::runtime::Runtime;

pub fn handle_init(path: &str) -> Result<()> {
    #[cfg(feature = "async-db")]
    {
        let db_path = Path::new(path);
        let runtime = Runtime::new().map_err(|e| anyhow!(e.to_string()))?;
        runtime.block_on(async {
            use crate::store::durable::{bootstrap_durable_store, DurableConfig};
            bootstrap_durable_store(db_path, DurableConfig::default())
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            Ok::<(), anyhow::Error>(())
        })?;
        println!("Database initialized at {}", path);
        Ok(())
    }
    #[cfg(not(feature = "async-db"))]
    {
        let _ = path;
        Err(anyhow!("async-db feature is required"))
    }
}

pub fn handle_status(path: &str) -> Result<()> {
    #[cfg(feature = "async-db")]
    {
        let db_path = Path::new(path);
        let runtime = Runtime::new().map_err(|e| anyhow!(e.to_string()))?;
        let status = runtime.block_on(async {
            use crate::store_async::{bootstrap_async_store, read_store_pragmas_async};
            let bootstrap = bootstrap_async_store(db_path)
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            let pragmas = read_store_pragmas_async(&bootstrap.pool)
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            Ok::<_, anyhow::Error>(pragmas)
        })?;
        println!(
            "{}",
            serde_json::json!({
                "path": path,
                "schema_version": 2,
                "journal_mode": status.journal_mode,
                "synchronous": status.synchronous,
                "wal_autocheckpoint": status.wal_autocheckpoint,
                "foreign_keys": status.foreign_keys,
                "busy_timeout": status.busy_timeout,
            })
        );
        Ok(())
    }
    #[cfg(not(feature = "async-db"))]
    {
        let _ = path;
        Err(anyhow!("async-db feature is required"))
    }
}

pub fn handle_revision(path: &str) -> Result<()> {
    #[cfg(feature = "async-db")]
    {
        let db_path = Path::new(path);
        let runtime = Runtime::new().map_err(|e| anyhow!(e.to_string()))?;
        let revision = runtime.block_on(async {
            use crate::store_async::{bootstrap_async_store, fetch_latest_revision};
            let bootstrap = bootstrap_async_store(db_path)
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            let rev = fetch_latest_revision(&bootstrap.pool)
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            Ok::<_, anyhow::Error>(rev)
        })?;
        println!("{}", revision);
        Ok(())
    }
    #[cfg(not(feature = "async-db"))]
    {
        let _ = path;
        Err(anyhow!("async-db feature is required"))
    }
}

pub fn handle_events(path: &str, since: i64) -> Result<()> {
    #[cfg(feature = "async-db")]
    {
        let db_path = Path::new(path);
        let runtime = Runtime::new().map_err(|e| anyhow!(e.to_string()))?;
        let events = runtime.block_on(async {
            use crate::store_async::{bootstrap_async_store, fetch_events_since};
            let bootstrap = bootstrap_async_store(db_path)
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            let evs = fetch_events_since(&bootstrap.pool, since)
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            Ok::<_, anyhow::Error>(evs)
        })?;
        for event in events {
            println!(
                "{}",
                serde_json::json!({
                    "op_id": event.op_id,
                    "revision": event.revision,
                    "timestamp": event.timestamp,
                })
            );
        }
        Ok(())
    }
    #[cfg(not(feature = "async-db"))]
    {
        let _ = (path, since);
        Err(anyhow!("async-db feature is required"))
    }
}

pub fn handle_conflict_diff(path: &str, assumed_revision: i64) -> Result<()> {
    #[cfg(feature = "async-db")]
    {
        let db_path = Path::new(path);
        let runtime = Runtime::new().map_err(|e| anyhow!(e.to_string()))?;
        let diff = runtime.block_on(async {
            use crate::store::durable::generate_conflict_diff;
            use crate::store_async::create_async_pool;
            let pool = create_async_pool(db_path)
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            let d = generate_conflict_diff(&pool, assumed_revision)
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            Ok::<_, anyhow::Error>(d)
        })?;
        println!(
            "{}",
            serde_json::json!({
                "assumed_revision": diff.assumed_revision,
                "actual_revision": diff.actual_revision,
                "changes_count": diff.changes.len(),
                "first_change_timestamp": diff.first_change_timestamp,
                "first_change_author": diff.first_change_author,
            })
        );
        Ok(())
    }
    #[cfg(not(feature = "async-db"))]
    {
        let _ = (path, assumed_revision);
        Err(anyhow!("async-db feature is required"))
    }
}
