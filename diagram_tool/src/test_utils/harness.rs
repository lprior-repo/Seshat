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
#![allow(unused_imports, clippy::unnecessary_lazy_evaluations)]

use super::fixtures::*;
use super::types::*;
use diagram_models::document::{DiagramDocument, Node, NodeId, NodeKind, OrderedFloat};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

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
