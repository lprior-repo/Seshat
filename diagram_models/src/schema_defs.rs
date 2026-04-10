//! Schema definitions for `SQLite` database - single source of truth
//!
//! This module provides centralized schema definitions used by the async store.
//! IMPORTANT: This must match the actual schema in `diagram_tool/src/store_async/bootstrap.rs`

#![allow(dead_code)]

use smallvec::smallvec;

/// Events table schema - consolidated definition
/// Columns: id (auto-increment), `operation_id` (unique), revision, payload (JSON), timestamp
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

/// Events `operation_id` index for idempotency checks
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

/// AI Documents key index for efficient lookups
pub const SCHEMA_AI_DOCUMENTS_KEY_INDEX: &str = r"
CREATE INDEX IF NOT EXISTS idx_ai_documents_key ON ai_documents(key)
";

pub const SCHEMA_SAGA_OPERATIONS_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS saga_operations (
    operation_id TEXT NOT NULL PRIMARY KEY,
    state TEXT NOT NULL DEFAULT 'started',
    current_step INTEGER NOT NULL DEFAULT 0,
    total_steps INTEGER NOT NULL,
    started_at INTEGER NOT NULL,
    completed_at INTEGER,
    final_revision INTEGER,
    error_message TEXT,
    author_id TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT ''
)
";

pub const SCHEMA_SAGA_OPERATIONS_STATE_INDEX: &str = r"
CREATE INDEX IF NOT EXISTS idx_saga_operations_state ON saga_operations(state)
";

pub const SCHEMA_STEP_JOURNAL_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS step_journal (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation_id TEXT NOT NULL,
    step_index INTEGER NOT NULL,
    step_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    event_revision INTEGER,
    created_at INTEGER NOT NULL,
    started_at INTEGER,
    completed_at INTEGER,
    error_message TEXT,
    FOREIGN KEY (operation_id) REFERENCES saga_operations(operation_id),
    UNIQUE(operation_id, step_index)
)
";

pub const SCHEMA_STEP_JOURNAL_OPERATION_INDEX: &str = r"
CREATE INDEX IF NOT EXISTS idx_step_journal_operation ON step_journal(operation_id, step_index)
";

/// All schema statements for initial setup.
///
/// Uses `SmallVec<[&'static str; 8]>` to avoid heap allocation for the
/// fixed-size collection of schema statements.
#[must_use]
pub fn all_schema_statements() -> smallvec::SmallVec<[&'static str; 12]> {
    smallvec![
        SCHEMA_VERSION_TABLE,
        SCHEMA_EVENTS_TABLE,
        SCHEMA_EVENTS_REVISION_INDEX,
        SCHEMA_EVENTS_OPERATION_ID_INDEX,
        SCHEMA_SNAPSHOTS_TABLE,
        SCHEMA_SNAPSHOTS_REVISION_INDEX,
        SCHEMA_AI_DOCUMENTS_TABLE,
        SCHEMA_AI_DOCUMENTS_KEY_INDEX,
        SCHEMA_SAGA_OPERATIONS_TABLE,
        SCHEMA_SAGA_OPERATIONS_STATE_INDEX,
        SCHEMA_STEP_JOURNAL_TABLE,
        SCHEMA_STEP_JOURNAL_OPERATION_INDEX,
    ]
}
