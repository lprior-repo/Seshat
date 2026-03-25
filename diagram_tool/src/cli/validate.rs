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
pub enum ValidateError {
    #[error("Failed to load input file: {0}")]
    LoadFailed(#[from] physical_io::Error),
}

pub struct ValidateCommand {
    input: PathBuf,
}

impl ValidateCommand {
    pub fn new(input: PathBuf) -> Self {
        Self { input }
    }

    fn execute_inner(&self) -> Result<(), ValidateError> {
        let _doc = physical_io::load_document(&self.input)?;
        Ok(())
    }
}

impl super::commands::Command for ValidateCommand {
    fn name(&self) -> &'static str {
        "validate"
    }

    fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.execute_inner()?;
        Ok(())
    }
}
