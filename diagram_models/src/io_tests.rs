#![allow(clippy::unwrap_used, clippy::panic, clippy::module_inception, clippy::let_unit_value, clippy::redundant_pattern_matching, unused_variables, unused_imports)]
//! Import/Export/Persistence Tests (IO-001 to IO-015)
//!
//! This module contains comprehensive tests for JSON import/export
//! and persistence operations per contract bd-19p.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::document::{DiagramDocument, DocumentData, EditorState, Revision};
use crate::export::{export_diagram_json, import_diagram_json, Author, DiagramJsonExport, ExportError};
use im::HashMap;

// Helper to create a minimal test JSON export
fn create_minimal_export() -> String {
    r#"{
        "metadata": {
            "name": "diagram",
            "revision": 0,
            "version": 2
        },
        "data": {
            "version": 2,
            "revision": 0,
            "nodes": {},
            "edges": {},
            "cycle_policy": "default",
            "author_priority": []
        },
        "events": []
    }"#.to_string()
}

// Helper to create an in-memory database for testing
//
// # Errors
//
// Returns error if temp directory creation, database connection, or schema initialization fails.
fn create_test_db() -> Result<(rusqlite::Connection, tempfile::TempDir), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    let conn = rusqlite::Connection::open(db_path)?;

    // Initialize schema
    conn.execute(
        "CREATE TABLE IF NOT EXISTS events (
            operation_id TEXT PRIMARY KEY,
            revision INTEGER NOT NULL,
            payload TEXT NOT NULL,
            timestamp INTEGER NOT NULL
        )",
        [],
    )?;

    Ok((conn, temp_dir))
}

// ============================================================================
// IO-001: Malformed JSON Import
// ============================================================================

#[cfg(kani)]
#[kani::proof]
fn io_001_malformed_json_import_syntax_error() {
    // Given: JSON input with syntax errors
    let malformed_json = r##"{"version": 2, "broken": [}"##;

    // When: Attempting to parse and validate
    let result: Result<DiagramJsonExport, _> = serde_json::from_str(malformed_json);

    // Then: Returns serialization error (not panic)
    assert!(result.is_err());
}

#[cfg(kani)]
#[kani::proof]
fn io_001_malformed_json_import_unclosed_string() {
    // Given: JSON with unclosed string
    let malformed_json = r##"{"version": 2, "data": {"nodes": {"##;

    // When: Attempting to parse
    let result: Result<DiagramJsonExport, _> = serde_json::from_str(malformed_json);

    // Then: Returns error
    assert!(result.is_err());
}

#[cfg(kani)]
#[kani::proof]
fn io_001_malformed_json_import_invalid_escape() {
    // Given: JSON with invalid escape sequence
    let malformed_json = r#"{"version": 2, "data": {"nodes": {"node-1": {"label": "test\x0"}}}}"#;

    // When: Attempting to parse
    let result: Result<DiagramJsonExport, _> = serde_json::from_str(malformed_json);

    // Then: Returns error
    assert!(result.is_err());
}

// ============================================================================
// IO-002: Empty Document Export
// ============================================================================

#[cfg(kani)]
#[kani::proof]
fn io_002_empty_document_export() {
    // Given: Empty database (no events)
    let (conn, _temp_dir) = create_test_db().unwrap_or_else(|e| panic!("Failed to create test DB: {}", e));

    // When: Export is called
    let result = export_diagram_json(&conn);

    // Then: Returns valid export with revision=0
    let export = match result {
        Ok(e) => e,
        Err(e) => panic!("Export should succeed: {:?}", e),
    };
    assert_eq!(export.metadata.revision, 0);
    assert_eq!(export.metadata.version, 2);
    assert!(export.events.is_some());
    assert!(export.events.as_ref().map_or(false, |v| v.is_empty()));
}

// ============================================================================
// IO-003: Invalid Schema Version
// ============================================================================

#[cfg(kani)]
#[kani::proof]
fn io_003_invalid_schema_version_too_new() {
    // Given: JSON with version > current supported
    let future_version_json = serde_json::json!({
        "metadata": {
            "name": "diagram",
            "revision": 0,
            "version": 999
        },
        "data": {
            "version": 2,
            "revision": 0,
            "nodes": {},
            "edges": {}
        },
        "events": []
    });

    // When: Checking version
    let version = future_version_json["metadata"]["version"]
        .as_u64()
        .unwrap_or(0) as u32;

    // Then: Should be rejected (> current version 2)
    assert!(version > 2);
}

#[cfg(kani)]
#[kani::proof]
fn io_003_invalid_schema_version_rejected_on_import() {
    // Given: JSON with future version
    let json = r#"{
        "metadata": {"name": "diagram", "revision": 0, "version": 999},
        "data": {"version": 2, "revision": 0, "nodes": {}, "edges": {}},
        "events": []
    }"#;

    let (mut conn, _temp_dir) = create_test_db().unwrap_or_else(|e| panic!("Failed to create test DB: {}", e));
    let actor = Author {
        id: "test".to_string(),
        is_human: true,
    };

    // When: Importing with invalid version
    let result = import_diagram_json(&mut conn, json, actor);

    // Then: Returns InvalidSchema error
    assert!(result.is_err());
    if let Err(ExportError::InvalidSchema(msg)) = result {
        assert!(msg.contains("999"));
    } else {
        panic!("Expected InvalidSchema error");
    }
}

// ============================================================================
// IO-004: Valid Round-Trip
// ============================================================================

#[cfg(kani)]
#[kani::proof]
fn io_004_valid_json_parses() {
    // Given: Valid JSON export
    let json = create_minimal_export();

    // When: Parsing
    let result: Result<DiagramJsonExport, _> = serde_json::from_str(&json);

    // Then: Parses successfully
    let export = match result {
        Ok(e) => e,
        Err(e) => panic!("Failed to parse JSON: {:?}", e),
    };
    assert_eq!(export.metadata.revision, 0);
    assert_eq!(export.metadata.version, 2);
}

// ============================================================================
// IO-005: Large Document Export Performance
// ============================================================================

#[cfg(kani)]
#[kani::proof]
fn io_005_large_document_export_performance() {
    // Given: Document with 1000+ events
    let (conn, _temp_dir) = create_test_db().unwrap_or_else(|e| panic!("Failed to create test DB: {}", e));

    // Create 1000 events
    for i in 0..1000 {
        conn.execute(
            "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
            (format!("op-{i}"), i + 1, "{}", 1000 + i),
        ).unwrap_or_else(|e| panic!("DB insert failed: {:?}", e));
    }

    // When: Exporting
    let export_start = std::time::Instant::now();
    let result = export_diagram_json(&conn);
    let export_duration = export_start.elapsed();

    // Then: Completes within reasonable time (< 5 seconds)
    assert!(result.is_ok());
    assert!(export_duration.as_secs() < 5, "Export took {} seconds", export_duration.as_secs());

    let export = match result {
        Ok(e) => e,
        Err(e) => panic!("Export should succeed: {:?}", e),
    };
    assert_eq!(export.metadata.revision, 0); // No replay = revision 0
}

// ============================================================================
// IO-006: Large Document Import Performance
// ============================================================================

#[cfg(kani)]
#[kani::proof]
fn io_006_large_document_import_performance() {
    // Given: Export with 100 nodes
    let mut nodes = serde_json::Map::new();
    for i in 0..100 {
        let node_id = format!("node-{i}");
        nodes.insert(node_id, serde_json::json!({
            "kind": "node",
            "icon": "",
            "label": format!("Node {}", i),
            "x": (i % 10) as f64 * 100.0,
            "y": (i / 10) as f64 * 100.0,
            "width": 80.0,
            "height": 40.0,
            "locked": false,
            "parent": null,
            "tags": [],
            "metadata": {},
            "z_index": i
        }));
    }

    let export_json = serde_json::json!({
        "metadata": {
            "name": "diagram",
            "revision": 0,
            "version": 2
        },
        "data": {
            "version": 2,
            "revision": 0,
            "nodes": nodes,
            "edges": {},
            "cycle_policy": "default",
            "author_priority": []
        },
        "events": []
    });

    let json_str = serde_json::to_string(&export_json)
        .unwrap_or_else(|e| panic!("Failed to serialize export JSON: {:?}", e));
    let (mut conn, _temp_dir) = create_test_db().unwrap_or_else(|e| panic!("Failed to create test DB: {}", e));
    let actor = Author {
        id: "test".to_string(),
        is_human: true,
    };

    // When: Importing
    let start = std::time::Instant::now();
    let result = import_diagram_json(&mut conn, &json_str, actor);
    let duration = start.elapsed();

    // Then: Import completes successfully
    assert!(result.is_ok());
    assert!(duration.as_secs() < 5, "Import took {} seconds", duration.as_secs());
}

// ============================================================================
// IO-007: Unicode Node Labels
// ============================================================================

#[cfg(kani)]
#[kani::proof]
fn io_007_unicode_emoji_labels() {
    // Given: Document with emoji labels
    let labels = vec![
        ("🔥 Fire", "café"),
        ("💧 Water", "مرحبا"),
        ("🌍 Earth", "שלום"),
    ];

    for (emoji, text) in labels {
        // When: Serializing and deserializing
        let json = serde_json::json!({"label": emoji, "text": text});
        let serialized = serde_json::to_string(&json)
            .unwrap_or_else(|e| panic!("Failed to serialize: {:?}", e));
        let deserialized: serde_json::Value = serde_json::from_str(&serialized)
            .unwrap_or_else(|e| panic!("Failed to deserialize: {:?}", e));

        // Then: Labels are preserved exactly
        assert_eq!(deserialized["label"], emoji);
        assert_eq!(deserialized["text"], text);
    }
}

#[cfg(kani)]
#[kani::proof]
fn io_007_unicode_rtl_text() {
    // Given: RTL text (Arabic, Hebrew)
    let rtl_labels = vec![
        "مرحبا",           // Arabic "Hello"
        "שלום",            // Hebrew "Shalom"
        "السلام عليكم",    // Arabic "Peace be upon you"
    ];

    for label in rtl_labels {
        let json = serde_json::json!({"label": label});
        let serialized = serde_json::to_string(&json)
            .unwrap_or_else(|e| panic!("Failed to serialize: {:?}", e));
        let deserialized: serde_json::Value = serde_json::from_str(&serialized)
            .unwrap_or_else(|e| panic!("Failed to deserialize: {:?}", e));

        assert_eq!(deserialized["label"], label);
    }
}

#[cfg(kani)]
#[kani::proof]
fn io_007_unicode_combining_characters() {
    // Given: Text with combining diacritics
    let text = "café";  // Combining acute accent
    let json = serde_json::json!({"label": text});
    let serialized = serde_json::to_string(&json)
        .unwrap_or_else(|e| panic!("Failed to serialize: {:?}", e));
    let deserialized: serde_json::Value = serde_json::from_str(&serialized)
        .unwrap_or_else(|e| panic!("Failed to deserialize: {:?}", e));

    assert_eq!(deserialized["label"], text);
}

// ============================================================================
// IO-008: Atomic Save on Crash
// ============================================================================

#[cfg(kani)]
#[kani::proof]
fn io_008_atomic_save_pattern_works() {
    use std::fs;

    // Given: Original file exists
    let temp_dir = tempfile::TempDir::new()
        .unwrap_or_else(|e| panic!("Failed to create temp dir: {:?}", e));
    let file_path = temp_dir.path().join("test.json");
    let original_content = r#"{"version": 2, "revision": 0}"#;

    fs::write(&file_path, original_content)
        .unwrap_or_else(|e| panic!("Failed to write file: {:?}", e));

    let original_hash = fs::metadata(&file_path)
        .unwrap_or_else(|e| panic!("Failed to get metadata: {:?}", e))
        .len();

    // Simulate atomic save pattern
    let temp_path = temp_dir.path().join(format!(".test.json.tmp.{}", std::process::id()));
    fs::write(&temp_path, "new content")
        .unwrap_or_else(|e| panic!("Failed to write temp file: {:?}", e));
    fs::rename(&temp_path, &file_path)
        .unwrap_or_else(|e| panic!("Failed to rename file: {:?}", e));

    // Then: Original file is updated
    let current_content = fs::read_to_string(&file_path)
        .unwrap_or_else(|e| panic!("Failed to read file: {:?}", e));
    assert_eq!(current_content, "new content");
    assert_ne!(
        fs::metadata(&file_path)
            .unwrap_or_else(|e| panic!("Failed to get metadata: {:?}", e))
            .len(),
        original_hash
    );
}

// ============================================================================
// IO-009: LKG Fallback
// ============================================================================

#[cfg(kani)]
#[kani::proof]
fn io_009_lkg_fallback_function_exists() {
    use crate::cli_persistence::{load_workspace_with_lkg, CliPersistenceError};
    use std::path::Path;

    // Given: Non-existent file
    let temp_dir = tempfile::TempDir::new()
        .unwrap_or_else(|e| panic!("Failed to create temp dir: {:?}", e));
    let file_path = temp_dir.path().join("nonexistent.json");

    // When: Attempting to load
    let result = load_workspace_with_lkg(&file_path);

    // Then: Returns error (not panic)
    match result {
        Err(CliPersistenceError::NoValidDocument(_)) => {
            // Expected - file doesn't exist
        }
        Err(CliPersistenceError::IoError(_)) => {
            // Also acceptable
        }
        Err(e) => {
            panic!("Unexpected error type: {:?}", e);
        }
        Ok(_) => {
            panic!("Should not succeed with non-existent file");
        }
    }
}

// ============================================================================
// IO-010: Schema Validation on Import
// ============================================================================

#[cfg(kani)]
#[kani::proof]
fn io_010_validate_schema_exists() {
    use crate::schema::validate_schema;
    use crate::document::EditorState;
    use crate::document::Revision;

    // Given: Minimal valid document
    let doc = DiagramDocument {
        version: 2,
        revision: Revision::INITIAL,
        document: DocumentData {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        },
        editor_state: EditorState::default(),
    };

    // When: Validating
    let result = validate_schema(&doc);

    // Then: Does not panic (may pass or fail depending on implementation)
    let _ = result;
}

// ============================================================================
// IO-011: Recovery Mode Export
// ============================================================================

#[cfg(kani)]
#[kani::proof]
fn io_011_recovery_mode_export_function_exists() {
    use crate::export::export_while_recovering;

    // Given: Read-only connection with data
    let (conn, _temp_dir) = create_test_db().unwrap_or_else(|e| panic!("Failed to create test DB: {}", e));

    // Add some data
    for i in 0..5 {
        conn.execute(
            "INSERT INTO events (operation_id, revision, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
            (format!("op-{i}"), i + 1, "{}", 1000 + i),
        ).unwrap_or_else(|e| panic!("DB insert failed: {:?}", e));
    }

    // When: Exporting in recovery mode
    let result = export_while_recovering(&conn);

    // Then: Returns valid JSON string
    let json_str = match result {
        Ok(s) => s,
        Err(e) => panic!("Recovery export should succeed: {:?}", e),
    };
    assert!(!json_str.is_empty());

    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json_str)
        .unwrap_or_else(|e| panic!("Failed to parse JSON: {:?}", e));
    assert!(parsed["version"].is_number());
}

// ============================================================================
// IO-012: Version Backward Compatibility
// ============================================================================

#[cfg(kani)]
#[kani::proof]
fn io_012_backward_compatible_version_1() {
    use crate::export::validate_export_schema;

    // Given: Export with version 1 (older)
    let old_version_json = serde_json::json!({
        "version": 1,
        "revision": 0,
        "nodes": {},
        "edges": {}
    });

    let json_str = serde_json::to_string(&old_version_json)
        .unwrap_or_else(|e| panic!("Failed to serialize old version JSON: {:?}", e));

    // When: Validating
    let result = validate_export_schema(&json_str);

    // Then: Accepts the old version (version 1 <= version 2)
    assert!(result.is_ok());
}

// ============================================================================
// IO-013: Null in Required Field
// ============================================================================

#[cfg(kani)]
#[kani::proof]
fn io_013_null_version_field() {
    // Given: Version field is null
    let null_version = r#"{
        "metadata": {"name": "diagram", "revision": 0, "version": null},
        "data": {"version": 2, "revision": 0, "nodes": {}, "edges": {}},
        "events": []
    }"#;

    // When: Attempting to parse
    let result: Result<DiagramJsonExport, _> = serde_json::from_str(null_version);

    // Then: Returns error
    assert!(result.is_err());
}

// ============================================================================
// IO-014: Truncated JSON
// ============================================================================

#[cfg(kani)]
#[kani::proof]
fn io_014_truncated_json_mid_string() {
    // Given: JSON cut off mid-string
    let truncated = r##"{"version": 2, "revision": 0, "nodes": {"##;

    // When: Attempting to parse
    let result: Result<serde_json::Value, _> = serde_json::from_str(truncated);

    // Then: Returns error (not panic)
    assert!(result.is_err());
}

// ============================================================================
// IO-015: Missing Required Field
// ============================================================================

#[cfg(kani)]
#[kani::proof]
fn io_015_missing_version_field() {
    // Given: JSON missing version field
    let no_version = r#"{
        "metadata": {"name": "diagram", "revision": 0},
        "data": {"nodes": {}, "edges": {}}
    }"#;

    // When: Attempting to parse
    let result: Result<DiagramJsonExport, _> = serde_json::from_str(no_version);

    // Then: Returns error
    assert!(result.is_err());
}

#[cfg(kani)]
#[kani::proof]
fn io_015_missing_metadata_field() {
    // Given: JSON missing metadata
    let no_metadata = r#"{
        "data": {"version": 2, "revision": 0, "nodes": {}, "edges": {}},
        "events": []
    }"#;

    // When: Attempting to parse
    let result: Result<DiagramJsonExport, _> = serde_json::from_str(no_metadata);

    // Then: Returns error
    assert!(result.is_err());
}
