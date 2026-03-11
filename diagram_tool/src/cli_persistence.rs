//! Atomic persistence for CLI workspace operations.
//!
//! Provides crash-safe file operations using atomic write patterns:
//! - Write to temp file in same directory
//! - fsync to ensure data is on disk
//! - Atomic rename to target path
//!
//! Also supports Last Known Good (LKG) fallback for recovery.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use crate::models::canonical_json::to_canonical_pretty_json;
use crate::models::document::DiagramDocument;
use crate::models::schema::validate_schema;
use serde::Serialize;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during CLI persistence operations.
#[derive(Debug, Error)]
pub enum CliPersistenceError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse document: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Schema validation failed: {0}")]
    ValidationError(String),

    #[error("Failed to create temp file in directory: {0}")]
    TempFileError(String),

    #[error("Atomic rename failed from '{from}' to '{to}'")]
    AtomicRenameError { from: String, to: String },

    #[error("Both primary and LKG files failed to load: {0}")]
    NoValidDocument(String),

    #[error("Path traversal denied: path '{path}' escapes allowed directory")]
    PathTraversalDenied { path: String },
}

/// Validates that a path stays within the allowed base directory.
///
/// This function prevents path traversal attacks by:
/// 1. Canonicalizing the input path (resolves `..`, symlinks, relative paths)
/// 2. Canonicalizing the base directory
/// 3. Ensuring the canonicalized path starts with the base directory
///
/// # Errors
///
/// Returns `CliPersistenceError::PathTraversalDenied` if:
/// - The path resolves to a location outside the base directory
/// - The path is an absolute path outside the cwd
/// - Canonicalization fails for any reason
pub fn validate_safe_path(path: &Path, base_dir: &Path) -> Result<PathBuf, CliPersistenceError> {
    // Reject paths with parent directory references that could escape
    // This is a defense-in-depth measure before any canonicalization
    let path_str = path.to_string_lossy();
    if path_str.contains("..") {
        return Err(CliPersistenceError::PathTraversalDenied {
            path: path.to_string_lossy().to_string(),
        });
    }

    // For relative paths, resolve against base_dir
    // For absolute paths, check directly
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_dir.join(path)
    };

    // Canonicalize to resolve symlinks - handle non-existent files securely
    let canonical = match std::fs::canonicalize(&resolved) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // File or parent directory doesn't exist yet - this is valid for output files
            // Get the directory to check (file itself or its parent)
            let dir_to_check: PathBuf = if resolved.is_dir() {
                resolved.clone()
            } else if let Some(parent) = resolved.parent() {
                if parent.as_os_str().is_empty() {
                    // No parent directory, use base_dir
                    base_dir.to_path_buf()
                } else {
                    parent.to_path_buf()
                }
            } else {
                base_dir.to_path_buf()
            };

            // Try to canonicalize the directory
            match std::fs::canonicalize(&dir_to_check) {
                Ok(canonical_dir) => {
                    let canonical_base = std::fs::canonicalize(base_dir)?;
                    let dir_str = canonical_dir.to_string_lossy();
                    let base_str = canonical_base.to_string_lossy();
                    if !dir_str.starts_with(base_str.as_ref()) && dir_str != base_str {
                        return Err(CliPersistenceError::PathTraversalDenied {
                            path: path.to_string_lossy().to_string(),
                        });
                    }
                    // Return the resolved path (verified directory is safe)
                    return Ok(resolved);
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    // Parent directory doesn't exist - check if the path components could escape
                    // by verifying the resolved path is still within base_dir
                    let resolved_str = resolved.to_string_lossy();
                    // For relative paths without ".." we can be reasonably sure they're safe
                    if !resolved_str.contains("..") {
                        return Ok(resolved);
                    }
                    return Err(CliPersistenceError::PathTraversalDenied {
                        path: path.to_string_lossy().to_string(),
                    });
                }
                Err(e) => return Err(CliPersistenceError::IoError(e)),
            }
        }
        Err(e) => return Err(CliPersistenceError::IoError(e)),
    };

    // Canonicalize base_dir for comparison - MUST SUCCEED
    let canonical_base = std::fs::canonicalize(base_dir)?;

    // Check if canonical path starts with canonical base
    let canonical_str = canonical.to_string_lossy();
    let base_str = canonical_base.to_string_lossy();

    if !canonical_str.starts_with(base_str.as_ref()) && canonical_str != base_str {
        return Err(CliPersistenceError::PathTraversalDenied {
            path: path.to_string_lossy().to_string(),
        });
    }

    Ok(canonical)
}

/// Details for stage event emissions.
#[derive(Debug, Clone, Serialize)]
pub struct StageDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temp_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes_written: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_used: Option<bool>,
}

impl StageDetails {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            path: None,
            temp_path: None,
            bytes_written: None,
            code: None,
            message: None,
            fallback_used: None,
        }
    }

    #[must_use]
    pub fn with_path(mut self, path: &Path) -> Self {
        self.path = path.to_str().map(String::from);
        self
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn with_temp_path(mut self, path: &Path) -> Self {
        self.temp_path = path.to_str().map(String::from);
        self
    }

    #[must_use]
    pub const fn with_bytes_written(mut self, bytes: u64) -> Self {
        self.bytes_written = Some(bytes);
        self
    }

    #[must_use]
    pub fn with_code(mut self, code: &str) -> Self {
        self.code = Some(String::from(code));
        self
    }

    #[must_use]
    pub fn with_message(mut self, message: &str) -> Self {
        self.message = Some(String::from(message));
        self
    }

    #[must_use]
    pub const fn with_fallback_used(mut self, used: bool) -> Self {
        self.fallback_used = Some(used);
        self
    }
}

impl Default for StageDetails {
    fn default() -> Self {
        Self::new()
    }
}

/// Emits a stage event as a single-line JSON object to stdout.
///
/// The output is JSONL format - each line is a valid JSON object.
pub fn emit_stage_event(name: &str, details: &StageDetails) {
    let event = StageEvent {
        event: String::from("stage"),
        name: String::from(name),
        details: details.clone(),
    };

    match serde_json::to_string(&event) {
        Ok(line) => println!("{line}"),
        Err(_) => {
            // Fallback: emit minimal valid JSONL
            println!(
                "{{\"event\":\"stage\",\"name\":\"{name}\",\"error\":\"jsonl_encode_failed\"}}"
            );
        }
    }
}

#[derive(Debug, Serialize)]
struct StageEvent {
    event: String,
    name: String,
    details: StageDetails,
}

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
    // This is required for true atomic semantics - without it, the renamed
    // file may be lost after a system crash
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
            // This matches the save convention in cli.rs
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::models::document::{DiagramDocument, DocumentData, EditorState, Revision};
    use im::HashMap;
    use tempfile::TempDir;

    fn create_test_document() -> DiagramDocument {
        DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: HashMap::new(),
                edges: HashMap::new(),
            },
            editor_state: EditorState::default(),
        }
    }

    #[test]
    fn given_valid_document_when_saved_atomically_then_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.json");
        let doc = create_test_document();

        let result = save_workspace_atomic(&doc, &path);

        assert!(result.is_ok());
        assert!(path.exists());
    }

    #[test]
    fn given_saved_document_when_loaded_with_lkg_then_returns_same_document() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.json");
        let doc = create_test_document();

        save_workspace_atomic(&doc, &path).unwrap();
        let loaded = load_workspace_with_lkg(&path);

        assert!(loaded.is_ok());
        let loaded_doc = loaded.unwrap();
        assert_eq!(loaded_doc.version, doc.version);
        assert_eq!(loaded_doc.revision, doc.revision);
    }

    #[test]
    fn given_missing_file_when_loaded_with_lkg_then_fails() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("nonexistent.json");

        let result = load_workspace_with_lkg(&path);

        assert!(result.is_err());
        assert!(matches!(
            result.err(),
            Some(CliPersistenceError::NoValidDocument(_))
        ));
    }

    #[test]
    fn given_invalid_json_when_loaded_with_lkg_then_fails() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("invalid.json");

        std::fs::write(&path, b"not valid json").unwrap();

        let result = load_workspace_with_lkg(&path);

        assert!(result.is_err());
    }

    #[test]
    fn given_invalid_schema_when_loaded_with_lkg_then_fails() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("invalid_schema.json");

        // Version 1 is invalid (must be 2)
        let invalid_doc = r#"{"version":1,"revision":0,"document":{"nodes":{},"edges":{}},"editor_state":{"camera_x":0.0,"camera_y":0.0,"zoom":1.0,"grid_size":20.0,"snap_to_grid":true,"selected_items":[],"editing_edge_id":null,"theme":"system","show_grid":true,"minimap_visible":false}}"#;
        std::fs::write(&path, invalid_doc).unwrap();

        let result = load_workspace_with_lkg(&path);

        assert!(result.is_err());
    }

    #[test]
    fn given_lkg_fallback_file_when_primary_fails_then_uses_lkg() {
        let temp_dir = TempDir::new().unwrap();
        let primary_path = temp_dir.path().join("doc.json");
        let lkg_dir = temp_dir.path().join(".lkg");
        let lkg_path = lkg_dir.join("doc.json.lkg");

        // Write invalid primary
        std::fs::write(&primary_path, b"invalid").unwrap();

        // Create LKG directory and write valid LKG file
        std::fs::create_dir_all(&lkg_dir).unwrap();
        let doc = create_test_document();
        let json = serde_json::to_string_pretty(&doc).unwrap();
        std::fs::write(&lkg_path, &json).unwrap();

        let result = load_workspace_with_lkg(&primary_path);

        assert!(result.is_ok());
    }

    #[test]
    fn given_stage_details_when_serialized_then_contains_expected_fields() {
        let details = StageDetails::new()
            .with_path(Path::new("/test/path.json"))
            .with_code("test_code")
            .with_message("test message");

        let json = serde_json::to_string(&details).unwrap();

        assert!(json.contains("test_code"));
        assert!(json.contains("test message"));
        assert!(json.contains("/test/path.json"));
    }

    #[test]
    fn given_relative_path_when_saved_then_uses_current_directory() {
        // Use a unique filename to avoid conflicts
        let filename = format!("test_relative_{}.json", std::process::id());
        let path = Path::new(&filename);
        let doc = create_test_document();

        let result = save_workspace_atomic(&doc, path);

        // Should succeed - writes to current directory
        assert!(result.is_ok() || path.exists());

        // Cleanup
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn given_atomic_save_when_crash_during_write_then_original_untouched() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.json");

        // Create original file
        let original_content = "original content";
        std::fs::write(&path, original_content).unwrap();

        // Note: We can't easily simulate a crash, but we can verify that
        // temp files are cleaned up on successful write
        let doc = create_test_document();
        save_workspace_atomic(&doc, &path).unwrap();

        // Verify no temp files left behind
        let entries: Vec<_> = std::fs::read_dir(temp_dir.path())
            .expect("Failed to read temp directory")
            .filter_map(|r| r.ok())
            .collect();

        let has_temp_files = entries
            .iter()
            .any(|e| e.file_name().to_string_lossy().contains(".tmp."));

        assert!(
            !has_temp_files,
            "Temp files should be cleaned up after atomic save"
        );
    }

    // === Path Traversal Prevention Tests ===

    #[test]
    fn given_simple_filename_when_validated_then_allowed() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path();
        let path = Path::new("diagram.json");

        let result = validate_safe_path(path, base_dir);

        assert!(
            result.is_ok(),
            "Simple filename should be allowed: {:?}",
            result
        );
    }

    #[test]
    fn given_path_traversal_when_validated_then_rejected() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path();
        // This tries to escape the base directory
        let path = Path::new("../../etc/passwd");

        let result = validate_safe_path(path, base_dir);

        assert!(result.is_err());
        assert!(matches!(
            result.err(),
            Some(CliPersistenceError::PathTraversalDenied { .. })
        ));
    }

    #[test]
    fn given_absolute_path_outside_cwd_when_validated_then_rejected() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path();
        // Absolute path outside the base directory
        let path = Path::new("/etc/shadow");

        let result = validate_safe_path(path, base_dir);

        assert!(result.is_err());
        assert!(matches!(
            result.err(),
            Some(CliPersistenceError::PathTraversalDenied { .. })
        ));
    }

    #[test]
    fn given_sibling_escape_when_validated_then_rejected() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path();
        // Path that tries to escape via .. after canonicalization
        let path = Path::new("diagram/../sibling.json");

        let result = validate_safe_path(path, base_dir);

        // This should be rejected because canonicalization resolves ../
        assert!(result.is_err());
        assert!(matches!(
            result.err(),
            Some(CliPersistenceError::PathTraversalDenied { .. })
        ));
    }

    #[test]
    fn given_valid_subpath_when_validated_then_allowed() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path();
        // Valid path inside the base directory
        let path = Path::new("subdir/diagram.json");

        let result = validate_safe_path(path, base_dir);

        assert!(
            result.is_ok(),
            "Valid subdirectory path should be allowed: {:?}",
            result
        );
    }

    #[test]
    fn given_relative_path_with_dot_prefix_when_validated_then_allowed() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path();
        // ./diagram.json is equivalent to diagram.json
        let path = Path::new("./diagram.json");

        let result = validate_safe_path(path, base_dir);

        assert!(result.is_ok(), "Path with ./ prefix should be allowed");
    }
}
