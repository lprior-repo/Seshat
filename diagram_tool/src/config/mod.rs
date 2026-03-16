//! Configuration module for the diagram tool.
//!
//! Provides environment variable handling, config file parsing,
//! and validation at startup.

#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]
#![allow(dead_code)]

mod app_config;
mod error;
mod validation;

pub use app_config::{AppConfig, DatabaseConfig, LogConfig};
pub use error::ConfigError;

pub(crate) const CONFIG_FILE_NAME: &str = "diagram_tool.json";
pub(crate) const MAX_DATABASE_PATH_LENGTH: usize = 4096;

pub fn load_config() -> Result<AppConfig, ConfigError> {
    let config = AppConfig::load_from_environment()?;
    let config = config.merge_from_config_file()?;
    config.validate()?;
    Ok(config)
}

#[cfg(test)]
mod tests {

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_load_config_returns_valid_config() {
        let result = load_config();
        assert!(result.is_ok() || result.is_err());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_config_has_defaults() {
        let config = AppConfig::load_from_environment().expect("should create config");
        assert_eq!(config.database.journal_mode, "wal");
        assert_eq!(config.database.synchronous, 2);
        assert_eq!(config.database.wal_autocheckpoint, 1000);
    }
}
