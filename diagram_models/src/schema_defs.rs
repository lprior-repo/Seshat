//! Schema definitions for `SQLite` database - single source of truth
//!
//! This module provides centralized schema definitions used by the async store.
//! IMPORTANT: This must match the actual schema in diagram_tool/src/store_async/bootstrap.rs

#![allow(dead_code)]

use smallvec::smallvec;

/// Events table schema - consolidated definition
/// Columns: id (auto-increment), operation_id (unique), revision, payload (JSON), timestamp
pub const SCHEMA_EVENTS_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation_id TEXT NOT NULL UNIQUE,
    revision INTEGER NOT NULL,
    payload TEXT NOT NULL,
    timestamp TEXT NOT NULL
)
";

/// Events revision index for ordered retrieval
pub const SCHEMA_EVENTS_REVISION_INDEX: &str = r"
CREATE INDEX IF NOT EXISTS idx_events_revision ON events(revision)
";

/// Events operation_id index for idempotency checks
pub const SCHEMA_EVENTS_OPERATION_ID_INDEX: &str = r"
CREATE INDEX IF NOT EXISTS idx_events_operation_id ON events(operation_id)
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
pub const SCHEMA_AI_DOCUMENTS_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS ai_documents (
    id TEXT NOT NULL PRIMARY KEY,
    key TEXT NOT NULL,
    json_payload TEXT NOT NULL,
    location_type TEXT NOT NULL,
    location_data TEXT NOT NULL,
    created_at INTEGER NOT NULL
)
";

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
        SCHEMA_EVENTS_OPERATION_ID_INDEX,
        SCHEMA_SNAPSHOTS_TABLE,
        SCHEMA_SNAPSHOTS_REVISION_INDEX,
        SCHEMA_AI_DOCUMENTS_TABLE,
    ]
}
