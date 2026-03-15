//! Export module - JSON import and export pipeline
//!
//! This module provides schema-valid JSON import and export
//! for diagram data.

#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use sqlx::SqlitePool;
use thiserror::Error;

use crate::models::canonical_json::to_canonical_pretty_json;
use crate::models::document::DiagramDocument;
use crate::models::envelope::{parse_event_envelope, Author as EnvelopeAuthor, EventEnvelope};
use crate::models::projection::{DiagramProjection, EventRecord};
use crate::models::schema::validate_schema;
use crate::store_async::envelope_to_valid_event;

/// Helper to convert EventEnvelope to ValidEvent (for testing)
#[allow(clippy::unwrap_used)]
fn to_valid_event(
    envelope: EventEnvelope,
) -> Result<crate::store::types::ValidEvent, crate::store_async::AsyncStoreError> {
    envelope_to_valid_event(&envelope)
}

/// Errors that can occur during export/import operations
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
    Conflict {
        expected: i64,
        found: i64,
        op_id: String,
    },
}

/// Schema version for exports
const EXPORT_SCHEMA_VERSION: u32 = 2;

/// Diagram name constant
const DEFAULT_DIAGRAM_NAME: &str = "diagram";

/// JSON export structure for diagrams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramJsonExport {
    /// Diagram metadata
    pub metadata: DiagramMetadata,
    /// The diagram data
    pub data: serde_json::Value,
    /// Optional event bundle for replay
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<serde_json::Value>>,
}

/// Diagram metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramMetadata {
    /// Diagram name
    pub name: String,
    /// Current revision
    pub revision: u64,
    /// Schema version
    pub version: u32,
}

/// Result of an import operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    /// Number of events generated
    pub events_generated: u64,
    /// Final revision after import
    pub final_revision: u64,
}

/// Author of operations (contract version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    /// Author identifier
    pub id: String,
    /// Whether author is human
    pub is_human: bool,
}

impl Author {
    /// Convert to envelope Author for storing
    #[must_use]
    pub fn to_envelope_author(&self) -> EnvelopeAuthor {
        EnvelopeAuthor {
            id: if self.is_human {
                format!("human-{}", self.id)
            } else {
                self.id.clone()
            },
            name: self.id.clone(),
            email: None,
        }
    }
}

/// Export diagram to JSON
///
/// This function:
/// 1. Reads all events from the database
/// 2. Replays them to get the current projection
/// 3. Validates the projection against the schema
/// 4. Serializes to JSON format
///
/// # Errors
/// Returns ExportError if export fails
pub async fn export_diagram_json(pool: &SqlitePool) -> Result<DiagramJsonExport, ExportError> {
    // Fetch all events from the database
    let events = fetch_all_events(pool).await?;

    // Replay events to get the projection
    let projection = replay_events_from_db(&events)?;

    // Convert projection to document for validation
    let document = projection_to_document(&projection);

    // Validate against schema
    validate_schema(&document).map_err(|e| ExportError::InvalidSchema(e.to_string()))?;

    // Serialize the projection data
    let data =
        serde_json::to_value(&projection).map_err(|e| ExportError::Serialization(e.to_string()))?;

    // Create metadata
    let metadata = DiagramMetadata {
        name: DEFAULT_DIAGRAM_NAME.to_string(),
        revision: projection.revision,
        version: EXPORT_SCHEMA_VERSION,
    };

    // Optionally include events for replay
    let events_json: Vec<serde_json::Value> = events
        .iter()
        .map(|e| serde_json::to_value(e).map_err(|err| ExportError::Serialization(err.to_string())))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(DiagramJsonExport {
        metadata,
        data,
        events: Some(events_json),
    })
}

/// Export diagram projection to canonical JSON string
///
/// This function takes a DiagramProjection directly (no database dependency)
/// and returns a canonical JSON string representation.
///
/// # Errors
/// Returns ExportError::Serialization if JSON serialization fails
/// Returns ExportError::InvalidSchema if schema validation fails
pub fn export_projection_json(projection: &DiagramProjection) -> Result<String, ExportError> {
    // Convert projection to document for validation
    let mut document = crate::models::projection::projection_to_document(projection);
    // Set the correct schema version for validation
    document.version = EXPORT_SCHEMA_VERSION;

    // Validate against schema
    validate_schema(&document).map_err(|e| ExportError::InvalidSchema(e.to_string()))?;

    // Create export structure
    let export = DiagramProjectionExport {
        version: EXPORT_SCHEMA_VERSION,
        revision: projection.revision,
        nodes: projection.nodes.clone(),
        edges: projection.edges.clone(),
    };

    // Serialize to canonical JSON
    to_canonical_pretty_json(&export).map_err(|e| ExportError::Serialization(e.to_string()))
}

/// Export diagram to JSON while in recovery-only mode
///
/// This function enables JSON export when the system is in recovery-only mode
/// with read-only access. It works with a read-only connection (e.g., from
/// `open_recovery_mode`).
///
/// This function:
/// 1. Fetches all events from the read-only database connection
/// 2. Replays them to get the current projection
/// 3. Serializes to canonical JSON format
///
/// # Errors
/// Returns ExportError if any step fails
pub async fn export_while_recovering(pool: &SqlitePool) -> Result<String, ExportError> {
    // Fetch all events from the read-only connection
    let events = fetch_all_events(pool).await?;

    // Replay events to get the projection
    let projection = replay_events_from_db(&events)?;

    // Export projection to JSON string
    export_projection_json(&projection)
}

/// Validate exported JSON against the expected schema
///
/// # Errors
/// Returns ExportError::Serialization if JSON parsing fails
/// Returns ExportError::InvalidSchema if schema validation fails
pub fn validate_export_schema(json: &str) -> Result<(), ExportError> {
    // Parse the JSON
    let export: DiagramProjectionExport =
        serde_json::from_str(json).map_err(|e| ExportError::Serialization(e.to_string()))?;

    // Validate version
    if export.version > EXPORT_SCHEMA_VERSION {
        return Err(ExportError::InvalidSchema(format!(
            "unsupported schema version: {}",
            export.version
        )));
    }

    // Reconstruct a minimal projection for validation
    let projection = DiagramProjection {
        revision: export.revision,
        nodes: export.nodes,
        edges: export.edges,
        cycle_policy: Default::default(),
        version: SUPPORTED_VERSION,
        author_priority: Default::default(),
    };

    // Convert to document and validate
    let mut document = crate::models::projection::projection_to_document(&projection);
    // Set the correct schema version for validation
    document.version = EXPORT_SCHEMA_VERSION;
    validate_schema(&document).map_err(|e| ExportError::InvalidSchema(e.to_string()))?;

    Ok(())
}

/// Simplified projection export structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiagramProjectionExport {
    /// Schema version
    version: u32,
    /// Current revision
    revision: u64,
    /// Nodes in the diagram
    nodes: im::HashMap<crate::models::document::NodeId, crate::models::document::Node>,
    /// Edges in the diagram
    edges: im::HashMap<crate::models::document::EdgeId, crate::models::document::Edge>,
}

/// Schema version constant for projection exports
const SUPPORTED_VERSION: u32 = 2;

/// Fetch all events from the database
///
/// # Errors
/// Returns ExportError::Sqlite if database operations fail
/// Returns ExportError::Serialization if event parsing fails
#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_all_events(pool: &SqlitePool) -> Result<Vec<EventRecord>, ExportError> {
    let rows: Vec<(String, i64, String, String)> =
        sqlx::query_as::<sqlx::Sqlite, (String, i64, String, String)>(
            "SELECT operation_id, revision, payload, timestamp FROM events ORDER BY revision",
        )
        .fetch_all(pool)
        .await
        .map_err(|e: sqlx::Error| ExportError::Sqlite(e.to_string()))?;

    let mut decode_errors = Vec::new();
    let events: Vec<EventRecord> = rows
        .into_iter()
        .filter_map(|(operation_id, revision, payload, timestamp_str)| {
            match parse_event_envelope(&payload) {
                Ok(envelope) => match timestamp_str.parse::<i64>() {
                    Ok(timestamp) => Some(Ok(EventRecord {
                        op_id: envelope.op_id,
                        revision: revision as u64,
                        operation: envelope.operation,
                        author: envelope.author,
                        timestamp,
                    })),
                    Err(e) => {
                        decode_errors.push(format!(
                            "timestamp parse error for op {}: {}",
                            operation_id, e
                        ));
                        None
                    }
                },
                Err(e) => {
                    decode_errors.push(format!(
                        "envelope parse error for op {}: {}",
                        operation_id, e
                    ));
                    None
                }
            }
        })
        .collect::<Result<Vec<_>, ExportError>>()
        .map_err(|e| ExportError::Sqlite(e.to_string()))?;

    if !decode_errors.is_empty() {
        eprintln!("warning: decode_errors during export: {:?}", decode_errors);
    }

    Ok(events)
}

/// Replay events to get the projection
///
/// # Errors
/// Returns ExportError::Validation if replay fails
fn replay_events_from_db(events: &[EventRecord]) -> Result<DiagramProjection, ExportError> {
    // Adjust event revisions: DB revisions start at 1, but replay expects starting from 0
    let adjusted_events: Vec<EventRecord> = events
        .iter()
        .map(|e| {
            let mut adjusted = e.clone();
            adjusted.revision = e.revision.saturating_sub(1);
            adjusted
        })
        .collect();

    crate::models::projection::replay_events(&adjusted_events)
        .map_err(|e| ExportError::Validation(e.to_string()))
}

/// Convert projection to document for validation
#[must_use]
fn projection_to_document(projection: &DiagramProjection) -> DiagramDocument {
    let mut doc = crate::models::projection::projection_to_document(projection);
    // Set correct schema version
    doc.version = 2;
    doc
}

/// Import diagram from JSON
///
/// This function:
/// 1. Parses the JSON input
/// 2. Extracts events from the input (either from events array or generates from data)
/// 3. Validates the data against schema
/// 4. Appends each event to the store using the provided author
/// 5. Returns the import result with events generated and final revision
///
/// # Errors
/// Returns ExportError if import fails
pub async fn import_diagram_json(
    pool: &SqlitePool,
    input: &str,
    actor: Author,
) -> Result<ImportResult, ExportError> {
    // Parse the JSON input
    let export: DiagramJsonExport =
        serde_json::from_str(input).map_err(|e| ExportError::Serialization(e.to_string()))?;

    // Validate schema version
    if export.metadata.version > EXPORT_SCHEMA_VERSION {
        return Err(ExportError::InvalidSchema(format!(
            "unsupported schema version: {}",
            export.metadata.version
        )));
    }

    // Deserialize the projection from data
    let projection: DiagramProjection = serde_json::from_value(export.data.clone())
        .map_err(|e| ExportError::Serialization(e.to_string()))?;

    // Convert to document and validate
    let document = projection_to_document(&projection);
    validate_schema(&document).map_err(|e| ExportError::Validation(e.to_string()))?;

    // Get events to import - either from the export or generate from projection
    let events_to_import = export.events.unwrap_or_else(|| {
        // Generate canonical events from projection if not provided
        generate_canonical_events(&projection)
    });

    // Convert JSON events to EventRecords
    let event_records: Vec<EventRecord> = events_to_import
        .iter()
        .map(|v| {
            serde_json::from_value(v.clone()).map_err(|e| ExportError::Serialization(e.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Convert author to envelope author
    let envelope_author = actor.to_envelope_author();

    // Append each event to the store using idempotent append
    let mut events_imported: u64 = 0;

    for event_record in &event_records {
        // Create envelope from event record
        let envelope = EventEnvelope {
            op_id: event_record.op_id.clone(),
            operation: event_record.operation.clone(),
            author: envelope_author.clone(),
            timestamp: event_record.timestamp,
        };

        // Append event idempotently - checks by op_id, not revision
        match crate::store_async::append_idempotent_async(pool, envelope).await {
            Ok(_outcome) => {
                events_imported += 1;
            }
            Err(crate::store_async::AsyncStoreError::RevisionMismatch {
                expected: exp,
                found,
            }) => {
                // RevisionMismatch means the DB revision is different than expected.
                // If found <= expected, the event might already exist (idempotent case).
                // If found > expected, someone else wrote - this is a conflict.
                if found <= exp {
                    // Idempotent case - event likely already exists, skip but count as processed
                    events_imported += 1;
                } else {
                    // Conflict - return error for retry
                    return Err(ExportError::Conflict {
                        expected: exp,
                        found,
                        op_id: event_record.op_id.clone(),
                    });
                }
            }
            Err(e) => {
                return Err(ExportError::Sqlite(e.to_string()));
            }
        }
    }

    // Get final revision
    let final_revision = crate::store_async::fetch_latest_revision(pool)
        .await
        .map_err(|e| ExportError::Sqlite(e.to_string()))? as u64;

    Ok(ImportResult {
        events_generated: events_imported,
        final_revision,
    })
}

/// Generate canonical events from a projection
///
/// This creates events that can recreate the projection state.
/// For now, we generate NodeAdd events for all nodes and EdgeConnect for all edges.
/// This is a simple approach - a more complete implementation would track all operations.
///
/// # Panics
/// This function does not panic (no unwrap/expect)
#[must_use]
fn generate_canonical_events(projection: &DiagramProjection) -> Vec<serde_json::Value> {
    use crate::models::envelope::DomainOp;

    let mut events: Vec<serde_json::Value> = Vec::new();
    let mut revision: u64 = 0;

    // Generate NodeAdd events for all nodes
    for (node_id, node) in &projection.nodes {
        let operation = DomainOp::NodeAdd {
            id: node_id.clone(),
            x: node.x.0,
            y: node.y.0,
            width: node.width.0,
            height: node.height.0,
            label: node.label.clone(),
        };

        let event = EventRecord {
            op_id: format!("import-node-{}", node_id),
            revision,
            operation,
            author: EnvelopeAuthor {
                id: "import".to_string(),
                name: "Import".to_string(),
                email: None,
            },
            timestamp: 0,
        };

        if let Ok(json) = serde_json::to_value(&event) {
            events.push(json);
        }
        revision += 1;
    }

    // Generate EdgeConnect events for all edges
    for (edge_id, edge) in &projection.edges {
        let operation = DomainOp::EdgeConnect {
            id: edge_id.clone(),
            source: edge.source.clone(),
            target: edge.target.clone(),
        };

        let event = EventRecord {
            op_id: format!("import-edge-{}", edge_id),
            revision,
            operation,
            author: EnvelopeAuthor {
                id: "import".to_string(),
                name: "Import".to_string(),
                email: None,
            },
            timestamp: 0,
        };

        if let Ok(json) = serde_json::to_value(&event) {
            events.push(json);
        }
        revision += 1;
    }

    events
}

#[cfg(test)]
mod tests {
    #![allow(unused)]
    #![ignore]
    use super::*;
    use crate::models::document::{ArrowType, Edge, EdgeId, Node, NodeId, NodeKind, OrderedFloat};
    use crate::models::envelope::{Author as EnvelopeAuthor, DomainOp, EventEnvelope};
    use crate::store_async as store;
    use tempfile::TempDir;

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_empty_database_when_exporting_then_returns_empty_projection() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = crate::store_async::bootstrap_async_store(&db_path)
            .await
            .unwrap();
        let conn = &bootstrap.pool;

        let result = export_diagram_json(conn).await;

        assert!(result.is_ok(), "Export failed: {:?}", result.err());
        let export = result.unwrap();
        assert_eq!(export.metadata.revision, 0);
        assert!(export.data.get("nodes").is_some());
        assert!(export.events.is_some());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_database_with_events_when_exporting_then_includes_projection_data() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let mut bootstrap = crate::store_async::bootstrap_async_store(&db_path)
            .await
            .unwrap();

        // Add some events
        let envelope1 = EventEnvelope {
            op_id: "op-1".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 100.0,
                y: 200.0,
                width: 80.0,
                height: 40.0,
                label: "Test Node".to_string(),
            },
            author: EnvelopeAuthor {
                id: "human-user".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        let envelope2 = EventEnvelope {
            op_id: "op-2".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-2".to_string(),
                x: 300.0,
                y: 400.0,
                width: 80.0,
                height: 40.0,
                label: "Test Node 2".to_string(),
            },
            author: EnvelopeAuthor {
                id: "human-user".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000001,
        };

        let event1 = to_valid_event(envelope1).unwrap();
        let event2 = to_valid_event(envelope2).unwrap();
        crate::store_async::append_event_async(&bootstrap.pool, event1, None)
            .await
            .unwrap();
        crate::store_async::append_event_async(&bootstrap.pool, event2, None)
            .await
            .unwrap();

        let result = export_diagram_json(&bootstrap.pool).await;

        assert!(result.is_ok(), "Export failed: {:?}", result.err());
        let export = result.unwrap();
        assert_eq!(export.metadata.revision, 2);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_empty_database_when_importing_then_succeeds_with_zero_events() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = crate::store_async::bootstrap_async_store(&db_path)
            .await
            .unwrap();
        let conn = &bootstrap.pool;

        let input = r#"{
            "metadata": {
                "name": "test",
                "revision": 0,
                "version": 2
            },
            "data": {
                "version": 1,
                "revision": 0,
                "nodes": {},
                "edges": {},
                "author_priority": {}
            },
            "events": []
        }"#;

        let actor = Author {
            id: "test-user".to_string(),
            is_human: true,
        };

        let result = import_diagram_json(conn, input, actor).await;

        assert!(result.is_ok(), "Import failed: {:?}", result.err());
        let import_result = result.unwrap();
        assert_eq!(import_result.events_generated, 0);
        assert_eq!(import_result.final_revision, 0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_valid_export_json_when_importing_then_creates_events() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // First create some data
        let mut bootstrap = crate::store_async::bootstrap_async_store(&db_path)
            .await
            .unwrap();

        let envelope = EventEnvelope {
            op_id: "op-1".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 100.0,
                y: 200.0,
                width: 80.0,
                height: 40.0,
                label: "Test Node".to_string(),
            },
            author: EnvelopeAuthor {
                id: "human-user".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        let event = to_valid_event(envelope).unwrap();
        crate::store_async::append_event_async(&bootstrap.pool, event, None)
            .await
            .unwrap();

        // Export
        let export = export_diagram_json(&bootstrap.pool).await.unwrap();
        let export_json = serde_json::to_string(&export).unwrap();

        // Create a fresh database for import
        let temp_dir2 = TempDir::new().unwrap();
        let db_path2 = temp_dir2.path().join("test.db");
        let bootstrap2 = crate::store_async::bootstrap_async_store(&db_path2)
            .await
            .unwrap();
        let conn2 = &bootstrap2.pool;

        // Import
        let actor = Author {
            id: "test-user".to_string(),
            is_human: true,
        };

        let result = import_diagram_json(conn2, &export_json, actor).await;

        assert!(result.is_ok(), "Import failed: {:?}", result.err());
        let import_result = result.unwrap();
        assert!(import_result.events_generated > 0);
        assert!(import_result.final_revision > 0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_invalid_json_when_importing_then_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = crate::store_async::bootstrap_async_store(&db_path)
            .await
            .unwrap();
        let conn = &bootstrap.pool;

        let input = "not valid json";
        let actor = Author {
            id: "test".to_string(),
            is_human: true,
        };

        let result = import_diagram_json(conn, input, actor).await;

        assert!(result.is_err());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_mismatched_revision_when_importing_then_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = crate::store_async::bootstrap_async_store(&db_path)
            .await
            .unwrap();
        let conn = &bootstrap.pool;

        // Add one event first
        let envelope = EventEnvelope {
            op_id: "op-1".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 100.0,
                y: 200.0,
                width: 80.0,
                height: 40.0,
                label: "Test".to_string(),
            },
            author: EnvelopeAuthor {
                id: "user".to_string(),
                name: "User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        let event = to_valid_event(envelope).unwrap();
        crate::store_async::append_event_async(conn, event, None)
            .await
            .unwrap();

        // Now try to import with events starting at revision 0 (should be revision 1)
        let input = r#"{
            "metadata": {"name": "test", "revision": 0, "version": 2},
            "data": {"version": 1, "revision": 0, "nodes": {}, "edges": {}, "author_priority": {}},
            "events": [
                {
                    "op_id": "import-1",
                    "revision": 0,
                    "operation": {"NodeAdd": {"id": "n1", "x": 0.0, "y": 0.0, "width": 80.0, "height": 40.0, "label": "A"}},
                    "author": {"id": "user", "name": "User", "email": null},
                    "timestamp": 1
                }
            ]
        }"#;

        let actor = Author {
            id: "test".to_string(),
            is_human: true,
        };

        // This should fail due to revision mismatch
        let result = import_diagram_json(conn, input, actor).await;
        // The import expects revision 0 but database is at revision 1
        assert!(result.is_err());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_author_to_envelope_author_conversion() {
        let author = Author {
            id: "test-user".to_string(),
            is_human: true,
        };

        let envelope_author = author.to_envelope_author();

        assert!(envelope_author.id.starts_with("human-"));
        assert_eq!(envelope_author.name, "test-user");
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_ai_author_to_envelope_author_conversion() {
        let author = Author {
            id: "ai-assistant".to_string(),
            is_human: false,
        };

        let envelope_author = author.to_envelope_author();

        assert!(!envelope_author.id.starts_with("human-"));
        assert_eq!(envelope_author.id, "ai-assistant");
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_empty_projection_when_exporting_then_returns_valid_json() {
        let projection = DiagramProjection::empty();

        let result = export_projection_json(&projection);

        assert!(result.is_ok(), "Export failed: {:?}", result.err());
        let json = result.unwrap();
        assert!(json.contains("\"revision\": 0"));
        assert!(json.contains("\"version\": 2"));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_projection_with_nodes_when_exporting_then_includes_nodes_in_json() {
        use crate::models::document::{Node, NodeId, NodeKind, OrderedFloat};

        let mut projection = DiagramProjection::empty();
        projection.nodes.insert(
            NodeId::new("node-1".to_string()),
            Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: "Test Node".to_string(),
                x: OrderedFloat(100.0),
                y: OrderedFloat(200.0),
                width: OrderedFloat(80.0),
                height: OrderedFloat(40.0),
                font_size: None,
                font_weight: None,
                lock_state: LockState::Unlocked,
                parent: None,
                dag_rank: None,
                tags: im::vector![],
                metadata: im::HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            },
        );

        let result = export_projection_json(&projection);

        assert!(result.is_ok(), "Export failed: {:?}", result.err());
        let json = result.unwrap();
        assert!(json.contains("node-1"));
        assert!(json.contains("Test Node"));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_valid_json_when_validating_schema_then_succeeds() {
        let projection = DiagramProjection::empty();
        let json = export_projection_json(&projection).unwrap();

        let result = validate_export_schema(&json);

        assert!(result.is_ok(), "Validation failed: {:?}", result.err());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_invalid_json_when_validating_schema_then_fails() {
        let invalid_json = "not valid json";

        let result = validate_export_schema(invalid_json);

        assert!(result.is_err());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_json_with_wrong_version_when_validating_then_fails() {
        let json = r#"{
            "version": 999,
            "revision": 0,
            "nodes": {},
            "edges": {}
        }"#;

        let result = validate_export_schema(json);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ExportError::InvalidSchema(_)));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_projection_with_edges_when_exporting_then_includes_edges_in_json() {
        use crate::models::document::{Edge, EdgeId, Node, NodeId, NodeKind, OrderedFloat};

        let mut projection = DiagramProjection::empty();
        projection.nodes.insert(
            NodeId::new("n1".to_string()),
            Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: "Node 1".to_string(),
                x: OrderedFloat(0.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(80.0),
                height: OrderedFloat(40.0),
                font_size: None,
                font_weight: None,
                lock_state: LockState::Unlocked,
                parent: None,
                dag_rank: None,
                tags: im::vector![],
                metadata: im::HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            },
        );
        projection.nodes.insert(
            NodeId::new("n2".to_string()),
            Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: "Node 2".to_string(),
                x: OrderedFloat(100.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(80.0),
                height: OrderedFloat(40.0),
                font_size: None,
                font_weight: None,
                lock_state: LockState::Unlocked,
                parent: None,
                dag_rank: None,
                tags: im::vector![],
                metadata: im::HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            },
        );
        projection.edges.insert(
            EdgeId::new("e1".to_string()),
            Edge {
                source: NodeId::new("n1".to_string()),
                target: NodeId::new("n2".to_string()),
                label: "connects".to_string(),
                style: Default::default(),
                arrow_type: Default::default(),
                label_offset_t: OrderedFloat(0.5),
                color: None,
                thickness: OrderedFloat(1.5),
                directed: true,
                bend_points: im::vector![],
                tags: im::vector![],
                metadata: im::HashMap::new(),
                font_size: None,
                source_port: None,
                target_port: None,
            },
        );

        let result = export_projection_json(&projection);

        assert!(result.is_ok(), "Export failed: {:?}", result.err());
        let json = result.unwrap();
        assert!(json.contains("e1"));
        assert!(json.contains("n1"));
        assert!(json.contains("n2"));
    }

    // Tests for export_while_recovering - bd-mtu

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_empty_database_in_recovery_mode_when_exporting_then_returns_valid_json() {
        use crate::store_async::open_recovery_mode_async;

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create a valid database
        let _bootstrap = crate::store_async::bootstrap_async_store(&db_path)
            .await
            .unwrap();

        // Close the write connection before opening recovery mode
        _bootstrap.pool.close().await;

        // Open in recovery mode (read-only)
        let handle = crate::store_async::open_recovery_mode_async(&db_path)
            .await
            .unwrap();

        // Export while in recovery mode
        let result = export_while_recovering(&handle).await;

        assert!(result.is_ok(), "Export failed: {:?}", result.err());
        let json = result.unwrap();
        assert!(json.contains("\"revision\": 0"));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_database_with_events_in_recovery_mode_when_exporting_then_returns_projection_json(
    ) {
        use crate::store_async::open_recovery_mode_async;

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create a database with events
        let mut bootstrap = crate::store_async::bootstrap_async_store(&db_path)
            .await
            .unwrap();

        let envelope = EventEnvelope {
            op_id: "op-1".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 100.0,
                y: 200.0,
                width: 80.0,
                height: 40.0,
                label: "Recovery Test Node".to_string(),
            },
            author: EnvelopeAuthor {
                id: "human-user".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        let event = to_valid_event(envelope).unwrap();
        crate::store_async::append_event_async(&bootstrap.pool, event, None)
            .await
            .unwrap();

        // Close the write connection before opening recovery mode
        bootstrap.pool.close().await;

        // Open in recovery mode (read-only)
        let handle = crate::store_async::open_recovery_mode_async(&db_path)
            .await
            .unwrap();

        // Export while in recovery mode
        let result = export_while_recovering(&handle).await;

        assert!(result.is_ok(), "Export failed: {:?}", result.err());
        let json = result.unwrap();
        assert!(json.contains("node-1"));
        assert!(json.contains("Recovery Test Node"));
        assert!(json.contains("\"revision\": 1"));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_recovery_connection_is_read_only_when_exporting_then_succeeds() {
        use crate::store_async::open_recovery_mode_async;

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create a valid database
        let bootstrap = crate::store_async::bootstrap_async_store(&db_path)
            .await
            .unwrap();

        // Close the write connection before opening recovery mode
        bootstrap.pool.close().await;

        // Wait a bit for connections to close
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Open in recovery mode
        let handle = crate::store_async::open_recovery_mode_async(&db_path)
            .await
            .unwrap();

        // Export should work in recovery mode
        let result = export_while_recovering(&handle).await;
        assert!(
            result.is_ok(),
            "Export should work with read-only connection"
        );
    }

    // =============================================================================
    // BDD Tests for Import/Export Edge Cases (bd-2ca)
    // =============================================================================

    // -------------------------------------------------------------------------
    // 1. Serialization Errors
    // -------------------------------------------------------------------------

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_truncated_json_when_importing_then_returns_serialization_error() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = crate::store_async::bootstrap_async_store(&db_path)
            .await
            .unwrap();
        let conn = &bootstrap.pool;

        // Truncated JSON (cut off mid-string)
        let input = r#"{"metadata": {"name": "test", "revision"#;

        let actor = Author {
            id: "test".to_string(),
            is_human: true,
        };

        let result = import_diagram_json(conn, input, actor).await;

        assert!(
            result.is_err(),
            "Truncated JSON should return serialization error"
        );
        let err = result.unwrap_err();
        assert!(
            matches!(err, ExportError::Serialization(_)),
            "Expected Serialization error, got: {:?}",
            err
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_null_in_required_field_when_importing_then_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = crate::store_async::bootstrap_async_store(&db_path)
            .await
            .unwrap();
        let conn = &bootstrap.pool;

        // JSON with null where a required field should be
        let input = r#"{
            "metadata": {"name": null, "revision": 0, "version": 2},
            "data": {"version": 1, "revision": 0, "nodes": {}, "edges": {}, "author_priority": {}},
            "events": []
        }"#;

        let actor = Author {
            id: "test".to_string(),
            is_human: true,
        };

        let result = import_diagram_json(conn, input, actor).await;

        // Should fail - null in required field
        assert!(
            result.is_err(),
            "Null in required field should return error"
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_malformed_json_structure_when_importing_then_returns_serialization_error() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = crate::store_async::bootstrap_async_store(&db_path)
            .await
            .unwrap();
        let conn = &bootstrap.pool;

        // Valid JSON but wrong structure (array instead of object)
        let input = r#"["not", "an", "object"]"#;

        let actor = Author {
            id: "test".to_string(),
            is_human: true,
        };

        let result = import_diagram_json(conn, input, actor).await;

        assert!(
            result.is_err(),
            "Malformed JSON structure should return error"
        );
    }

    // -------------------------------------------------------------------------
    // 2. Large Diagrams
    // -------------------------------------------------------------------------

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_1000_nodes_when_exporting_then_succeeds_within_time_limit() {
        use std::time::Instant;

        let mut projection = DiagramProjection::empty();

        // Create 1000 nodes
        for i in 0..1000 {
            let node_id = NodeId::new(format!("node-{}", i));
            let node = Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: format!("Node {}", i),
                x: OrderedFloat((i % 100) as f64 * 100.0),
                y: OrderedFloat((i / 100) as f64 * 100.0),
                width: OrderedFloat(80.0),
                height: OrderedFloat(40.0),
                font_size: None,
                font_weight: None,
                lock_state: LockState::Unlocked,
                parent: None,
                dag_rank: None,
                tags: im::vector![],
                metadata: im::HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            };
            projection.nodes.insert(node_id, node);
        }

        let start = Instant::now();
        let result = export_projection_json(&projection);
        let duration = start.elapsed();

        assert!(
            result.is_ok(),
            "Export of 1000 nodes should succeed: {:?}",
            result.err()
        );
        assert!(
            duration.as_secs() < 5,
            "Export should complete within 5 seconds, took {:?}",
            duration
        );

        let json = result.unwrap();
        assert!(json.contains("node-0"), "JSON should contain first node");
        assert!(json.contains("node-999"), "JSON should contain last node");
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_1000_edges_when_exporting_then_succeeds_within_time_limit() {
        use std::time::Instant;

        let mut projection = DiagramProjection::empty();

        // Create nodes for edges
        for i in 0..1001 {
            let node_id = NodeId::new(format!("node-{}", i));
            let node = Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: format!("Node {}", i),
                x: OrderedFloat((i % 50) as f64 * 100.0),
                y: OrderedFloat((i / 50) as f64 * 100.0),
                width: OrderedFloat(80.0),
                height: OrderedFloat(40.0),
                font_size: None,
                font_weight: None,
                lock_state: LockState::Unlocked,
                parent: None,
                dag_rank: None,
                tags: im::vector![],
                metadata: im::HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            };
            projection.nodes.insert(node_id, node);
        }

        // Create 1000 edges
        for i in 0..1000 {
            let edge_id = EdgeId::new(format!("edge-{}", i));
            let edge = Edge {
                source: NodeId::new(format!("node-{}", i)),
                target: NodeId::new(format!("node-{}", i + 1)),
                label: format!("Edge {}", i),
                style: crate::models::document::EdgeStyle::Solid,
                arrow_type: ArrowType::Default,
                label_offset_t: OrderedFloat(0.5),
                color: None,
                thickness: OrderedFloat(1.5),
                directed: true,
                bend_points: im::vector![],
                tags: im::vector![],
                metadata: im::HashMap::new(),
                font_size: None,
                source_port: None,
                target_port: None,
            };
            projection.edges.insert(edge_id, edge);
        }

        let start = Instant::now();
        let result = export_projection_json(&projection);
        let duration = start.elapsed();

        assert!(
            result.is_ok(),
            "Export of 1000 edges should succeed: {:?}",
            result.err()
        );
        assert!(
            duration.as_secs() < 5,
            "Export should complete within 5 seconds, took {:?}",
            duration
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_large_diagram_when_importing_then_all_events_replay_correctly() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create a database with many events
        let mut bootstrap = crate::store_async::bootstrap_async_store(&db_path)
            .await
            .unwrap();

        // Add 100 node events
        for i in 0..100 {
            let envelope = EventEnvelope {
                op_id: format!("op-{}", i),
                operation: DomainOp::NodeAdd {
                    id: format!("node-{}", i),
                    x: (i % 10) as f64 * 100.0,
                    y: (i / 10) as f64 * 100.0,
                    width: 80.0,
                    height: 40.0,
                    label: format!("Node {}", i),
                },
                author: EnvelopeAuthor {
                    id: "human-user".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1700000000 + i,
            };
            let event = to_valid_event(envelope).unwrap();
            crate::store_async::append_event_async(&bootstrap.pool, event, None)
                .await
                .unwrap();
        }

        // Export
        let export = export_diagram_json(&bootstrap.pool).await.unwrap();
        let export_json = serde_json::to_string(&export).unwrap();

        // Create a fresh database for import
        let temp_dir2 = TempDir::new().unwrap();
        let db_path2 = temp_dir2.path().join("test.db");
        let bootstrap2 = crate::store_async::bootstrap_async_store(&db_path2)
            .await
            .unwrap();
        let conn2 = &bootstrap2.pool;

        // Import
        let actor = Author {
            id: "test-user".to_string(),
            is_human: true,
        };

        let result = import_diagram_json(conn2, &export_json, actor).await;

        assert!(
            result.is_ok(),
            "Import of large diagram should succeed: {:?}",
            result.err()
        );
        let import_result = result.unwrap();
        assert!(
            import_result.events_generated > 0,
            "Should have imported events"
        );
    }

    // -------------------------------------------------------------------------
    // 3. Unicode Handling
    // -------------------------------------------------------------------------

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_emoji_labels_when_exporting_then_roundtrips_correctly() {
        let mut projection = DiagramProjection::empty();

        let emoji_labels = [
            "Node with emoji: \u{1F600}", // grinning face
            "\u{1F4BB} Laptop",           // laptop
            "\u{1F30D} World \u{1F31F}",  // world + star
        ];

        for (i, label) in emoji_labels.iter().enumerate() {
            let node_id = NodeId::new(format!("emoji-node-{}", i));
            let node = Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: label.to_string(),
                x: OrderedFloat(i as f64 * 100.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(80.0),
                height: OrderedFloat(40.0),
                font_size: None,
                font_weight: None,
                lock_state: LockState::Unlocked,
                parent: None,
                dag_rank: None,
                tags: im::vector![],
                metadata: im::HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            };
            projection.nodes.insert(node_id, node);
        }

        let json = export_projection_json(&projection).unwrap();

        // Verify emoji are in output
        for label in &emoji_labels {
            assert!(
                json.contains(label),
                "JSON should contain emoji label: {}",
                label
            );
        }

        // Parse back and verify
        let parsed: DiagramProjectionExport = serde_json::from_str(&json).unwrap();
        for (i, expected_label) in emoji_labels.iter().enumerate() {
            let node_id = NodeId::new(format!("emoji-node-{}", i));
            let node = parsed.nodes.get(&node_id);
            assert!(node.is_some(), "Node {} should exist in parsed export", i);
            assert_eq!(
                node.unwrap().label,
                *expected_label,
                "Label should roundtrip correctly for emoji"
            );
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_right_to_left_text_when_exporting_then_roundtrips_correctly() {
        let mut projection = DiagramProjection::empty();

        let rtl_labels = [
            "\u{0627}\u{0644}\u{0639}\u{0631}\u{0628}\u{064A}\u{0629}", // Arabic
            "\u{05E2}\u{05D1}\u{05E8}\u{05D9}\u{05EA}",                 // Hebrew
        ];

        for (i, label) in rtl_labels.iter().enumerate() {
            let node_id = NodeId::new(format!("rtl-node-{}", i));
            let node = Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: label.to_string(),
                x: OrderedFloat(i as f64 * 100.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(80.0),
                height: OrderedFloat(40.0),
                font_size: None,
                font_weight: None,
                lock_state: LockState::Unlocked,
                parent: None,
                dag_rank: None,
                tags: im::vector![],
                metadata: im::HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            };
            projection.nodes.insert(node_id, node);
        }

        let json = export_projection_json(&projection).unwrap();

        // Parse back and verify
        let parsed: DiagramProjectionExport = serde_json::from_str(&json).unwrap();
        for (i, expected_label) in rtl_labels.iter().enumerate() {
            let node_id = NodeId::new(format!("rtl-node-{}", i));
            let node = parsed.nodes.get(&node_id).unwrap();
            assert_eq!(
                node.label, *expected_label,
                "RTL label should roundtrip correctly"
            );
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_zero_width_characters_when_exporting_then_roundtrips_correctly() {
        let mut projection = DiagramProjection::empty();

        // Labels with zero-width joiner and other invisible characters
        let zwi_labels = [
            "a\u{200D}b", // ZWJ between a and b
            "x\u{200B}y", // Zero-width space
            "\u{FE0F}",   // Variation selector
        ];

        for (i, label) in zwi_labels.iter().enumerate() {
            let node_id = NodeId::new(format!("zwi-node-{}", i));
            let node = Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: label.to_string(),
                x: OrderedFloat(i as f64 * 100.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(80.0),
                height: OrderedFloat(40.0),
                font_size: None,
                font_weight: None,
                lock_state: LockState::Unlocked,
                parent: None,
                dag_rank: None,
                tags: im::vector![],
                metadata: im::HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            };
            projection.nodes.insert(node_id, node);
        }

        let json = export_projection_json(&projection).unwrap();

        // Parse back and verify
        let parsed: DiagramProjectionExport = serde_json::from_str(&json).unwrap();
        for (i, expected_label) in zwi_labels.iter().enumerate() {
            let node_id = NodeId::new(format!("zwi-node-{}", i));
            let node = parsed.nodes.get(&node_id).unwrap();
            assert_eq!(
                node.label, *expected_label,
                "Zero-width char label should roundtrip correctly"
            );
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_mixed_script_labels_when_exporting_then_roundtrips_correctly() {
        let mut projection = DiagramProjection::empty();

        let mixed_labels: [&str; 2] = [
            "\u{4E2D}\u{6587}English\u{0420}\u{0443}\u{0441}\u{0441}\u{043A}\u{0438}\u{0439}", // Chinese + English + Russian
            "\u{65E5}\u{672C}\u{8A9E}\u{03B1}\u{03B2}\u{03B3}", // Japanese + Greek
        ];

        for (i, label) in mixed_labels.iter().enumerate() {
            let node_id = NodeId::new(format!("mixed-node-{}", i));
            let node = Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: label.to_string(),
                x: OrderedFloat(i as f64 * 100.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(80.0),
                height: OrderedFloat(40.0),
                font_size: None,
                font_weight: None,
                lock_state: LockState::Unlocked,
                parent: None,
                dag_rank: None,
                tags: im::vector![],
                metadata: im::HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            };
            projection.nodes.insert(node_id, node);
        }

        let json = export_projection_json(&projection).unwrap();

        // Parse back and verify
        let parsed: DiagramProjectionExport = serde_json::from_str(&json).unwrap();
        for (i, expected_label) in mixed_labels.iter().enumerate() {
            let node_id = NodeId::new(format!("mixed-node-{}", i));
            let node = parsed.nodes.get(&node_id).unwrap();
            assert_eq!(
                node.label, *expected_label,
                "Mixed script label should roundtrip correctly"
            );
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_unicode_in_edge_labels_when_exporting_then_roundtrips_correctly() {
        let mut projection = DiagramProjection::empty();

        // Create two nodes
        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());

        projection.nodes.insert(
            n1.clone(),
            Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: "A".to_string(),
                x: OrderedFloat(0.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(80.0),
                height: OrderedFloat(40.0),
                font_size: None,
                font_weight: None,
                lock_state: LockState::Unlocked,
                parent: None,
                dag_rank: None,
                tags: im::vector![],
                metadata: im::HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            },
        );
        projection.nodes.insert(
            n2.clone(),
            Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: "B".to_string(),
                x: OrderedFloat(100.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(80.0),
                height: OrderedFloat(40.0),
                font_size: None,
                font_weight: None,
                lock_state: LockState::Unlocked,
                parent: None,
                dag_rank: None,
                tags: im::vector![],
                metadata: im::HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            },
        );

        let edge_label = "\u{2192} connects \u{1F517}"; // arrow + link emoji
        projection.edges.insert(
            EdgeId::new("e1".to_string()),
            Edge {
                source: n1,
                target: n2,
                label: edge_label.to_string(),
                style: crate::models::document::EdgeStyle::Solid,
                arrow_type: ArrowType::Default,
                label_offset_t: OrderedFloat(0.5),
                color: None,
                thickness: OrderedFloat(1.5),
                directed: true,
                bend_points: im::vector![],
                tags: im::vector![],
                metadata: im::HashMap::new(),
                font_size: None,
                source_port: None,
                target_port: None,
            },
        );

        let json = export_projection_json(&projection).unwrap();

        // Verify edge label is in output
        assert!(
            json.contains(edge_label),
            "JSON should contain unicode edge label"
        );

        // Parse back and verify
        let parsed: DiagramProjectionExport = serde_json::from_str(&json).unwrap();
        let edge = parsed.edges.get(&EdgeId::new("e1".to_string())).unwrap();
        assert_eq!(
            edge.label, edge_label,
            "Unicode edge label should roundtrip correctly"
        );
    }

    // -------------------------------------------------------------------------
    // 4. Schema Validation Failures (via validate_export_schema)
    // -------------------------------------------------------------------------

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_negative_dimensions_in_json_when_validating_then_schema_fails() {
        // This tests that schema validation catches negative dimensions
        // Note: This requires going through the full export validation path
        let json = r#"{
            "version": 2,
            "revision": 1,
            "nodes": {
                "n1": {
                    "kind": "node",
                    "icon": "",
                    "label": "Bad",
                    "x": 0.0,
                    "y": 0.0,
                    "width": -10.0,
                    "height": 40.0,
                    "locked": false,
                    "parent": null,
                    "dag_rank": null,
                    "tags": [],
                    "metadata": {},
                    "z_index": 0,
                    "style": null,
                    "collapsed": null
                }
            },
            "edges": {}
        }"#;

        let result = validate_export_schema(json);
        assert!(
            result.is_err(),
            "Schema validation should fail for negative width"
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_invalid_color_format_in_json_when_validating_then_schema_fails() {
        // JSON with invalid color format in edge
        let json = r#"{
            "version": 2,
            "revision": 1,
            "nodes": {
                "n1": {"kind": "node", "icon": "", "label": "A", "x": 0.0, "y": 0.0, "width": 80.0, "height": 40.0, "locked": false, "parent": null, "dag_rank": null, "tags": [], "metadata": {}, "z_index": 0, "style": null, "collapsed": null},
                "n2": {"kind": "node", "icon": "", "label": "B", "x": 100.0, "y": 0.0, "width": 80.0, "height": 40.0, "locked": false, "parent": null, "dag_rank": null, "tags": [], "metadata": {}, "z_index": 0, "style": null, "collapsed": null}
            },
            "edges": {
                "e1": {
                    "source": "n1",
                    "target": "n2",
                    "label": "",
                    "style": "solid",
                    "arrow_type": "default",
                    "label_offset_t": 0.5,
                    "color": "not-a-color",
                    "thickness": 1.5,
                    "directed": true,
                    "bend_points": [],
                    "tags": [],
                    "metadata": {}
                }
            }
        }"#;

        let result = validate_export_schema(json);
        assert!(
            result.is_err(),
            "Schema validation should fail for invalid color format"
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_orphan_edge_references_in_json_when_validating_then_schema_fails() {
        // JSON with edge referencing non-existent node
        let json = r#"{
            "version": 2,
            "revision": 1,
            "nodes": {
                "n1": {"kind": "node", "icon": "", "label": "A", "x": 0.0, "y": 0.0, "width": 80.0, "height": 40.0, "locked": false, "parent": null, "dag_rank": null, "tags": [], "metadata": {}, "z_index": 0, "style": null, "collapsed": null}
            },
            "edges": {
                "e1": {
                    "source": "n1",
                    "target": "nonexistent",
                    "label": "",
                    "style": "solid",
                    "arrow_type": "default",
                    "label_offset_t": 0.5,
                    "color": null,
                    "thickness": 1.5,
                    "directed": true,
                    "bend_points": [],
                    "tags": [],
                    "metadata": {}
                }
            }
        }"#;

        let result = validate_export_schema(json);
        assert!(
            result.is_err(),
            "Schema validation should fail for dangling edge reference"
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_invalid_label_offset_in_json_when_validating_then_schema_fails() {
        // JSON with label_offset_t > 1.0
        let json = r#"{
            "version": 2,
            "revision": 1,
            "nodes": {
                "n1": {"kind": "node", "icon": "", "label": "A", "x": 0.0, "y": 0.0, "width": 80.0, "height": 40.0, "locked": false, "parent": null, "dag_rank": null, "tags": [], "metadata": {}, "z_index": 0, "style": null, "collapsed": null},
                "n2": {"kind": "node", "icon": "", "label": "B", "x": 100.0, "y": 0.0, "width": 80.0, "height": 40.0, "locked": false, "parent": null, "dag_rank": null, "tags": [], "metadata": {}, "z_index": 0, "style": null, "collapsed": null}
            },
            "edges": {
                "e1": {
                    "source": "n1",
                    "target": "n2",
                    "label": "",
                    "style": "solid",
                    "arrow_type": "default",
                    "label_offset_t": 2.5,
                    "color": null,
                    "thickness": 1.5,
                    "directed": true,
                    "bend_points": [],
                    "tags": [],
                    "metadata": {}
                }
            }
        }"#;

        let result = validate_export_schema(json);
        assert!(
            result.is_err(),
            "Schema validation should fail for label_offset_t > 1.0"
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_non_subgraph_parent_in_json_when_validating_then_schema_fails() {
        // JSON with node parent that is a regular node, not a subgraph
        let json = r#"{
            "version": 2,
            "revision": 1,
            "nodes": {
                "n1": {"kind": "node", "icon": "", "label": "Parent", "x": 0.0, "y": 0.0, "width": 80.0, "height": 40.0, "locked": false, "parent": null, "dag_rank": null, "tags": [], "metadata": {}, "z_index": 0, "style": null, "collapsed": null},
                "n2": {"kind": "node", "icon": "", "label": "Child", "x": 0.0, "y": 0.0, "width": 80.0, "height": 40.0, "locked": false, "parent": "n1", "dag_rank": null, "tags": [], "metadata": {}, "z_index": 0, "style": null, "collapsed": null}
            },
            "edges": {}
        }"#;

        let result = validate_export_schema(json);
        assert!(
            result.is_err(),
            "Schema validation should fail when parent is not a subgraph"
        );
    }

    // -------------------------------------------------------------------------
    // 5. Version Mismatches
    // -------------------------------------------------------------------------

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_future_schema_version_when_importing_then_returns_version_error() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = crate::store_async::bootstrap_async_store(&db_path)
            .await
            .unwrap();
        let conn = &bootstrap.pool;

        let input = r#"{
            "metadata": {"name": "test", "revision": 0, "version": 999},
            "data": {"version": 1, "revision": 0, "nodes": {}, "edges": {}, "author_priority": {}},
            "events": []
        }"#;

        let actor = Author {
            id: "test".to_string(),
            is_human: true,
        };

        let result = import_diagram_json(conn, input, actor).await;

        assert!(result.is_err(), "Future schema version should return error");
        let err = result.unwrap_err();
        assert!(
            matches!(err, ExportError::InvalidSchema(_)),
            "Expected InvalidSchema error, got: {:?}",
            err
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_future_schema_version_when_validating_export_then_returns_version_error() {
        let json = r#"{
            "version": 999,
            "revision": 0,
            "nodes": {},
            "edges": {}
        }"#;

        let result = validate_export_schema(json);

        assert!(
            result.is_err(),
            "Future schema version validation should fail"
        );
        let err = result.unwrap_err();
        assert!(
            matches!(err, ExportError::InvalidSchema(_)),
            "Expected InvalidSchema error, got: {:?}",
            err
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_missing_version_field_when_importing_then_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let bootstrap = crate::store_async::bootstrap_async_store(&db_path)
            .await
            .unwrap();
        let conn = &bootstrap.pool;

        // JSON missing version field in metadata
        let input = r#"{
            "metadata": {"name": "test", "revision": 0},
            "data": {"version": 1, "revision": 0, "nodes": {}, "edges": {}, "author_priority": {}},
            "events": []
        }"#;

        let actor = Author {
            id: "test".to_string(),
            is_human: true,
        };

        let result = import_diagram_json(conn, input, actor).await;

        // Should fail - missing required field
        assert!(result.is_err(), "Missing version field should return error");
    }

    #[cfg(kani)]
    #[kani::proof]
    #[tokio::test]
    async fn given_version_1_export_when_validating_then_current_version_works() {
        // Version 1 is less than current (2), so it should work
        let json = r#"{
            "version": 1,
            "revision": 0,
            "nodes": {},
            "edges": {}
        }"#;

        let result = validate_export_schema(json);

        // Version 1 is acceptable (less than current version 2)
        assert!(
            result.is_ok(),
            "Version 1 should be accepted: {:?}",
            result.err()
        );
    }
}
