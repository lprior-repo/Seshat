//! Configuration error types.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {path}")]
    FileRead {
        path: std::path::PathBuf,
        source: std::io::Error,
    },

    #[error("Failed to parse config file: {path}")]
    FileParse {
        path: std::path::PathBuf,
        source: serde_json::Error,
    },

    #[error("Invalid database path: {0}")]
    InvalidDatabasePath(String),

    #[error("Invalid journal mode: {0}")]
    InvalidJournalMode(String),

    #[error("Invalid synchronous mode: {0}")]
    InvalidSynchronousMode(i32),

    #[error("Invalid WAL autocheckpoint: {0}")]
    InvalidWalAutocheckpoint(i32),

    #[error("Invalid log level: {0}")]
    InvalidLogLevel(String),

    #[error("Invalid log format: {0}")]
    InvalidLogFormat(String),

    #[error("Configuration validation failed: {0}")]
    ValidationFailed(String),
}

impl ConfigError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::FileRead { .. } => "config_file_read_error",
            Self::FileParse { .. } => "config_file_parse_error",
            Self::InvalidDatabasePath(_) => "invalid_database_path",
            Self::InvalidJournalMode(_) => "invalid_journal_mode",
            Self::InvalidSynchronousMode(_) => "invalid_synchronous_mode",
            Self::InvalidWalAutocheckpoint(_) => "invalid_wal_autocheckpoint",
            Self::InvalidLogLevel(_) => "invalid_log_level",
            Self::InvalidLogFormat(_) => "invalid_log_format",
            Self::ValidationFailed(_) => "config_validation_failed",
        }
    }
}
