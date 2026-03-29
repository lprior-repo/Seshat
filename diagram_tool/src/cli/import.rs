#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

use std::path::PathBuf;

use diagram_models::physical_io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImportError {
    #[error("Failed to load input file: {0}")]
    LoadFailed(#[from] physical_io::Error),
}

pub struct ImportCommand {
    input: PathBuf,
}

impl ImportCommand {
    pub fn new(input: PathBuf) -> Self {
        Self { input }
    }

    fn execute_inner(&self) -> Result<(), ImportError> {
        let _doc = physical_io::load_document(&self.input)?;
        Ok(())
    }
}

impl super::commands::Command for ImportCommand {
    fn name(&self) -> &'static str {
        "import"
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
    fn import_command_new_creates_command() {
        let cmd = ImportCommand::new(std::path::PathBuf::from("/tmp/test.json"));
        assert_eq!(cmd.name(), "import");
    }

    #[test]
    fn import_command_succeeds_on_valid_document() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("valid.json");

        let doc = create_test_doc();
        physical_io::save_document(&input_path, &doc).unwrap();

        let cmd = ImportCommand::new(input_path);
        let result = cmd.execute();
        assert!(result.is_ok(), "import should succeed on valid document");
    }

    #[test]
    fn import_command_fails_on_missing_file() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("nonexistent.json");

        let cmd = ImportCommand::new(input_path);
        let result = cmd.execute();
        assert!(result.is_err(), "import should fail on missing file");

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Failed to load input file"),
            "error should be LoadFailed, got: {err_msg}"
        );
    }

    #[test]
    fn import_command_name_returns_import() {
        let cmd = ImportCommand::new(std::path::PathBuf::from("test.json"));
        assert_eq!(cmd.name(), "import");
    }
}
