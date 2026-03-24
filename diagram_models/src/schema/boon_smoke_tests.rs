//! Smoke tests for the `boon` JSON Schema validation dependency.
//!
//! These tests verify that `boon` can compile a JSON Schema (draft-2020-12)
//! and validate instances against it, producing actionable error messages.
//!
//! Contract: seshat-ni4
//! Tests: HP-01, EP-01a, EP-01b

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

use boon::{Compiler, Schemas};
use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// Pure data helpers
// ---------------------------------------------------------------------------

/// Minimal draft-2020-12 schema requiring a `"name"` property.
fn test_schema() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "required": ["name"],
        "properties": {
            "name": { "type": "string" }
        }
    })
}

/// Compile `test_schema()` via boon and return the compiled artefacts.
///
/// # Errors
///
/// Returns a `String` describing why compilation failed (add_resource or
/// compile step).
fn compile_test_schema() -> Result<(Schemas, boon::SchemaIndex), String> {
    let schema = test_schema();
    let mut schemas = Schemas::new();
    let mut compiler = Compiler::new();

    compiler
        .add_resource("test-schema.json", schema)
        .map_err(|e| format!("add_resource failed: {e}"))?;

    let sch_index = compiler
        .compile("test-schema.json", &mut schemas)
        .map_err(|e| format!("compile failed: {e}"))?;

    assert!(
        schemas.contains(sch_index),
        "compiled schema index must belong to this schemas instance"
    );

    Ok((schemas, sch_index))
}

// ---------------------------------------------------------------------------
// HP-01: boon_validates_valid_instance
// ---------------------------------------------------------------------------

#[test]
fn boon_validates_valid_instance() -> Result<(), String> {
    let (schemas, sch_index) = compile_test_schema()?;
    let instance = json!({ "name": "seshat" });

    let result = schemas.validate(&instance, sch_index);
    assert!(
        result.is_ok(),
        "valid instance should pass validation: {result:?}"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// EP-01a: boon_rejects_missing_required_property
// ---------------------------------------------------------------------------

#[test]
fn boon_rejects_missing_required_property() -> Result<(), String> {
    let (schemas, sch_index) = compile_test_schema()?;
    let instance = json!({});

    let result = schemas.validate(&instance, sch_index);
    assert!(
        result.is_err(),
        "missing required property 'name' should produce at least one error"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// EP-01b: boon_error_identifies_missing_property_name
// ---------------------------------------------------------------------------

#[test]
fn boon_error_identifies_missing_property_name() -> Result<(), String> {
    let (schemas, sch_index) = compile_test_schema()?;
    let instance = json!({});

    let result = schemas.validate(&instance, sch_index);

    match result {
        Err(validation_error) => {
            let msg = validation_error.to_string();
            assert!(
                msg.contains("name"),
                "error message should reference missing property 'name', got: {msg}"
            );
        }
        Ok(()) => return Err("expected validation error but got success".to_string()),
    }

    Ok(())
}
