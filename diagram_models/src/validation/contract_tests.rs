#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]

use super::test_helpers::{make_edge, make_node};
use crate::document::{DiagramDocument, OrderedFloat};
use crate::validation::{validate_document, validate_document_data, ValidationCode};

// --- Automated Contract Verification ---

#[test]
fn cv01_production_paths_only_use_new_validator() {
    let app_validation = include_str!("../../../diagram_tool/src/app/validation.rs");
    let pipeline_validation =
        include_str!("../../../diagram_tool/src/mutation/pipeline_stages/validation.rs");
    let ui_helpers = include_str!("../../../diagram_tool/src/mutation/ui_helpers.rs");

    assert!(
        !app_validation.contains("validate_schema"),
        "app/validation.rs still calls validate_schema"
    );
    assert!(
        !pipeline_validation.contains("validate_schema"),
        "mutation/pipeline_stages/validation.rs still calls validate_schema"
    );
    assert!(
        !ui_helpers.contains("validate_schema"),
        "mutation/ui_helpers.rs still calls validate_schema"
    );

    assert!(
        app_validation.contains("validate_document"),
        "app/validation.rs does not call validate_document"
    );
    assert!(
        pipeline_validation.contains("validate_document"),
        "mutation/pipeline_stages/validation.rs does not call validate_document"
    );
}

#[test]
fn cv02_validation_module_is_anyhow_free() {
    let rules = include_str!("rules.rs");
    let types = include_str!("types.rs");
    let label = include_str!("label.rs");

    for (name, source) in [
        ("rules.rs", rules),
        ("types.rs", types),
        ("label.rs", label),
    ] {
        assert!(
            !source.contains("anyhow"),
            "{} contains 'anyhow' -- validation module must be anyhow-free",
            name
        );
        assert!(
            !source.contains("bail!"),
            "{} contains 'bail!' -- validation module must not use bail! macro",
            name
        );
        assert!(
            !source.contains("bail"),
            "{} contains 'bail' -- validation module must not use bail",
            name
        );
    }
}

#[test]
fn cv03_all_issues_use_validation_code_constants() {
    let rules_source = include_str!("rules.rs");
    let lines: Vec<&str> = rules_source.lines().collect();
    let mut error_call_count = 0;
    for (i, line) in lines.iter().enumerate() {
        if line.contains("ValidationIssue::error(") {
            error_call_count += 1;
            // Check this line and the next 10 lines for ValidationCode::
            // (handles match arms that define code variable above the call)
            let end = std::cmp::min(i + 11, lines.len());
            let start = i.saturating_sub(10);
            let window = lines[start..end].join("\n");
            assert!(
                window.contains("ValidationCode::"),
                "ValidationIssue::error() call does not use ValidationCode:: constant near: {}",
                line.trim()
            );
        }
    }
    assert!(
        error_call_count > 0,
        "No ValidationIssue::error() calls found in rules.rs -- test is vacuous"
    );
}

#[test]
fn cv04_validate_document_is_pure() {
    let mut doc = DiagramDocument::default();
    let (nid, mut node) = make_node("A");
    node.x = OrderedFloat::new_unchecked(f64::NAN);
    doc.document.nodes = doc.document.nodes.update(nid, node);

    let issues1 = validate_document(&doc);
    let issues2 = validate_document(&doc);
    assert_eq!(issues1, issues2);
}

#[test]
fn cv05_validate_document_data_skips_editor_state() {
    let mut doc = DiagramDocument::default();
    doc.editor_state.camera_x = OrderedFloat::new_unchecked(f64::NAN);
    let issues = validate_document_data(&doc.document);
    assert!(issues.is_empty());
}

#[test]
fn cv06_new_codes_defined_in_types_rs() {
    let _ = ValidationCode::INVALID_VERSION;
    let _ = ValidationCode::PARENT_CYCLE;
    let _ = ValidationCode::EDGE_INVALID_OFFSET;
    let _ = ValidationCode::EDGE_INVALID_THICKNESS;
    let _ = ValidationCode::EDGE_INVALID_COLOR;
    let _ = ValidationCode::EDGE_INVALID_FONT_SIZE;
    let _ = ValidationCode::EDITOR_INVALID_STATE;
}

#[test]
fn cv08_production_files_do_not_import_schema_validator() {
    let app_validation = include_str!("../../../diagram_tool/src/app/validation.rs");
    let pipeline_validation =
        include_str!("../../../diagram_tool/src/mutation/pipeline_stages/validation.rs");
    let ui_helpers = include_str!("../../../diagram_tool/src/mutation/ui_helpers.rs");

    for (name, source) in [
        ("app/validation.rs", app_validation),
        ("pipeline_stages/validation.rs", pipeline_validation),
        ("ui_helpers.rs", ui_helpers),
    ] {
        assert!(
            !source.contains("schema::validate_schema"),
            "{} still imports schema::validate_schema",
            name
        );
        assert!(
            !source.contains("use diagram_models::schema"),
            "{} still imports from diagram_models::schema",
            name
        );
    }
}

#[test]
fn cv09_validate_document_never_produces_schema_code() {
    let mut doc = DiagramDocument::default();
    doc.version = 1;

    let (nan_id, mut nan_node) = make_node("nan");
    nan_node.x = OrderedFloat::new_unchecked(f64::NAN);
    nan_node.width = OrderedFloat::new_unchecked(-1.0);
    nan_node.parent = Some(crate::document::NodeId::new("missing".to_string()));
    doc.document.nodes = doc.document.nodes.update(nan_id, nan_node);

    let (eid, mut edge) = make_edge("e1", "missing_src", "missing_tgt");
    edge.label_offset_t = OrderedFloat::new_unchecked(5.0);
    edge.thickness = OrderedFloat::new_unchecked(f64::NAN);
    edge.color = Some("bad".to_string());
    edge.font_size = Some(OrderedFloat::new_unchecked(f64::INFINITY));
    doc.document.edges = doc.document.edges.update(eid, edge);

    doc.editor_state.camera_x = OrderedFloat::new_unchecked(f64::NAN);
    doc.editor_state.zoom = OrderedFloat::new_unchecked(f64::INFINITY);

    let issues = validate_document(&doc);
    assert!(
        issues.iter().all(|i| i.code != ValidationCode::SCHEMA),
        "validate_document produced SCHEMA code -- forbidden. Issues: {:?}",
        issues
    );
}
