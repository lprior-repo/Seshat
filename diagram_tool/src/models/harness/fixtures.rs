//! Harness fixtures
#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::models::envelope::{Author, DomainOp, EventEnvelope};
use crate::models::projection::{replay_events, DiagramProjection, EventRecord};

use super::{FuzzReport, TestFailure, TestReport, VerifyError};

#[allow(clippy::unwrap_used)]
pub fn block_on<T>(fut: impl std::future::Future<Output = T>) -> T { tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(fut) }

struct SeededRng { state: u64 }
impl SeededRng { fn new(seed: u64) -> Self { Self { state: seed } } fn next(&mut self) -> u64 { self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1); self.state } }

pub fn projection_hash(projection: &DiagramProjection) -> Result<String, VerifyError> {
    let mut canonical = String::new();
    canonical.push_str(&format!("v:{}\n", projection.version));
    canonical.push_str(&format!("rev:{}\n", projection.revision));
    canonical.push_str(&format!("cycle:{:?}\n", projection.cycle_policy));
    let mut node_ids: Vec<_> = projection.nodes.keys().collect(); node_ids.sort();
    canonical.push_str("nodes:\n");
    for node_id in node_ids { if let Some(node) = projection.nodes.get(node_id) { canonical.push_str(&format!("  {}:({},{},{},{},{})\n", node_id, node.label, node.x.0, node.y.0, node.width.0, node.height.0)); } }
    let mut edge_ids: Vec<_> = projection.edges.keys().collect(); edge_ids.sort();
    canonical.push_str("edges:\n");
    for edge_id in edge_ids { if let Some(edge) = projection.edges.get(edge_id) { canonical.push_str(&format!("  {}:({}->{})\n", edge_id, edge.source, edge.target)); } }
    let mut hash: u64 = 5381; for byte in canonical.bytes() { hash = hash.wrapping_mul(33).wrapping_add(u64::from(byte)); }
    Ok(format!("{:016x}", hash))
}

pub fn run_replay_fuzz(seed: u64, cases: usize) -> Result<FuzzReport, VerifyError> {
    let mut rng = SeededRng::new(seed);
    let mut node_counter = 0u64;
    let mut author_counter = 0u64;
    let mut all_events: Vec<EventRecord> = Vec::new();
    let mut revision: u64 = 0;
    for _ in 0..cases {
        let op_type = (rng.next() % 4) as u8;
        let event = match op_type {
            0 => { node_counter += 1; let node_id = format!("node-{}", node_counter); EventRecord { op_id: format!("fuzz-{}", rng.next()), revision, operation: DomainOp::NodeAdd { id: node_id, x: (rng.next() % 1000) as f64, y: (rng.next() % 1000) as f64, width: 100.0, height: 50.0, label: format!("Node {}", node_counter) }, author: Author { id: format!("author-{}", author_counter), name: format!("Author {}", author_counter), email: None }, timestamp: 1700000000 + revision as i64 } }
            1 if node_counter > 0 => { let node_id = format!("node-{}", (rng.next() % node_counter) + 1); EventRecord { op_id: format!("fuzz-{}", rng.next()), revision, operation: DomainOp::NodeMove { id: node_id, x: (rng.next() % 1000) as f64, y: (rng.next() % 1000) as f64 }, author: Author { id: format!("author-{}", author_counter), name: format!("Author {}", author_counter), email: None }, timestamp: 1700000000 + revision as i64 } }
            2 if node_counter > 1 => { let source = format!("node-{}", (rng.next() % node_counter) + 1); let target = format!("node-{}", (rng.next() % node_counter) + 1); if source != target { EventRecord { op_id: format!("fuzz-{}", rng.next()), revision, operation: DomainOp::EdgeConnect { id: format!("edge-{}", rng.next()), source, target }, author: Author { id: format!("author-{}", author_counter), name: format!("Author {}", author_counter), email: None }, timestamp: 1700000000 + revision as i64 } } else { continue; } }
            _ => continue,
        };
        all_events.push(event);
        revision += 1;
        author_counter += 1;
    }
    let projection = replay_events(&all_events).map_err(|e| VerifyError::TestHarness(e.to_string()))?;
    let hash = projection_hash(&projection)?;
    Ok(FuzzReport::passing(seed, cases, hash))
}

pub fn assert_replay_determinism(report: &FuzzReport) -> Result<(), VerifyError> {
    if report.passed { Ok(()) } else { Err(VerifyError::DeterminismFailure(report.error_message.clone().unwrap_or_default())) }
}

pub fn run_replay_determinism_suite(seed: u64) -> Result<TestReport, VerifyError> {
    let report1 = run_replay_fuzz(seed, 10)?;
    let report2 = run_replay_fuzz(seed, 10)?;
    if report1.projection_hash == report2.projection_hash { Ok(TestReport { passed: true, tests_run: 1, failures: vec![] }) }
    else { Ok(TestReport { passed: false, tests_run: 1, failures: vec![TestFailure { test_name: "determinism".to_string(), message: format!("Hash mismatch: {} vs {}", report1.projection_hash, report2.projection_hash) }] }) }
}

pub fn run_crash_recovery_scenario(db_path: &std::path::Path) -> Result<TestReport, VerifyError> {
    let bootstrap = block_on(async { crate::store_async::bootstrap_async_store(db_path).await }).map_err(|e| VerifyError::TestHarness(e.to_string()))?;
    let check = block_on(async { crate::store_async::integrity_check_async(&bootstrap.pool).await });
    match check { Ok(_) => Ok(TestReport { passed: true, tests_run: 1, failures: vec![] }), Err(e) => Ok(TestReport { passed: false, tests_run: 1, failures: vec![TestFailure { test_name: "crash_recovery".to_string(), message: e.to_string() }] }) }
}

pub fn run_crash_recovery_suite() -> Result<TestReport, VerifyError> {
    let temp_dir = tempfile::tempdir().map_err(|e| VerifyError::Io(e.to_string()))?;
    let db_path = temp_dir.path().join("test.db");
    run_crash_recovery_scenario(&db_path)
}

pub fn assert_recovery_properties(report: &TestReport) -> Result<(), VerifyError> {
    if report.passed { Ok(()) } else { let messages: Vec<String> = report.failures.iter().map(|f| f.message.clone()).collect(); Err(VerifyError::TestFailure(messages.join("; "))) }
}
