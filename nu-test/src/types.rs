use std::path::Path;
use thiserror::Error;

pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExitCode(pub u8);

impl ExitCode {
    pub fn new(code: i32) -> Result<Self, Error> {
        if !(0..=255).contains(&code) {
            return Err(Error::ExitCodeOutOfRange(code));
        }
        Ok(Self(code as u8))
    }

    pub fn as_i32(self) -> i32 {
        self.0 as i32
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonEmptyString(String);

impl NonEmptyString {
    pub fn new(s: impl Into<String>) -> Result<Self, Error> {
        let s = s.into();
        if s.is_empty() {
            return Err(Error::EmptyCommand);
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedCommand(String);

impl CheckedCommand {
    pub const MAX_LEN: usize = 8192;

    pub fn new(cmd: impl Into<String>) -> Result<Self, Error> {
        let cmd = cmd.into();
        if cmd.is_empty() {
            return Err(Error::EmptyCommand);
        }
        if cmd.len() > Self::MAX_LEN {
            return Err(Error::CommandTooLong(cmd.len()));
        }
        Ok(Self(cmd))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct NuResult {
    pub stdout: String,
    pub stdout_valid: bool,
    pub stderr: String,
    pub stderr_valid: bool,
    pub exit_code: i32,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Nushell not found in PATH")]
    NushellNotFound,

    #[error("Command cannot be empty")]
    EmptyCommand,

    #[error("Command too long: {0} bytes (max 8192)")]
    CommandTooLong(usize),

    #[error("Exit code out of range: {0}")]
    ExitCodeOutOfRange(i32),

    #[error("Working directory does not exist: {0}")]
    WorkingDirectoryNotFound(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),
}

pub fn find_nushell() -> Result<String, Error> {
    std::env::var("NU_PATH")
        .or_else(|_| std::env::var("PATH"))
        .ok()
        .and_then(|path| {
            path.split(':')
                .map(|dir| format!("{}/nu", dir))
                .find(|p| std::path::Path::new(p).exists())
        })
        .ok_or(Error::NushellNotFound)
}

pub fn validate_working_directory(cwd: &Path) -> Result<(), Error> {
    if !cwd.exists() {
        return Err(Error::WorkingDirectoryNotFound(cwd.display().to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_empty_string_accepts_valid() {
        assert!(NonEmptyString::new("echo hello").is_ok());
    }

    #[test]
    fn test_non_empty_string_rejects_empty() {
        assert!(NonEmptyString::new("").is_err());
    }

    #[test]
    fn test_checked_command_accepts_at_max_length() {
        let cmd = "a".repeat(8192);
        assert!(CheckedCommand::new(&cmd).is_ok());
    }

    #[test]
    fn test_checked_command_rejects_too_long() {
        let cmd = "a".repeat(8193);
        assert!(CheckedCommand::new(&cmd).is_err());
    }

    #[test]
    fn test_checked_command_rejects_empty() {
        assert!(CheckedCommand::new("").is_err());
    }

    #[test]
    fn test_exit_code_from_i32_valid_range() {
        assert!(ExitCode::new(0).is_ok());
        assert!(ExitCode::new(127).is_ok());
        assert!(ExitCode::new(255).is_ok());
    }

    #[test]
    fn test_exit_code_from_i32_invalid_range() {
        assert!(ExitCode::new(-1).is_err());
        assert!(ExitCode::new(256).is_err());
    }

    #[test]
    fn test_validate_working_directory_valid() {
        assert!(validate_working_directory(std::path::Path::new(".")).is_ok());
    }

    #[test]
    fn test_validate_working_directory_invalid() {
        assert!(validate_working_directory(std::path::Path::new("/nonexistent/path")).is_err());
    }
}
