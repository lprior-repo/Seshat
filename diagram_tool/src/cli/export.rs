#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use std::path::PathBuf;

use diagram_models::physical_io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExportError {
    #[error("Failed to load input file: {0}")]
    LoadFailed(#[from] physical_io::Error),

    #[error("Failed to write output: {0}")]
    WriteFailed(#[from] std::io::Error),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

pub struct ExportCommand {
    input: PathBuf,
    format: String,
}

impl ExportCommand {
    pub fn new(input: PathBuf, format: String) -> Self {
        Self { input, format }
    }

    fn execute_inner(&self) -> Result<(), ExportError> {
        let doc = physical_io::load_document(&self.input)?;

        match self.format.as_str() {
            "json" => {
                let json = serde_json::to_string_pretty(&doc)
                    .map_err(|e| ExportError::WriteFailed(std::io::Error::other(e)))?;
                println!("{json}");
            }
            _ => return Err(ExportError::UnsupportedFormat(self.format.clone())),
        }

        Ok(())
    }
}

impl super::commands::Command for ExportCommand {
    fn name(&self) -> &'static str {
        "export"
    }

    fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.execute_inner()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use crate::cli::commands::Command;
    use diagram_models::document::{DiagramDocument, DocumentData, EditorState, Revision};
    use diagram_models::physical_io;
    use im::HashMap;
    use tempfile::tempdir;

    fn create_test_doc() -> DiagramDocument {
        DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: HashMap::new(),
                edges: HashMap::new(),
            },
            editor_state: EditorState::default(),
        }
    }

    #[test]
    fn export_command_new_creates_command() {
        let cmd = ExportCommand::new(
            std::path::PathBuf::from("/tmp/test.json"),
            "json".to_string(),
        );
        assert_eq!(cmd.name(), "export");
    }

    #[test]
    fn export_command_succeeds_on_json_format() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("valid.json");

        let doc = create_test_doc();
        physical_io::save_document(&input_path, &doc).unwrap();

        let cmd = ExportCommand::new(input_path, "json".to_string());
        let result = cmd.execute();
        assert!(result.is_ok(), "export should succeed with json format");
    }

    #[test]
    fn export_command_fails_on_missing_file() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("nonexistent.json");

        let cmd = ExportCommand::new(input_path, "json".to_string());
        let result = cmd.execute();
        assert!(result.is_err(), "export should fail on missing file");

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Failed to load input file"),
            "error should be LoadFailed, got: {err_msg}"
        );
    }

    #[test]
    fn export_command_fails_on_unsupported_format() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("valid.json");

        let doc = create_test_doc();
        physical_io::save_document(&input_path, &doc).unwrap();

        let cmd = ExportCommand::new(input_path, "csv".to_string());
        let result = cmd.execute();
        assert!(result.is_err(), "export should fail on unsupported format");

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Unsupported format"),
            "error should be UnsupportedFormat, got: {err_msg}"
        );
    }

    #[test]
    fn export_command_name_returns_export() {
        let cmd = ExportCommand::new(std::path::PathBuf::from("test.json"), "json".to_string());
        assert_eq!(cmd.name(), "export");
    }
}
