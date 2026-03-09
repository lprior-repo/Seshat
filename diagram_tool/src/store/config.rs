use rusqlite::Connection;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JournalMode {
    Wal,
    Delete,
    Truncate,
    Persist,
    Memory,
    Off,
    Other(String),
}

impl From<String> for JournalMode {
    fn from(s: String) -> Self {
        match s.to_uppercase().as_str() {
            "WAL" => Self::Wal,
            "DELETE" => Self::Delete,
            "TRUNCATE" => Self::Truncate,
            "PERSIST" => Self::Persist,
            "MEMORY" => Self::Memory,
            "OFF" => Self::Off,
            _ => Self::Other(s),
        }
    }
}

impl std::fmt::Display for JournalMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wal => write!(f, "WAL"),
            Self::Delete => write!(f, "DELETE"),
            Self::Truncate => write!(f, "TRUNCATE"),
            Self::Persist => write!(f, "PERSIST"),
            Self::Memory => write!(f, "MEMORY"),
            Self::Off => write!(f, "OFF"),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SynchronousMode {
    Off = 0,
    Normal = 1,
    Full = 2,
    Extra = 3,
}

impl TryFrom<i32> for SynchronousMode {
    type Error = String;
    fn try_from(val: i32) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(Self::Off),
            1 => Ok(Self::Normal),
            2 => Ok(Self::Full),
            3 => Ok(Self::Extra),
            _ => Err(format!("Invalid synchronous mode: {}", val)),
        }
    }
}

impl std::fmt::Display for SynchronousMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as i32)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WalAutoCheckpoint(pub u32);

#[derive(Debug, Clone)]
pub struct StorePragmas {
    pub journal_mode: JournalMode,
    pub synchronous: SynchronousMode,
    pub wal_autocheckpoint: WalAutoCheckpoint,
}

#[derive(Debug)]
pub struct StoreBootstrap {
    pub conn: Connection,
    pub db_path: PathBuf,
    pub schema_version: i32,
}

#[derive(Debug)]
pub struct StoreConfig {
    pub pragmas: StorePragmas,
    pub schema_version: i32,
}
