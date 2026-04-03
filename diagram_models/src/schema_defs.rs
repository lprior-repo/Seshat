//! Schema definitions for `SQLite` database - single source of truth
//!
//! This module provides centralized schema definitions used by store.rs

#![allow(dead_code)]

use smallvec::smallvec;

/// Events table schema - consolidated definition
pub const SCHEMA_EVENTS_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS events (
    id TEXT NOT NULL PRIMARY KEY,
    revision INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    payload TEXT NOT NULL,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
";

/// Events revision index
pub const SCHEMA_EVENTS_REVISION_INDEX: &str = r"
CREATE INDEX IF NOT EXISTS idx_events_revision ON events(revision)
";

/// Events type index
pub const SCHEMA_EVENTS_TYPE_INDEX: &str = r"
CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type)
";

/// Snapshots table schema
pub const SCHEMA_SNAPSHOTS_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS snapshots (
    id INTEGER NOT NULL PRIMARY KEY,
    revision INTEGER NOT NULL UNIQUE,
    payload TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
";

/// Snapshots revision index
pub const SCHEMA_SNAPSHOTS_REVISION_INDEX: &str = r"
CREATE INDEX IF NOT EXISTS idx_snapshots_revision ON snapshots(revision DESC)
";

/// Schema version table
pub const SCHEMA_VERSION_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER NOT NULL PRIMARY KEY,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
";

/// AI Documents table schema
pub const SCHEMA_AI_DOCUMENTS_TABLE: &str = "CREATE TABLE IF NOT EXISTS ai_documents (\
     id TEXT PRIMARY KEY,\
     key TEXT NOT NULL,\
     json_payload TEXT NOT NULL,\
     location_type TEXT NOT NULL,\
     location_data TEXT NOT NULL,\
     created_at INTEGER\
 )";

/// All schema statements for initial setup.
///
/// Uses `SmallVec<[&'static str; 7]>` to avoid heap allocation for the
/// fixed-size collection of exactly 7 schema statements.
#[must_use]
pub fn all_schema_statements() -> smallvec::SmallVec<[&'static str; 7]> {
    smallvec![
        SCHEMA_VERSION_TABLE,
        SCHEMA_EVENTS_TABLE,
        SCHEMA_EVENTS_REVISION_INDEX,
        SCHEMA_EVENTS_TYPE_INDEX,
        SCHEMA_SNAPSHOTS_TABLE,
        SCHEMA_SNAPSHOTS_REVISION_INDEX,
        SCHEMA_AI_DOCUMENTS_TABLE,
    ]
}
