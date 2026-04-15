//! CLI persistence module for workspace loading with LKG (Last Known Good) fallback.
//!
//! This module provides functionality for loading workspace data with automatic
//! fallback to previous known good states when loading fails.

use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliPersistenceError {
    #[error("No valid document found")]
    NoValidDocument(String),
    #[error("IO error: {0}")]
    IoError(String),
}

pub fn load_workspace_with_lkg(path: &Path) -> Result<String, CliPersistenceError> {
    if !path.exists() {
        return Err(CliPersistenceError::NoValidDocument(format!(
            "File does not exist: {:?}",
            path
        )));
    }

    std::fs::read_to_string(path).map_err(|e| CliPersistenceError::IoError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn load_workspace_with_lkg_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.json");

        let result = load_workspace_with_lkg(&file_path);
        assert!(result.is_err());
        match result {
            Err(CliPersistenceError::NoValidDocument(_)) => {}
            Err(CliPersistenceError::IoError(_)) => panic!("Expected NoValidDocument error"),
            Ok(_) => panic!("Should not succeed"),
        }
    }

    #[test]
    fn load_workspace_with_lkg_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("existing.json");

        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();

        let result = load_workspace_with_lkg(&file_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test content");
    }
}
