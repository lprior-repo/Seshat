//! Application configuration structures.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub logging: LogConfig,
    pub debug: DebugConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
    pub journal_mode: String,
    pub synchronous: i32,
    pub wal_autocheckpoint: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogConfig {
    pub level: String,
    pub format: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DebugConfig {
    pub enable_validation_panel: bool,
    pub enable_perf_metrics: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: String::new(),
            journal_mode: "wal".to_string(),
            synchronous: 2,
            wal_autocheckpoint: 1000,
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
        }
    }
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enable_validation_panel: false,
            enable_perf_metrics: false,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            logging: LogConfig::default(),
            debug: DebugConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn load_from_environment() -> Result<Self, super::ConfigError> {
        let mut config = Self::default();

        if let Ok(path) = std::env::var("DIAGRAM_TOOL_DB_PATH") {
            config.database.path = path;
        }

        if let Ok(journal_mode) = std::env::var("DIAGRAM_TOOL_JOURNAL_MODE") {
            config.database.journal_mode = journal_mode;
        }

        if let Ok(synchronous) = std::env::var("DIAGRAM_TOOL_SYNCHRONOUS") {
            config.database.synchronous = synchronous.parse().unwrap_or(2);
        }

        if let Ok(checkpoint) = std::env::var("DIAGRAM_TOOL_WAL_AUTOCHECKPOINT") {
            config.database.wal_autocheckpoint = checkpoint.parse().unwrap_or(1000);
        }

        if let Ok(level) = std::env::var("DIAGRAM_TOOL_LOG_LEVEL") {
            config.logging.level = level;
        }

        if let Ok(format) = std::env::var("DIAGRAM_TOOL_LOG_FORMAT") {
            config.logging.format = format;
        }

        if let Ok(validation) = std::env::var("DIAGRAM_TOOL_DEBUG_VALIDATION") {
            config.debug.enable_validation_panel = validation == "1" || validation.to_lowercase() == "true";
        }

        if let Ok(perf) = std::env::var("DIAGRAM_TOOL_DEBUG_PERF") {
            config.debug.enable_perf_metrics = perf == "1" || perf.to_lowercase() == "true";
        }

        Ok(config)
    }

    pub fn merge_from_config_file(mut self) -> Result<Self, super::ConfigError> {
        let config_path = self.find_config_file()?;
        
        if let Some(path) = config_path {
            if path.exists() {
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| super::ConfigError::FileRead {
                        path: path.clone(),
                        source: e,
                    })?;
                
                let file_config: FileConfig = serde_json::from_str(&content)
                    .map_err(|e| super::ConfigError::FileParse {
                        path: path.clone(),
                        source: e,
                    })?;
                
                self = file_config.merge_into(self);
            }
        }
        
        Ok(self)
    }

    pub fn validate(&self) -> Result<(), super::ConfigError> {
        super::validation::ConfigValidator::validate(self)
    }

    fn find_config_file(&self) -> Result<Option<PathBuf>, super::ConfigError> {
        let mut search_paths: Vec<PathBuf> = vec![PathBuf::from(".")];

        if let Some(config_dir) = dirs::config_dir() {
            search_paths.push(config_dir.join("diagram_tool"));
        }
        
        if let Some(home_dir) = dirs::home_dir() {
            search_paths.push(home_dir.join(".config").join("diagram_tool"));
        }

        for path in search_paths.iter() {
            let config_file = path.join(super::CONFIG_FILE_NAME);
            if config_file.exists() {
                return Ok(Some(config_file));
            }
        }

        Ok(None)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct FileConfig {
    database: Option<DatabaseFileConfig>,
    logging: Option<LogFileConfig>,
    debug: Option<DebugFileConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct DatabaseFileConfig {
    path: Option<String>,
    journal_mode: Option<String>,
    synchronous: Option<i32>,
    wal_autocheckpoint: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct LogFileConfig {
    level: Option<String>,
    format: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct DebugFileConfig {
    enable_validation_panel: Option<bool>,
    enable_perf_metrics: Option<bool>,
}

impl FileConfig {
    fn merge_into(self, mut config: AppConfig) -> AppConfig {
        if let Some(db) = self.database {
            if let Some(path) = db.path {
                config.database.path = path;
            }
            if let Some(journal_mode) = db.journal_mode {
                config.database.journal_mode = journal_mode;
            }
            if let Some(synchronous) = db.synchronous {
                config.database.synchronous = synchronous;
            }
            if let Some(checkpoint) = db.wal_autocheckpoint {
                config.database.wal_autocheckpoint = checkpoint;
            }
        }

        if let Some(log) = self.logging {
            if let Some(level) = log.level {
                config.logging.level = level;
            }
            if let Some(format) = log.format {
                config.logging.format = format;
            }
        }

        if let Some(debug) = self.debug {
            if let Some(validation) = debug.enable_validation_panel {
                config.debug.enable_validation_panel = validation;
            }
            if let Some(perf) = debug.enable_perf_metrics {
                config.debug.enable_perf_metrics = perf;
            }
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_database_config() {
        let db = DatabaseConfig::default();
        assert_eq!(db.journal_mode, "wal");
        assert_eq!(db.synchronous, 2);
        assert_eq!(db.wal_autocheckpoint, 1000);
    }

    #[test]
    fn test_load_from_environment_uses_defaults() {
        let config = AppConfig::load_from_environment().expect("should load");
        assert_eq!(config.database.journal_mode, "wal");
    }

    #[test]
    fn test_file_config_merge() {
        let file_config = FileConfig {
            database: Some(DatabaseFileConfig {
                path: Some("/custom/path.db".to_string()),
                journal_mode: None,
                synchronous: None,
                wal_autocheckpoint: None,
            }),
            logging: None,
            debug: None,
        };

        let mut app_config = AppConfig::default();
        app_config = file_config.merge_into(app_config);
        
        assert_eq!(app_config.database.path, "/custom/path.db");
    }
}
