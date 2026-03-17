use anyhow::{anyhow, Result};
use std::path::Path;

use crate::cli::common::load_doc;
use crate::cli_persistence::{emit_stage_event, StageDetails};

pub fn handle(input: &str) -> Result<()> {
    emit_stage_event(
        "validating",
        &StageDetails::new().with_path(Path::new(input)),
    );
    let doc = load_doc(input)?;
    let issues = diagram_models::validation::validate_document(&doc);
    if !issues.is_empty() {
        return Err(anyhow!(
            "validation failed: {}",
            issues
                .iter()
                .map(|i| format!("{}: {}", i.code, i.message))
                .collect::<Vec<_>>()
                .join("; ")
        ));
    }
    Ok(())
}
