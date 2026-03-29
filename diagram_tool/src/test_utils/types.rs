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

#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(unused_imports, clippy::unnecessary_lazy_evaluations)]

use diagram_models::document::{DiagramDocument, Node, NodeId, NodeKind, OrderedFloat};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

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
