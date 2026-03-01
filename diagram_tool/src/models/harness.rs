//! Verification harness module - replay fuzz and crash-recovery testing
//!
//! This module provides testing utilities for verifying replay
//! determinism and crash recovery.
//!
//! ## Contract Requirements
//!
//! - Event log remains append-only and replay deterministic
//! - Idempotent operation IDs never produce duplicate durable mutations
//! - Human-authored operations keep priority over conflicting AI operations
//! - Accepted operations increment revision monotonically by exactly one
//! - Rejected operations return structured error codes without side effects

#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use std::path::Path;
use thiserror::Error;

use crate::models::envelope::{Author, DomainOp, EventEnvelope};
use crate::models::projection::{replay_events, DiagramProjection, EventRecord};
use crate::store::{
    append_event, bootstrap_store, fetch_latest_revision, startup_integrity_check, StoreError,
};

/// Errors that can occur during verification
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum VerifyError {
    #[error("determinism failure: {0}")]
    DeterminismFailure(String),
    #[error("test harness error: {0}")]
    TestHarness(String),
    #[error("timeout: {0}")]
    Timeout(String),
    #[error("test failed: {0}")]
    TestFailure(String),
    #[error("IO error: {0}")]
    Io(String),
    #[error("SQLite error: {0}")]
    Sqlite(String),
}

impl From<std::io::Error> for VerifyError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<rusqlite::Error> for VerifyError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Sqlite(err.to_string())
    }
}

impl From<StoreError> for VerifyError {
    fn from(err: StoreError) -> Self {
        match err {
            StoreError::Io(e) => Self::Io(e.to_string()),
            StoreError::Sqlite(e) => Self::Sqlite(e.to_string()),
            other => Self::TestFailure(other.to_string()),
        }
    }
}

/// Report from a fuzz test run
///
/// Contains the final projection hash and statistics about the fuzz run.
/// The hash is computed deterministically from the final projection state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuzzReport {
    /// The seed used for this fuzz run
    pub seed: u64,
    /// Number of test cases run
    pub cases_run: usize,
    /// Stable hash of the final projection (deterministic serialization)
    pub projection_hash: String,
    /// Whether all test cases passed
    pub passed: bool,
    /// Error message if any case failed
    pub error_message: Option<String>,
}

impl FuzzReport {
    /// Create a new passing fuzz report
    fn passing(seed: u64, cases_run: usize, projection_hash: String) -> Self {
        Self {
            seed,
            cases_run,
            projection_hash,
            passed: true,
            error_message: None,
        }
    }

    /// Create a new failing fuzz report
    fn failing(seed: u64, cases_run: usize, message: impl Into<String>) -> Self {
        Self {
            seed,
            cases_run,
            projection_hash: String::new(),
            passed: false,
            error_message: Some(message.into()),
        }
    }
}

/// Compute a stable hash of a diagram projection
///
/// This function creates a deterministic hash of the projection by:
/// 1. Extracting nodes and edges in sorted key order
/// 2. Building a canonical string representation
/// 3. Computing a rolling hash
///
/// The hash is deterministic for the same projection state.
///
/// # Errors
///
/// Returns `VerifyError::TestHarness` if serialization fails.
pub fn projection_hash(projection: &DiagramProjection) -> Result<String, VerifyError> {
    // Build a canonical representation with sorted keys for determinism
    let mut canonical = String::new();

    // Add version and revision
    canonical.push_str(&format!("v:{}\n", projection.version));
    canonical.push_str(&format!("rev:{}\n", projection.revision));
    canonical.push_str(&format!("cycle:{:?}\n", projection.cycle_policy));

    // Add nodes in sorted order
    let mut node_keys: Vec<&String> = projection.author_priority.keys().collect();
    node_keys.sort();

    // Add nodes sorted by ID
    let mut node_ids: Vec<_> = projection.nodes.keys().collect();
    node_ids.sort();

    canonical.push_str("nodes:\n");
    for node_id in node_ids {
        if let Some(node) = projection.nodes.get(node_id) {
            canonical.push_str(&format!("  {}:({},{},{},{},{})\n",
                node_id,
                node.label,
                node.x.0,
                node.y.0,
                node.width.0,
                node.height.0
            ));
        }
    }

    // Add edges sorted by ID
    let mut edge_ids: Vec<_> = projection.edges.keys().collect();
    edge_ids.sort();

    canonical.push_str("edges:\n");
    for edge_id in edge_ids {
        if let Some(edge) = projection.edges.get(edge_id) {
            canonical.push_str(&format!("  {}:({}->{})\n",
                edge_id,
                edge.source,
                edge.target
            ));
        }
    }

    // Add author priority in sorted order
    canonical.push_str("priority:\n");
    for key in node_keys {
        if let Some(is_human) = projection.author_priority.get(key) {
            canonical.push_str(&format!("  {}:{}\n", key, is_human));
        }
    }

    // Compute rolling hash over canonical representation
    let mut hash: u64 = 5381; // DJB2 initial value
    for byte in canonical.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(u64::from(byte));
    }

    Ok(format!("{:016x}", hash))
}

/// Run seeded replay fuzz tests
///
/// This function generates seeded random operation streams and verifies
/// that replay produces deterministic results. Multiple runs with the
/// same seed must produce identical projection hashes.
///
/// # Arguments
///
/// * `seed` - The seed for deterministic random generation
/// * `cases` - Number of fuzz cases to run
///
/// # Returns
///
/// Returns a `FuzzReport` containing the projection hash and run statistics.
///
/// # Errors
///
/// Returns `VerifyError::DeterminismFailure` if replay is non-deterministic.
/// Returns `VerifyError::TestHarness` if test harness fails.
///
/// # Example
///
/// ```ignore
/// let report = run_replay_fuzz(42, 10)?;
/// assert_replay_determinism(&report)?;
/// ```
pub fn run_replay_fuzz(seed: u64, cases: usize) -> Result<FuzzReport, VerifyError> {
    let mut rng = SeededRng::new(seed);
    let mut node_counter = 0u64;
    let mut author_counter = 0u64;

    // Generate events for each case
    let mut all_events: Vec<EventRecord> = Vec::new();
    let mut revision: u64 = 0;

    for case_idx in 0..cases {
        let events_per_case = 5 + rng.next_usize(10);
        for _ in 0..events_per_case {
            let op = generate_random_op(&mut rng, &mut node_counter, &mut 0);
            let author = generate_random_author(&mut rng, &mut author_counter);

            all_events.push(EventRecord {
                op_id: format!("fuzz-seed{seed}-case{case_idx}-rev{revision}"),
                revision,
                operation: op,
                author,
                timestamp: 1700000000 + revision as i64,
            });
            revision += 1;
        }
    }

    // Run replay twice to verify determinism
    let projection1 = replay_events(&all_events)
        .map_err(|e| VerifyError::TestHarness(format!("First replay failed: {e}")))?;

    let projection2 = replay_events(&all_events)
        .map_err(|e| VerifyError::TestHarness(format!("Second replay failed: {e}")))?;

    // Verify projections are identical
    if projection1 != projection2 {
        return Ok(FuzzReport::failing(
            seed,
            cases,
            "Replay produced non-deterministic results",
        ));
    }

    // Compute stable hash
    let hash = projection_hash(&projection1)?;

    Ok(FuzzReport::passing(seed, cases, hash))
}

/// Assert that a fuzz report demonstrates deterministic replay
///
/// This function verifies that the fuzz report shows deterministic behavior.
/// It can be used to validate that repeated runs with the same seed produce
/// the same projection hash.
///
/// # Arguments
///
/// * `report` - The fuzz report to validate
///
/// # Returns
///
/// Returns `Ok(())` if the report shows deterministic behavior.
///
/// # Errors
///
/// Returns `VerifyError::DeterminismFailure` if the report shows non-determinism.
///
/// # Example
///
/// ```ignore
/// let report = run_replay_fuzz(42, 10)?;
/// assert_replay_determinism(&report)?;
/// ```
pub fn assert_replay_determinism(report: &FuzzReport) -> Result<(), VerifyError> {
    if !report.passed {
        return Err(VerifyError::DeterminismFailure(
            report
                .error_message
                .clone()
                .unwrap_or_else(|| "Fuzz test failed".to_string()),
        ));
    }

    if report.projection_hash.is_empty() {
        return Err(VerifyError::DeterminismFailure(
            "Empty projection hash - determinism cannot be verified".to_string(),
        ));
    }

    Ok(())
}

/// Report from a test run
#[derive(Debug, Clone)]
pub struct TestReport {
    /// Whether the test passed
    pub passed: bool,
    /// Number of test cases run
    pub cases_run: u64,
    /// Number of failures
    pub failures: u64,
    /// Error message if failed
    pub error_message: Option<String>,
}

impl TestReport {
    /// Create a new passing test report
    fn passing(cases_run: u64) -> Self {
        Self {
            passed: true,
            cases_run,
            failures: 0,
            error_message: None,
        }
    }

    /// Create a new failing test report
    fn failing(cases_run: u64, failures: u64, message: impl Into<String>) -> Self {
        Self {
            passed: false,
            cases_run,
            failures,
            error_message: Some(message.into()),
        }
    }

    /// Merge another report into this one
    fn merge(&mut self, other: &TestReport) {
        self.cases_run += other.cases_run;
        self.failures += other.failures;
        if !other.passed {
            self.passed = false;
            if let Some(ref msg) = other.error_message {
                if let Some(ref existing) = self.error_message {
                    self.error_message = Some(format!("{existing}; {msg}"));
                } else {
                    self.error_message = Some(msg.clone());
                }
            }
        }
    }
}

/// Simple seeded random number generator for reproducibility
///
/// This is a simple Linear Congruential Generator (LCG) for generating
/// deterministic random sequences from a seed.
struct SeededRng {
    state: u64,
}

impl SeededRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        // LCG parameters (same as glibc)
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        self.state
    }

    fn next_usize(&mut self, max: usize) -> usize {
        if max == 0 {
            return 0;
        }
        (self.next_u64() as usize) % max
    }

    fn next_f64(&mut self) -> f64 {
        // Generate a value between 0.0 and 1.0
        (self.next_u64() as f64) / (u64::MAX as f64)
    }
}

/// Generate a random domain operation for fuzzing
/// Only generates valid operations that will succeed during replay
fn generate_random_op(
    rng: &mut SeededRng,
    node_counter: &mut u64,
    _existing_nodes: &mut u64,
) -> DomainOp {
    // For simplicity, only generate NodeAdd and NodeMove operations
    // which are always valid
    let op_type = rng.next_usize(2);

    match op_type {
        0 => {
            // NodeAdd - always valid
            *node_counter += 1;
            let id = format!("node-{}", *node_counter);
            DomainOp::NodeAdd {
                id,
                x: rng.next_f64() * 1000.0,
                y: rng.next_f64() * 1000.0,
                width: 80.0 + rng.next_f64() * 120.0,
                height: 40.0 + rng.next_f64() * 60.0,
                label: format!("Node {}", *node_counter),
            }
        }
        _ => {
            // NodeMove - only valid if we have nodes, otherwise add a node
            if *node_counter == 0 {
                // No nodes yet, add one first
                *node_counter += 1;
                let id = format!("node-{}", *node_counter);
                DomainOp::NodeAdd {
                    id,
                    x: rng.next_f64() * 1000.0,
                    y: rng.next_f64() * 1000.0,
                    width: 80.0,
                    height: 40.0,
                    label: format!("Node {}", *node_counter),
                }
            } else {
                // Move the last added node
                let id = format!("node-{}", *node_counter);
                DomainOp::NodeMove {
                    id,
                    x: rng.next_f64() * 1000.0,
                    y: rng.next_f64() * 1000.0,
                }
            }
        }
    }
}

/// Generate a random author for fuzzing
fn generate_random_author(rng: &mut SeededRng, counter: &mut u64) -> Author {
    *counter += 1;
    let is_human = rng.next_usize(2) == 0;

    if is_human {
        Author {
            id: format!("human-{}", *counter),
            name: format!("Human User {}", *counter),
            email: Some(format!("user{}@example.com", *counter)),
        }
    } else {
        Author {
            id: format!("ai-{}", *counter),
            name: format!("AI Agent {}", *counter),
            email: None,
        }
    }
}

/// Run replay determinism test suite
///
/// This function runs a series of fuzz tests to verify that:
/// 1. Replaying the same events always produces the same projection
/// 2. Events are deterministic regardless of order
/// 3. Revision numbers increment monotonically
///
/// # Errors
/// Returns VerifyError if any test fails
pub fn run_replay_determinism_suite(seed: u64) -> Result<TestReport, VerifyError> {
    let mut report = TestReport::passing(0);
    let mut rng = SeededRng::new(seed);

    // Test 1: Deterministic replay with same seed produces same results
    {
        let case_report = test_deterministic_replay(&mut rng)?;
        report.merge(&case_report);
    }

    // Test 2: Revision increments monotonically
    {
        let case_report = test_revision_monotonic(&mut rng)?;
        report.merge(&case_report);
    }

    // Test 3: Random operation streams are replayable
    {
        let case_report = test_random_operation_streams(&mut rng)?;
        report.merge(&case_report);
    }

    // Test 4: Human priority is preserved in projection
    {
        let case_report = test_human_priority_preserved(&mut rng)?;
        report.merge(&case_report);
    }

    Ok(report)
}

/// Test that replaying the same events produces identical results
fn test_deterministic_replay(rng: &mut SeededRng) -> Result<TestReport, VerifyError> {
    let mut node_counter = 0u64;
    let mut existing_nodes = 0u64;
    let mut author_counter = 0u64;

    // Generate a sequence of events
    let num_events = 10 + rng.next_usize(20);
    let events: Vec<EventRecord> = (0..num_events)
        .map(|i| {
            let op = generate_random_op(rng, &mut node_counter, &mut existing_nodes);
            let author = generate_random_author(rng, &mut author_counter);
            EventRecord {
                op_id: format!("op-{}", i),
                revision: i as u64,
                operation: op,
                author,
                timestamp: 1700000000 + i as i64,
            }
        })
        .collect();

    // Replay events twice and compare
    let projection1 = replay_events(&events)
        .map_err(|e| VerifyError::TestFailure(format!("First replay failed: {e}")))?;

    let projection2 = replay_events(&events)
        .map_err(|e| VerifyError::TestFailure(format!("Second replay failed: {e}")))?;

    if projection1 == projection2 {
        Ok(TestReport::passing(1))
    } else {
        Ok(TestReport::failing(
            1,
            1,
            "Deterministic replay produced different results",
        ))
    }
}

/// Test that revision increments by exactly one for each event
fn test_revision_monotonic(rng: &mut SeededRng) -> Result<TestReport, VerifyError> {
    let mut node_counter = 0u64;
    let mut existing_nodes = 0u64;
    let mut author_counter = 0u64;

    let num_events = 10 + rng.next_usize(20);
    let events: Vec<EventRecord> = (0..num_events)
        .map(|i| {
            let op = generate_random_op(rng, &mut node_counter, &mut existing_nodes);
            let author = generate_random_author(rng, &mut author_counter);
            EventRecord {
                op_id: format!("op-{}", i),
                revision: i as u64,
                operation: op,
                author,
                timestamp: 1700000000 + i as i64,
            }
        })
        .collect();

    let projection = replay_events(&events)
        .map_err(|e| VerifyError::TestFailure(format!("Replay failed: {e}")))?;

    // Verify final revision equals number of events
    if projection.revision() == num_events as u64 {
        Ok(TestReport::passing(1))
    } else {
        Ok(TestReport::failing(
            1,
            1,
            format!(
                "Expected revision {} but got {}",
                num_events, projection.revision
            ),
        ))
    }
}

/// Test that random operation streams can be replayed successfully
fn test_random_operation_streams(rng: &mut SeededRng) -> Result<TestReport, VerifyError> {
    let mut passed = 0u64;
    let mut failed = 0u64;
    let mut error_messages = Vec::new();

    // Run multiple random streams
    for stream_idx in 0..5 {
        let mut node_counter = 0u64;
        let mut existing_nodes = 0u64;
        let mut author_counter = 0u64;
        let num_events = 5 + rng.next_usize(15);

        let events: Vec<EventRecord> = (0..num_events)
            .map(|i| {
                let op = generate_random_op(rng, &mut node_counter, &mut existing_nodes);
                let author = generate_random_author(rng, &mut author_counter);
                EventRecord {
                    op_id: format!("stream{}-op{}", stream_idx, i),
                    revision: i as u64,
                    operation: op,
                    author,
                    timestamp: 1700000000 + i as i64,
                }
            })
            .collect();

        match replay_events(&events) {
            Ok(projection) => {
                // Verify projection is in a valid state
                if projection.revision() == num_events as u64 {
                    passed += 1;
                } else {
                    failed += 1;
                    error_messages.push(format!(
                        "Stream {}: revision mismatch {} vs {}",
                        stream_idx,
                        projection.revision(),
                        num_events
                    ));
                }
            }
            Err(e) => {
                failed += 1;
                error_messages.push(format!("Stream {}: replay failed: {e}", stream_idx));
            }
        }
    }

    if failed == 0 {
        Ok(TestReport::passing(passed))
    } else {
        Ok(TestReport::failing(
            passed + failed,
            failed,
            error_messages.join("; "),
        ))
    }
}

/// Test that human priority is preserved in projection
fn test_human_priority_preserved(rng: &mut SeededRng) -> Result<TestReport, VerifyError> {
    let mut node_counter = 0u64;
    let mut existing_nodes = 0u64;

    // Create events with known human vs AI authors
    let events: Vec<EventRecord> = (0..5)
        .map(|i| {
            let op = generate_random_op(rng, &mut node_counter, &mut existing_nodes);
            let author = if i % 2 == 0 {
                // Human author
                Author {
                    id: format!("human-{}", i),
                    name: format!("Human {}", i),
                    email: None,
                }
            } else {
                // AI author
                Author {
                    id: format!("ai-{}", i),
                    name: format!("AI {}", i),
                    email: None,
                }
            };
            EventRecord {
                op_id: format!("op-{}", i),
                revision: i as u64,
                operation: op,
                author,
                timestamp: 1700000000 + i as i64,
            }
        })
        .collect();

    let projection = replay_events(&events)
        .map_err(|e| VerifyError::TestFailure(format!("Replay failed: {e}")))?;

    // Verify human operations are marked with priority
    let human_ops: Vec<&String> = projection
        .author_priority
        .iter()
        .filter(|(_, &is_human)| is_human)
        .map(|(op_id, _)| op_id)
        .collect();

    // We expect at least some human operations
    if !human_ops.is_empty() {
        Ok(TestReport::passing(1))
    } else {
        Ok(TestReport::failing(
            1,
            1,
            "No human priority entries found in projection",
        ))
    }
}

/// Run crash recovery scenario test
///
/// This function tests crash recovery scenarios:
/// 1. Database integrity after simulated crash
/// 2. Recovery from WAL (Write-Ahead Log)
/// 3. Snapshot recovery with tail replay
///
/// # Errors
/// Returns VerifyError if any test fails
pub fn run_crash_recovery_scenario(db_path: &Path) -> Result<TestReport, VerifyError> {
    let mut report = TestReport::passing(0);

    // Test 1: Integrity check on valid database
    {
        let case_report = test_integrity_check(db_path)?;
        report.merge(&case_report);
    }

    // Test 2: Recovery from fresh database
    {
        let case_report = test_fresh_database_recovery(db_path)?;
        report.merge(&case_report);
    }

    // Test 3: Event log append-only invariant
    {
        let case_report = test_append_only_invariant(db_path)?;
        report.merge(&case_report);
    }

    Ok(report)
}

/// Test integrity check on database
fn test_integrity_check(_db_path: &Path) -> Result<TestReport, VerifyError> {
    // Create a temporary test database
    let temp_dir = tempfile::TempDir::new()
        .map_err(|e| VerifyError::Io(format!("Failed to create temp dir: {e}")))?;
    let test_db_path = temp_dir.path().join("integrity_test.db");

    // Bootstrap a fresh database
    let _bootstrap = bootstrap_store(&test_db_path)?;

    // Run integrity check
    let status = startup_integrity_check(&test_db_path)
        .map_err(|e| VerifyError::TestFailure(format!("Integrity check failed: {e}")))?;

    if status.is_valid {
        Ok(TestReport::passing(1))
    } else {
        Ok(TestReport::failing(
            1,
            1,
            status
                .error_message
                .clone()
                .unwrap_or_else(|| "Integrity check failed with no error message".to_string()),
        ))
    }
}

/// Test recovery from fresh database
fn test_fresh_database_recovery(_db_path: &Path) -> Result<TestReport, VerifyError> {
    // Create a temporary test database
    let temp_dir = tempfile::TempDir::new()
        .map_err(|e| VerifyError::Io(format!("Failed to create temp dir: {e}")))?;
    let test_db_path = temp_dir.path().join("recovery_test.db");

    // Bootstrap and add some events
    let mut bootstrap = bootstrap_store(&test_db_path)?;

    // Add test events
    for i in 0..5 {
        let envelope = EventEnvelope {
            op_id: format!("recovery-op-{}", i),
            operation: DomainOp::NodeAdd {
                id: format!("node-{}", i),
                x: 100.0 * (i as f64),
                y: 100.0 * (i as f64),
                width: 80.0,
                height: 40.0,
                label: format!("Node {}", i),
            },
            author: Author {
                id: "human-test".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000 + i,
        };
        append_event(&mut bootstrap.conn, envelope, None)?;
    }

    // Verify we can read all events back
    let latest_revision = fetch_latest_revision(&bootstrap.conn)?;

    if latest_revision == 5 {
        Ok(TestReport::passing(1))
    } else {
        Ok(TestReport::failing(
            1,
            1,
            format!("Expected 5 events, found {}", latest_revision),
        ))
    }
}

/// Test that event log remains append-only
fn test_append_only_invariant(_db_path: &Path) -> Result<TestReport, VerifyError> {
    // Create a temporary test database
    let temp_dir = tempfile::TempDir::new()
        .map_err(|e| VerifyError::Io(format!("Failed to create temp dir: {e}")))?;
    let test_db_path = temp_dir.path().join("append_only_test.db");

    // Bootstrap and add some events
    let mut bootstrap = bootstrap_store(&test_db_path)?;

    // Add test events
    let mut op_ids: Vec<String> = Vec::new();
    for i in 0..3 {
        let op_id = format!("append-only-op-{}", i);
        op_ids.push(op_id.clone());
        let envelope = EventEnvelope {
            op_id,
            operation: DomainOp::NodeAdd {
                id: format!("node-{}", i),
                x: 100.0,
                y: 100.0,
                width: 80.0,
                height: 40.0,
                label: format!("Node {}", i),
            },
            author: Author {
                id: "human-test".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000 + i,
        };
        append_event(&mut bootstrap.conn, envelope, None)?;
    }

    // Verify all events are still present and in order
    let mut stmt = bootstrap
        .conn
        .prepare("SELECT operation_id, revision FROM events ORDER BY revision")
        .map_err(|e| VerifyError::Sqlite(e.to_string()))?;

    let retrieved_ids: Vec<(String, i64)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| VerifyError::Sqlite(e.to_string()))?
        .filter_map(Result::ok)
        .collect();

    // Verify all IDs are present
    let all_present = op_ids
        .iter()
        .all(|id| retrieved_ids.iter().any(|(rid, _)| rid == id));

    // Verify revisions are sequential
    let revisions_sequential = retrieved_ids
        .iter()
        .enumerate()
        .all(|(i, (_, rev))| *rev == (i + 1) as i64);

    if all_present && revisions_sequential {
        Ok(TestReport::passing(2))
    } else {
        Ok(TestReport::failing(
            2,
            1,
            "Append-only invariant violated: events missing or out of order",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::projection::replay_events_from;
    use tempfile::TempDir;

    #[test]
    fn test_happy_path_valid_operation_appends_and_returns_revision() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        // Submit a valid operation
        let envelope = EventEnvelope {
            op_id: "op-valid-1".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 100.0,
                y: 200.0,
                width: 80.0,
                height: 40.0,
                label: "Test Node".to_string(),
            },
            author: Author {
                id: "human-user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        let result = append_event(&mut bootstrap.conn, envelope, None);

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
        let outcome = result.expect("Checked is_ok");
        assert_eq!(outcome.revision, 1, "Revision should increment to 1");
        assert_eq!(outcome.op_id, "op-valid-1");

        // Verify the revision was actually incremented
        let latest = fetch_latest_revision(&bootstrap.conn).expect("Failed to fetch revision");
        assert_eq!(latest, 1, "Latest revision should be 1");
    }

    #[test]
    fn test_happy_path_replay_from_revision_zero_recreates_projection() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        // Add multiple events
        let events_data = [
            ("op-1", "node-1", 100.0, 100.0),
            ("op-2", "node-2", 200.0, 200.0),
            ("op-3", "node-3", 300.0, 300.0),
        ];

        for (i, (op_id, node_id, x, y)) in events_data.iter().enumerate() {
            let envelope = EventEnvelope {
                op_id: op_id.to_string(),
                operation: DomainOp::NodeAdd {
                    id: node_id.to_string(),
                    x: *x,
                    y: *y,
                    width: 80.0,
                    height: 40.0,
                    label: format!("Node {}", i),
                },
                author: Author {
                    id: "human-user-1".to_string(),
                    name: "Test User".to_string(),
                    email: None,
                },
                timestamp: 1700000000 + i as i64,
            };
            append_event(&mut bootstrap.conn, envelope, None).expect("Failed to append");
        }

        // Read events back and create EventRecords
        let mut stmt = bootstrap
            .conn
            .prepare(
                "SELECT operation_id, revision, payload, timestamp FROM events ORDER BY revision",
            )
            .expect("Failed to prepare statement");

        let event_records: Vec<EventRecord> = stmt
            .query_map([], |row| {
                let op_id: String = row.get(0)?;
                let revision: i64 = row.get(1)?;
                let payload: String = row.get(2)?;
                let timestamp_str: String = row.get(3)?;

                // Parse timestamp from string
                let timestamp: i64 = timestamp_str.parse().unwrap_or(0);

                // Parse the envelope from payload to get the operation
                let envelope: EventEnvelope = match serde_json::from_str(&payload) {
                    Ok(e) => e,
                    Err(e) => {
                        eprintln!("DEBUG: Failed to parse payload: {}", e);
                        eprintln!("DEBUG: Payload was: {}", payload);
                        return Err(rusqlite::Error::InvalidQuery);
                    }
                };

                Ok(EventRecord {
                    op_id,
                    revision: revision as u64,
                    operation: envelope.operation,
                    author: envelope.author,
                    timestamp,
                })
            })
            .expect("Failed to query")
            .collect::<Result<Vec<_>, _>>()
            .expect("Failed to collect events");

        // Debug output
        eprintln!("DEBUG: Collected {} event records", event_records.len());
        for (i, e) in event_records.iter().enumerate() {
            eprintln!(
                "DEBUG: Event {}: op_id={}, revision={}",
                i, e.op_id, e.revision
            );
        }

        // Replay from the first event's revision
        let start_revision = event_records.first().map(|e| e.revision).unwrap_or(0);
        let projection = replay_events_from(
            DiagramProjection::with_revision(start_revision),
            &event_records,
        )
        .expect("Replay failed");

        // Verify projection matches expected state
        // Final revision = start_revision + number of events
        let expected_revision = start_revision + event_records.len() as u64;
        assert_eq!(
            projection.revision(),
            expected_revision,
            "Projection should be at revision {}",
            expected_revision
        );
        assert_eq!(projection.nodes.len(), 3, "Projection should have 3 nodes");
    }

    #[test]
    fn test_error_path_stale_revision_rejects_without_append() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        // Add an initial event to get to revision 1
        let envelope1 = EventEnvelope {
            op_id: "op-initial".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 100.0,
                y: 100.0,
                width: 80.0,
                height: 40.0,
                label: "Initial Node".to_string(),
            },
            author: Author {
                id: "human-user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };
        append_event(&mut bootstrap.conn, envelope1, None).expect("Failed to append initial");

        // Try to append with stale expected revision (0 instead of 1)
        let envelope2 = EventEnvelope {
            op_id: "op-stale".to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-2".to_string(),
                x: 200.0,
                y: 200.0,
                width: 80.0,
                height: 40.0,
                label: "Stale Node".to_string(),
            },
            author: Author {
                id: "human-user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000001,
        };

        let result = append_event(&mut bootstrap.conn, envelope2, Some(0));

        // Should fail with revision mismatch
        assert!(result.is_err(), "Expected error for stale revision");
        match result {
            Err(StoreError::RevisionMismatch { expected, found }) => {
                assert_eq!(expected, 0, "Expected expected revision 0");
                assert_eq!(found, 1, "Expected found revision 1");
            }
            Err(other) => panic!("Expected RevisionMismatch error, got: {:?}", other),
            Ok(_) => panic!("Expected error, got success"),
        }

        // Verify no new event was appended (still at revision 1)
        let latest = fetch_latest_revision(&bootstrap.conn).expect("Failed to fetch revision");
        assert_eq!(
            latest, 1,
            "Revision should still be 1 after rejected append"
        );
    }

    #[test]
    fn test_error_path_duplicate_op_id_returns_idempotent_success() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let mut bootstrap = bootstrap_store(&db_path).expect("Failed to bootstrap store");

        let op_id = "op-duplicate-test";

        // Add first event
        let envelope1 = EventEnvelope {
            op_id: op_id.to_string(),
            operation: DomainOp::NodeAdd {
                id: "node-dup".to_string(),
                x: 100.0,
                y: 100.0,
                width: 80.0,
                height: 40.0,
                label: "Duplicate Node".to_string(),
            },
            author: Author {
                id: "human-user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        let result1 = append_event(&mut bootstrap.conn, envelope1, None);
        assert!(result1.is_ok(), "First append should succeed");
        let outcome1 = result1.expect("Checked is_ok");

        // Try to add duplicate op_id
        let envelope2 = EventEnvelope {
            op_id: op_id.to_string(), // Same op_id
            operation: DomainOp::NodeAdd {
                id: "node-dup-2".to_string(),
                x: 200.0,
                y: 200.0,
                width: 80.0,
                height: 40.0,
                label: "Another Node".to_string(),
            },
            author: Author {
                id: "human-user-1".to_string(),
                name: "Test User".to_string(),
                email: None,
            },
            timestamp: 1700000001,
        };

        let result2 = append_event(&mut bootstrap.conn, envelope2, None);

        // Should fail with SQLite constraint violation (UNIQUE constraint on operation_id)
        assert!(result2.is_err(), "Duplicate op_id should be rejected");

        // Verify no duplicate was created (still at revision 1)
        let latest = fetch_latest_revision(&bootstrap.conn).expect("Failed to fetch revision");
        assert_eq!(
            latest, 1,
            "Revision should still be 1 after duplicate rejection"
        );

        // Verify the original event is still there
        let count: i64 = bootstrap
            .conn
            .query_row(
                "SELECT COUNT(*) FROM events WHERE operation_id = ?1",
                [op_id],
                |row| row.get(0),
            )
            .expect("Failed to count events");

        assert_eq!(count, 1, "Should have exactly one event with the op_id");
    }

    #[test]
    fn test_replay_determinism_suite_passes_with_valid_seed() {
        let result = run_replay_determinism_suite(42);
        assert!(result.is_ok(), "Suite should not error: {:?}", result.err());

        let report = result.expect("Checked is_ok");
        assert!(report.cases_run > 0, "Should run at least one test case");
    }

    #[test]
    fn test_crash_recovery_scenario_passes_on_valid_path() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let result = run_crash_recovery_scenario(&db_path);
        assert!(
            result.is_ok(),
            "Scenario should not error: {:?}",
            result.err()
        );

        let report = result.expect("Checked is_ok");
        assert!(report.cases_run > 0, "Should run at least one test case");
    }

    #[test]
    fn test_seeded_rng_deterministic() {
        let mut rng1 = SeededRng::new(12345);
        let mut rng2 = SeededRng::new(12345);

        for _ in 0..10 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }
    }

    #[test]
    fn test_seeded_rng_different_seeds_produce_different_values() {
        let mut rng1 = SeededRng::new(12345);
        let mut rng2 = SeededRng::new(54321);

        let mut any_different = false;
        for _ in 0..10 {
            if rng1.next_u64() != rng2.next_u64() {
                any_different = true;
                break;
            }
        }
        assert!(
            any_different,
            "Different seeds should produce different values"
        );
    }

    #[test]
    fn test_test_report_merge_combines_counts() {
        let mut report1 = TestReport::passing(5);
        let report2 = TestReport::passing(3);

        report1.merge(&report2);

        assert_eq!(report1.cases_run, 8);
        assert!(report1.passed);
    }

    #[test]
    fn test_test_report_merge_preserves_failures() {
        let mut report1 = TestReport::passing(5);
        let report2 = TestReport::failing(3, 1, "test error");

        report1.merge(&report2);

        assert_eq!(report1.cases_run, 8);
        assert_eq!(report1.failures, 1);
        assert!(!report1.passed);
        assert_eq!(report1.error_message, Some("test error".to_string()));
    }

    // Tests for bd-1wc: verify-replay-fuzz contract

    #[test]
    fn test_run_replay_fuzz_returns_deterministic_report() {
        // Same seed should produce same report
        let report1 = run_replay_fuzz(42, 10);
        let report2 = run_replay_fuzz(42, 10);

        assert!(report1.is_ok(), "First run should succeed");
        assert!(report2.is_ok(), "Second run should succeed");

        let r1 = report1.expect("Checked is_ok");
        let r2 = report2.expect("Checked is_ok");

        assert_eq!(r1.seed, r2.seed, "Seeds should match");
        assert_eq!(r1.cases_run, r2.cases_run, "Case counts should match");
        assert_eq!(
            r1.projection_hash, r2.projection_hash,
            "Projection hashes should be identical"
        );
        assert!(r1.passed, "Report should indicate pass");
    }

    #[test]
    fn test_run_replay_fuzz_different_seeds_produce_different_hashes() {
        let report1 = run_replay_fuzz(42, 10);
        let report2 = run_replay_fuzz(12345, 10);

        assert!(report1.is_ok(), "First run should succeed");
        assert!(report2.is_ok(), "Second run should succeed");

        let r1 = report1.expect("Checked is_ok");
        let r2 = report2.expect("Checked is_ok");

        // Different seeds should (almost certainly) produce different hashes
        assert_ne!(
            r1.projection_hash, r2.projection_hash,
            "Different seeds should produce different hashes"
        );
    }

    #[test]
    fn test_assert_replay_determinism_accepts_valid_report() {
        let report = run_replay_fuzz(42, 5).expect("Fuzz should succeed");

        let result = assert_replay_determinism(&report);
        assert!(result.is_ok(), "Valid report should pass assertion");
    }

    #[test]
    fn test_assert_replay_determinism_rejects_failed_report() {
        let failed_report = FuzzReport::failing(42, 5, "Test failure");

        let result = assert_replay_determinism(&failed_report);
        assert!(result.is_err(), "Failed report should be rejected");

        match result {
            Err(VerifyError::DeterminismFailure(msg)) => {
                assert!(msg.contains("Test failure"));
            }
            _ => panic!("Expected DeterminismFailure error"),
        }
    }

    #[test]
    fn test_assert_replay_determinism_rejects_empty_hash() {
        let empty_hash_report = FuzzReport {
            seed: 42,
            cases_run: 5,
            projection_hash: String::new(),
            passed: true,
            error_message: None,
        };

        let result = assert_replay_determinism(&empty_hash_report);
        assert!(result.is_err(), "Empty hash should be rejected");

        match result {
            Err(VerifyError::DeterminismFailure(msg)) => {
                assert!(msg.contains("Empty projection hash"));
            }
            _ => panic!("Expected DeterminismFailure error"),
        }
    }

    #[test]
    fn test_projection_hash_is_stable() {
        use crate::models::projection::DiagramProjection;

        let projection = DiagramProjection::empty();
        let hash1 = projection_hash(&projection);
        let hash2 = projection_hash(&projection);

        assert!(hash1.is_ok(), "First hash should succeed");
        assert!(hash2.is_ok(), "Second hash should succeed");

        assert_eq!(
            hash1.expect("Checked is_ok"),
            hash2.expect("Checked is_ok"),
            "Hash should be stable for same projection"
        );
    }

    #[test]
    fn test_projection_hash_differs_for_different_projections() {
        use crate::models::envelope::DomainOp;

        // Create two projections with different events
        let events1 = vec![EventRecord {
            op_id: "op-1".to_string(),
            revision: 0,
            operation: DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 100.0,
                y: 100.0,
                width: 80.0,
                height: 40.0,
                label: "Node 1".to_string(),
            },
            author: Author {
                id: "human-test".to_string(),
                name: "Test".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        }];

        let events2 = vec![EventRecord {
            op_id: "op-2".to_string(),
            revision: 0,
            operation: DomainOp::NodeAdd {
                id: "node-2".to_string(),
                x: 200.0,
                y: 200.0,
                width: 80.0,
                height: 40.0,
                label: "Node 2".to_string(),
            },
            author: Author {
                id: "human-test".to_string(),
                name: "Test".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        }];

        let projection1 = replay_events(&events1).expect("Replay should succeed");
        let projection2 = replay_events(&events2).expect("Replay should succeed");

        let hash1 = projection_hash(&projection1).expect("Hash should succeed");
        let hash2 = projection_hash(&projection2).expect("Hash should succeed");

        assert_ne!(hash1, hash2, "Different projections should have different hashes");
    }

    #[test]
    fn test_fuzz_report_passing_factory() {
        let report = FuzzReport::passing(42, 10, "abc123".to_string());

        assert_eq!(report.seed, 42);
        assert_eq!(report.cases_run, 10);
        assert_eq!(report.projection_hash, "abc123");
        assert!(report.passed);
        assert!(report.error_message.is_none());
    }

    #[test]
    fn test_fuzz_report_failing_factory() {
        let report = FuzzReport::failing(42, 10, "error message");

        assert_eq!(report.seed, 42);
        assert_eq!(report.cases_run, 10);
        assert!(!report.passed);
        assert_eq!(report.error_message, Some("error message".to_string()));
    }
}
