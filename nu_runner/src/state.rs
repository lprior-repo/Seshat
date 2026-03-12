//! State machine types for the Nushell command runner.
//!
//! Explicit state encoding following DDD principles.

use std::fmt;

/// Explicit state for the runner - ensures one command at a time.
///
/// This replaces the implicit `is_executing: bool` with an explicit enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerState {
    /// Ready to execute a command
    Ready,
    /// Currently executing a command
    Executing,
}

impl Default for RunnerState {
    fn default() -> Self {
        Self::Ready
    }
}

impl RunnerState {
    /// Returns true if the runner is ready to execute.
    #[must_use]
    pub const fn is_ready(self) -> bool {
        matches!(self, Self::Ready)
    }

    /// Returns true if the runner is currently executing.
    #[must_use]
    pub const fn is_executing(self) -> bool {
        matches!(self, Self::Executing)
    }

    /// Transitions to Executing state.
    ///
    /// # Errors
    /// Returns an error if already executing.
    pub fn start_executing(self) -> Result<Self, RunnerStateError> {
        match self {
            Self::Ready => Ok(Self::Executing),
            Self::Executing => Err(RunnerStateError::AlreadyExecuting),
        }
    }

    /// Transitions back to Ready state.
    pub fn finish_executing(self) -> Self {
        Self::Ready
    }
}

/// Error type for invalid state transitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunnerStateError {
    AlreadyExecuting,
}

impl fmt::Display for RunnerStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyExecuting => write!(f, "another command is already executing"),
        }
    }
}

impl std::error::Error for RunnerStateError {}
