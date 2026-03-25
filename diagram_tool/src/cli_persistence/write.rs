use crate::cli_persistence::{emit_stage_event, CliPersistenceError, StageDetails};
use diagram_models::canonical_json::to_canonical_pretty_json;
use diagram_models::document::DiagramDocument;
use diagram_models::schema::validate_schema;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

/// Atomically saves a workspace document to the specified path.
///
/// This function uses the atomic write pattern:
/// 1. Write to a temp file in the same directory as the target
/// 2. Sync the temp file to disk (fsync)
/// 3. Atomically rename temp file to target path
///
/// This ensures that:
/// - If the process crashes during write, the original file is untouched
/// - The file is either fully written or not written at all
/// - No partial/corrupted files are left behind
///
/// # Errors
///
/// Returns `CliPersistenceError` if any step fails:
/// - `TempFileError` if temp file cannot be created
/// - `IoError` if write or sync fails
/// - `AtomicRenameError` if rename fails
pub fn save_workspace_atomic(
    doc: &DiagramDocument,
    path: &Path,
) -> Result<(), CliPersistenceError> {
    // Validate before persistence - run the full validation pipeline
    validate_schema(doc).map_err(|e| CliPersistenceError::ValidationError(e.to_string()))?;

    // Get parent directory, defaulting to current directory for relative paths
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty());
    let parent = parent.unwrap_or_else(|| Path::new("."));

    // Create temp file in same directory for atomic rename
    let temp_path = parent.join(format!(
        ".{}.tmp.{}",
        path.file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default(),
        std::process::id()
    ));

    // Write to temp file
    let temp_file = File::create(&temp_path).map_err(|e| {
        CliPersistenceError::TempFileError(format!(
            "Failed to create temp file at {}: {}",
            temp_path.display(),
            e
        ))
    })?;

    let mut writer = BufWriter::new(temp_file);

    let json_content = to_canonical_pretty_json(doc)?;
    writer.write_all(json_content.as_bytes())?;
    writer.flush()?;

    // fsync to ensure data is on disk
    let file = writer
        .into_inner()
        .map_err(|e| CliPersistenceError::IoError(e.into_error()))?;
    file.sync_all()?;

    // Atomic rename
    fs::rename(&temp_path, path).map_err(|_| CliPersistenceError::AtomicRenameError {
        from: temp_path.display().to_string(),
        to: path.display().to_string(),
    })?;

    // Sync parent directory to ensure rename is durable
    if let Some(parent_dir) = path.parent() {
        let dir_file = std::fs::OpenOptions::new()
            .read(true)
            .open(parent_dir)
            .map_err(CliPersistenceError::IoError)?;
        dir_file.sync_all().map_err(CliPersistenceError::IoError)?;
    }

    // Emit success event
    emit_stage_event(
        "persisted",
        &StageDetails::new()
            .with_path(path)
            .with_bytes_written(json_content.len() as u64),
    );

    Ok(())
}
