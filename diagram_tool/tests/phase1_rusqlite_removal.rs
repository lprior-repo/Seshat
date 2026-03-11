//! Phase 1 Tests: rusqlite → sqlx migration validation
//!
//! These tests verify the dependency changes required for the migration:
//! - rusqlite removed from Cargo.toml
//! - sqlx/tokio available with correct features
//! - Async store module properly integrated

use thiserror::Error;
use diagram_tool::store_async::envelope_to_valid_event;

#[derive(Debug, Error)]
pub enum Phase1Error {
    #[error("Failed to read Cargo.toml: {0}")]
    Io(#[from] std::io::Error),
    #[error("Dependency not found: {0}")]
    DependencyNotFound(String),
    #[error("Dependency found unexpectedly: {0}")]
    DependencyFound(String),
    #[error("Feature mismatch: {0}")]
    FeatureMismatch(String),
    #[error("Store error: {0}")]
    Store(String),
    #[error("Async store error: {0}")]
    AsyncStore(String),
}

type Phase1Result<T> = Result<T, Phase1Error>;

const CARGO_TOML_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");

fn read_cargo_toml() -> Phase1Result<String> {
    std::fs::read_to_string(CARGO_TOML_PATH).map_err(Phase1Error::Io)
}

fn has_dependency(content: &str, dep_name: &str) -> bool {
    let dep_pattern = format!("{} ", dep_name);
    let dep_pattern_equals = format!("{}=", dep_name);

    content.contains(&dep_pattern) || content.contains(&dep_pattern_equals)
}

fn has_feature(content: &str, package: &str, feature: &str) -> bool {
    let dep_pattern = format!("{} = {{", package);
    if let Some(start) = content.find(&dep_pattern) {
        let rest = &content[start..];
        if let Some(block_start) = rest.find('{') {
            if let Some(block_end) = rest[block_start..].find('}') {
                let block = &rest[block_start + 1..block_start + block_end];
                // Check for exact feature match or "full" meta-feature which includes all
                return block.contains(feature) || block.contains("full");
            }
        }
    }
    false
}

#[tokio::test]
async fn test_rusqlite_removed() -> Phase1Result<()> {
    let content = read_cargo_toml()?;

    if has_dependency(&content, "rusqlite") {
        return Err(Phase1Error::DependencyFound(
            "rusqlite still present in Cargo.toml".to_string(),
        ));
    }

    Ok(())
}

#[tokio::test]
async fn test_sqlx_tokio_available() -> Phase1Result<()> {
    let content = read_cargo_toml()?;

    if !has_dependency(&content, "sqlx") {
        return Err(Phase1Error::DependencyNotFound(
            "sqlx not found in dependencies".to_string(),
        ));
    }

    if !has_dependency(&content, "tokio") {
        return Err(Phase1Error::DependencyNotFound(
            "tokio not found in dependencies".to_string(),
        ));
    }

    if !has_feature(&content, "sqlx", "runtime-tokio") || !has_feature(&content, "sqlx", "sqlite") {
        return Err(Phase1Error::FeatureMismatch(
            "sqlx missing runtime-tokio or sqlite feature".to_string(),
        ));
    }

    if !has_feature(&content, "tokio", "rt") || !has_feature(&content, "tokio", "sync") {
        return Err(Phase1Error::FeatureMismatch(
            "tokio missing rt or sync feature".to_string(),
        ));
    }

    Ok(())
}

#[tokio::test]
async fn test_sqlx_tokio_non_optional() -> Phase1Result<()> {
    let content = read_cargo_toml()?;

    let sqlx_optional = content.contains("sqlx = {") && content.contains("optional = true");
    let tokio_optional = content.contains("tokio = {") && content.contains("optional = true");

    if sqlx_optional {
        return Err(Phase1Error::FeatureMismatch(
            "sqlx should not be optional".to_string(),
        ));
    }

    if tokio_optional {
        return Err(Phase1Error::FeatureMismatch(
            "tokio should not be optional".to_string(),
        ));
    }

    Ok(())
}

#[tokio::test]
async fn test_store_module_imports() -> Phase1Result<()> {
    use diagram_tool::store_async::{bootstrap_async_store, AsyncStoreError};
    use sqlx::SqlitePool;

    let _error_type = AsyncStoreError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test"));

    let _pool_type: Option<SqlitePool> = None;

    let _bootstrap_fn = bootstrap_async_store;

    Ok(())
}

#[tokio::test]
async fn test_async_bootstrap() -> Phase1Result<()> {
    use diagram_tool::store_async::{bootstrap_async_store, fetch_latest_revision};
    use sqlx::SqlitePool;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().map_err(|e| Phase1Error::Io(e))?;

    let db_path = temp_dir.path().join("test.db");

    let bootstrap = bootstrap_async_store(&db_path)
        .await
        .map_err(|e| Phase1Error::AsyncStore(e.to_string()))?;

    let pool: SqlitePool = bootstrap.pool;

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
        return Err(Phase1Error::Store("events table not created".to_string()));
    }

    let schema_table_exists: (i32,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_version'",
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| Phase1Error::AsyncStore(e.to_string()))?;

    if schema_table_exists.0 == 0 {
        return Err(Phase1Error::Store(
            "schema_version table not created".to_string(),
        ));
    }

    pool.close().await;

    Ok(())
}

#[tokio::test]
async fn test_async_append() -> Phase1Result<()> {
    use diagram_tool::models::envelope::{Author, DomainOp, EventEnvelope};
    use diagram_tool::store_async::{
        append_event_async, bootstrap_async_store, fetch_latest_revision,
    };
    use tempfile::TempDir;

    let temp_dir = TempDir::new().map_err(|e| Phase1Error::Io(e))?;

    let db_path = temp_dir.path().join("test.db");

    let bootstrap = bootstrap_async_store(&db_path)
        .await
        .map_err(|e| Phase1Error::AsyncStore(e.to_string()))?;

    let pool = bootstrap.pool;

    let envelope = EventEnvelope {
        op_id: "test-op-phase1".to_string(),
        operation: DomainOp::NodeAdd {
            id: "node-1".to_string(),
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
            label: "Test Node".to_string(),
        },
        author: Author {
            id: "user-1".to_string(),
            name: "Test User".to_string(),
            email: None,
        },
        timestamp: 1700000000,
    };

        let valid_event = envelope_to_valid_event(&envelope).map_err(|e| Phase1Error::AsyncStore(e.to_string()))?;

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

#[tokio::test]
async fn test_async_append_increments_revision() -> Phase1Result<()> {
    use diagram_tool::models::envelope::{Author, DomainOp, EventEnvelope};
    use diagram_tool::store_async::{append_event_async, fetch_latest_revision};
    use tempfile::TempDir;

    let temp_dir = TempDir::new().map_err(|e| Phase1Error::Io(e))?;

    let db_path = temp_dir.path().join("test.db");

    let bootstrap = diagram_tool::store_async::bootstrap_async_store(&db_path)
        .await
        .map_err(|e| Phase1Error::AsyncStore(e.to_string()))?;

    let pool = bootstrap.pool;

    for i in 1..=3 {
        let envelope = EventEnvelope {
            op_id: format!("test-op-{}", i),
            operation: DomainOp::NodeAdd {
                id: format!("node-{}", i),
                x: 10.0 + (i as f64) * 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                label: format!("Test Node {}", i),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000 + i as i64,
        };

    let valid_event = envelope_to_valid_event(&envelope).map_err(|e| Phase1Error::AsyncStore(e.to_string()))?;

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

#[tokio::test]
async fn test_async_bootstrap_corrupted_db() -> Phase1Result<()> {
    use diagram_tool::store_async::bootstrap_async_store;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().map_err(|e| Phase1Error::Io(e))?;

    let db_path = temp_dir.path().join("corrupted.db");

    std::fs::write(&db_path, "not a valid sqlite database").map_err(|e| Phase1Error::Io(e))?;

    let result = bootstrap_async_store(&db_path).await;

    if result.is_ok() {
        if let Ok(b) = result {
            b.pool.close().await;
        }
        return Err(Phase1Error::Store(
            "Should fail with corrupted database".to_string(),
        ));
    }

    Ok(())
}
