//! Schema definitions for the `ai_documents` `SQLite` table.
//!
//! This module provides the schema constant and types for AI document storage.

mod ai_document;
mod error;
mod location;

pub use ai_document::{AiDocument, JsonPayload, LocationData};
pub use error::AiDocumentError;
pub use location::{LocationType, LocationTypeParseError};

/// SQL schema for creating the `ai_documents` table.
///
/// Columns:
/// - `id TEXT PRIMARY KEY` - Document identifier
/// - `key TEXT NOT NULL` - Document key
/// - `json_payload TEXT NOT NULL` - JSON-encoded document data
/// - `location_type TEXT NOT NULL` - Type of location reference
/// - `location_data TEXT NOT NULL` - Location reference data
/// - `created_at INTEGER NOT NULL` - Unix timestamp of creation
pub const SCHEMA_AI_DOCUMENTS_TABLE: &str = crate::schema_defs::SCHEMA_AI_DOCUMENTS_TABLE;
