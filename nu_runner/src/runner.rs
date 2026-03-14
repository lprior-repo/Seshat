//! Nushell command runner implementation (Actions layer).
//!
//! This module contains the impure I/O operations for executing Nushell commands.
//! Following the functional core / imperative shell pattern:
//! - Pure validation happens in the `validation` module
//! - Side effects are isolated to this module

use std::time::Duration;

use tokio::process::Command as TokioCommand;
use tokio::time::timeout;

use crate::errors::NuError;
use crate::state::RunnerState;
use crate::types::{NuConfig, NuOutput, NuRunner};
use crate::validation::{parse_output, validate_config};
use crate::newtypes::TimeoutMs;

// =============================================================================
// COMMAND BUILDING
// =============================================================================

/// Builds the tokio Command from validated inputs.
fn build_command(config: &NuConfig, command: &str) -> TokioCommand {
    let mut cmd = TokioCommand::new(config.nu_path.0.as_str());
    cmd.arg("-c").arg(command);

    // Apply environment (persists for runner lifetime - I3)
    for (key, value) in config.env.0.iter() {
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

/// Spawns the command process.
fn spawn_command(cmd: &mut TokioCommand) -> Result<tokio::process::Child, NuError> {
    cmd.spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                NuError::ExecutableNotFound
            } else {
                NuError::IoError(e.to_string())
            }
        })
}

// =============================================================================
// RUNNER IMPLEMENTATION
// =============================================================================

impl NuRunner {
    /// Creates a new `NuRunner` with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: NuConfig::default(),
            state: RunnerState::default(),
        }
    }

    /// Sets an environment variable for all subsequent commands.
    ///
    /// # Invariant (I3)
    /// Environment variables persist for the runner lifetime.
    #[must_use]
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.env.0.insert(key.into(), value.into());
        self
    }

    /// Sets the working directory for command execution.
    #[must_use]
    pub fn with_cwd(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.config.cwd = Some(path.into());
        self
    }

    /// Sets the timeout for command execution.
    ///
    /// # Postcondition (Q3)
    /// Timeout is enforced (30s default).
    /// Zero duration is ignored (keeps default).
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::missing_const_for_fn)]
    pub fn with_timeout(mut self, duration: Duration) -> Self {
        let ms = duration.as_millis() as u64;
        // Guard ensures ms > 0, so TimeoutMs::new() will always succeed
        // (it only returns Err for ZeroTimeout, i.e., ms == 0)
        if ms != 0 {
            self.config.timeout_ms = match TimeoutMs::new(ms) {
                Ok(t) => t,
                // This branch is unreachable due to the guard above,
                // but kept for explicitness and defense in depth
                Err(_) => TimeoutMs::default(),
            };
        }
        self
    }

    /// Sets a custom nushell executable path.
    #[must_use]
    pub fn with_nu_path(mut self, path: impl Into<String>) -> Self {
        self.config.nu_path.0 = path.into();
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
        // P1, P2, P3, and timeout validation
        let validated_command = validate_config(&self.config, command)?;

        // Check I2: One command at a time using explicit state
        self.state = self.state.start_executing()?;

        let result = self.execute_inner(validated_command.as_str()).await;

        // Mark as not executing (I1: Runner remains valid)
        self.state = self.state.finish_executing();

        result
    }

    /// Internal execution with timeout enforcement.
    async fn execute_inner(&self, command: &str) -> Result<NuOutput, NuError> {
        let mut cmd = build_command(&self.config, command);
        let timeout_duration = Duration::from_millis(self.config.timeout_ms.0);

        // Spawn the command process
        let child = spawn_command(&mut cmd)?;

        // Execute with timeout (Q3)
        self.wait_with_timeout(child, timeout_duration, command)
            .await
    }

    /// Waits for the child process with timeout enforcement.
    async fn wait_with_timeout(
        &self,
        child: tokio::process::Child,
        timeout_duration: Duration,
        command: &str,
    ) -> Result<NuOutput, NuError> {
        let result = timeout(timeout_duration, child.wait_with_output()).await;

        match result {
            Ok(Ok(output)) => Ok(parse_output(&output)),
            Ok(Err(e)) => Err(NuError::IoError(e.to_string())),
            Err(_) => {
                // Timeout exceeded (Q3)
                // Process is killed by timeout automatically
                Err(NuError::Timeout {
                    command: command.to_string(),
                    duration_ms: self.config.timeout_ms.0,
                })
            }
        }
    }

    /// Returns a reference to the current environment variables.
    ///
    /// # Invariant (I3)
    #[must_use]
    pub fn env(&self) -> &std::collections::HashMap<String, String> {
        &self.config.env.0
    }

    /// Returns the current working directory if set.
    #[must_use]
    pub fn cwd(&self) -> Option<&std::path::PathBuf> {
        self.config.cwd.as_ref()
    }

    /// Returns the current timeout setting.
    #[must_use]
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.config.timeout_ms.0)
    }
}

impl Default for NuRunner {
    fn default() -> Self {
        Self::new()
    }
}
