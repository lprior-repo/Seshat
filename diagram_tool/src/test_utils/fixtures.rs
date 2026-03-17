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

use super::types::*;
use diagram_models::document::{DiagramDocument, Node, NodeId, NodeKind, OrderedFloat};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

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
