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
