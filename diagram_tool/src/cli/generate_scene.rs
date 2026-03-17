use anyhow::{anyhow, Result};
use std::path::Path;

use crate::cli_persistence::{emit_stage_event, save_workspace_atomic, StageDetails};

pub fn handle(nodes: u32, seed: u64, output: &str) -> Result<()> {
    emit_stage_event(
        "generating_scene",
        &StageDetails::new()
            .with_path(Path::new(output))
            .with_code("started"),
    );

    let doc = crate::perf::generate_test_scene(nodes, seed);

    save_workspace_atomic(&doc, Path::new(output))
        .map_err(|e| anyhow!("Failed to save generated scene: {e}"))?;

    emit_stage_event(
        "generated_scene",
        &StageDetails::new()
            .with_path(Path::new(output))
            .with_code("success"),
    );
    Ok(())
}
