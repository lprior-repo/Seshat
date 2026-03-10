//! Export module

#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::models::envelope::Author as EnvelopeAuthor;

#[derive(Debug, Error, Clone)]
pub enum ExportError {
    #[error("invalid schema: {0}")]
    InvalidSchema(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("SQLite error: {0}")]
    Sqlite(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("conflict: expected revision {expected} but found {found} for op {op_id}")]
    Conflict { expected: i64, found: i64, op_id: String },
}

pub const EXPORT_SCHEMA_VERSION: u32 = 2;
pub const DEFAULT_DIAGRAM_NAME: &str = "diagram";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramJsonExport {
    pub metadata: DiagramMetadata,
    pub data: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramMetadata {
    pub name: String,
    pub revision: u64,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub events_generated: u64,
    pub final_revision: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub id: String,
    pub is_human: bool,
}

impl Author {
    #[must_use]
    pub fn to_envelope_author(&self) -> EnvelopeAuthor {
        EnvelopeAuthor {
            id: if self.is_human { format!("human-{}", self.id) } else { self.id.clone() },
            name: self.id.clone(),
            email: None,
        }
    }
}

pub mod ops;
pub use ops::*;
