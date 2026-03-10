//! Export operations
#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use sqlx::SqlitePool;
use crate::models::canonical_json::to_canonical_pretty_json;
use crate::models::envelope::{parse_event_envelope, EventEnvelope};
use crate::models::projection::{replay_events, DiagramProjection, EventRecord};
use crate::models::schema::validate_schema;

use super::{Author, DiagramJsonExport, DiagramMetadata, ExportError, ImportResult, EXPORT_SCHEMA_VERSION, DEFAULT_DIAGRAM_NAME};

const SUPPORTED_VERSION: u32 = 2;

pub async fn export_diagram_json(pool: &SqlitePool) -> Result<DiagramJsonExport, ExportError> {
    let events = fetch_all_events(pool).await?;
    let projection = replay_events_from_db(&events)?;
    let document = projection_to_document(&projection);
    validate_schema(&document).map_err(|e| ExportError::InvalidSchema(e.to_string()))?;
    let data = serde_json::to_value(&projection).map_err(|e| ExportError::Serialization(e.to_string()))?;
    let metadata = DiagramMetadata { name: DEFAULT_DIAGRAM_NAME.to_string(), revision: projection.revision, version: EXPORT_SCHEMA_VERSION };
    let events_json: Vec<serde_json::Value> = events.iter().map(|e| serde_json::to_value(e).map_err(|err| ExportError::Serialization(err.to_string()))).collect::<Result<Vec<_>, _>>()?;
    Ok(DiagramJsonExport { metadata, data, events: Some(events_json) })
}

pub fn export_projection_json(projection: &DiagramProjection) -> Result<String, ExportError> {
    let mut document = crate::models::projection::projection_to_document(projection);
    document.version = EXPORT_SCHEMA_VERSION;
    validate_schema(&document).map_err(|e| ExportError::InvalidSchema(e.to_string()))?;
    let export = DiagramProjectionExport { version: EXPORT_SCHEMA_VERSION, revision: projection.revision, nodes: projection.nodes.clone(), edges: projection.edges.clone() };
    to_canonical_pretty_json(&export).map_err(|e| ExportError::Serialization(e.to_string()))
}

pub async fn export_while_recovering(pool: &SqlitePool) -> Result<String, ExportError> {
    let events = fetch_all_events(pool).await?;
    let projection = replay_events_from_db(&events)?;
    export_projection_json(&projection)
}

pub fn validate_export_schema(json: &str) -> Result<(), ExportError> {
    let export: DiagramProjectionExport = serde_json::from_str(json).map_err(|e| ExportError::Serialization(e.to_string()))?;
    if export.version > EXPORT_SCHEMA_VERSION { return Err(ExportError::InvalidSchema(format!("unsupported schema version: {}", export.version))); }
    let projection = DiagramProjection { revision: export.revision, nodes: export.nodes, edges: export.edges, cycle_policy: Default::default(), version: SUPPORTED_VERSION, author_priority: Default::default() };
    let mut document = crate::models::projection::projection_to_document(&projection);
    document.version = EXPORT_SCHEMA_VERSION;
    validate_schema(&document).map_err(|e| ExportError::InvalidSchema(e.to_string()))?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiagramProjectionExport { version: u32, revision: u64, nodes: im::HashMap<crate::models::document::NodeId, crate::models::document::Node>, edges: im::HashMap<crate::models::document::EdgeId, crate::models::document::Edge> }

pub async fn fetch_all_events(pool: &SqlitePool) -> Result<Vec<EventRecord>, ExportError> {
    let rows: Vec<(String, i64, String, String)> = sqlx::query_as::<sqlx::Sqlite, (String, i64, String, String)>("SELECT operation_id, revision, payload, timestamp FROM events ORDER BY revision").fetch_all(pool).await.map_err(|e| ExportError::Sqlite(e.to_string()))?;
    let events: Vec<EventRecord> = rows.into_iter().filter_map(|(operation_id, revision, payload, timestamp_str)| {
        match parse_event_envelope(&payload) {
            Ok(envelope) => timestamp_str.parse::<i64>().ok().map(|timestamp| EventRecord { op_id: envelope.op_id, revision: revision as u64, operation: envelope.operation, author: envelope.author, timestamp }),
            Err(_) => None,
        }
    }).collect();
    Ok(events)
}

fn replay_events_from_db(events: &[EventRecord]) -> Result<DiagramProjection, ExportError> {
    let adjusted_events: Vec<EventRecord> = events.iter().map(|e| { let mut adjusted = e.clone(); adjusted.revision = e.revision.saturating_sub(1); adjusted }).collect();
    replay_events(&adjusted_events).map_err(|e| ExportError::Validation(e.to_string()))
}

#[must_use]
fn projection_to_document(projection: &DiagramProjection) -> crate::models::document::DiagramDocument {
    let mut doc = crate::models::projection::projection_to_document(projection);
    doc.version = 2;
    doc
}

pub async fn import_diagram_json(pool: &SqlitePool, input: &str, actor: Author) -> Result<ImportResult, ExportError> {
    let export: DiagramJsonExport = serde_json::from_str(input).map_err(|e| ExportError::Serialization(e.to_string()))?;
    if export.metadata.version > EXPORT_SCHEMA_VERSION { return Err(ExportError::InvalidSchema(format!("unsupported schema version: {}", export.metadata.version))); }
    let projection: DiagramProjection = serde_json::from_value(export.data.clone()).map_err(|e| ExportError::Serialization(e.to_string()))?;
    let document = projection_to_document(&projection);
    validate_schema(&document).map_err(|e| ExportError::Validation(e.to_string()))?;
    let events_to_import = export.events.unwrap_or_else(|| generate_canonical_events(&projection));
    let event_records: Vec<EventRecord> = events_to_import.iter().map(|v| serde_json::from_value(v.clone()).map_err(|e| ExportError::Serialization(e.to_string()))).collect::<Result<Vec<_>, _>>()?;
    let envelope_author = actor.to_envelope_author();
    let mut events_imported: u64 = 0;
    for event_record in &event_records {
        let envelope = EventEnvelope { op_id: event_record.op_id.clone(), operation: event_record.operation.clone(), author: envelope_author.clone(), timestamp: event_record.timestamp };
        let revision = event_record.revision as i64 + 1;
        match crate::store_async::append_event_async(pool, envelope, Some(revision as i64)).await {
            Ok(_) => events_imported += 1,
            Err(e) => { if let Some(conflict) = e.as_sqlite_error() { if conflict.code() == sqlx::error::SqliteErrorCode::ConstraintViolation { continue; } } return Err(ExportError::Sqlite(e.to_string())); }
        }
    }
    Ok(ImportResult { events_generated: events_imported, final_revision: event_records.len() as u64 })
}

fn generate_canonical_events(projection: &DiagramProjection) -> Vec<serde_json::Value> {
    let mut events = Vec::new();
    let mut revision: u64 = 0;
    for (id, node) in projection.nodes.iter() {
        let envelope = EventEnvelope { op_id: format!("gen-node-add-{}", id), operation: crate::models::envelope::DomainOp::NodeAdd { id: id.to_string(), x: node.x.0, y: node.y.0, width: node.width.0, height: node.height.0, label: node.label.clone() }, author: EnvelopeAuthor { id: "import".to_string(), name: "Import".to_string(), email: None }, timestamp: 1700000000 + revision as i64 };
        if let Ok(json) = serde_json::to_value(&envelope) { events.push(json); }
        revision += 1;
    }
    for (id, edge) in projection.edges.iter() {
        let envelope = EventEnvelope { op_id: format!("gen-edge-{}", id), operation: crate::models::envelope::DomainOp::EdgeConnect { id: id.to_string(), source: edge.source.to_string(), target: edge.target.to_string() }, author: EnvelopeAuthor { id: "import".to_string(), name: "Import".to_string(), email: None }, timestamp: 1700000000 + revision as i64 };
        if let Ok(json) = serde_json::to_value(&envelope) { events.push(json); }
        revision += 1;
    }
    events
}
