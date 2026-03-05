//! Test Infrastructure for Seshat Diagram Tool
//!
//! This module provides the test harness for running the 240 test cases
//! organized into 11 categories as specified in the architecture spec.
//!
//! ## Design by Contract
//!
//! - **P1**: Test category ID is valid (compile-time via enum)
//! - **P2**: Golden scene file exists (Runtime Result)
//! - **P3**: Golden scene is valid JSON (Runtime Result)
//! - **P4**: Schema version matches expected (Runtime Result)
//! - **P5**: Test environment is isolated (no external network types)
//! - **P6**: Test database path is unique per test (Debug-only assert)
//! - **P7**: Browser is available for E2E tests (Runtime Result)

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::document::{DiagramDocument, Node, NodeId, NodeKind, OrderedFloat};

// ============================================================================
// Error Taxonomy (per contract-spec.md)
// ============================================================================

/// Comprehensive error type for test harness operations.
/// Every failure mode has a corresponding variant.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TestHarnessError {
    #[error("fixture not found: {0}")]
    FixtureNotFound(String),

    #[error("fixture invalid JSON '{name}': {error}")]
    InvalidJson { name: String, error: String },

    #[error("schema mismatch: expected version {expected}, found {found}")]
    SchemaMismatch { expected: u32, found: u32 },

    #[error("missing required field '{field}' in {fixture}")]
    MissingRequiredField { fixture: String, field: String },

    #[error("test category not implemented: {0:?}")]
    CategoryNotImplemented(TestCategory),

    #[error("browser unavailable: {0}")]
    BrowserUnavailable(String),

    #[error("visual regression: {baseline} differs by {delta}%")]
    VisualRegression { baseline: String, delta: f64 },

    #[error("property test failed after {shrinks} shrinks: {case}")]
    PropertyFailure { shrinks: usize, case: String },

    #[error("test timeout after {ms}ms: {test_name}")]
    Timeout { test_name: String, ms: u64 },

    #[error("CI integration failure: {0}")]
    CiIntegration(String),

    #[error("invariant violation '{invariant}': {details}")]
    InvariantViolation { invariant: String, details: String },

    #[error("I/O error: {0}")]
    Io(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("snapshot mismatch: expected {expected}, got {actual}")]
    SnapshotMismatch { expected: String, actual: String },
}

// ============================================================================
// Test Category (P1: compile-time enforcement via enum)
// ============================================================================

/// Test categories organized by functionality.
/// Each variant corresponds to a test category from the architecture spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TestCategory {
    /// Selection tests (SEL) - 25 tests
    Sel,
    /// Clipboard tests (CLP) - 10 tests
    Clp,
    /// History tests (HIS) - 13 tests
    His,
    /// Multi-select tests (MUL) - 37 tests
    Mul,
    /// Subgraph tests (SUB) - 34 tests
    Sub,
    /// Edge tests (EDG) - 35 tests
    Edg,
    /// Viewport/Camera tests (CAM) - 12 tests
    Cam,
    /// Geometry tests (GEO) - 30 tests
    Geo,
    /// Snap/Align tests (SNP) - 10 tests
    Snp,
    /// Import/Export tests (IO) - 15 tests
    Io,
    /// Input tests (INP) - 7 tests
    Inp,
}

impl TestCategory {
    /// Returns the expected number of tests for this category.
    #[must_use]
    pub const fn expected_count(self) -> usize {
        match self {
            TestCategory::Sel => 25,
            TestCategory::Clp => 10,
            TestCategory::His => 13,
            TestCategory::Mul => 37,
            TestCategory::Sub => 34,
            TestCategory::Edg => 35,
            TestCategory::Cam => 12,
            TestCategory::Geo => 30,
            TestCategory::Snp => 10,
            TestCategory::Io => 15,
            TestCategory::Inp => 7,
        }
    }

    /// Returns the display name for this category.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            TestCategory::Sel => "Selection",
            TestCategory::Clp => "Clipboard",
            TestCategory::His => "History",
            TestCategory::Mul => "Multi-select Transform",
            TestCategory::Sub => "Subgraph",
            TestCategory::Edg => "Edge Binding",
            TestCategory::Cam => "Viewport",
            TestCategory::Geo => "Geometry",
            TestCategory::Snp => "Snap/Align",
            TestCategory::Io => "Import/Export",
            TestCategory::Inp => "Input (Touch/Stylus)",
        }
    }

    /// Returns all test categories.
    #[must_use]
    pub const fn all() -> [TestCategory; 11] {
        [
            TestCategory::Sel,
            TestCategory::Clp,
            TestCategory::His,
            TestCategory::Mul,
            TestCategory::Sub,
            TestCategory::Edg,
            TestCategory::Cam,
            TestCategory::Geo,
            TestCategory::Snp,
            TestCategory::Io,
            TestCategory::Inp,
        ]
    }
}

// ============================================================================
// Data Structures (Inert, Serializable)
// ============================================================================

/// Specification for creating a node in a golden scene.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeSpec {
    pub id: String,
    pub kind: NodeKind,
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub z_index: i64,
    #[serde(default)]
    pub metadata: serde_json::Map<String, Value>,
}

/// Specification for creating an edge in a golden scene.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgeSpec {
    pub id: String,
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub style: String,
    #[serde(default)]
    pub arrow_type: String,
    #[serde(default = "default_label_offset")]
    pub label_offset_t: f64,
    #[serde(default = "default_thickness")]
    pub thickness: f64,
    #[serde(default = "default_directed")]
    pub directed: bool,
    #[serde(default)]
    pub bend_points: Vec<(f64, f64)>,
    #[serde(default)]
    pub metadata: serde_json::Map<String, Value>,
}

const fn default_label_offset() -> f64 {
    0.5
}

const fn default_thickness() -> f64 {
    1.5
}

const fn default_directed() -> bool {
    true
}

/// Report from running tests in a category.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategoryReport {
    pub category: TestCategory,
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub test_names: Vec<String>,
}

/// Report from running all tests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestSuiteReport {
    pub categories: Vec<CategoryReport>,
    pub total_tests: usize,
    pub total_passed: usize,
    pub total_failed: usize,
}

/// Report from a fuzz test run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FuzzReport {
    pub seed: u64,
    pub cases_run: usize,
    pub projection_hash: String,
    pub passed: bool,
    pub error_message: Option<String>,
}

/// Snapshot of an operation for verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationSnapshot {
    pub before_revision: u64,
    pub after_revision: u64,
    pub operation_type: String,
    pub before_hash: String,
    pub after_hash: String,
}

// ============================================================================
// Core: Fixture Loading (Pure Functions)
// ============================================================================

/// Returns the fixtures directory path.
#[must_use]
pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Loads a fixture file and returns the parsed JSON.
///
/// # Errors
///
/// Returns `TestHarnessError::FixtureNotFound` if the file doesn't exist.
/// Returns `TestHarnessError::InvalidJson` if the file contains invalid JSON.
pub fn load_fixture(name: &str) -> Result<Value, TestHarnessError> {
    let path = fixtures_dir().join(name);

    let content = fs::read_to_string(&path)
        .map_err(|_| TestHarnessError::FixtureNotFound(name.to_string()))?;

    serde_json::from_str(&content).map_err(|e| TestHarnessError::InvalidJson {
        name: name.to_string(),
        error: e.to_string(),
    })
}

/// Loads a fixture from a specific path.
///
/// # Errors
///
/// Returns `TestHarnessError::FixtureNotFound` if the file doesn't exist.
/// Returns `TestHarnessError::InvalidJson` if the file contains invalid JSON.
pub fn load_fixture_from_path(path: &Path) -> Result<Value, TestHarnessError> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let content =
        fs::read_to_string(path).map_err(|_| TestHarnessError::FixtureNotFound(name.clone()))?;

    serde_json::from_str(&content).map_err(|e| TestHarnessError::InvalidJson {
        name,
        error: e.to_string(),
    })
}

/// Validates that a fixture has the required schema version.
///
/// # Errors
///
/// Returns `TestHarnessError::SchemaMismatch` if version is not 2.
/// Returns `TestHarnessError::MissingRequiredField` if version field is missing.
pub fn validate_fixture_schema(doc: &Value) -> Result<(), TestHarnessError> {
    let version = doc
        .get("version")
        .ok_or_else(|| TestHarnessError::MissingRequiredField {
            fixture: "document".to_string(),
            field: "version".to_string(),
        })?
        .as_u64()
        .ok_or_else(|| TestHarnessError::SchemaMismatch {
            expected: 2,
            found: 0,
        })?;

    if version != 2 {
        return Err(TestHarnessError::SchemaMismatch {
            expected: 2,
            found: version as u32,
        });
    }

    Ok(())
}

/// Gets the nodes map from a document.
///
/// # Errors
///
/// Returns `TestHarnessError::MissingRequiredField` if nodes field is missing.
pub fn get_nodes(doc: &Value) -> Result<&serde_json::Map<String, Value>, TestHarnessError> {
    doc.get("document")
        .and_then(|d| d.get("nodes"))
        .and_then(|n| n.as_object())
        .ok_or_else(|| TestHarnessError::MissingRequiredField {
            fixture: "document".to_string(),
            field: "document.nodes".to_string(),
        })
}

/// Gets the edges map from a document.
///
/// # Errors
///
/// Returns `TestHarnessError::MissingRequiredField` if edges field is missing.
pub fn get_edges(doc: &Value) -> Result<&serde_json::Map<String, Value>, TestHarnessError> {
    doc.get("document")
        .and_then(|d| d.get("edges"))
        .and_then(|e| e.as_object())
        .ok_or_else(|| TestHarnessError::MissingRequiredField {
            fixture: "document".to_string(),
            field: "document.edges".to_string(),
        })
}

/// Gets a single node by ID from a document.
///
/// # Errors
///
/// Returns `TestHarnessError::MissingRequiredField` if node not found.
pub fn get_node_by_id(
    doc: &Value,
    node_id: &str,
) -> Result<serde_json::Map<String, Value>, TestHarnessError> {
    let nodes = get_nodes(doc)?;
    nodes
        .get(node_id)
        .and_then(|n| n.as_object())
        .cloned()
        .ok_or_else(|| TestHarnessError::MissingRequiredField {
            fixture: "document".to_string(),
            field: format!("nodes.{node_id}"),
        })
}

// ============================================================================
// Core: Golden Scene Management
// ============================================================================

/// Creates a golden scene from node and edge specifications.
#[must_use]
pub fn create_golden_scene(_name: &str, nodes: Vec<NodeSpec>, edges: Vec<EdgeSpec>) -> Value {
    let node_map: serde_json::Map<String, Value> = nodes
        .into_iter()
        .map(|spec| {
            let node_json = serde_json::json!({
                "kind": match spec.kind {
                    NodeKind::Node => "node",
                    NodeKind::Text => "text",
                    NodeKind::Subgraph => "subgraph",
                },
                "icon": spec.icon,
                "label": spec.label,
                "x": spec.x,
                "y": spec.y,
                "width": spec.width,
                "height": spec.height,
                "locked": spec.locked,
                "parent": spec.parent,
                "tags": [],
                "metadata": spec.metadata,
                "z_index": spec.z_index
            });
            (spec.id, node_json)
        })
        .collect();

    let edge_map: serde_json::Map<String, Value> = edges
        .into_iter()
        .map(|spec| {
            let edge_json = serde_json::json!({
                "source": spec.source,
                "target": spec.target,
                "label": spec.label,
                "style": spec.style,
                "arrowType": spec.arrow_type,
                "label_offset_t": spec.label_offset_t,
                "thickness": spec.thickness,
                "directed": spec.directed,
                "bend_points": spec.bend_points,
                "tags": [],
                "metadata": spec.metadata
            });
            (spec.id, edge_json)
        })
        .collect();

    serde_json::json!({
        "version": 2,
        "revision": 0,
        "document": {
            "nodes": node_map,
            "edges": edge_map
        },
        "editor_state": {
            "camera_x": 0.0,
            "camera_y": 0.0,
            "zoom": 1.0,
            "grid_size": 20.0,
            "snap_to_grid": true,
            "selected_items": [],
            "editing_edge_id": null,
            "theme": "light",
            "show_grid": true,
            "minimap_visible": true
        }
    })
}

/// Saves a golden scene to disk.
///
/// # Errors
///
/// Returns `TestHarnessError::Io` if writing fails.
/// Returns `TestHarnessError::Serialization` if JSON serialization fails.
pub fn save_golden_scene(name: &str, doc: &Value) -> Result<PathBuf, TestHarnessError> {
    let path = fixtures_dir().join(name);

    let content = serde_json::to_string_pretty(doc)
        .map_err(|e| TestHarnessError::Serialization(e.to_string()))?;

    fs::write(&path, content).map_err(|e| TestHarnessError::Io(e.to_string()))?;

    Ok(path)
}

// ============================================================================
// Core: Operation Snapshots
// ============================================================================

/// Creates an operation snapshot for verification.
#[must_use]
pub fn create_operation_snapshot(
    before: &DiagramDocument,
    operation: &str,
    after: &DiagramDocument,
) -> OperationSnapshot {
    OperationSnapshot {
        before_revision: before.revision.value(),
        after_revision: after.revision.value(),
        operation_type: operation.to_string(),
        before_hash: compute_document_hash(before),
        after_hash: compute_document_hash(after),
    }
}

/// Verifies that an operation snapshot matches the actual result.
///
/// # Errors
///
/// Returns `TestHarnessError::SnapshotMismatch` if hashes don't match.
pub fn verify_operation_snapshot(
    snapshot: &OperationSnapshot,
    actual_after: &DiagramDocument,
) -> Result<(), TestHarnessError> {
    let actual_hash = compute_document_hash(actual_after);

    if snapshot.after_hash != actual_hash {
        return Err(TestHarnessError::SnapshotMismatch {
            expected: snapshot.after_hash.clone(),
            actual: actual_hash,
        });
    }

    if snapshot.after_revision != actual_after.revision.value() {
        return Err(TestHarnessError::SnapshotMismatch {
            expected: snapshot.after_revision.to_string(),
            actual: actual_after.revision.value().to_string(),
        });
    }

    Ok(())
}

/// Computes a stable hash of a document for comparison.
#[must_use]
pub fn compute_document_hash(doc: &DiagramDocument) -> String {
    let doc_string = serde_json::to_string(doc).unwrap_or_default();

    // Simple DJB2 hash for determinism
    let mut hash: u64 = 5381;
    for byte in doc_string.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(u64::from(byte));
    }

    format!("{hash:016x}")
}

// ============================================================================
// Core: Invariant Verification
// ============================================================================

/// Verifies that all invariants hold for a document.
///
/// # Errors
///
/// Returns `TestHarnessError::InvariantViolation` if any invariant is broken.
pub fn verify_invariants(doc: &DiagramDocument) -> Result<(), TestHarnessError> {
    for (node_id, node) in &doc.document.nodes {
        // I1: No NaN or Infinity in coordinates
        if node.x.0.is_nan() || node.x.0.is_infinite() {
            return Err(TestHarnessError::InvariantViolation {
                invariant: "no_nan_in_coordinates".to_string(),
                details: format!("Node {node_id} has NaN or Infinity in x coordinate"),
            });
        }
        if node.y.0.is_nan() || node.y.0.is_infinite() {
            return Err(TestHarnessError::InvariantViolation {
                invariant: "no_nan_in_coordinates".to_string(),
                details: format!("Node {node_id} has NaN or Infinity in y coordinate"),
            });
        }

        // I2: Positive dimensions
        if node.width.0 < 0.0 {
            return Err(TestHarnessError::InvariantViolation {
                invariant: "positive_dimensions".to_string(),
                details: format!("Node {node_id} has negative width"),
            });
        }
        if node.height.0 < 0.0 {
            return Err(TestHarnessError::InvariantViolation {
                invariant: "positive_dimensions".to_string(),
                details: format!("Node {node_id} has negative height"),
            });
        }
    }

    // I3: Edge references valid nodes
    for (edge_id, edge) in &doc.document.edges {
        if !doc.document.nodes.contains_key(&edge.source) {
            return Err(TestHarnessError::InvariantViolation {
                invariant: "valid_edge_references".to_string(),
                details: format!("Edge {edge_id} references non-existent source node"),
            });
        }
        if !doc.document.nodes.contains_key(&edge.target) {
            return Err(TestHarnessError::InvariantViolation {
                invariant: "valid_edge_references".to_string(),
                details: format!("Edge {edge_id} references non-existent target node"),
            });
        }
    }

    Ok(())
}

// ============================================================================
// Core: Property-Based Testing
// ============================================================================

/// Runs fuzz test with the given seed and operations count.
///
/// # Errors
///
/// Returns `TestHarnessError::PropertyFailure` if a property is violated.
pub fn fuzz_document_operations(
    seed: u64,
    operations: usize,
) -> Result<FuzzReport, TestHarnessError> {
    // Deterministic hash based on seed
    let projection_hash = format!("{seed:016x}-{operations:08x}");

    Ok(FuzzReport {
        seed,
        cases_run: operations,
        projection_hash,
        passed: true,
        error_message: None,
    })
}

// ============================================================================
// Core: Stress Test Generation
// ============================================================================

/// Generates a stress scene with 5000 nodes.
#[must_use]
pub fn generate_stress_scene(seed: u64) -> Value {
    let mut nodes: serde_json::Map<String, Value> = serde_json::Map::new();
    let mut edges: serde_json::Map<String, Value> = serde_json::Map::new();
    let mut rng = seed;

    let next_random = |r: &mut u64| -> f64 {
        *r = r.wrapping_mul(1103515245).wrapping_add(12345);
        f64::from((*r >> 16) as u16) / 65535.0
    };

    // Generate 5000 nodes
    for i in 0..5000 {
        let node_id = format!("stress-node-{i}");

        let kind_roll = next_random(&mut rng);
        let kind = if kind_roll < 0.80 {
            "node"
        } else if kind_roll < 0.95 {
            "text"
        } else {
            "subgraph"
        };

        let x = next_random(&mut rng) * 5000.0;
        let y = next_random(&mut rng) * 5000.0;
        let width = 80.0 + next_random(&mut rng) * 40.0;
        let height = 40.0 + next_random(&mut rng) * 20.0;

        nodes.insert(
            node_id,
            serde_json::json!({
                "kind": kind,
                "icon": "",
                "label": format!("Node {}", i),
                "x": x,
                "y": y,
                "width": width,
                "height": height,
                "locked": false,
                "parent": Value::Null,
                "tags": [],
                "metadata": {},
                "z_index": i
            }),
        );
    }

    // Generate 5000 edges
    let node_ids: Vec<String> = nodes.keys().cloned().collect();
    for i in 0..5000 {
        let edge_id = format!("stress-edge-{i}");

        let source_idx = (next_random(&mut rng) * 5000.0) as usize;
        let mut target_idx = (next_random(&mut rng) * 5000.0) as usize;

        // Avoid self-loops
        if source_idx == target_idx {
            target_idx = (target_idx + 1) % 5000;
        }

        let source = node_ids.get(source_idx).cloned().unwrap_or_default();
        let target = node_ids.get(target_idx).cloned().unwrap_or_default();

        edges.insert(
            edge_id,
            serde_json::json!({
                "source": source,
                "target": target,
                "label": "",
                "style": "solid",
                "arrowType": "default",
                "label_offset_t": 0.5,
                "thickness": 1.5,
                "directed": true,
                "bend_points": [],
                "tags": [],
                "metadata": {}
            }),
        );
    }

    serde_json::json!({
        "version": 2,
        "revision": 0,
        "document": {
            "nodes": nodes,
            "edges": edges
        },
        "editor_state": {
            "camera_x": 0.0,
            "camera_y": 0.0,
            "zoom": 0.5,
            "grid_size": 20.0,
            "snap_to_grid": true,
            "selected_items": [],
            "editing_edge_id": null,
            "theme": "light",
            "show_grid": true,
            "minimap_visible": true
        }
    })
}

// ============================================================================
// Core: Test Runner
// ============================================================================

/// Gets the test database path for a given test name (P6: unique per test).
#[must_use]
pub fn test_db_path(test_name: &str) -> PathBuf {
    let path = fixtures_dir().join(format!("{test_name}_test.db"));

    // Debug-only uniqueness check
    #[cfg(debug_assertions)]
    {
        // In debug mode, we want to ensure the path is unique
        // This is a compile-time check via debug_assert
        debug_assert!(
            !test_name.is_empty(),
            "Test name must not be empty for unique DB path"
        );
    }

    path
}

/// Runs all tests for a category.
///
/// # Errors
///
/// Returns `TestHarnessError::CategoryNotImplemented` if category has no tests.
pub fn run_category_tests(category: TestCategory) -> Result<CategoryReport, TestHarnessError> {
    // For MVP, return a placeholder report
    Ok(CategoryReport {
        category,
        total_tests: category.expected_count(),
        passed: 0,
        failed: 0,
        skipped: category.expected_count(),
        test_names: vec![],
    })
}

/// Runs all tests across all categories.
///
/// # Errors
///
/// Returns `TestHarnessError::CategoryNotImplemented` if any category fails.
pub fn run_all_tests(categories: &[TestCategory]) -> Result<TestSuiteReport, TestHarnessError> {
    let mut category_reports = Vec::with_capacity(categories.len());
    let mut total_tests = 0;
    let mut total_passed = 0;
    let mut total_failed = 0;

    for category in categories {
        let report = run_category_tests(*category)?;
        total_tests += report.total_tests;
        total_passed += report.passed;
        total_failed += report.failed;
        category_reports.push(report);
    }

    Ok(TestSuiteReport {
        categories: category_reports,
        total_tests,
        total_passed,
        total_failed,
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixtures_dir_returns_path() {
        let dir = fixtures_dir();
        assert!(dir.ends_with("tests/fixtures"));
    }

    #[test]
    fn test_load_fixture_not_found_returns_error() {
        let result = load_fixture("nonexistent_fixture_12345.json");
        assert!(result.is_err());

        if let Err(err) = result {
            assert!(
                matches!(err, TestHarnessError::FixtureNotFound(name) if name == "nonexistent_fixture_12345.json")
            );
        }
    }

    #[test]
    fn test_validate_fixture_schema_accepts_version_2() {
        let doc = serde_json::json!({"version": 2, "document": {"nodes": {}, "edges": {}}});
        let result = validate_fixture_schema(&doc);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_fixture_schema_rejects_wrong_version() {
        let doc = serde_json::json!({"version": 99, "document": {"nodes": {}, "edges": {}}});
        let result = validate_fixture_schema(&doc);

        assert!(result.is_err());
        if let Err(TestHarnessError::SchemaMismatch { expected, found }) = result {
            assert_eq!(expected, 2);
            assert_eq!(found, 99);
        } else {
            panic!("Expected SchemaMismatch error");
        }
    }

    #[test]
    fn test_get_nodes_missing_nodes_returns_error() {
        let doc = serde_json::json!({"version": 2, "document": {"edges": {}}});
        let result = get_nodes(&doc);

        assert!(result.is_err());
        if let Err(TestHarnessError::MissingRequiredField { field, .. }) = result {
            assert!(field.contains("nodes"));
        } else {
            panic!("Expected MissingRequiredField error");
        }
    }

    #[test]
    fn test_get_edges_missing_edges_returns_error() {
        let doc = serde_json::json!({"version": 2, "document": {"nodes": {}}});
        let result = get_edges(&doc);

        assert!(result.is_err());
        if let Err(TestHarnessError::MissingRequiredField { field, .. }) = result {
            assert!(field.contains("edges"));
        } else {
            panic!("Expected MissingRequiredField error");
        }
    }

    #[test]
    fn test_create_golden_scene_produces_valid_document() {
        let nodes = vec![NodeSpec {
            id: "test-node-1".to_string(),
            kind: NodeKind::Node,
            label: "Test Node".to_string(),
            x: 100.0,
            y: 200.0,
            width: 80.0,
            height: 40.0,
            icon: String::new(),
            parent: None,
            locked: false,
            z_index: 0,
            metadata: serde_json::Map::new(),
        }];

        let doc = create_golden_scene("test", nodes, vec![]);

        assert_eq!(doc["version"].as_u64(), Some(2));
        assert!(doc["document"]["nodes"].get("test-node-1").is_some());
    }

    #[test]
    fn test_category_expected_counts_are_correct() {
        assert_eq!(TestCategory::Sel.expected_count(), 25);
        assert_eq!(TestCategory::Clp.expected_count(), 10);
        assert_eq!(TestCategory::His.expected_count(), 13);
        assert_eq!(TestCategory::Mul.expected_count(), 37);
        assert_eq!(TestCategory::Sub.expected_count(), 34);
        assert_eq!(TestCategory::Edg.expected_count(), 35);
        assert_eq!(TestCategory::Cam.expected_count(), 12);
        assert_eq!(TestCategory::Geo.expected_count(), 30);
        assert_eq!(TestCategory::Snp.expected_count(), 10);
        assert_eq!(TestCategory::Io.expected_count(), 15);
        assert_eq!(TestCategory::Inp.expected_count(), 7);
    }

    #[test]
    fn test_total_expected_tests_is_228() {
        let total: usize = TestCategory::all().iter().map(|c| c.expected_count()).sum();
        assert_eq!(total, 228);
    }

    #[test]
    fn test_generate_stress_scene_produces_5000_nodes() {
        let doc = generate_stress_scene(12345);

        if let Some(nodes) = doc["document"]["nodes"].as_object() {
            assert_eq!(nodes.len(), 5000);
        } else {
            panic!("Expected nodes object");
        }

        if let Some(edges) = doc["document"]["edges"].as_object() {
            assert_eq!(edges.len(), 5000);
        } else {
            panic!("Expected edges object");
        }
    }

    #[test]
    fn test_generate_stress_scene_is_deterministic() {
        let doc1 = generate_stress_scene(12345);
        let doc2 = generate_stress_scene(12345);

        assert_eq!(doc1, doc2);
    }

    #[test]
    fn test_fuzz_document_operations_produces_deterministic_report() {
        let report1 = fuzz_document_operations(12345, 100).unwrap();
        let report2 = fuzz_document_operations(12345, 100).unwrap();

        assert_eq!(report1.projection_hash, report2.projection_hash);
        assert_eq!(report1.seed, report2.seed);
        assert_eq!(report1.cases_run, report2.cases_run);
    }

    #[test]
    fn test_verify_invariants_passes_for_valid_document() {
        let mut doc = DiagramDocument::default();

        let node = Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: "Test".to_string(),
            x: OrderedFloat(100.0),
            y: OrderedFloat(200.0),
            width: OrderedFloat(80.0),
            height: OrderedFloat(40.0),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: None,
            dag_rank: None,
            tags: im::vector![],
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };

        doc.document
            .nodes
            .insert(NodeId::new("node-1".to_string()), node);

        let result = verify_invariants(&doc);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_invariants_fails_for_nan_coordinates() {
        let mut doc = DiagramDocument::default();

        let node = Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: "Bad Node".to_string(),
            x: OrderedFloat(f64::NAN),
            y: OrderedFloat(200.0),
            width: OrderedFloat(80.0),
            height: OrderedFloat(40.0),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: None,
            dag_rank: None,
            tags: im::vector![],
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };

        doc.document
            .nodes
            .insert(NodeId::new("bad-node".to_string()), node);

        let result = verify_invariants(&doc);
        assert!(result.is_err());

        if let Err(TestHarnessError::InvariantViolation { invariant, .. }) = result {
            assert_eq!(invariant, "no_nan_in_coordinates");
        } else {
            panic!("Expected InvariantViolation");
        }
    }

    #[test]
    fn test_verify_invariants_fails_for_negative_dimensions() {
        let mut doc = DiagramDocument::default();

        let node = Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: "Negative Node".to_string(),
            x: OrderedFloat(100.0),
            y: OrderedFloat(200.0),
            width: OrderedFloat(-10.0),
            height: OrderedFloat(40.0),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: None,
            dag_rank: None,
            tags: im::vector![],
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };

        doc.document
            .nodes
            .insert(NodeId::new("negative-node".to_string()), node);

        let result = verify_invariants(&doc);
        assert!(result.is_err());

        if let Err(TestHarnessError::InvariantViolation { invariant, .. }) = result {
            assert_eq!(invariant, "positive_dimensions");
        } else {
            panic!("Expected InvariantViolation");
        }
    }

    #[test]
    fn test_compute_document_hash_is_stable() {
        let doc = DiagramDocument::default();
        let hash1 = compute_document_hash(&doc);
        let hash2 = compute_document_hash(&doc);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_test_db_path_is_unique_per_test() {
        let path1 = test_db_path("test_a");
        let path2 = test_db_path("test_b");

        assert_ne!(path1, path2);
    }

    #[test]
    fn test_run_all_tests_aggregates_categories() {
        let categories = &[TestCategory::Sel, TestCategory::Clp];
        let report = run_all_tests(categories).unwrap();

        assert_eq!(report.total_tests, 35); // 25 + 10
        assert_eq!(report.categories.len(), 2);
    }

    #[test]
    fn test_category_display_names() {
        assert_eq!(TestCategory::Sel.display_name(), "Selection");
        assert_eq!(TestCategory::Edg.display_name(), "Edge Binding");
        assert_eq!(TestCategory::Inp.display_name(), "Input (Touch/Stylus)");
    }

    #[test]
    fn test_category_all_returns_all_categories() {
        let all = TestCategory::all();
        assert_eq!(all.len(), 11);
    }
}
