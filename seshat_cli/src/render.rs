use crate::domain::{RenderCommand, RenderSource};
use crate::error::RenderError;
use std::io::Read;
use std::path::PathBuf;

pub fn map_render_subcommand(input: Option<PathBuf>, output: PathBuf) -> RenderCommand {
    let source = input.map_or(RenderSource::Stdin, RenderSource::File);
    RenderCommand {
        input: source,
        output,
    }
}

pub fn execute_render<R: Read>(cmd: &RenderCommand, mut stdin: R) -> Result<(), RenderError> {
    // 1. Read document
    let doc = match &cmd.input {
        RenderSource::File(path) => {
            if !path.exists() {
                return Err(RenderError::FileNotFound(path.clone()));
            }
            crate::show::load_document_from_path(path).map_err(|e| match e {
                crate::error::ShowError::FileNotFound(p) => RenderError::FileNotFound(p),
                crate::error::ShowError::IoError(msg) => RenderError::IoError(msg),
                crate::error::ShowError::InvalidUtf8 => RenderError::InvalidUtf8,
                crate::error::ShowError::EmptyInput => RenderError::EmptyInput,
                crate::error::ShowError::JsonDeserialize(msg) => RenderError::JsonDeserialize(msg),
                crate::error::ShowError::InvalidDocument(_) => {
                    RenderError::JsonDeserialize("Invalid document structure".to_string())
                }
                _ => RenderError::IoError(e.to_string()),
            })?
        }
        RenderSource::Stdin => {
            crate::show::load_document_from_reader(&mut stdin).map_err(|e| match e {
                crate::error::ShowError::IoError(msg) => RenderError::IoError(msg),
                crate::error::ShowError::InvalidUtf8 => RenderError::InvalidUtf8,
                crate::error::ShowError::EmptyInput => RenderError::EmptyInput,
                crate::error::ShowError::JsonDeserialize(msg) => RenderError::JsonDeserialize(msg),
                crate::error::ShowError::InvalidDocument(_) => {
                    RenderError::JsonDeserialize("Invalid document structure".to_string())
                }
                _ => RenderError::IoError(e.to_string()),
            })?
        }
    };

    let ext = cmd
        .output
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "png" => {
            diagram_tool::export::png::export_png(&doc, &cmd.output)
                .map_err(|e| RenderError::ExportFailure(e.to_string()))?;
        }
        "svg" => {
            let svg = diagram_tool::export::svg::generate_svg_string(&doc);
            std::fs::write(&cmd.output, svg).map_err(|e| RenderError::IoError(e.to_string()))?;
        }
        other => {
            return Err(RenderError::UnsupportedFormat(other.to_string()));
        }
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use tempfile::tempdir;

    const VALID_DOC_JSON: &str = r#"{
        "version": 2,
        "revision": 0,
        "document": { "nodes": {}, "edges": {} },
        "editor_state": { "camera_x": 0.0, "camera_y": 0.0, "zoom": 1.0 }
    }"#;

    #[test]
    fn map_render_subcommand_with_input_returns_file_source() {
        let input = PathBuf::from("/tmp/input.json");
        let output = PathBuf::from("/tmp/out.svg");
        let cmd = map_render_subcommand(Some(input.clone()), output);
        assert_eq!(cmd.input, RenderSource::File(input));
    }

    #[test]
    fn map_render_subcommand_no_input_returns_stdin_source() {
        let output = PathBuf::from("/tmp/out.svg");
        let cmd = map_render_subcommand(None, output);
        assert_eq!(cmd.input, RenderSource::Stdin);
    }

    #[test]
    fn execute_render_missing_input_file() {
        let dir = tempdir().expect("tempdir");
        let missing = dir.path().join("nonexistent.json");
        let output = dir.path().join("out.svg");
        let cmd = RenderCommand {
            input: RenderSource::File(missing.clone()),
            output,
        };
        let err = execute_render(&cmd, Cursor::new("")).expect_err("should fail");
        assert!(matches!(err, RenderError::FileNotFound(p) if p == missing));
    }

    #[test]
    fn execute_render_empty_stdin() {
        let dir = tempdir().expect("tempdir");
        let output = dir.path().join("out.svg");
        let cmd = RenderCommand {
            input: RenderSource::Stdin,
            output,
        };
        let err = execute_render(&cmd, Cursor::new("")).expect_err("should fail");
        assert!(matches!(err, RenderError::EmptyInput));
    }

    #[test]
    fn execute_render_invalid_json_stdin() {
        let dir = tempdir().expect("tempdir");
        let output = dir.path().join("out.svg");
        let cmd = RenderCommand {
            input: RenderSource::Stdin,
            output,
        };
        let err = execute_render(&cmd, Cursor::new("not json at all")).expect_err("should fail");
        assert!(matches!(err, RenderError::JsonDeserialize(_)));
    }

    #[test]
    fn execute_render_unsupported_format() {
        let dir = tempdir().expect("tempdir");
        let output = dir.path().join("out.bmp");
        let cmd = RenderCommand {
            input: RenderSource::Stdin,
            output,
        };
        let err = execute_render(&cmd, Cursor::new(VALID_DOC_JSON)).expect_err("should fail");
        assert!(matches!(err, RenderError::UnsupportedFormat(ref s) if s == "bmp"));
    }

    #[test]
    fn execute_render_valid_doc_creates_svg() {
        let dir = tempdir().expect("tempdir");
        let output = dir.path().join("out.svg");
        let cmd = RenderCommand {
            input: RenderSource::Stdin,
            output: output.clone(),
        };
        execute_render(&cmd, Cursor::new(VALID_DOC_JSON)).expect("should succeed");
        let contents = std::fs::read_to_string(&output).expect("read output");
        assert!(contents.starts_with("<svg"));
        assert!(contents.contains("</svg>"));
    }

    #[test]
    fn execute_render_valid_doc_from_file_creates_svg() {
        let dir = tempdir().expect("tempdir");
        let input_path = dir.path().join("input.json");
        std::fs::write(&input_path, VALID_DOC_JSON).expect("write input");
        let output = dir.path().join("out.svg");
        let cmd = RenderCommand {
            input: RenderSource::File(input_path),
            output: output.clone(),
        };
        execute_render(&cmd, Cursor::new("")).expect("should succeed");
        let contents = std::fs::read_to_string(&output).expect("read output");
        assert!(contents.starts_with("<svg"));
        assert!(contents.contains("</svg>"));
    }
}
