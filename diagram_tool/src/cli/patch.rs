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
