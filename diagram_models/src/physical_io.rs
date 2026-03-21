mod builder;
mod migration;

use crate::document::DiagramDocument;
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

pub use builder::DiagramBuilder;
pub use migration::migrate_schema;

#[derive(thiserror::Error, Debug)]
#[allow(clippy::enum_variant_names)]
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

/// Saves a document to a file.
///
/// # Errors
///
/// Returns `Error` if saving fails.
pub fn save_document(path: &Path, doc: &DiagramDocument) -> Result<(), Error> {
    validate_document_floats(doc)?;

    let json_value =
        serde_json::to_value(doc).map_err(|e| Error::SerializationFailed(e.to_string()))?;
    validate_serialization(&json_value)?;

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &json_value)
        .map_err(|e| Error::SerializationFailed(e.to_string()))?;
    writer.flush().map_err(Error::IoError)?;

    Ok(())
}

fn check_finite(val: f64, msg: &str) -> Result<(), Error> {
    if val.is_finite() {
        Ok(())
    } else {
        Err(Error::SerializationFailed(msg.into()))
    }
}

fn validate_document_floats(doc: &DiagramDocument) -> Result<(), Error> {
    let es = &doc.editor_state;
    check_finite(es.camera_x.0, "Non-finite camera_x")?;
    check_finite(es.camera_y.0, "Non-finite camera_y")?;
    check_finite(es.zoom.0, "Non-finite zoom")?;

    for (id, node) in &doc.document.nodes {
        check_finite(node.x.0, &format!("Non-finite x in node {id}"))?;
        check_finite(node.y.0, &format!("Non-finite y in node {id}"))?;
        check_finite(node.width.0, &format!("Non-finite width in node {id}"))?;
        check_finite(node.height.0, &format!("Non-finite height in node {id}"))?;
    }

    for (id, edge) in &doc.document.edges {
        check_finite(
            edge.label_offset_t.0,
            &format!("Non-finite label_offset_t in edge {id}"),
        )?;
        check_finite(
            edge.thickness.0,
            &format!("Non-finite thickness in edge {id}"),
        )?;
        if let Some(ref fs) = edge.font_size {
            check_finite(fs.0, &format!("Non-finite font_size in edge {id}"))?;
        }
    }

    Ok(())
}

fn validate_serialization(value: &Value) -> Result<(), Error> {
    check_depth(value, 0)
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
            if num.as_f64().is_some_and(|f| !f.is_finite()) {
                return Err(Error::SerializationFailed("Non-finite float".into()));
            }
        }
        _ => {}
    }
    Ok(())
}

/// Loads a document from a file.
///
/// # Errors
///
/// Returns `Error` if loading fails.
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

fn get_field<'a>(obj: &'a serde_json::Map<String, Value>, name: &str) -> Result<&'a Value, Error> {
    let val = obj
        .get(name)
        .ok_or_else(|| Error::MissingField(name.into()))?;
    if val.is_null() {
        Err(Error::InvalidNull(name.into()))
    } else {
        Ok(val)
    }
}

fn get_object<'a>(
    obj: &'a serde_json::Map<String, Value>,
    name: &str,
) -> Result<&'a serde_json::Map<String, Value>, Error> {
    let val = get_field(obj, name)?;
    val.as_object().ok_or_else(|| Error::TypeMismatch {
        field: name.into(),
        expected: "object".into(),
        found: type_of(val),
    })
}

fn validate_structure(json: &Value) -> Result<(), Error> {
    let obj = json.as_object().ok_or_else(|| Error::TypeMismatch {
        field: "root".into(),
        expected: "object".into(),
        found: type_of(json),
    })?;

    let version = get_field(obj, "version")?;
    if !version.is_number() {
        return Err(Error::TypeMismatch {
            field: "version".into(),
            expected: "number".into(),
            found: type_of(version),
        });
    }

    let doc = get_object(obj, "document")?;
    for field in ["nodes", "edges"] {
        let items = get_object(doc, field)?;
        let item_name = &field[..field.len() - 1]; // "node" or "edge"

        for val in items.values() {
            if val.is_null() {
                return Err(Error::InvalidNull(item_name.into()));
            }
            if let Some(item_obj) = val.as_object() {
                if let Some(meta) = item_obj.get("metadata") {
                    if meta.is_null() {
                        return Err(Error::InvalidNull("metadata".into()));
                    }
                    if !meta.is_object() {
                        return Err(Error::TypeMismatch {
                            field: "metadata".into(),
                            expected: "object".into(),
                            found: type_of(meta),
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
