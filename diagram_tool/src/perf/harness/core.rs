//! Benchmark harness for running performance tests.

use std::{collections::HashMap, path::PathBuf};

use crate::perf::{
    benchmark::{Benchmark, BenchmarkConfig, BenchmarkResult},
    error::PerfError,
    fps::FpsReport,
    BASELINE_NODE_COUNT, TARGET_FPS,
};

use super::{baseline::Baseline, operation::Operation};

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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_harness_quick_benchmark() {
        let temp_dir = tempfile::tempdir().unwrap();
        let harness = BenchmarkHarness::new(temp_dir.path().to_path_buf()).with_node_count(100);

        let results = harness.quick_benchmark();
        assert!(results.is_ok());

        let results = results.unwrap();
        assert_eq!(results.len(), 5); // All 5 operations
    }
}
