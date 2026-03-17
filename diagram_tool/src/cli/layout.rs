use anyhow::{anyhow, Result};
use std::path::Path;

use crate::cli::common::load_doc;
use crate::cli_persistence::{emit_stage_event, save_workspace_atomic, StageDetails};
use crate::mutation::{ops::apply_layout, pipeline::run_mutation};

pub fn handle(input: &str, output: &str) -> Result<()> {
    emit_stage_event(
        "validating",
        &StageDetails::new().with_path(Path::new(input)),
    );
    let doc = load_doc(input)?;
    let laid_out_doc = run_mutation(&doc, |current| Ok(apply_layout(current, 200.0)))
        .map_err(|err| anyhow!(err.to_string()))?;
    save_workspace_atomic(&laid_out_doc, Path::new(output))
        .map_err(|e| anyhow!("Failed to save workspace: {e}"))?;
    Ok(())
}
