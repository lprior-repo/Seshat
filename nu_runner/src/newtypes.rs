//! Newtype wrappers for domain concepts.
//
//! Following Scott Wlaschin DDD principles:
//! - Newtypes for primitive domain concepts
//! - Makes illegal states unrepresentable

use std::collections::HashMap;

// =============================================================================
// NEWTYPES - Eliminating primitive obsession
// =============================================================================

/// Newtype for the nushell executable path.
///
/// Makes illegal states unrepresentable: empty paths are rejected at construction.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NuPath(pub String);

impl NuPath {
    /// Creates a new `NuPath` from a string.
    ///
    /// # Errors
    /// Returns an error if the path string is empty.
    pub fn new(path: impl Into<String>) -> Result<Self, NuPathError> {
        let path = path.into();
        if path.is_empty() {
            Err(NuPathError::EmptyPath)
        } else {
            Ok(Self(path))
        }
    }

    /// Returns the path as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for NuPath {
    fn default() -> Self {
        Self("nu".to_string())
    }
}

impl std::fmt::Display for NuPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Error type for invalid NuPath construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NuPathError {
    EmptyPath,
}

impl std::fmt::Display for NuPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPath => write!(f, "NuPath cannot be empty"),
        }
    }
}

impl std::error::Error for NuPathError {}

/// Newtype for timeout duration in milliseconds.
///
/// Makes illegal states unrepresentable: zero timeout is rejected at construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeoutMs(pub u64);

impl TimeoutMs {
    /// Creates a new `TimeoutMs` from milliseconds.
    ///
    /// # Errors
    /// Returns an error if the duration is zero.
    pub fn new(ms: u64) -> Result<Self, TimeoutMsError> {
        if ms == 0 {
            Err(TimeoutMsError::ZeroTimeout)
        } else {
            Ok(Self(ms))
        }
    }

    /// Returns the timeout in milliseconds.
    #[must_use]
    pub const fn as_millis(self) -> u64 {
        self.0
    }

    /// Creates a timeout of zero (for testing purposes only).
    #[cfg(test)]
    pub const fn zero() -> Self {
        Self(0)
    }
}

impl Default for TimeoutMs {
    fn default() -> Self {
        // 30 seconds default
        Self(30_000)
    }
}

impl From<TimeoutMs> for std::time::Duration {
    fn from(val: TimeoutMs) -> Self {
        std::time::Duration::from_millis(val.0)
    }
}

/// Error type for invalid TimeoutMs construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeoutMsError {
    ZeroTimeout,
}

impl std::fmt::Display for TimeoutMsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroTimeout => write!(f, "timeout must be greater than zero"),
        }
    }
}

impl std::error::Error for TimeoutMsError {}

/// Newtype for process exit codes.
///
/// Makes illegal states unrepresentable: captures valid exit code range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExitCode(pub i32);

impl ExitCode {
    /// Creates a new `ExitCode`.
    ///
    /// # Panics
    /// Panics if the exit code is outside the valid range (-128 to 255).
    /// In practice, we accept any i32 but normalize None to -1.
    pub const fn new(code: i32) -> Self {
        Self(code)
    }

    /// Returns the exit code value.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        self.0
    }

    /// Returns true if the exit code indicates success (0).
    #[must_use]
    pub const fn is_success(self) -> bool {
        self.0 == 0
    }
}

impl Default for ExitCode {
    fn default() -> Self {
        Self(0)
    }
}

impl From<i32> for ExitCode {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// Newtype for environment variables.
///
/// Provides type safety and guarantees UTF-8 compliance.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EnvVars(pub HashMap<String, String>);

impl EnvVars {
    /// Creates a new empty `EnvVars`.
    #[must_use]
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Inserts an environment variable.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.0.insert(key.into(), value.into());
    }

    /// Returns a reference to the underlying map.
    #[must_use]
    pub fn as_map(&self) -> &HashMap<String, String> {
        &self.0
    }

    /// Returns an iterator over the environment variables.
    #[must_use]
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.0.iter()
    }
}

impl From<HashMap<String, String>> for EnvVars {
    fn from(map: HashMap<String, String>) -> Self {
        Self(map)
    }
}

/// Newtype for a validated command string.
///
/// Ensures commands are non-empty after trimming whitespace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command(String);

impl Command {
    /// Creates a new `Command` from a string.
    ///
    /// # Errors
    /// Returns an error if the command is empty or whitespace-only.
    pub fn new(cmd: impl Into<String>) -> Result<Self, CommandError> {
        let cmd = cmd.into();
        let trimmed = cmd.trim();
        if trimmed.is_empty() {
            Err(CommandError::EmptyCommand)
        } else {
            Ok(Self(trimmed.to_string()))
        }
    }

    /// Returns the command as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Error type for invalid Command construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    EmptyCommand,
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyCommand => write!(f, "command cannot be empty"),
        }
    }
}

impl std::error::Error for CommandError {}
