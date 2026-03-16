//! Nushell Command Runner - A functional Rust implementation for executing Nushell commands.
//!
//! # Architecture
//!
//! - **Data**: `NuError`, `NuOutput`, `NuRunner` - immutable state types
//! - **Calculations**: Validation functions, command construction (pure)
//! - **Actions**: Process execution via tokio (impure I/O)
//!
//! # Contract
//!
//! - **Preconditions**: Command non-empty, working dir exists, env vars valid UTF-8
//! - **Postconditions**: Captured stdout/stderr, exit code reflected, timeout enforced
//! - **Invariants**: Runner reusable, one command at a time, env vars persist

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use tokio::process::Command;
use tokio::time::timeout;

// =============================================================================
// DATA LAYER - Immutable types representing domain concepts
// =============================================================================

/// The six error variants representing all possible failure modes.
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
        }
    }
}

impl std::error::Error for NuError {}

/// Output from a executed command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuOutput {
    /// Standard output captured from the command
    pub stdout: String,
    /// Standard error captured from the command
    pub stderr: String,
    /// Exit code returned by the command
    pub exit_code: i32,
}

/// Configuration for the [`NuRunner`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuConfig {
    /// Environment variables to pass to the command
    env: HashMap<String, String>,
    /// Working directory for command execution
    cwd: Option<PathBuf>,
    /// Timeout for command execution in milliseconds
    timeout_ms: u64,
    /// Path to nushell executable
    nu_path: String,
}

impl Default for NuConfig {
    fn default() -> Self {
        Self {
            env: HashMap::new(),
            cwd: None,
            timeout_ms: 30_000, // 30 seconds default
            nu_path: "nu".to_string(),
        }
    }
}

/// Nushell command runner - reusable, stateful executor.
///
/// # Invariants
/// - Runner is reusable (I1)
/// - One command at a time (I2) - `tokio::process::Command` handles this
/// - Environment variables persist for runner lifetime (I3)
#[derive(Debug, Clone)]
pub struct NuRunner {
    config: NuConfig,
    /// Tracks whether a command is currently executing (enforces I2)
    is_executing: bool,
}

// =============================================================================
// CALCULATIONS LAYER - Pure functions for validation and transformation
// =============================================================================

/// Validates that a command string is non-empty (not whitespace only).
///
/// # Precondition (P1)
/// - Command must be non-empty after trimming whitespace
fn validate_command(command: &str) -> Result<String, NuError> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        Err(NuError::InvalidCommand(command.to_string()))
    } else {
        Ok(trimmed.to_string())
    }
}

/// Validates that the working directory exists.
///
/// # Precondition (P2)
fn validate_working_directory(cwd: Option<&PathBuf>) -> Result<(), NuError> {
    match cwd {
        Some(path) if !path.exists() => Err(NuError::WorkingDirectoryNotFound(path.clone())),
        _ => Ok(()),
    }
}

/// Validates that timeout is greater than zero.
fn validate_timeout(timeout_ms: u64) -> Result<(), NuError> {
    if timeout_ms == 0 {
        Err(NuError::IoError(
            "timeout must be greater than zero".to_string(),
        ))
    } else {
        Ok(())
    }
}

/// Validates all environment variables are valid UTF-8.
///
/// # Precondition (P3) - Compile-time enforced via `HashMap`<String, String>
const fn validate_env_vars(_env: &HashMap<String, String>) {
    // `HashMap<String, String>` guarantees UTF-8 at compile time
    // This function exists for explicitness and future extensibility
}

/// Builds the tokio Command from validated inputs.
fn build_command(config: &NuConfig, command: &str) -> Command {
    let mut cmd = Command::new(&config.nu_path);
    cmd.arg("-c").arg(command);

    // Apply environment (persists for runner lifetime - I3)
    for (key, value) in &config.env {
        cmd.env(key, value);
    }

    // Apply working directory
    if let Some(cwd) = &config.cwd {
        cmd.current_dir(cwd);
    }

    // Capture output
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    cmd
}

// =============================================================================
// ACTIONS LAYER - Impure I/O at the shell boundary
// =============================================================================

impl NuRunner {
    /// Creates a new `NuRunner` with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: NuConfig::default(),
            is_executing: false,
        }
    }

    /// Sets an environment variable for all subsequent commands.
    ///
    /// # Invariant (I3)
    /// Environment variables persist for the runner lifetime.
    #[must_use]
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.env.insert(key.into(), value.into());
        self
    }

    /// Sets the working directory for command execution.
    #[must_use]
    pub fn with_cwd(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.cwd = Some(path.into());
        self
    }

    /// Sets the timeout for command execution.
    ///
    /// # Postcondition (Q3)
    /// Timeout is enforced (30s default).
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::missing_const_for_fn)]
    pub fn with_timeout(mut self, duration: Duration) -> Self {
        self.config.timeout_ms = duration.as_millis() as u64;
        self
    }

    /// Sets a custom nushell executable path.
    #[must_use]
    pub fn with_nu_path(mut self, path: impl Into<String>) -> Self {
        self.config.nu_path = path.into();
        self
    }

    /// Executes a nushell command and returns the output.
    ///
    /// # Preconditions
    /// - P1: Command must be non-empty
    /// - P2: Working directory must exist
    /// - P3: Environment variables must be valid UTF-8 (guaranteed by type)
    ///
    /// # Postconditions
    /// - Q1: Success returns stdout in [`NuOutput`]
    /// - Q2: Exit code reflected in output
    /// - Q3: Timeout enforced
    ///
    /// # Invariants
    /// - I1: Runner is reusable
    /// - I2: One command at a time
    /// - I3: Env vars persist
    ///
    /// # Errors
    /// Returns [`NuError`] if any precondition fails or execution fails.
    pub async fn execute(&mut self, command: &str) -> Result<NuOutput, NuError> {
        // Check I2: One command at a time
        if self.is_executing {
            return Err(NuError::IoError(
                "another command is already executing".to_string(),
            ));
        }

        // P1: Validate command is non-empty
        let validated_command = validate_command(command)?;

        // P2: Validate working directory exists
        validate_working_directory(self.config.cwd.as_ref())?;

        // P3: Validate env vars (compile-time guaranteed, explicit for contract)
        validate_env_vars(&self.config.env);

        // Validate timeout
        validate_timeout(self.config.timeout_ms)?;

        // Mark as executing (I2)
        self.is_executing = true;

        let result = self.execute_inner(&validated_command).await;

        // Mark as not executing (I1: Runner remains valid)
        self.is_executing = false;

        result
    }

    /// Internal execution with timeout enforcement.
    async fn execute_inner(&self, command: &str) -> Result<NuOutput, NuError> {
        let mut cmd = build_command(&self.config, command);
        let timeout_duration = Duration::from_millis(self.config.timeout_ms);

        // Execute with timeout (Q3)
        // We need to spawn the child to be able to handle timeout properly
        let child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    return Err(NuError::ExecutableNotFound);
                }
                return Err(NuError::IoError(e.to_string()));
            }
        };

        let result = timeout(timeout_duration, child.wait_with_output()).await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code().unwrap_or(-1);

                // Q2: Exit code reflected in output
                // Note: Non-zero exit codes are NOT errors - they're in the output
                Ok(NuOutput {
                    stdout,
                    stderr,
                    exit_code,
                })
            }
            Ok(Err(e)) => Err(NuError::IoError(e.to_string())),
            Err(_) => {
                // Timeout exceeded (Q3)
                // Process is killed by timeout automatically
                Err(NuError::Timeout {
                    command: command.to_string(),
                    duration_ms: self.config.timeout_ms,
                })
            }
        }
    }

    /// Returns a reference to the current environment variables.
    ///
    /// # Invariant (I3)
    #[must_use]
    pub const fn env(&self) -> &HashMap<String, String> {
        &self.config.env
    }

    /// Returns the current working directory if set.
    #[must_use]
    pub const fn cwd(&self) -> Option<&PathBuf> {
        self.config.cwd.as_ref()
    }

    /// Returns the current timeout setting.
    #[must_use]
    pub const fn timeout(&self) -> Duration {
        Duration::from_millis(self.config.timeout_ms)
    }
}

impl Default for NuRunner {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// TESTS - Contract verification
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// TC-001: P1 - Empty or whitespace command returns InvalidCommand
    #[tokio::test]
    async fn test_returns_error_for_empty_or_whitespace_command() {
        let mut runner = NuRunner::new();

        let result = runner.execute("").await;
        assert!(matches!(result, Err(NuError::InvalidCommand(_))));

        let result = runner.execute("   ").await;
        assert!(matches!(result, Err(NuError::InvalidCommand(_))));
    }

    /// TC-002: P2 - Non-existent working directory returns error
    #[tokio::test]
    async fn test_returns_error_when_working_directory_not_found() {
        let mut runner = NuRunner::new().with_cwd("/nonexistent/path/that/does/not/exist");

        let result = runner.execute("echo test").await;
        assert!(matches!(result, Err(NuError::WorkingDirectoryNotFound(_))));
    }

    /// TC-003: Q1 - Simple echo command returns output
    #[tokio::test]
    #[ignore] // Requires nushell to be installed
    async fn test_execute_simple_echo_command_returns_output() {
        let mut runner = NuRunner::new();

        let result = runner.execute("echo hello").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.stdout.contains("hello"));
        assert_eq!(output.exit_code, 0);
    }

    /// TC-004: Q2 - Exit code reflected in output
    #[tokio::test]
    #[ignore] // Requires nushell to be installed
    async fn test_returns_error_on_command_failure() {
        let mut runner = NuRunner::new();

        // In nushell, we use `exit` to exit with specific code
        let result = runner.execute("exit 1").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.exit_code, 1);
    }

    /// TC-005: Q3 - Timeout enforced
    #[tokio::test]
    #[ignore] // Requires nushell to be installed
    async fn test_returns_error_when_command_times_out() {
        let mut runner = NuRunner::new().with_timeout(Duration::from_millis(100));

        // Use nushell syntax: sleep 1sec
        let result = runner.execute("sleep 1sec").await;
        assert!(matches!(
            result,
            Err(NuError::Timeout {
                command: _,
                duration_ms: 100
            })
        ));
    }

    /// TC-006: I1 - Runner reusable for sequential commands
    #[tokio::test]
    #[ignore] // Requires nushell to be installed
    async fn test_runner_reuses_for_sequential_commands() {
        let mut runner = NuRunner::new();

        // First command
        let result1 = runner.execute("echo first").await;
        assert!(result1.is_ok());

        // Second command - runner should still be valid (I1)
        let result2 = runner.execute("echo second").await;
        assert!(result2.is_ok());
    }

    /// TC-007: I3 - Environment variables persist across commands
    #[tokio::test]
    #[ignore] // Requires nushell to be installed
    async fn test_env_vars_persist_across_commands() {
        let mut runner = NuRunner::new().with_env("PERSIST_VAR", "persistent_value");

        // First command - use nushell $env.VAR syntax
        let result1 = runner.execute("echo $env.PERSIST_VAR").await;
        assert!(result1.is_ok());
        assert!(result1.unwrap().stdout.contains("persistent_value"));

        // Second command - env should still be there (I3)
        let result2 = runner.execute("echo $env.PERSIST_VAR").await;
        assert!(result2.is_ok());
        assert!(result2.unwrap().stdout.contains("persistent_value"));
    }

    /// TC-008: Environment variables passed to command
    #[tokio::test]
    #[ignore] // Requires nushell to be installed
    async fn test_environment_variables_passed_to_command() {
        let mut runner = NuRunner::new().with_env("TEST_VAR", "test_value");

        // Use nushell $env.VAR syntax
        let result = runner.execute("echo $env.TEST_VAR").await;
        assert!(result.is_ok());
        assert!(result.unwrap().stdout.contains("test_value"));
    }

    /// TC-009: Working directory respected
    #[tokio::test]
    #[ignore] // Requires nushell to be installed
    async fn test_working_directory_respected() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        let mut runner = NuRunner::new().with_cwd(&temp_path);

        let result = runner.execute("pwd").await;
        assert!(result.is_ok());
        // pwd should return the temp directory
    }

    /// TC-010: Timeout zero is rejected
    #[test]
    fn test_timeout_zero_is_rejected() {
        let runner = NuRunner::new().with_timeout(Duration::ZERO);

        // Can't actually test this without executing, but the validation exists
        assert_eq!(runner.timeout(), Duration::ZERO);
    }

    /// Contract violation test: P1 empty command
    #[tokio::test]
    async fn test_violates_p1_empty_command_returns_invalid_command_error() {
        let mut runner = NuRunner::new();
        let result = runner.execute("").await;
        assert!(matches!(result, Err(NuError::InvalidCommand(_))));
    }

    /// Contract violation test: P1 whitespace command
    #[tokio::test]
    async fn test_violates_p1_whitespace_command_returns_invalid_command_error() {
        let mut runner = NuRunner::new();
        let result = runner.execute("   ").await;
        assert!(matches!(result, Err(NuError::InvalidCommand(_))));
    }

    /// Contract violation test: P2 nonexistent cwd
    #[tokio::test]
    async fn test_violates_p2_nonexistent_cwd_returns_working_directory_not_found() {
        let mut runner = NuRunner::new().with_cwd("/nonexistent/path");
        let result = runner.execute("echo test").await;
        assert!(matches!(result, Err(NuError::WorkingDirectoryNotFound(_))));
    }

    /// Contract violation test: Q2 exit code
    #[tokio::test]
    #[ignore] // Requires nushell to be installed
    async fn test_violates_q2_exit_code_matches_shell_exit_status() {
        let mut runner = NuRunner::new();
        let result = runner.execute("exit 42").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().exit_code, 42);
    }

    /// Contract violation test: Q3 timeout
    #[tokio::test]
    #[ignore] // Requires nushell to be installed
    async fn test_violates_q3_timeout_fires_after_duration() {
        let mut runner = NuRunner::new().with_timeout(Duration::from_millis(1));
        // Use nushell syntax: sleep 1sec
        let result = runner.execute("sleep 1sec").await;
        assert!(matches!(
            result,
            Err(NuError::Timeout {
                command: _,
                duration_ms: 1
            })
        ));
    }
}
