#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
use super::*;
use diagram_models::document::DiagramDocument;

#[path = "fixtures.rs"]
mod fixtures;

#[cfg(kani)]
#[kani::proof]
fn given_document_when_serialized_then_round_trips() {
    let doc = DiagramDocument::default();
    let json = serde_json::to_string_pretty(&doc).unwrap();
    let loaded: DiagramDocument = serde_json::from_str(&json).unwrap();
    assert_eq!(doc.revision, loaded.revision);
}

#[cfg(kani)]
#[kani::proof]
fn given_ts_style_json_when_parsed_then_document_loads() {
    let loaded = super::parse_diagram_document_with_compat(fixtures::TS_STYLE_JSON);
    assert!(loaded.is_ok(), "{:?}", loaded.err());
}

#[cfg(kani)]
#[kani::proof]
fn given_legacy_font_size_keys_when_parsed_then_document_loads() {
    let loaded = super::parse_diagram_document_with_compat(fixtures::LEGACY_FONT_SIZE_JSON);
    assert!(loaded.is_ok(), "{:?}", loaded.err());
}

#[cfg(kani)]
#[kani::proof]
fn given_equivalent_legacy_aliases_when_parsed_then_canonical_json_is_identical() {
    use diagram_models::canonical_json::to_canonical_pretty_json;

    let parsed_a = super::parse_diagram_document_with_compat(fixtures::LEGACY_A).unwrap();
    let parsed_b = super::parse_diagram_document_with_compat(fixtures::LEGACY_B).unwrap();

    let canonical_a = to_canonical_pretty_json(&parsed_a).unwrap();
    let canonical_b = to_canonical_pretty_json(&parsed_b).unwrap();

    assert_eq!(canonical_a, canonical_b);
}

/// IO-TEST-5: Import Older Version Migration (bd-1u1)
#[cfg(kani)]
#[kani::proof]
fn given_version_1_document_when_import_then_migrates_to_current_version() {
    let result = super::parse_diagram_document_with_compat(fixtures::VERSION_1_DOCUMENT);
    assert!(
        result.is_ok(),
        "Version 1 document should parse: {:?}",
        result.err()
    );
    let doc = result.expect("should have document");
    assert!(
        doc.document
            .nodes
            .contains_key(&diagram_models::document::NodeId::new(
                "legacy_node".to_string()
            )),
        "Legacy node should be present"
    );
}

/// IO-TEST-5b: Version migration with legacy field names
#[cfg(kani)]
#[kani::proof]
fn given_older_document_with_legacy_fields_when_import_then_fields_remapped() {
    let result = super::parse_diagram_document_with_compat(fixtures::LEGACY_FIELDS_DOCUMENT);
    assert!(
        result.is_ok(),
        "Legacy fields should be remapped: {:?}",
        result.err()
    );
    let doc = result.expect("should have document");

    assert!(
        doc.document
            .nodes
            .contains_key(&diagram_models::document::NodeId::new(
                "legacy_fields".to_string()
            )),
        "Node should be parsed"
    );
    assert!(
        doc.document
            .edges
            .contains_key(&diagram_models::document::EdgeId::new(
                "legacy_edge".to_string()
            )),
        "Edge should be parsed"
    );

    let node = doc
        .document
        .nodes
        .get(&diagram_models::document::NodeId::new(
            "legacy_fields".to_string(),
        ))
        .expect("node should exist");
    assert_eq!(
        node.dag_rank,
        Some(5),
        "dagRank should be remapped to dag_rank"
    );
}

/// IO-TEST-5c: Version field is required
#[cfg(kani)]
#[kani::proof]
fn given_document_without_version_when_import_then_fails_gracefully() {
    let result = super::parse_diagram_document_with_compat(fixtures::NO_VERSION_DOCUMENT);
    assert!(
        result.is_err(),
        "Document without version should fail: expected error, got {:?}",
        result.ok()
    );
    let err_msg = result.expect_err("should have error");
    assert!(
        err_msg.contains("version"),
        "Error message should mention version field: {}",
        err_msg
    );
}

// ============================================================================
// Non-Kani tests (run with `cargo test`) for icon migration
// ============================================================================

/// Constructs a minimal valid document JSON string with the given node metadata.
fn make_doc_with_node_metadata(
    icon_field: &str,
    metadata_key: &str,
    metadata_value: &str,
) -> String {
    format!(
        r#"{{
            "version": 2,
            "document": {{
                "nodes": {{
                    "test-node": {{
                        "kind": "node",
                        "icon": "{icon_field}",
                        "label": "Test",
                        "x": 0.0, "y": 0.0,
                        "width": 64.0, "height": 64.0,
                        "z_index": 0,
                        "metadata": {{
                            "{metadata_key}": "{metadata_value}"
                        }}
                    }}
                }},
                "edges": {{}},
                "editor_state": {{
                    "camera_x": 0.0, "camera_y": 0.0, "zoom": 1.0,
                    "snap_to_grid": false, "grid_size": {{ "x": 20.0, "y": 20.0 }},
                    "selected_items": {{}}
                }},
                "revision": {{"counter": 0, "author": "test"}}
            }}
        }}"#,
        icon_field = icon_field,
        metadata_key = metadata_key,
        metadata_value = metadata_value.replace('\\', "\\\\").replace('"', "\\\"")
    )
}

#[test]
fn given_icon_data_url_base64_when_migrated_then_converted_to_url_path() {
    let json = make_doc_with_node_metadata(
        "aws/compute/ec2.png",
        "icon_data_url",
        "data:image/png;base64,iVBORw0KGgo=",
    );
    let result = super::parse_diagram_document_with_compat(&json);
    assert!(result.is_ok(), "Should parse: {:?}", result.err());
    let doc = result.expect("should have doc");
    let node = doc
        .document
        .nodes
        .values()
        .next()
        .expect("should have one node");

    // The old base64 data URL must NOT be present
    assert!(
        node.metadata.get("icon_data_url").is_none(),
        "Old icon_data_url key must be removed"
    );
    // Must have icon_url with a proper URL path (derived from node.icon)
    let icon_url = node
        .metadata
        .get("icon_url")
        .and_then(|v| v.as_str())
        .expect("icon_url should exist");
    assert_eq!(
        icon_url, "/assets/resources/aws/compute/ec2.png",
        "Base64 data URL must be converted to /assets/resources/ URL path"
    );
}

#[test]
fn given_icon_data_url_path_when_migrated_then_remapped_as_is() {
    // Edge case: if somehow icon_data_url contains a path (not base64), remap it
    let json = make_doc_with_node_metadata(
        "aws/compute/ec2.png",
        "icon_data_url",
        "/assets/resources/aws/compute/ec2.png",
    );
    let result = super::parse_diagram_document_with_compat(&json);
    assert!(result.is_ok(), "Should parse: {:?}", result.err());
    let doc = result.expect("should have doc");
    let node = doc
        .document
        .nodes
        .values()
        .next()
        .expect("should have one node");

    assert!(
        node.metadata.get("icon_data_url").is_none(),
        "Old key must be removed"
    );
    assert_eq!(
        node.metadata.get("icon_url").and_then(|v| v.as_str()),
        Some("/assets/resources/aws/compute/ec2.png"),
        "Non-base64 value should be remapped as-is"
    );
}

#[test]
fn given_both_icon_data_url_and_icon_url_when_migrated_then_icon_url_kept() {
    // If both old and new keys exist, new key wins (old key is removed)
    let json = r#"{
        "version": 2,
        "document": {
            "nodes": {
                "test-node": {
                    "kind": "node",
                    "icon": "aws/compute/ec2.png",
                    "label": "Test",
                    "x": 0.0, "y": 0.0,
                    "width": 64.0, "height": 64.0,
                    "z_index": 0,
                    "metadata": {
                        "icon_url": "/assets/resources/aws/compute/ec2.png",
                        "icon_data_url": "data:image/png;base64,old"
                    }
                }
            },
            "edges": {},
            "editor_state": {
                "camera_x": 0.0, "camera_y": 0.0, "zoom": 1.0,
                "snap_to_grid": false, "grid_size": {"x": 20.0, "y": 20.0},
                "selected_items": {}
            },
            "revision": {"counter": 0, "author": "test"}
        }
    }"#;
    let result = super::parse_diagram_document_with_compat(json);
    assert!(result.is_ok(), "Should parse: {:?}", result.err());
    let doc = result.expect("should have doc");
    let node = doc
        .document
        .nodes
        .values()
        .next()
        .expect("should have one node");

    // Old key removed
    assert!(node.metadata.get("icon_data_url").is_none());
    // New key preserved
    assert_eq!(
        node.metadata.get("icon_url").and_then(|v| v.as_str()),
        Some("/assets/resources/aws/compute/ec2.png"),
        "Existing icon_url must be preserved, old data URL must be dropped"
    );
}

#[test]
fn given_no_icon_metadata_when_migrated_then_no_icon_url_added() {
    let json = make_doc_with_node_metadata("aws/compute/ec2.png", "label", "EC2");
    let result = super::parse_diagram_document_with_compat(&json);
    assert!(result.is_ok(), "Should parse: {:?}", result.err());
    let doc = result.expect("should have doc");
    let node = doc
        .document
        .nodes
        .values()
        .next()
        .expect("should have one node");

    assert!(
        node.metadata.get("icon_url").is_none(),
        "icon_url must NOT be added if no icon_data_url existed"
    );
    assert!(
        node.metadata.get("icon_data_url").is_none(),
        "icon_data_url must not exist"
    );
}
