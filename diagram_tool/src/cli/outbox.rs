use anyhow::{anyhow, Result};
use std::path::Path;

#[cfg(feature = "async-db")]
use tokio::runtime::Runtime;

pub fn handle_list(path: &str, limit: u32) -> Result<()> {
    #[cfg(feature = "async-db")]
    {
        let db_path = Path::new(path);
        let runtime = Runtime::new().map_err(|e| anyhow!(e.to_string()))?;
        let entries = runtime.block_on(async {
            use crate::store::durable::{
                bootstrap_durable_store, get_pending_outbox, DurableConfig,
            };
            let bootstrap = bootstrap_durable_store(db_path, DurableConfig::default())
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            let ent = get_pending_outbox(&bootstrap.pool, limit)
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            Ok::<_, anyhow::Error>(ent)
        })?;
        for entry in entries {
            println!(
                "{}",
                serde_json::json!({
                    "id": entry.id,
                    "side_effect_type": entry.side_effect_type.as_str(),
                    "status": entry.status.as_str(),
                    "retry_count": entry.retry_count,
                    "max_retries": entry.max_retries,
                    "created_at": entry.created_at,
                })
            );
        }
        Ok(())
    }
    #[cfg(not(feature = "async-db"))]
    {
        let _ = (path, limit);
        Err(anyhow!("async-db feature is required"))
    }
}

pub fn handle_add(path: &str, id: &str, side_effect_type: &str, payload: &str) -> Result<()> {
    #[cfg(feature = "async-db")]
    {
        let db_path = Path::new(path);
        let runtime = Runtime::new().map_err(|e| anyhow!(e.to_string()))?;
        runtime.block_on(async {
            use crate::store::durable::{add_outbox_entry, bootstrap_durable_store, DurableConfig};
            use crate::store::types::SideEffectType;
            use crate::store_async::fetch_latest_revision;
            let bootstrap = bootstrap_durable_store(db_path, DurableConfig::default())
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            let revision = fetch_latest_revision(&bootstrap.pool)
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            let se_type = SideEffectType::from_str(side_effect_type)
                .ok_or_else(|| anyhow!("Invalid side effect type: {}", side_effect_type))?;
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| anyhow!(e.to_string()))?
                .as_secs() as i64;
            add_outbox_entry(
                &bootstrap.pool,
                id.to_string(),
                se_type,
                payload.to_string(),
                revision,
                3,
                timestamp,
            )
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
            Ok::<(), anyhow::Error>(())
        })?;
        println!("Outbox entry {} added", id);
        Ok(())
    }
    #[cfg(not(feature = "async-db"))]
    {
        let _ = (path, id, side_effect_type, payload);
        Err(anyhow!("async-db feature is required"))
    }
}
