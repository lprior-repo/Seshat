//! Validation functions (Calculations layer).
//!
//! This module contains pure functions for input validation.
//! Following the "Parse, don't validate" principle - these functions
//! convert raw inputs into validated domain types at the boundary.

use std::path::PathBuf;

use crate::errors::NuError;
use crate::newtypes::{Command, EnvVars, ExitCode, NuPath, NuPathError, TimeoutMs, TimeoutMsError};
use crate::types::NuConfig;

// =============================================================================
// COMMAND VALIDATION
// =============================================================================

/// Validates that a command string is non-empty (not whitespace only).
///
/// # Precondition (P1)
/// - Command must be non-empty after trimming whitespace
///
/// # Returns
/// - `Ok(Command)` - validated command newtype
/// - `Err(NuError::InvalidCommand)` - if command is empty/whitespace
pub fn validate_command(command: &str) -> Result<Command, NuError> {
    Command::new(command).map_err(NuError::from)
}

/// Parses a raw command string into a validated `Command`.
///
/// Alias for `validate_command` for semantic clarity at boundaries.
pub fn parse_command(command: &str) -> Result<Command, NuError> {
    validate_command(command)
}

// =============================================================================
// WORKING DIRECTORY VALIDATION
// =============================================================================

/// Validates that the working directory exists (if provided).
///
/// # Precondition (P2)
/// - If cwd is Some(path), path must exist
///
/// # Returns
/// - `Ok(())` - directory is valid or not set
/// - `Err(NuError::WorkingDirectoryNotFound)` - if directory doesn't exist
pub fn validate_working_directory(cwd: Option<&PathBuf>) -> Result<(), NuError> {
    match cwd {
        Some(path) if !path.exists() => Err(NuError::WorkingDirectoryNotFound(path.clone())),
        _ => Ok(()),
    }
}

/// Validates a PathBuf as a working directory.
///
/// Alias for `validate_working_directory` for semantic clarity.
pub fn parse_working_directory(cwd: Option<PathBuf>) -> Result<Option<PathBuf>, NuError> {
    if let Some(ref path) = cwd {
        validate_working_directory(Some(path))?;
    }
    Ok(cwd)
}

// =============================================================================
// TIMEOUT VALIDATION
// =============================================================================

/// Validates that timeout is greater than zero.
///
/// # Returns
/// - `Ok(TimeoutMs)` - validated timeout newtype
/// - `Err(NuError::IoError)` - if timeout is zero
pub fn validate_timeout(timeout_ms: u64) -> Result<TimeoutMs, NuError> {
    TimeoutMs::new(timeout_ms).map_err(|e| match e {
        TimeoutMsError::ZeroTimeout => {
            NuError::IoError("timeout must be greater than zero".to_string())
        }
    })
}

/// Parses a u64 into a validated `TimeoutMs`.
///
/// Alias for `validate_timeout` for semantic clarity.
pub fn parse_timeout(timeout_ms: u64) -> Result<TimeoutMs, NuError> {
    validate_timeout(timeout_ms)
}

// =============================================================================
// ENVIRONMENT VARIABLES VALIDATION
// =============================================================================

/// Validates all environment variables are valid UTF-8.
///
/// Since we use `EnvVars` (which wraps `HashMap<String, String>`),
/// UTF-8 validation is guaranteed at compile time.
///
/// This function exists for explicitness and future extensibility.
pub const fn validate_env_vars(_env: &EnvVars) -> Result<(), NuError> {
    // `EnvVars` guarantees UTF-8 at compile time via `HashMap<String, String>`
    Ok(())
}

// =============================================================================
// NUSHELL PATH VALIDATION
// =============================================================================

/// Validates the nushell executable path is not empty.
///
/// # Returns
/// - `Ok(NuPath)` - validated path newtype
/// - `Err(NuError::IoError)` - if path is empty
pub fn validate_nu_path(path: &str) -> Result<NuPath, NuError> {
    NuPath::new(path).map_err(|e| match e {
        NuPathError::EmptyPath => NuError::IoError("nu_path cannot be empty".to_string()),
    })
}

/// Parses a string into a validated `NuPath`.
///
/// Alias for `validate_nu_path` for semantic clarity.
pub fn parse_nu_path(path: &str) -> Result<NuPath, NuError> {
    validate_nu_path(path)
}

// =============================================================================
// CONFIGURATION VALIDATION
// =============================================================================

/// Validates all configuration preconditions at once.
///
/// This is the main entry point for validating the runner configuration
/// before execution.
///
/// # Preconditions
/// - P1: Command must be non-empty
/// - P2: Working directory must exist (if set)
/// - P3: Environment variables must be valid UTF-8 (guaranteed by type)
///
/// # Returns
/// - `Ok(())` - all preconditions met
/// - `Err(NuError)` - any precondition failure
pub fn validate_config(config: &NuConfig, command: &str) -> Result<Command, NuError> {
    // P1: Validate command
    let validated_command = validate_command(command)?;

    // P2: Validate working directory
    validate_working_directory(config.cwd())?;

    // P3: Validate env vars (compile-time guaranteed, explicit for contract)
    validate_env_vars(config.env())?;

    // Validate timeout - just access it to ensure it's valid
    let _ = config.timeout_ms();

    Ok(validated_command)
}

// =============================================================================
// OUTPUT PARSING
// =============================================================================

/// Parses the raw process output into `NuOutput`.
///
/// Note: This does NOT validate exit codes - non-zero exit codes are
/// valid outcomes, not errors. The caller must decide whether to treat
/// a non-zero exit code as an error.
pub fn parse_output(output: &std::process::Output) -> crate::types::NuOutput {
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Exit code, or -1 if unavailable (e.g., process killed by signal).
    // When `ProcessOutput::status.code()` returns `None` (which happens on
    // abnormal termination like signal kills), the code falls back to -1.
    let exit_code = output.status.code().unwrap_or(-1);

    // Q2: Exit code reflected in output
    // Note: Non-zero exit codes are NOT errors - they're in the output
    crate::types::NuOutput {
        stdout,
        stderr,
        exit_code: ExitCode::new(exit_code),
    }
}
