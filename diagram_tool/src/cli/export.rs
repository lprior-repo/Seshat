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
