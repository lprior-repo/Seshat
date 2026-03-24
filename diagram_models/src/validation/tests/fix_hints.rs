//! Tests for ValidationCode::default_fix_hint() and ValidationIssue fix_hint behavior.

use crate::validation::types::{ValidationCode, ValidationIssue, ValidationSeverity};

macro_rules! assert_hint {
    ($code:expr, $expected:expr) => {
        assert_eq!($code.default_fix_hint(), Some($expected));
    };
}

#[test]
fn all_known_codes_have_hints() {
    assert_hint!(ValidationCode::EDGE_DANGLING, "Remove edge {edge_id} or create missing node {missing_node_id}");
    assert_hint!(ValidationCode::INVALID_PARENT, "Set node {node_id}.parent to None or reference an existing Subgraph node");
    assert_hint!(ValidationCode::INVALID_NUMERIC, "Ensure numeric field {field_name} is finite. Got: {actual_value}");
    assert_hint!(ValidationCode::DAG_CYCLE, "Remove one edge in the cycle to break it. Cycle path: {cycle_path}");
    assert_hint!(ValidationCode::DAG_DISCONNECTED, "Add edges to connect all {n} components into a single connected graph");
    assert_hint!(ValidationCode::INTERNAL_ERROR, "This is a bug. Report to developers with reproduction steps.");
    assert_hint!(ValidationCode::SCHEMA, "Fix schema violation at {path}: expected {expected}, got {actual}");
    assert_hint!(ValidationCode::INVALID_VERSION, "Set document version to 2. Current: {actual}");
    assert_hint!(ValidationCode::PARENT_CYCLE, "Break the parent chain cycle by setting {node_id}.parent = None");
    assert_hint!(ValidationCode::EDGE_INVALID_OFFSET, "Set edge {edge_id}.label_offset_t to a value in [0.0, 1.0]. Got: {actual}");
    assert_hint!(ValidationCode::EDGE_INVALID_THICKNESS, "Set edge {edge_id}.thickness to a finite non-negative value. Got: {actual}");
    assert_hint!(ValidationCode::EDGE_INVALID_COLOR, "Set edge {edge_id}.color to hex format #RGB, #RGBA, #RRGGBB, or #RRGGBBAA. Got: {actual}");
    assert_hint!(ValidationCode::EDGE_INVALID_FONT_SIZE, "Set edge {edge_id}.font_size to a finite value. Got: {actual}");
    assert_hint!(ValidationCode::EDITOR_INVALID_STATE, "Set editor.{field} to a finite value. Got: {actual}");
}

#[test]
fn unknown_codes_return_none() {
    assert_eq!(ValidationCode::from("unknown-code").default_fix_hint(), None);
    assert_eq!(ValidationCode::from("").default_fix_hint(), None);
    assert_eq!(ValidationCode::from("日本語").default_fix_hint(), None);
    assert_eq!(ValidationCode::from("x".repeat(1000)).default_fix_hint(), None);
}

#[test]
fn error_populates_fix_hint() {
    let issue = ValidationIssue::error(
        ValidationCode::EDGE_DANGLING,
        "Edge source does not exist",
        Some("edge-1".to_string()),
    );
    assert_eq!(issue.fix_hint, Some("Remove edge {edge_id} or create missing node {missing_node_id}".to_string()));
    assert_eq!(issue.severity, ValidationSeverity::Error);
}

#[test]
fn with_fix_hint_sets_custom() {
    let issue = ValidationIssue::with_fix_hint(
        ValidationSeverity::Error,
        ValidationCode::EDGE_DANGLING,
        "Custom error",
        None,
        "Custom fix: delete the edge",
    );
    assert_eq!(issue.fix_hint, Some("Custom fix: delete the edge".to_string()));
}

#[test]
fn error_edge_cases() {
    // Empty message still gets hint
    let issue = ValidationIssue::error(ValidationCode::INVALID_VERSION, "", None);
    assert!(issue.fix_hint.is_some());
    assert_eq!(issue.message, "");

    // None subject still gets hint
    let issue = ValidationIssue::error(ValidationCode::INVALID_VERSION, "Bad version", None);
    assert!(issue.fix_hint.is_some());
    assert_eq!(issue.subject, None);

    // Unknown code has no hint
    let issue = ValidationIssue::error(ValidationCode::from("unknown"), "Unknown error", None);
    assert_eq!(issue.fix_hint, None);
}
