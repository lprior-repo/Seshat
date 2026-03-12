//! Core domain types for the Nushell command runner.
//!
//! This module contains the main domain types: NuOutput, NuConfig, and NuRunner.

use std::path::PathBuf;

use crate::newtypes::{EnvVars, ExitCode, NuPath, TimeoutMs};
use crate::state::RunnerState;

// =============================================================================
// OUTPUT TYPE
// =============================================================================

/// Output from a executed command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuOutput {
    /// Standard output captured from the command
    pub stdout: String,
    /// Standard error captured from the command
    pub stderr: String,
    /// Exit code returned by the command
    pub exit_code: ExitCode,
}

// =============================================================================
// CONFIGURATION TYPE
// =============================================================================

/// Configuration for the [`NuRunner`].
///
/// Uses newtypes for all primitive fields to eliminate primitive obsession.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuConfig {
    /// Environment variables to pass to the command
    pub env: EnvVars,
    /// Working directory for command execution
    pub cwd: Option<PathBuf>,
    /// Timeout for command execution in milliseconds
    pub timeout_ms: TimeoutMs,
    /// Path to nushell executable
    pub nu_path: NuPath,
}

impl Default for NuConfig {
    fn default() -> Self {
        Self {
            env: EnvVars::new(),
            cwd: None,
            timeout_ms: TimeoutMs::default(),
            nu_path: NuPath::default(),
        }
    }
}

impl NuConfig {
    /// Returns a reference to the environment variables.
    #[must_use]
    pub const fn env(&self) -> &EnvVars {
        &self.env
    }

    /// Returns the working directory if set.
    #[must_use]
    pub const fn cwd(&self) -> Option<&PathBuf> {
        self.cwd.as_ref()
    }

    /// Returns the timeout setting.
    #[must_use]
    pub const fn timeout_ms(&self) -> TimeoutMs {
        self.timeout_ms
    }

    /// Returns the nushell executable path.
    #[must_use]
    pub const fn nu_path(&self) -> &NuPath {
        &self.nu_path
    }

    /// Returns the timeout as a `Duration`.
    #[must_use]
    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_ms.0)
    }
}

// =============================================================================
// RUNNER TYPE (just the struct, impl is in runner.rs)
// =============================================================================

/// Nushell command runner - reusable, stateful executor.
///
/// # Invariants
/// - Runner is reusable (I1)
/// - One command at a time (I2) - enforced by `RunnerState` enum
/// - Environment variables persist for runner lifetime (I3)
#[derive(Debug)]
pub struct NuRunner {
    /// Configuration (pub for builder pattern)
    pub config: NuConfig,
    /// Explicit state tracking (replaces `is_executing: bool`)
    pub state: RunnerState,
}
