use crate::models::document::DiagramDocument;
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Parse Error: {0}")]
    ParseError(String),

    #[error("Missing Field: {0}")]
    MissingField(String),

    #[error("Type Mismatch: field {field} expected {expected}, found {found}")]
    TypeMismatch {
        field: String,
        expected: String,
        found: String,
    },

    #[error("Invalid Null: {0}")]
    InvalidNull(String),

    #[error("Unsupported Version: {0}")]
    UnsupportedVersion(String),

    #[error("Serialization Failed: {0}")]
    SerializationFailed(String),

    #[error("Recursion Limit Exceeded")]
    RecursionLimitExceeded,
}

pub fn save_document(path: &Path, doc: &DiagramDocument) -> Result<(), Error> {
    // Validate document floats BEFORE serialization
    // (serde_json converts NaN to null, making it undetectable after serialization)
    validate_document_floats(doc)?;

    let json_value =
        serde_json::to_value(doc).map_err(|e| Error::SerializationFailed(e.to_string()))?;
    validate_serialization(&json_value)?;

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &json_value)
        .map_err(|e| Error::SerializationFailed(e.to_string()))?;
    // Explicitly flush to catch IO errors (e.g., disk full)
    writer.flush().map_err(|e| Error::IoError(e))?;

    Ok(())
}

/// Validates that all float fields in the document are finite.
/// Must be called BEFORE serde_json::to_value because that function converts NaN to null.
fn validate_document_floats(doc: &DiagramDocument) -> Result<(), Error> {
    // Check editor state
    if !doc.editor_state.camera_x.0.is_finite() {
        return Err(Error::SerializationFailed("Non-finite camera_x".into()));
    }
    if !doc.editor_state.camera_y.0.is_finite() {
        return Err(Error::SerializationFailed("Non-finite camera_y".into()));
    }
    if !doc.editor_state.zoom.0.is_finite() {
        return Err(Error::SerializationFailed("Non-finite zoom".into()));
    }

    // Check all nodes (ID is the map key, not a field on Node)
    for (node_id, node) in &doc.document.nodes {
        if !node.x.0.is_finite() {
            return Err(Error::SerializationFailed(format!(
                "Non-finite x in node {}",
                node_id
            )));
        }
        if !node.y.0.is_finite() {
            return Err(Error::SerializationFailed(format!(
                "Non-finite y in node {}",
                node_id
            )));
        }
        if !node.width.0.is_finite() {
            return Err(Error::SerializationFailed(format!(
                "Non-finite width in node {}",
                node_id
            )));
        }
        if !node.height.0.is_finite() {
            return Err(Error::SerializationFailed(format!(
                "Non-finite height in node {}",
                node_id
            )));
        }
    }

    // Check all edges (ID is the map key, not a field on Edge)
    for (edge_id, edge) in &doc.document.edges {
        if !edge.label_offset_t.0.is_finite() {
            return Err(Error::SerializationFailed(format!(
                "Non-finite label_offset_t in edge {}",
                edge_id
            )));
        }
        if !edge.thickness.0.is_finite() {
            return Err(Error::SerializationFailed(format!(
                "Non-finite thickness in edge {}",
                edge_id
            )));
        }
        if let Some(ref fs) = &edge.font_size {
            if !fs.0.is_finite() {
                return Err(Error::SerializationFailed(format!(
                    "Non-finite font_size in edge {}",
                    edge_id
                )));
            }
        }
    }

    Ok(())
}

fn validate_serialization(value: &Value) -> Result<(), Error> {
    check_depth(value, 0)?;
    Ok(())
}

fn check_depth(value: &Value, depth: usize) -> Result<(), Error> {
    if depth > 100 {
        return Err(Error::RecursionLimitExceeded);
    }
    match value {
        Value::Array(arr) => {
            for item in arr {
                check_depth(item, depth + 1)?;
            }
        }
        Value::Object(map) => {
            for val in map.values() {
                check_depth(val, depth + 1)?;
            }
        }
        Value::Number(num) => {
            if num.as_f64().map(|f| !f.is_finite()).unwrap_or(false) {
                return Err(Error::SerializationFailed("Non-finite float".into()));
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn load_document(path: &Path) -> Result<DiagramDocument, Error> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;

    let raw_json: Value =
        serde_json::from_str(&contents).map_err(|e| Error::ParseError(e.to_string()))?;

    check_depth(&raw_json, 0)?;
    validate_structure(&raw_json)?;

    let migrated_json = migrate_schema(raw_json)?;

    serde_json::from_value(migrated_json).map_err(|e| Error::ParseError(e.to_string()))
}

fn validate_structure(json: &Value) -> Result<(), Error> {
    let obj = json.as_object().ok_or_else(|| Error::TypeMismatch {
        field: "root".into(),
        expected: "object".into(),
        found: type_of(json),
    })?;

    // Check version
    let version_val = obj
        .get("version")
        .ok_or_else(|| Error::MissingField("version".into()))?;
    if version_val.is_null() {
        return Err(Error::InvalidNull("version".into()));
    }
    if !version_val.is_number() {
        return Err(Error::TypeMismatch {
            field: "version".into(),
            expected: "number".into(),
            found: type_of(version_val),
        });
    }

    // Check document sub-object
    let document_val = obj
        .get("document")
        .ok_or_else(|| Error::MissingField("document".into()))?;
    if document_val.is_null() {
        return Err(Error::InvalidNull("document".into()));
    }
    let document_obj = document_val
        .as_object()
        .ok_or_else(|| Error::TypeMismatch {
            field: "document".into(),
            expected: "object".into(),
            found: type_of(document_val),
        })?;

    // Check nodes and edges
    for field in ["nodes", "edges"] {
        let val = document_obj
            .get(field)
            .ok_or_else(|| Error::MissingField(field.into()))?;
        if val.is_null() {
            return Err(Error::InvalidNull(field.into()));
        }
        if !val.is_object() {
            return Err(Error::TypeMismatch {
                field: field.into(),
                expected: "object".into(),
                found: type_of(val),
            });
        }
    }

    // Specific field checks on nodes
    if let Some(nodes) = document_obj.get("nodes").and_then(|v| v.as_object()) {
        for node_val in nodes.values() {
            if node_val.is_null() {
                return Err(Error::InvalidNull("node".into()));
            }
            if let Some(node_obj) = node_val.as_object() {
                if let Some(metadata) = node_obj.get("metadata") {
                    if metadata.is_null() {
                        return Err(Error::InvalidNull("metadata".into()));
                    }
                    if !metadata.is_object() {
                        return Err(Error::TypeMismatch {
                            field: "metadata".into(),
                            expected: "object".into(),
                            found: type_of(metadata),
                        });
                    }
                }
            }
        }
    }

    // Specific field checks on edges
    if let Some(edges) = document_obj.get("edges").and_then(|v| v.as_object()) {
        for edge_val in edges.values() {
            if edge_val.is_null() {
                return Err(Error::InvalidNull("edge".into()));
            }
            if let Some(edge_obj) = edge_val.as_object() {
                if let Some(metadata) = edge_obj.get("metadata") {
                    if metadata.is_null() {
                        return Err(Error::InvalidNull("metadata".into()));
                    }
                    if !metadata.is_object() {
                        return Err(Error::TypeMismatch {
                            field: "metadata".into(),
                            expected: "object".into(),
                            found: type_of(metadata),
                        });
                    }
                }
            }
        }
    }

    Ok(())
}

fn type_of(val: &Value) -> String {
    match val {
        Value::Null => "null".into(),
        Value::Bool(_) => "boolean".into(),
        Value::Number(_) => "number".into(),
        Value::String(_) => "string".into(),
        Value::Array(_) => "array".into(),
        Value::Object(_) => "object".into(),
    }
}

pub fn migrate_schema(raw_json: serde_json::Value) -> Result<serde_json::Value, Error> {
    let mut obj = match raw_json {
        Value::Object(map) => map,
        _ => return Err(Error::ParseError("Root is not an object".into())),
    };

    let version = obj
        .get("version")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| Error::MissingField("version".into()))?;

    if version < 0.9 {
        return Err(Error::UnsupportedVersion(version.to_string()));
    } else if version == 0.9 {
        // Migrate 0.9 to 1.0 (then to 2.0)
        obj.insert("version".into(), Value::Number(serde_json::Number::from(2)));
        if !obj.contains_key("revision") {
            obj.insert(
                "revision".into(),
                Value::Number(serde_json::Number::from(0)),
            );
        }
    } else if version == 1.0 {
        obj.insert("version".into(), Value::Number(serde_json::Number::from(2)));
        if !obj.contains_key("revision") {
            obj.insert(
                "revision".into(),
                Value::Number(serde_json::Number::from(0)),
            );
        }
    } else if version > 2.0 {
        return Err(Error::UnsupportedVersion(version.to_string()));
    }

    Ok(Value::Object(obj))
}

pub struct DiagramBuilder {
    doc: DiagramDocument,
}

impl DiagramBuilder {
    pub fn new() -> Self {
        Self {
            doc: DiagramDocument::default(),
        }
    }

    pub fn build(self) -> DiagramDocument {
        self.doc
    }
}

impl Default for DiagramBuilder {
    fn default() -> Self {
        Self::new()
    }
}
