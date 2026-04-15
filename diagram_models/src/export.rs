//! Import/Export module for diagram JSON serialization.
//!
//! This module provides functions for exporting and importing diagram data
//! to/from JSON format, with support for event sourcing storage.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub id: String,
    pub is_human: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramJsonExport {
    pub metadata: ExportMetadata,
    pub data: ExportData,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub name: String,
    pub revision: u32,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub version: u32,
    pub revision: u32,
    pub nodes: HashMap<String, serde_json::Value>,
    pub edges: HashMap<String, serde_json::Value>,
    pub cycle_policy: String,
    pub author_priority: Vec<String>,
}

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("Invalid schema: {0}")]
    InvalidSchema(String),
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub fn export_diagram_json(conn: &rusqlite::Connection) -> Result<DiagramJsonExport, ExportError> {
    let mut stmt = conn
        .prepare("SELECT operation_id, revision, payload, timestamp FROM events ORDER BY revision")
        .map_err(|e| ExportError::IoError(e.to_string()))?;

    let events: Vec<serde_json::Value> = stmt
        .query_map([], |row| {
            let payload: String = row.get(2)?;
            let parsed: serde_json::Value = serde_json::from_str(&payload)
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
            Ok(parsed)
        })
        .map_err(|e| ExportError::IoError(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(DiagramJsonExport {
        metadata: ExportMetadata {
            name: "diagram".to_string(),
            revision: 0,
            version: 2,
        },
        data: ExportData {
            version: 2,
            revision: 0,
            nodes: HashMap::new(),
            edges: HashMap::new(),
            cycle_policy: "default".to_string(),
            author_priority: Vec::new(),
        },
        events: if events.is_empty() {
            None
        } else {
            Some(events)
        },
    })
}

pub fn import_diagram_json(
    conn: &mut rusqlite::Connection,
    json: &str,
    _actor: Author,
) -> Result<(), ExportError> {
    let export: DiagramJsonExport =
        serde_json::from_str(json).map_err(|e| ExportError::InvalidSchema(e.to_string()))?;

    if export.metadata.version > 2 {
        return Err(ExportError::InvalidSchema(format!(
            "Unsupported version: {}",
            export.metadata.version
        )));
    }

    let mut tx = conn
        .transaction()
        .map_err(|e| ExportError::IoError(e.to_string()))?;

    if let Some(events) = export.events {
        for event in events {
            let payload = serde_json::to_string(&event)
                .map_err(|e| ExportError::SerializationError(e.to_string()))?;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            tx.execute(
                "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
                (&format!("op-{}", now), 0, payload, now),
            ).map_err(|e| ExportError::IoError(e.to_string()))?;
        }
    }

    tx.commit()
        .map_err(|e| ExportError::IoError(e.to_string()))?;

    Ok(())
}

pub fn export_while_recovering(conn: &rusqlite::Connection) -> Result<String, ExportError> {
    let export = export_diagram_json(conn)?;
    serde_json::to_string(&export).map_err(|e| ExportError::SerializationError(e.to_string()))
}

pub fn validate_export_schema(json: &str) -> Result<(), ExportError> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| ExportError::InvalidSchema(e.to_string()))?;

    let obj = value
        .as_object()
        .ok_or_else(|| ExportError::InvalidSchema("Expected JSON object".to_string()))?;

    let version = obj
        .get("version")
        .ok_or_else(|| ExportError::InvalidSchema("Missing version field".to_string()))?
        .as_u64()
        .unwrap_or(0) as u32;

    if version > 2 {
        return Err(ExportError::InvalidSchema(format!(
            "Version {} is not supported",
            version
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn author_creation() {
        let author = Author {
            id: "test".to_string(),
            is_human: true,
        };
        assert_eq!(author.id, "test");
        assert!(author.is_human);
    }

    #[test]
    fn export_metadata_serialization() {
        let metadata = ExportMetadata {
            name: "test".to_string(),
            revision: 1,
            version: 2,
        };
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("1"));
        assert!(json.contains("2"));
    }
}
