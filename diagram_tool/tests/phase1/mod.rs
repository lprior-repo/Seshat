#![cfg(not(target_arch = "wasm32"))]

use thiserror::Error;
use tempfile::TempDir;
use diagram_tool::store_async::{bootstrap_async_store, AsyncStoreBootstrap};

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

pub type Phase1Result<T> = Result<T, Phase1Error>;

const CARGO_TOML_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");

pub fn read_cargo_toml() -> Phase1Result<String> {
    std::fs::read_to_string(CARGO_TOML_PATH).map_err(Phase1Error::Io)
}

pub fn has_dependency(content: &str, dep_name: &str) -> bool {
    let dep_pattern = format!("{} ", dep_name);
    let dep_pattern_equals = format!("{}=", dep_name);

    content.contains(&dep_pattern) || content.contains(&dep_pattern_equals)
}

pub fn has_feature(content: &str, package: &str, feature: &str) -> bool {
    let dep_pattern = format!("{} = {{", package);
    if let Some(start) = content.find(&dep_pattern) {
        let rest = &content[start..];
        if let Some(block_start) = rest.find('{') {
            if let Some(block_end) = rest[block_start..].find('}') {
                let block = &rest[block_start + 1..block_start + block_end];
                return block.contains(feature) || block.contains("full");
            }
        }
    }
    false
}

pub async fn setup_test_db() -> Phase1Result<(TempDir, AsyncStoreBootstrap)> {
    let temp_dir = TempDir::new().map_err(Phase1Error::Io)?;
    let db_path = temp_dir.path().join("test.db");
    let bootstrap = bootstrap_async_store(&db_path)
        .await
        .map_err(|e| Phase1Error::AsyncStore(e.to_string()))?;
    Ok((temp_dir, bootstrap))
}
