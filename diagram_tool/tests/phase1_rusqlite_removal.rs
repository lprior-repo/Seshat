#![allow(clippy::io_other_error, clippy::unnecessary_cast)]
#![cfg(not(target_arch = "wasm32"))]

mod phase1;

use diagram_models::document::types::NodeId;
use diagram_models::envelope::{Author, DomainOp, EventEnvelope};
use diagram_tool::store_async::{
    append_event_async, envelope_to_valid_event, fetch_latest_revision,
};
use phase1::{
    has_dependency, has_feature, read_cargo_toml, setup_test_db, Phase1Error, Phase1Result,
};
use tempfile::TempDir;

#[cfg_attr(kani, kani::proof)]
async fn test_rusqlite_removed() -> Phase1Result<()> {
    let content = read_cargo_toml()?;
    if has_dependency(&content, "rusqlite") {
        return Err(Phase1Error::DependencyFound(
            "rusqlite still present in Cargo.toml".into(),
        ));
    }
    Ok(())
}

#[cfg_attr(kani, kani::proof)]
async fn test_sqlx_tokio_available() -> Phase1Result<()> {
    let content = read_cargo_toml()?;
    if !has_dependency(&content, "sqlx") {
        return Err(Phase1Error::DependencyNotFound("sqlx not found".into()));
    }
    if !has_dependency(&content, "tokio") {
        return Err(Phase1Error::DependencyNotFound("tokio not found".into()));
    }
    if !has_feature(&content, "sqlx", "runtime-tokio") || !has_feature(&content, "sqlx", "sqlite") {
        return Err(Phase1Error::FeatureMismatch("sqlx missing features".into()));
    }
    if !has_feature(&content, "tokio", "rt") || !has_feature(&content, "tokio", "sync") {
        return Err(Phase1Error::FeatureMismatch(
            "tokio missing features".into(),
        ));
    }
    Ok(())
}

#[cfg_attr(kani, kani::proof)]
async fn test_sqlx_tokio_non_optional() -> Phase1Result<()> {
    let content = read_cargo_toml()?;
    if content.contains("sqlx = {") && content.contains("optional = true") {
        return Err(Phase1Error::FeatureMismatch(
            "sqlx should not be optional".into(),
        ));
    }
    if content.contains("tokio = {") && content.contains("optional = true") {
        return Err(Phase1Error::FeatureMismatch(
            "tokio should not be optional".into(),
        ));
    }
    Ok(())
}

#[cfg_attr(kani, kani::proof)]
async fn test_store_module_imports() -> Phase1Result<()> {
    use diagram_tool::store_async::{bootstrap_async_store, AsyncStoreError};
    use sqlx::SqlitePool;
    let _error_type = AsyncStoreError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test"));
    let _pool_type: Option<SqlitePool> = None;
    let _bootstrap_fn = bootstrap_async_store;
    Ok(())
}

#[cfg_attr(kani, kani::proof)]
async fn test_async_bootstrap() -> Phase1Result<()> {
    let (_temp_dir, bootstrap) = setup_test_db().await?;
    let pool = bootstrap.pool;

    if bootstrap.schema_version != 1 {
        return Err(Phase1Error::Store(format!(
            "Expected schema version 1, got {}",
            bootstrap.schema_version
        )));
    }

    let revision = fetch_latest_revision(&pool)
        .await
        .map_err(|e| Phase1Error::AsyncStore(e.to_string()))?;
    if revision != 0 {
        return Err(Phase1Error::Store(format!(
            "Expected initial revision 0, got {}",
            revision
        )));
    }

    let table_exists: (i32,) =
        sqlx::query_as("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='events'")
            .fetch_one(&pool)
            .await
            .map_err(|e| Phase1Error::AsyncStore(e.to_string()))?;
    if table_exists.0 == 0 {
        return Err(Phase1Error::Store("events table not created".into()));
    }

    let schema_table_exists: (i32,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_version'",
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| Phase1Error::AsyncStore(e.to_string()))?;
    if schema_table_exists.0 == 0 {
        return Err(Phase1Error::Store(
            "schema_version table not created".into(),
        ));
    }

    pool.close().await;
    Ok(())
}

#[cfg_attr(kani, kani::proof)]
async fn test_async_append() -> Phase1Result<()> {
    let (_temp_dir, bootstrap) = setup_test_db().await?;
    let pool = bootstrap.pool;

    let envelope = EventEnvelope {
        op_id: "test-op-phase1".into(),
        operation: DomainOp::NodeAdd {
            id: NodeId::new("node-1".into()),
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
            label: "Test Node".into(),
        },
        author: Author {
            id: "user-1".into(),
            name: "Test User".into(),
            email: None,
        },
        timestamp: 1700000000,
    };

    let valid_event =
        envelope_to_valid_event(&envelope).map_err(|e| Phase1Error::AsyncStore(e.to_string()))?;
    let result = append_event_async(&pool, valid_event, None)
        .await
        .map_err(|e| Phase1Error::AsyncStore(e.to_string()))?;

    if result.revision != 1 {
        return Err(Phase1Error::Store(format!(
            "Expected revision 1, got {}",
            result.revision
        )));
    }
    if result.op_id != "test-op-phase1" {
        return Err(Phase1Error::Store(format!(
            "Expected op_id 'test-op-phase1', got '{}'",
            result.op_id
        )));
    }

    let latest_revision = fetch_latest_revision(&pool)
        .await
        .map_err(|e| Phase1Error::AsyncStore(e.to_string()))?;
    if latest_revision != 1 {
        return Err(Phase1Error::Store(format!(
            "Expected latest revision 1, got {}",
            latest_revision
        )));
    }

    pool.close().await;
    Ok(())
}

#[cfg_attr(kani, kani::proof)]
async fn test_async_append_increments_revision() -> Phase1Result<()> {
    let (_temp_dir, bootstrap) = setup_test_db().await?;
    let pool = bootstrap.pool;

    // Precondition: ceiling is 3
    for i in 1..=3 {
        let envelope = EventEnvelope {
            op_id: format!("test-op-{}", i),
            operation: DomainOp::NodeAdd {
                id: NodeId::new(format!("node-{}", i)),
                x: 10.0 + (i as f64) * 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: format!("Test Node {}", i),
            },
            author: Author {
                id: "user-1".into(),
                name: "Test User".into(),
                email: None,
            },
            timestamp: 1700000000 + i as i64,
        };

        let valid_event = envelope_to_valid_event(&envelope)
            .map_err(|e| Phase1Error::AsyncStore(e.to_string()))?;
        let result = append_event_async(&pool, valid_event, None)
            .await
            .map_err(|e| Phase1Error::AsyncStore(e.to_string()))?;

        if result.revision != i {
            return Err(Phase1Error::Store(format!(
                "Expected revision {}, got {}",
                i, result.revision
            )));
        }
    }

    let latest_revision = fetch_latest_revision(&pool)
        .await
        .map_err(|e| Phase1Error::AsyncStore(e.to_string()))?;
    if latest_revision != 3 {
        return Err(Phase1Error::Store(format!(
            "Expected final revision 3, got {}",
            latest_revision
        )));
    }

    pool.close().await;
    Ok(())
}

#[cfg_attr(kani, kani::proof)]
async fn test_async_bootstrap_corrupted_db() -> Phase1Result<()> {
    use diagram_tool::store_async::bootstrap_async_store;
    let temp_dir = TempDir::new().map_err(Phase1Error::Io)?;
    let db_path = temp_dir.path().join("corrupted.db");

    std::fs::write(&db_path, "not a valid sqlite database").map_err(Phase1Error::Io)?;

    let result = bootstrap_async_store(&db_path).await;
    if let Ok(b) = result {
        b.pool.close().await;
        return Err(Phase1Error::Store(
            "Should fail with corrupted database".into(),
        ));
    }
    Ok(())
}
