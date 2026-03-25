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
