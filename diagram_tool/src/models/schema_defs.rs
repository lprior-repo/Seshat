//! Schema definitions for `SQLite` database - single source of truth
//!
//! This module provides centralized schema definitions to avoid duplication
//! between store.rs and models/events.rs

/// Events table schema - consolidated definition
pub const SCHEMA_EVENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS events (
    id TEXT NOT NULL PRIMARY KEY,
    revision INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    payload TEXT NOT NULL,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
"#;

/// Events revision index
pub const SCHEMA_EVENTS_REVISION_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_events_revision ON events(revision)
"#;

/// Events type index
pub const SCHEMA_EVENTS_TYPE_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type)
"#;

/// Snapshots table schema
pub const SCHEMA_SNAPSHOTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS snapshots (
    id INTEGER NOT NULL PRIMARY KEY,
    revision INTEGER NOT NULL UNIQUE,
    payload TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
"#;

/// Snapshots revision index
pub const SCHEMA_SNAPSHOTS_REVISION_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_snapshots_revision ON snapshots(revision DESC)
"#;

/// Schema version table
pub const SCHEMA_VERSION_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER NOT NULL PRIMARY KEY,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
"#;

/// All schema statements for initial setup
#[must_use]
pub fn all_schema_statements() -> Vec<&'static str> {
    vec![
        SCHEMA_VERSION_TABLE,
        SCHEMA_EVENTS_TABLE,
        SCHEMA_EVENTS_REVISION_INDEX,
        SCHEMA_EVENTS_TYPE_INDEX,
        SCHEMA_SNAPSHOTS_TABLE,
        SCHEMA_SNAPSHOTS_REVISION_INDEX,
    ]
}
