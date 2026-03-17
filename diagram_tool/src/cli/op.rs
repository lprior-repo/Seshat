use anyhow::{anyhow, Result};
use std::path::Path;

#[cfg(feature = "async-db")]
use tokio::runtime::Runtime;

pub fn handle_start(
    path: &str,
    operation_id: &str,
    total_steps: u32,
    author_id: &str,
    description: &str,
) -> Result<()> {
    #[cfg(feature = "async-db")]
    {
        let db_path = Path::new(path);
        let runtime = Runtime::new().map_err(|e| anyhow!(e.to_string()))?;
        runtime.block_on(async {
            use crate::store::durable::{bootstrap_durable_store, start_operation, DurableConfig};
            let bootstrap = bootstrap_durable_store(db_path, DurableConfig::default())
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| anyhow!(e.to_string()))?
                .as_secs() as i64;
            start_operation(
                &bootstrap.pool,
                operation_id.to_string(),
                total_steps,
                author_id.to_string(),
                description.to_string(),
                timestamp,
            )
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
            Ok::<(), anyhow::Error>(())
        })?;
        println!("Operation {} started", operation_id);
        Ok(())
    }
    #[cfg(not(feature = "async-db"))]
    {
        let _ = (path, operation_id, total_steps, author_id, description);
        Err(anyhow!("async-db feature is required"))
    }
}

pub fn handle_status(path: &str, operation_id: &str) -> Result<()> {
    #[cfg(feature = "async-db")]
    {
        let db_path = Path::new(path);
        let runtime = Runtime::new().map_err(|e| anyhow!(e.to_string()))?;
        let op = runtime.block_on(async {
            use crate::store::durable::{bootstrap_durable_store, get_operation, DurableConfig};
            let bootstrap = bootstrap_durable_store(db_path, DurableConfig::default())
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            let o = get_operation(&bootstrap.pool, operation_id)
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            Ok::<_, anyhow::Error>(o)
        })?;
        println!(
            "{}",
            serde_json::json!({
                "operation_id": op.operation_id,
                "state": op.state.as_str(),
                "current_step": op.current_step,
                "total_steps": op.total_steps,
                "started_at": op.started_at,
                "completed_at": op.completed_at,
                "final_revision": op.final_revision,
                "error_message": op.error_message,
            })
        );
        Ok(())
    }
    #[cfg(not(feature = "async-db"))]
    {
        let _ = (path, operation_id);
        Err(anyhow!("async-db feature is required"))
    }
}

pub fn handle_list(path: &str, state: &str) -> Result<()> {
    #[cfg(feature = "async-db")]
    {
        let db_path = Path::new(path);
        let runtime = Runtime::new().map_err(|e| anyhow!(e.to_string()))?;
        let ops = runtime.block_on(async {
            use crate::store::durable::{
                bootstrap_durable_store, get_operations_by_state, DurableConfig,
            };
            use crate::store::types::OperationState;
            let bootstrap = bootstrap_durable_store(db_path, DurableConfig::default())
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            let op_state = OperationState::from_str(state)
                .ok_or_else(|| anyhow!("Invalid state: {}", state))?;
            let o = get_operations_by_state(&bootstrap.pool, op_state)
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            Ok::<_, anyhow::Error>(o)
        })?;
        for op in ops {
            println!(
                "{}",
                serde_json::json!({
                    "operation_id": op.operation_id,
                    "state": op.state.as_str(),
                    "current_step": op.current_step,
                    "total_steps": op.total_steps,
                })
            );
        }
        Ok(())
    }
    #[cfg(not(feature = "async-db"))]
    {
        let _ = (path, state);
        Err(anyhow!("async-db feature is required"))
    }
}

pub fn handle_complete(path: &str, operation_id: &str) -> Result<()> {
    #[cfg(feature = "async-db")]
    {
        let db_path = Path::new(path);
        let runtime = Runtime::new().map_err(|e| anyhow!(e.to_string()))?;
        runtime.block_on(async {
            use crate::store::durable::{
                bootstrap_durable_store, update_operation_state, DurableConfig,
            };
            use crate::store::types::OperationState;
            use crate::store_async::fetch_latest_revision;
            let bootstrap = bootstrap_durable_store(db_path, DurableConfig::default())
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            let revision = fetch_latest_revision(&bootstrap.pool)
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            update_operation_state(
                &bootstrap.pool,
                operation_id,
                OperationState::Completed,
                None,
                Some(revision),
                None,
            )
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
            Ok::<(), anyhow::Error>(())
        })?;
        println!("Operation {} completed", operation_id);
        Ok(())
    }
    #[cfg(not(feature = "async-db"))]
    {
        let _ = (path, operation_id);
        Err(anyhow!("async-db feature is required"))
    }
}

pub fn handle_fail(path: &str, operation_id: &str, error: &str) -> Result<()> {
    #[cfg(feature = "async-db")]
    {
        let db_path = Path::new(path);
        let runtime = Runtime::new().map_err(|e| anyhow!(e.to_string()))?;
        runtime.block_on(async {
            use crate::store::durable::{
                bootstrap_durable_store, update_operation_state, DurableConfig,
            };
            use crate::store::types::OperationState;
            let bootstrap = bootstrap_durable_store(db_path, DurableConfig::default())
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            update_operation_state(
                &bootstrap.pool,
                operation_id,
                OperationState::Failed,
                None,
                None,
                Some(error.to_string()),
            )
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
            Ok::<(), anyhow::Error>(())
        })?;
        println!("Operation {} failed: {}", operation_id, error);
        Ok(())
    }
    #[cfg(not(feature = "async-db"))]
    {
        let _ = (path, operation_id, error);
        Err(anyhow!("async-db feature is required"))
    }
}
