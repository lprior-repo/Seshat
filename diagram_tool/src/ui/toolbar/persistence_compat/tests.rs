use super::*;
use diagram_models::document::DiagramDocument;

#[path = "fixtures.rs"]
mod fixtures;

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_document_when_serialized_then_round_trips() {
    let doc = DiagramDocument::default();
    let json = serde_json::to_string_pretty(&doc).unwrap();
    let loaded: DiagramDocument = serde_json::from_str(&json).unwrap();
    assert_eq!(doc.revision, loaded.revision);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_ts_style_json_when_parsed_then_document_loads() {
    let loaded = super::parse_diagram_document_with_compat(fixtures::TS_STYLE_JSON);
    assert!(loaded.is_ok(), "{:?}", loaded.err());
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_legacy_font_size_keys_when_parsed_then_document_loads() {
    let loaded = super::parse_diagram_document_with_compat(fixtures::LEGACY_FONT_SIZE_JSON);
    assert!(loaded.is_ok(), "{:?}", loaded.err());
}

#[cfg(kani)]
#[kani::proof]
#[test]
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
#[test]
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
#[test]
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
#[test]
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
