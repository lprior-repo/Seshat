#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use std::path::PathBuf;

use diagram_models::physical_io;
use thiserror::Error;

use crate::export::svg::generate_svg_string;

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Failed to load input file: {0}")]
    LoadFailed(#[from] physical_io::Error),

    #[error("Failed to write output file: {0}")]
    WriteFailed(#[from] std::io::Error),

    #[error("Unsupported output format")]
    UnsupportedFormat,
}

pub struct RenderCommand {
    input: PathBuf,
    output: PathBuf,
}

impl RenderCommand {
    pub fn new(input: PathBuf, output: PathBuf) -> Self {
        Self { input, output }
    }

    fn execute_inner(&self) -> Result<(), RenderError> {
        let doc = physical_io::load_document(&self.input)?;
        let output_str = self.output.to_string_lossy().to_lowercase();

        if output_str.ends_with(".svg") {
            let svg = generate_svg_string(&doc);
            std::fs::write(&self.output, svg)?;
        } else {
            return Err(RenderError::UnsupportedFormat);
        }

        Ok(())
    }
}

impl super::commands::Command for RenderCommand {
    fn name(&self) -> &'static str {
        "render"
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
    fn render_command_new_creates_command() {
        let cmd = RenderCommand::new(
            std::path::PathBuf::from("/tmp/test.json"),
            std::path::PathBuf::from("/tmp/out.svg"),
        );
        assert_eq!(cmd.name(), "render");
    }

    #[test]
    fn render_command_succeeds_and_creates_svg() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("input.json");
        let output_path = dir.path().join("output.svg");

        let doc = create_test_doc();
        physical_io::save_document(&input_path, &doc).unwrap();

        let cmd = RenderCommand::new(input_path, output_path.clone());
        let result = cmd.execute();
        assert!(
            result.is_ok(),
            "render should succeed on valid input with .svg output"
        );

        assert!(output_path.exists(), "SVG output file should be created");

        let svg_content = fs::read_to_string(&output_path).unwrap();
        assert!(
            svg_content.contains("<svg"),
            "SVG output should contain <svg tag, got: {svg_content}"
        );
    }

    #[test]
    fn render_command_fails_on_missing_input() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("nonexistent.json");
        let output_path = dir.path().join("output.svg");

        let cmd = RenderCommand::new(input_path, output_path);
        let result = cmd.execute();
        assert!(result.is_err(), "render should fail on missing input");

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Failed to load input file"),
            "error should be LoadFailed, got: {err_msg}"
        );
    }

    #[test]
    fn render_command_fails_on_unsupported_format() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("input.json");
        let output_path = dir.path().join("output.png");

        let doc = create_test_doc();
        physical_io::save_document(&input_path, &doc).unwrap();

        let cmd = RenderCommand::new(input_path, output_path);
        let result = cmd.execute();
        assert!(result.is_err(), "render should fail on unsupported format");

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Unsupported output format"),
            "error should be UnsupportedFormat, got: {err_msg}"
        );
    }

    #[test]
    fn render_command_fails_on_invalid_json() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("invalid.json");
        let output_path = dir.path().join("output.svg");

        fs::write(&input_path, "not valid json{{{").unwrap();

        let cmd = RenderCommand::new(input_path, output_path);
        let result = cmd.execute();
        assert!(result.is_err(), "render should fail on invalid JSON");
    }

    #[test]
    fn render_command_name_returns_render() {
        let cmd = RenderCommand::new(
            std::path::PathBuf::from("input.json"),
            std::path::PathBuf::from("output.svg"),
        );
        assert_eq!(cmd.name(), "render");
    }
}
