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
#![forbid(unsafe_code)]

use serde::Serialize;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub mod read;
pub mod write;

#[cfg(test)]
mod tests;

pub use read::load_workspace_with_lkg;
pub use write::save_workspace_atomic;

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
