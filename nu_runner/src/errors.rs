//! Domain error types.
//
//! Explicit error taxonomy following DDD principles.

use std::path::PathBuf;

use crate::newtypes::CommandError;
use crate::state::RunnerStateError;

// =============================================================================
// DOMAIN ERRORS - Explicit error taxonomy
// =============================================================================

/// The six error variants representing all possible failure modes.
///
/// This is the main error type that users of the library will handle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NuError {
    /// Empty or whitespace-only command string
    InvalidCommand(String),
    /// Working directory does not exist
    WorkingDirectoryNotFound(PathBuf),
    /// Command exceeded the configured timeout
    Timeout { command: String, duration_ms: u64 },
    /// Nushell executable not found in PATH
    ExecutableNotFound,
    /// Command failed with non-zero exit code (includes stderr output)
    CommandFailed { code: i32, stderr: String },
    /// I/O error (permissions, etc.)
    IoError(String),
    /// Runner is in invalid state for the requested operation
    InvalidState(RunnerStateError),
}

impl From<CommandError> for NuError {
    fn from(err: CommandError) -> Self {
        match err {
            CommandError::EmptyCommand => Self::InvalidCommand(String::new()),
        }
    }
}

impl From<RunnerStateError> for NuError {
    fn from(err: RunnerStateError) -> Self {
        Self::InvalidState(err)
    }
}

impl std::fmt::Display for NuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCommand(cmd) => write!(f, "Invalid command: '{cmd}'"),
            Self::WorkingDirectoryNotFound(path) => {
                write!(f, "Working directory not found: {}", path.display())
            }
            Self::Timeout {
                command,
                duration_ms,
            } => write!(f, "Command timed out after {duration_ms}ms: '{command}'"),
            Self::ExecutableNotFound => write!(f, "Nushell executable not found in PATH"),
            Self::CommandFailed { code, stderr } => {
                write!(f, "Command failed with exit code {code}: {stderr}")
            }
            Self::IoError(msg) => write!(f, "I/O error: {msg}"),
            Self::InvalidState(err) => write!(f, "Invalid state: {err}"),
        }
    }
}

impl std::error::Error for NuError {}
