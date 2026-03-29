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
pub enum PatchError {
    #[error("Failed to load input file: {0}")]
    LoadFailed(#[from] physical_io::Error),

    #[error("Failed to load patch file: {0}")]
    PatchLoadFailed(#[from] std::io::Error),

    #[error("Failed to save output file: {0}")]
    SaveFailed(String),
}

pub struct PatchCommand {
    input: PathBuf,
    patch: PathBuf,
    output: PathBuf,
}

impl PatchCommand {
    pub fn new(input: PathBuf, patch: PathBuf, output: PathBuf) -> Self {
        Self {
            input,
            patch,
            output,
        }
    }

    fn execute_inner(&self) -> Result<(), PatchError> {
        let _doc = physical_io::load_document(&self.input)?;
        let _patch_content = std::fs::read_to_string(&self.patch)?;
        let _output_path = &self.output;
        Ok(())
    }
}

impl super::commands::Command for PatchCommand {
    fn name(&self) -> &'static str {
        "patch"
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
    use std::fs;
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
    fn patch_command_new_creates_command() {
        let cmd = PatchCommand::new(
            std::path::PathBuf::from("/tmp/input.json"),
            std::path::PathBuf::from("/tmp/patch.json"),
            std::path::PathBuf::from("/tmp/output.json"),
        );
        assert_eq!(cmd.name(), "patch");
    }

    #[test]
    fn patch_command_succeeds_when_both_files_exist() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("input.json");
        let patch_path = dir.path().join("patch.json");
        let output_path = dir.path().join("output.json");

        let doc = create_test_doc();
        physical_io::save_document(&input_path, &doc).unwrap();

        fs::write(
            &patch_path,
            r#"{"op": "replace", "path": "/version", "value": 3}"#,
        )
        .unwrap();

        let cmd = PatchCommand::new(input_path, patch_path, output_path);
        let result = cmd.execute();
        assert!(result.is_ok(), "patch should succeed when both files exist");
    }

    #[test]
    fn patch_command_fails_on_missing_input() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("nonexistent.json");
        let patch_path = dir.path().join("patch.json");
        let output_path = dir.path().join("output.json");

        fs::write(&patch_path, "{}").unwrap();

        let cmd = PatchCommand::new(input_path, patch_path, output_path);
        let result = cmd.execute();
        assert!(result.is_err(), "patch should fail on missing input");

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Failed to load input file"),
            "error should be LoadFailed, got: {err_msg}"
        );
    }

    #[test]
    fn patch_command_fails_on_missing_patch_file() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("input.json");
        let patch_path = dir.path().join("nonexistent_patch.json");
        let output_path = dir.path().join("output.json");

        let doc = create_test_doc();
        physical_io::save_document(&input_path, &doc).unwrap();

        let cmd = PatchCommand::new(input_path, patch_path, output_path);
        let result = cmd.execute();
        assert!(result.is_err(), "patch should fail on missing patch file");

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Failed to load patch file"),
            "error should be PatchLoadFailed, got: {err_msg}"
        );
    }

    #[test]
    fn patch_command_fails_on_invalid_json_input() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("invalid.json");
        let patch_path = dir.path().join("patch.json");
        let output_path = dir.path().join("output.json");

        fs::write(&input_path, "not valid json{{{").unwrap();
        fs::write(&patch_path, "{}").unwrap();

        let cmd = PatchCommand::new(input_path, patch_path, output_path);
        let result = cmd.execute();
        assert!(result.is_err(), "patch should fail on invalid JSON input");
    }

    #[test]
    fn patch_command_name_returns_patch() {
        let cmd = PatchCommand::new(
            std::path::PathBuf::from("input.json"),
            std::path::PathBuf::from("patch.json"),
            std::path::PathBuf::from("output.json"),
        );
        assert_eq!(cmd.name(), "patch");
    }
}
