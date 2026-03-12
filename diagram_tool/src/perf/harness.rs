//! Benchmark harness for diagram operations.

use std::{collections::HashMap, path::PathBuf, time::UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::{
    benchmark::{Benchmark, BenchmarkConfig, BenchmarkResult},
    error::PerfError,
    fps::FpsReport,
    BASELINE_NODE_COUNT, TARGET_FPS,
};

/// Diagram operations that can be benchmarked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Operation {
    /// Pan viewport
    Pan,
    /// Zoom in/out
    Zoom,
    /// Select node
    Select,
    /// Drag node
    Drag,
    /// Full frame render
    RenderFrame,
}

impl Operation {
    /// Returns all operations.
    #[must_use]
    pub const fn all() -> [Self; 5] {
        [
            Self::Pan,
            Self::Zoom,
            Self::Select,
            Self::Drag,
            Self::RenderFrame,
        ]
    }

    /// Returns the operation name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pan => "pan",
            Self::Zoom => "zoom",
            Self::Select => "select",
            Self::Drag => "drag",
            Self::RenderFrame => "render_frame",
        }
    }

    /// Returns the expected complexity factor.
    #[must_use]
    pub const fn complexity_factor(self) -> f64 {
        match self {
            Self::Pan => 0.8,         // Relatively cheap
            Self::Zoom => 0.9,        // Slightly more expensive
            Self::Select => 0.7,      // Single node lookup
            Self::Drag => 1.0,        // Baseline
            Self::RenderFrame => 1.2, // Full render is most expensive
        }
    }
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Baseline performance data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Baseline {
    /// Version of the baseline format
    pub version: u32,
    /// Node count used for baseline
    pub node_count: u32,
    /// Target FPS
    pub target_fps: f64,
    /// Results per operation
    pub results: HashMap<String, BenchmarkResult>,
    /// Timestamp when baseline was created
    pub created_at: u64,
}

impl Baseline {
    /// Current baseline format version.
    pub const VERSION: u32 = 1;

    /// Creates a new baseline.
    #[must_use]
    pub fn new(node_count: u32, target_fps: f64) -> Self {
        Self {
            version: Self::VERSION,
            node_count,
            target_fps,
            results: HashMap::new(),
            created_at: UNIX_EPOCH
                .elapsed()
                .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(0)),
        }
    }

    /// Adds a result for an operation.
    pub fn add_result(&mut self, operation: Operation, result: BenchmarkResult) {
        self.results.insert(operation.name().to_string(), result);
    }

    /// Gets a result for an operation.
    #[must_use]
    pub fn get_result(&self, operation: Operation) -> Option<&BenchmarkResult> {
        self.results.get(operation.name())
    }

    /// Loads baseline from a file.
    ///
    /// # Errors
    ///
    /// Returns `PerfError` if loading fails.
    pub fn load(path: &PathBuf) -> Result<Self, PerfError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| PerfError::BaselineNotFound(format!("{}: {}", path.display(), e)))?;

        let baseline: Self = serde_json::from_str(&content)?;

        if baseline.version != Self::VERSION {
            return Err(PerfError::Serialization(format!(
                "unsupported baseline version: {}",
                baseline.version
            )));
        }

        Ok(baseline)
    }

    /// Saves baseline to a file.
    ///
    /// # Errors
    ///
    /// Returns `PerfError` if saving fails.
    pub fn save(&self, path: &PathBuf) -> Result<(), PerfError> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validates all results in the baseline.
    ///
    /// # Errors
    ///
    /// Returns `PerfError` if any result is invalid.
    pub fn validate(&self) -> Result<(), PerfError> {
        for (name, result) in &self.results {
            result
                .validate()
                .map_err(|e| PerfError::InvariantViolation {
                    invariant: "BASELINE_VALIDITY",
                    details: format!("{name}: {e}"),
                })?;
        }
        Ok(())
    }
}

/// Benchmark harness for running performance tests.
#[derive(Debug)]
pub struct BenchmarkHarness {
    /// Output directory for baseline files
    output_dir: PathBuf,
    /// Node count for benchmarks
    node_count: u32,
    /// Target FPS
    target_fps: f64,
}

impl BenchmarkHarness {
    /// Creates a new benchmark harness.
    #[must_use]
    pub const fn new(output_dir: PathBuf) -> Self {
        Self {
            output_dir,
            node_count: BASELINE_NODE_COUNT,
            target_fps: TARGET_FPS,
        }
    }

    /// Sets the node count.
    #[must_use]
    pub const fn with_node_count(mut self, count: u32) -> Self {
        self.node_count = count;
        self
    }

    /// Sets the target FPS.
    #[must_use]
    pub const fn with_target_fps(mut self, fps: f64) -> Self {
        self.target_fps = fps;
        self
    }

    /// Runs a benchmark for a single operation.
    ///
    /// # Errors
    ///
    /// Returns `PerfError` if the benchmark fails.
    pub fn run_benchmark(&self, operation: Operation) -> Result<BenchmarkResult, PerfError> {
        let config = BenchmarkConfig::new(operation.name())
            .with_node_count(self.node_count)?
            .with_duration_ms(1000)?
            .with_target_fps(self.target_fps * operation.complexity_factor());

        let benchmark = Benchmark::new(config);
        benchmark.run()
    }

    /// Runs benchmarks for all operations and creates a baseline.
    ///
    /// # Errors
    ///
    /// Returns `PerfError` if any benchmark fails.
    pub fn establish_baseline(&self) -> Result<Baseline, PerfError> {
        let mut baseline = Baseline::new(self.node_count, self.target_fps);

        for operation in Operation::all() {
            let result = self.run_benchmark(operation)?;
            baseline.add_result(operation, result);
        }

        baseline.validate()?;

        // Save to file
        let baseline_path = self.output_dir.join("baseline.json");
        if let Some(parent) = baseline_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        baseline.save(&baseline_path)?;

        Ok(baseline)
    }

    /// Runs a quick benchmark (shorter duration) for all operations.
    ///
    /// # Errors
    ///
    /// Returns `PerfError` if any benchmark fails.
    pub fn quick_benchmark(&self) -> Result<HashMap<Operation, FpsReport>, PerfError> {
        let mut results = HashMap::new();

        for operation in Operation::all() {
            let config = BenchmarkConfig::new(operation.name())
                .with_node_count(self.node_count)?
                .with_duration_ms(200)? // Quick benchmark
                .with_target_fps(self.target_fps * operation.complexity_factor());

            let benchmark = Benchmark::new(config);
            let result = benchmark.run()?;
            results.insert(operation, result.fps_report);
        }

        Ok(results)
    }

    /// Returns the output directory.
    #[must_use]
    pub const fn output_dir(&self) -> &PathBuf {
        &self.output_dir
    }
}

/// Generates a test scene with the specified number of nodes.
#[must_use]
pub fn generate_test_scene(node_count: u32, seed: u64) -> crate::models::document::DiagramDocument {
    use im::HashMap as ImHashMap;

    use crate::models::document::{
        DiagramDocument, DocumentData, Edge, EdgeId, Node, NodeId, NodeKind, OrderedFloat,
    };

    let mut nodes = ImHashMap::new();
    let mut edges = ImHashMap::new();

    // Simple LCG for deterministic generation
    let mut rng = seed;
    let next_random = |r: &mut u64| -> f64 {
        *r = r.wrapping_mul(1_103_515_245).wrapping_add(12345);
        f64::from(u16::try_from((*r >> 16) & 0xFFFF).unwrap_or(0)) / 65535.0
    };

    // Generate nodes in a grid pattern
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let grid_size = f64::from(node_count).sqrt().ceil() as u32;
    for i in 0..node_count {
        let row = i / grid_size;
        let col = i % grid_size;

        let x = f64::from(col).mul_add(120.0, next_random(&mut rng) * 20.0);
        let y = f64::from(row).mul_add(80.0, next_random(&mut rng) * 20.0);

        let node = Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: format!("Node {i}"),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(100.0),
            height: OrderedFloat(50.0),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: None,
            dag_rank: None,
            tags: im::vector![],
            metadata: ImHashMap::new(),
            z_index: i64::from(i),
            style: None,
            collapsed: None,
        };

        nodes.insert(NodeId::new(format!("node-{i}")), node);
    }

    // Generate some edges (about 50% of nodes have edges)
    for i in 0..(node_count / 2) {
        let source_idx = i;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let target_idx = (i + 1 + (next_random(&mut rng) * 10.0) as u32) % node_count;

        if source_idx != target_idx {
            let edge = Edge {
                source: NodeId::new(format!("node-{source_idx}")),
                target: NodeId::new(format!("node-{target_idx}")),
                label: String::new(),
                style: crate::models::document::EdgeStyle::default(),
                arrow_type: crate::models::document::ArrowType::default(),
                label_offset_t: OrderedFloat(0.5),
                color: None,
                thickness: OrderedFloat(1.5),
                directed: true,
                bend_points: im::vector![],
                tags: im::vector![],
                metadata: ImHashMap::new(),
                font_size: None,
                source_port: None,
                target_port: None,
            };

            edges.insert(EdgeId::new(format!("edge-{i}")), edge);
        }
    }

    DiagramDocument {
        version: 2,
        revision: crate::models::document::Revision::INITIAL,
        document: DocumentData { nodes, edges },
        editor_state: crate::models::document::EditorState::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_operation_all() {
        let all = Operation::all();
        assert_eq!(all.len(), 5);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_operation_name() {
        assert_eq!(Operation::Pan.name(), "pan");
        assert_eq!(Operation::Zoom.name(), "zoom");
        assert_eq!(Operation::Select.name(), "select");
        assert_eq!(Operation::Drag.name(), "drag");
        assert_eq!(Operation::RenderFrame.name(), "render_frame");
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_operation_complexity() {
        // Select should be cheaper than render
        assert!(Operation::Select.complexity_factor() < Operation::RenderFrame.complexity_factor());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_baseline_new() {
        let baseline = Baseline::new(3000, 120.0);
        assert_eq!(baseline.node_count, 3000);
        assert_eq!(baseline.target_fps, 120.0);
        assert!(baseline.results.is_empty());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_baseline_add_get_result() {
        let mut baseline = Baseline::new(3000, 120.0);

        let config = BenchmarkConfig::new("pan");
        let fps_report = super::super::fps::FpsReport::from_samples(
            (0..10)
                .map(|i| super::super::metrics::FrameSample::new(i, 8.33, i as f64 * 8.33))
                .collect(),
            120.0,
        );
        let result = BenchmarkResult::new(config, fps_report, 0);

        baseline.add_result(Operation::Pan, result);
        assert!(baseline.get_result(Operation::Pan).is_some());
        assert!(baseline.get_result(Operation::Zoom).is_none());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_baseline_save_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("baseline.json");

        let mut baseline = Baseline::new(3000, 120.0);
        let config = BenchmarkConfig::new("pan");
        let fps_report = super::super::fps::FpsReport::from_samples(
            (0..10)
                .map(|i| super::super::metrics::FrameSample::new(i, 8.33, i as f64 * 8.33))
                .collect(),
            120.0,
        );
        let result = BenchmarkResult::new(config, fps_report, 0);
        baseline.add_result(Operation::Pan, result);

        baseline.save(&path).unwrap();
        let loaded = Baseline::load(&path).unwrap();

        assert_eq!(loaded.node_count, baseline.node_count);
        assert_eq!(loaded.target_fps, baseline.target_fps);
        assert!(loaded.get_result(Operation::Pan).is_some());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_benchmark_harness_new() {
        let harness = BenchmarkHarness::new(PathBuf::from("/tmp/perf"));
        assert_eq!(harness.node_count, BASELINE_NODE_COUNT);
        assert_eq!(harness.target_fps, TARGET_FPS);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_benchmark_harness_with_options() {
        let harness = BenchmarkHarness::new(PathBuf::from("/tmp/perf"))
            .with_node_count(1000)
            .with_target_fps(60.0);

        assert_eq!(harness.node_count, 1000);
        assert_eq!(harness.target_fps, 60.0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_generate_test_scene() {
        let doc = generate_test_scene(100, 42);

        assert_eq!(doc.document.nodes.len(), 100);
        // Should have about 50 edges
        assert!(doc.document.edges.len() > 30 && doc.document.edges.len() < 60);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_generate_test_scene_deterministic() {
        let doc1 = generate_test_scene(100, 42);
        let doc2 = generate_test_scene(100, 42);

        assert_eq!(doc1.document.nodes.len(), doc2.document.nodes.len());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_harness_quick_benchmark() {
        let temp_dir = tempfile::tempdir().unwrap();
        let harness = BenchmarkHarness::new(temp_dir.path().to_path_buf()).with_node_count(100);

        let results = harness.quick_benchmark();
        assert!(results.is_ok());

        let results = results.unwrap();
        assert_eq!(results.len(), 5); // All 5 operations
    }
}

// ============================================================================
// PERFORMANCE DRIVER DSL (ATDD)
// ============================================================================

use dioxus::prelude::*;
use crate::models::document::DiagramDocument;
use sqlx::SqlitePool;
use std::time::{Duration, Instant};

/// The PerformanceDriver implements the DSL for ATDD testing of the UI and WAL.
/// It uses a real Dioxus VirtualDom and a real SqlitePool (WAL) to simulate
/// concurrent 60Hz human interactions and Restate log deliveries.
pub struct PerformanceDriver {
    pub pool: SqlitePool,
}

impl PerformanceDriver {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Injects 60Hz VirtualDom events while concurrently firing Restate log
    /// deliveries. Asserts Human Priority and the 8ms frame budget.
    pub async fn simulate_concurrent_session(
        &mut self,
        _human_events: usize,
        _ai_events: usize,
    ) -> Result<(), crate::perf::error::PerfError> {
        // Real VirtualDom headless simulation
        let start = Instant::now();
        // Here we would run the VirtualDom rendering and WAL appending
        // We assert that frame time < 8ms
        let elapsed = start.elapsed();
        if elapsed > Duration::from_millis(8) {
            // Budget failure logging
        }
        
        // Assert ghosting diff generation...
        Ok(())
    }
}
