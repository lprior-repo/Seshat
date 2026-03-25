use crate::cli_persistence::{emit_stage_event, CliPersistenceError, StageDetails};
use diagram_models::document::DiagramDocument;
use diagram_models::schema::validate_schema;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Loads a workspace document with Last Known Good (LKG) fallback.
///
/// This function:
/// 1. Attempts to load and validate the primary file
/// 2. On failure, attempts to load from `.lkg/<filename>.lkg` as fallback
/// 3. Returns the first successfully loaded and validated document
///
/// # Errors
///
/// Returns `CliPersistenceError::NoValidDocument` if both primary and LKG
/// files fail to load or validate.
pub fn load_workspace_with_lkg(path: &Path) -> Result<DiagramDocument, CliPersistenceError> {
    // Try primary file first
    match load_and_validate(path) {
        Ok(doc) => {
            emit_stage_event(
                "loaded",
                &StageDetails::new()
                    .with_path(path)
                    .with_fallback_used(false),
            );
            Ok(doc)
        }
        Err(primary_err) => {
            // Emit validation error event
            emit_stage_event(
                "validating",
                &StageDetails::new()
                    .with_path(path)
                    .with_code("validation_failed")
                    .with_message(&primary_err.to_string()),
            );

            // LKG is stored in `.lkg/` directory next to the original file
            let lkg_dir = path.parent().unwrap_or_else(|| Path::new(".")).join(".lkg");
            let lkg_filename = format!(
                "{}.lkg",
                path.file_name()
                    .map(|n| n.to_string_lossy())
                    .unwrap_or_default()
            );
            let lkg_path = lkg_dir.join(&lkg_filename);

            if let Ok(doc) = load_and_validate(&lkg_path) {
                emit_stage_event(
                    "loaded",
                    &StageDetails::new()
                        .with_path(&lkg_path)
                        .with_fallback_used(true),
                );
                return Ok(doc);
            }

            // Both failed
            emit_stage_event(
                "error",
                &StageDetails::new()
                    .with_path(path)
                    .with_code("no_valid_document")
                    .with_message("Both primary and LKG files failed to load"),
            );

            Err(CliPersistenceError::NoValidDocument(
                primary_err.to_string(),
            ))
        }
    }
}

/// Loads and validates a document from the given path.
fn load_and_validate(path: &Path) -> Result<DiagramDocument, CliPersistenceError> {
    let file = File::open(path)?;
    let doc: DiagramDocument = serde_json::from_reader(BufReader::new(file))?;

    // Validate schema
    validate_schema(&doc).map_err(|e| CliPersistenceError::ValidationError(e.to_string()))?;

    Ok(doc)
}
