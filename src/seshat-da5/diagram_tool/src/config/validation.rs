//! Configuration validation.

use super::app_config::{AppConfig, DatabaseConfig, LogConfig};
use super::error::ConfigError;

pub struct ConfigValidator;

impl ConfigValidator {
    pub fn validate(config: &AppConfig) -> Result<(), ConfigError> {
        Self::validate_database(&config.database)?;
        Self::validate_logging(&config.logging)?;
        Ok(())
    }

    fn validate_database(config: &DatabaseConfig) -> Result<(), ConfigError> {
        if !config.path.is_empty() && config.path.len() > MAX_DATABASE_PATH_LENGTH {
            return Err(ConfigError::InvalidDatabasePath(format!(
                "Path too long: {} (max {})",
                config.path.len(),
                MAX_DATABASE_PATH_LENGTH
            )));
        }

        let valid_journal_modes = ["wal", "delete", "truncate", "persist", "memory", "off"];
        if !valid_journal_modes.contains(&config.journal_mode.as_str()) {
            return Err(ConfigError::InvalidJournalMode(config.journal_mode.clone()));
        }

        let valid_synchronous = [0, 1, 2, 3];
        if !valid_synchronous.contains(&config.synchronous) {
            return Err(ConfigError::InvalidSynchronousMode(config.synchronous));
        }

        if config.wal_autocheckpoint < 0 || config.wal_autocheckpoint > 10000 {
            return Err(ConfigError::InvalidWalAutocheckpoint(
                config.wal_autocheckpoint,
            ));
        }

        Ok(())
    }

    fn validate_logging(config: &LogConfig) -> Result<(), ConfigError> {
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&config.level.as_str()) {
            return Err(ConfigError::InvalidLogLevel(config.level.clone()));
        }

        let valid_formats = ["json", "text", "pretty"];
        if !valid_formats.contains(&config.format.as_str()) {
            return Err(ConfigError::InvalidLogFormat(config.format.clone()));
        }

        Ok(())
    }
}

const MAX_DATABASE_PATH_LENGTH: usize = 4096;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_database_config() {
        let config = DatabaseConfig::default();
        assert!(ConfigValidator::validate_database(&config).is_ok());
    }

    #[test]
    fn test_invalid_journal_mode() {
        let mut config = DatabaseConfig::default();
        config.journal_mode = "invalid".to_string();
        let result = ConfigValidator::validate_database(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_synchronous() {
        let mut config = DatabaseConfig::default();
        config.synchronous = 99;
        let result = ConfigValidator::validate_database(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_wal_checkpoint() {
        let mut config = DatabaseConfig::default();
        config.wal_autocheckpoint = -1;
        let result = ConfigValidator::validate_database(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_log_config() {
        let config = LogConfig::default();
        assert!(ConfigValidator::validate_logging(&config).is_ok());
    }

    #[test]
    fn test_invalid_log_level() {
        let mut config = LogConfig::default();
        config.level = "invalid".to_string();
        let result = ConfigValidator::validate_logging(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_full_config_validation() {
        let config = AppConfig::default();
        assert!(ConfigValidator::validate(&config).is_ok());
    }
}
