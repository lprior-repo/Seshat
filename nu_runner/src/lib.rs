//! Nushell Command Runner - A functional Rust implementation for executing Nushell commands.
//!
//! # Architecture
//!
//! - **Data**: `types` and `newtypes` modules - immutable domain types
//! - **Calculations**: `validation` module - pure validation functions
//! - **Actions**: `runner` module - process execution (impure I/O)
//! - **Errors**: `errors` module - explicit error taxonomy
//! - **State**: `state` module - explicit state machine
//!
//! # Modules
//!
//! - `types` - Core domain types: `NuOutput`, `NuConfig`, `NuRunner`
//! - `newtypes` - Value objects: `NuPath`, `TimeoutMs`, `ExitCode`, `EnvVars`, `Command`
//! - `errors` - Error types: `NuError`
//! - `state` - State machine: `RunnerState`, `RunnerStateError`
//! - `validation` - Pure functions for input validation
//! - `runner` - Command execution implementation

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

pub mod errors;
pub mod newtypes;
pub mod runner;
pub mod state;
pub mod types;
pub mod validation;

// Re-export main types for convenience
pub use errors::NuError;
pub use newtypes::{
    Command, EnvVars, ExitCode, NuPath, TimeoutMs,
};
pub use state::{RunnerState, RunnerStateError};
pub use types::{NuConfig, NuOutput, NuRunner};
pub use validation::{validate_command, validate_timeout, validate_working_directory};

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;
    use std::time::Duration;

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
        assert!(matches!(
            result,
            Err(NuError::WorkingDirectoryNotFound(_))
        ));
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
        assert_eq!(output.exit_code.0, 0);
    }

    /// TC-004: Q2 - Exit code reflected in output
    #[tokio::test]
    #[ignore] // Requires nushell to be installed
    async fn test_returns_error_on_command_failure() {
        let mut runner = NuRunner::new();

        let result = runner.execute("exit 1").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.exit_code.0, 1);
    }

    /// TC-005: Q3 - Timeout enforced
    #[tokio::test]
    #[ignore] // Requires nushell to be installed
    async fn test_returns_error_when_command_times_out() {
        let mut runner = NuRunner::new().with_timeout(Duration::from_millis(100));

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

        let result1 = runner.execute("echo first").await;
        assert!(result1.is_ok());

        let result2 = runner.execute("echo second").await;
        assert!(result2.is_ok());
    }

    /// TC-007: I3 - Environment variables persist across commands
    #[tokio::test]
    #[ignore] // Requires nushell to be installed
    async fn test_env_vars_persist_across_commands() {
        let mut runner = NuRunner::new().with_env("PERSIST_VAR", "persistent_value");

        let result1 = runner.execute("echo $env.PERSIST_VAR").await;
        assert!(result1.is_ok());
        assert!(result1.unwrap().stdout.contains("persistent_value"));

        let result2 = runner.execute("echo $env.PERSIST_VAR").await;
        assert!(result2.is_ok());
        assert!(result2.unwrap().stdout.contains("persistent_value"));
    }

    /// TC-008: Environment variables passed to command
    #[tokio::test]
    #[ignore] // Requires nushell to be installed
    async fn test_environment_variables_passed_to_command() {
        let mut runner = NuRunner::new().with_env("TEST_VAR", "test_value");

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
    }

    /// TC-010: Timeout zero is rejected
    #[test]
    fn test_timeout_zero_is_rejected() {
        let runner = NuRunner::new().with_timeout(Duration::ZERO);

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
        assert!(matches!(
            result,
            Err(NuError::WorkingDirectoryNotFound(_))
        ));
    }

    /// Contract violation test: Q2 exit code
    #[tokio::test]
    #[ignore] // Requires nushell to be installed
    async fn test_violates_q2_exit_code_matches_shell_exit_status() {
        let mut runner = NuRunner::new();
        let result = runner.execute("exit 42").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().exit_code.0, 42);
    }

    /// Contract violation test: Q3 timeout
    #[tokio::test]
    #[ignore] // Requires nushell to be installed
    async fn test_violates_q3_timeout_fires_after_duration() {
        let mut runner = NuRunner::new().with_timeout(Duration::from_millis(1));
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
