#![allow(clippy::unwrap_used)]
use std::path::PathBuf;

use crate::perf::{
    benchmark::BenchmarkResult, error::PerfError, harness::BenchmarkHarness, Baseline, Operation,
    MIN_ACCEPTABLE_FPS, TARGET_FPS,
};

use super::result::RegressionResult;

/// Regression test runner.
#[derive(Debug)]
pub struct RegressionTest {
    /// Baseline to compare against
    baseline: Baseline,
    /// Threshold for regression detection (FPS drop)
    threshold_fps: f64,
}

impl RegressionTest {
    /// Creates a new regression test from a baseline file.
    ///
    /// # Errors
    ///
    /// Returns `PerfError::BaselineNotFound` if the file doesn't exist.
    pub fn from_file(path: &PathBuf) -> Result<Self, PerfError> {
        let baseline = Baseline::load(path)?;
        Ok(Self {
            baseline,
            threshold_fps: MIN_ACCEPTABLE_FPS - TARGET_FPS, // ~20 FPS drop allowed
        })
    }

    /// Creates a new regression test from an existing baseline.
    #[must_use]
    pub const fn from_baseline(baseline: Baseline) -> Self {
        Self {
            baseline,
            threshold_fps: 20.0, // Allow up to 20 FPS drop
        }
    }

    /// Sets the regression threshold.
    #[must_use]
    pub const fn with_threshold(mut self, threshold_fps: f64) -> Self {
        self.threshold_fps = threshold_fps;
        self
    }

    /// Compares a benchmark result against the baseline.
    ///
    /// # Errors
    ///
    /// Returns `PerfError::BaselineNotFound` if the operation is not in the baseline.
    pub fn compare(&self, result: &BenchmarkResult) -> Result<RegressionResult, PerfError> {
        let operation_name = result.config.operation.as_str();

        // Parse operation name to Operation enum
        let operation = match operation_name {
            "pan" => Operation::Pan,
            "zoom" => Operation::Zoom,
            "select" => Operation::Select,
            "drag" => Operation::Drag,
            "render_frame" => Operation::RenderFrame,
            _ => {
                return Err(PerfError::BaselineNotFound(format!(
                    "unknown operation: {operation_name}"
                )))
            }
        };

        let baseline_result = self
            .baseline
            .get_result(operation)
            .ok_or_else(|| PerfError::BaselineNotFound(format!("operation: {operation_name}")))?;

        let baseline_fps = baseline_result.fps_report.mean_fps;
        let current_fps = result.fps_report.mean_fps;

        Ok(RegressionResult::new(
            operation,
            current_fps,
            baseline_fps,
            self.threshold_fps,
        ))
    }

    /// Runs regression tests for all operations in the baseline.
    ///
    /// # Errors
    ///
    /// Returns `PerfError` if any test fails to run.
    pub fn run_all(&self, harness: &BenchmarkHarness) -> Result<Vec<RegressionResult>, PerfError> {
        let mut results = Vec::new();

        for operation in Operation::all() {
            if self.baseline.get_result(operation).is_none() {
                continue;
            }

            let benchmark_result = harness.run_benchmark(operation)?;
            let regression_result = self.compare(&benchmark_result)?;
            results.push(regression_result);
        }

        Ok(results)
    }

    /// Checks if any operations regressed.
    #[must_use]
    pub fn any_regressions(results: &[RegressionResult]) -> bool {
        results.iter().any(|r| !r.passed)
    }

    /// Returns a summary of all results.
    #[must_use]
    pub fn summarize_results(results: &[RegressionResult]) -> String {
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = results.len() - passed;

        let mut summary = format!("Regression Test Summary: {passed} passed, {failed} failed\n");

        for result in results {
            summary.push_str(&result.summary());
            summary.push('\n');
        }

        summary
    }

    /// Returns the baseline.
    #[must_use]
    pub const fn baseline(&self) -> &Baseline {
        &self.baseline
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::perf::{benchmark::BenchmarkConfig, fps::FpsReport, metrics::FrameSample};

    fn make_test_result(operation: &str, fps: f64) -> BenchmarkResult {
        let config = BenchmarkConfig::new(operation)
            .with_node_count(3000)
            .unwrap();

        let samples: Vec<FrameSample> = (0..10)
            .map(|i| FrameSample::new(i, 1000.0 / fps, i as f64 * (1000.0 / fps)))
            .collect();

        let fps_report = FpsReport::from_samples(samples, fps);
        BenchmarkResult::new(config, fps_report, 0)
    }

    fn make_test_baseline() -> Baseline {
        let mut baseline = Baseline::new(3000, 120.0);

        baseline.add_result(Operation::Pan, make_test_result("pan", 120.0));
        baseline.add_result(Operation::Zoom, make_test_result("zoom", 115.0));
        baseline.add_result(Operation::Select, make_test_result("select", 125.0));
        baseline.add_result(Operation::Drag, make_test_result("drag", 118.0));
        baseline.add_result(
            Operation::RenderFrame,
            make_test_result("render_frame", 110.0),
        );

        baseline
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_regression_test_from_baseline() {
        let baseline = make_test_baseline();
        let test = RegressionTest::from_baseline(baseline);

        let result = make_test_result("pan", 100.0);
        let regression = test.compare(&result);

        assert!(regression.is_ok());
        let regression = regression.unwrap();
        assert!(regression.passed); // 20 FPS drop is at threshold
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_regression_test_unknown_operation() {
        let baseline = make_test_baseline();
        let test = RegressionTest::from_baseline(baseline);

        let result = make_test_result("unknown", 100.0);
        let regression = test.compare(&result);

        assert!(regression.is_err());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_any_regressions() {
        let passing: Vec<RegressionResult> = vec![
            RegressionResult::new(Operation::Pan, 120.0, 120.0, 20.0),
            RegressionResult::new(Operation::Zoom, 115.0, 120.0, 20.0),
        ];
        assert!(!RegressionTest::any_regressions(&passing));

        let failing: Vec<RegressionResult> = vec![
            RegressionResult::new(Operation::Pan, 120.0, 120.0, 20.0),
            RegressionResult::new(Operation::Zoom, 90.0, 120.0, 20.0), // 30 FPS drop
        ];
        assert!(RegressionTest::any_regressions(&failing));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_summarize_results() {
        let results: Vec<RegressionResult> = vec![
            RegressionResult::new(Operation::Pan, 120.0, 120.0, 20.0),
            RegressionResult::new(Operation::Zoom, 115.0, 120.0, 20.0),
        ];

        let summary = RegressionTest::summarize_results(&results);
        assert!(summary.contains("2 passed"));
        assert!(summary.contains("0 failed"));
    }
}
