//! Performance regression testing infrastructure.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{
    benchmark::BenchmarkResult,
    error::PerfError,
    harness::{Baseline, Operation},
    MIN_ACCEPTABLE_FPS,
};

/// Result of a regression test.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegressionResult {
    /// Operation that was tested
    pub operation: Operation,
    /// Whether the test passed (no significant regression)
    pub passed: bool,
    /// FPS delta from baseline (negative = regression)
    pub delta_fps: f64,
    /// Percentage change from baseline
    pub delta_percent: f64,
    /// Current FPS
    pub current_fps: f64,
    /// Baseline FPS
    pub baseline_fps: f64,
    /// Threshold for regression detection
    pub threshold_fps: f64,
}

impl RegressionResult {
    /// Creates a new regression result.
    #[must_use]
    pub fn new(
        operation: Operation,
        current_fps: f64,
        baseline_fps: f64,
        threshold_fps: f64,
    ) -> Self {
        let delta_fps = current_fps - baseline_fps;
        let delta_percent = if baseline_fps > 0.0 {
            (delta_fps / baseline_fps) * 100.0
        } else {
            0.0
        };
        let passed = delta_fps >= -threshold_fps;

        Self {
            operation,
            passed,
            delta_fps,
            delta_percent,
            current_fps,
            baseline_fps,
            threshold_fps,
        }
    }

    /// Returns a summary string.
    #[must_use]
    pub fn summary(&self) -> String {
        let status = if self.passed { "PASS" } else { "FAIL" };
        format!(
            "{}: {} - current {:.1} FPS, baseline {:.1} FPS, delta {:+.1} FPS ({:+.1}%)",
            status,
            self.operation,
            self.current_fps,
            self.baseline_fps,
            self.delta_fps,
            self.delta_percent
        )
    }
}

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
            threshold_fps: MIN_ACCEPTABLE_FPS - super::TARGET_FPS, // ~20 FPS drop allowed
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
    pub fn run_all(
        &self,
        harness: &super::harness::BenchmarkHarness,
    ) -> Result<Vec<RegressionResult>, PerfError> {
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
            summary.push_str(&format!("  {}\n", result.summary()));
        }

        summary
    }

    /// Returns the baseline.
    #[must_use]
    pub const fn baseline(&self) -> &Baseline {
        &self.baseline
    }
}

/// Performance report for CI integration.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceReport {
    /// Report version
    pub version: u32,
    /// Git commit hash (if available)
    pub commit_hash: Option<String>,
    /// Timestamp of report generation
    pub timestamp_ms: u64,
    /// Regression test results
    pub regression_results: Vec<RegressionResult>,
    /// Whether all tests passed
    pub all_passed: bool,
    /// Machine info (OS, CPU, etc.)
    pub machine_info: MachineInfo,
}

/// Machine information for reproducibility.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineInfo {
    /// Operating system
    pub os: String,
    /// CPU cores
    pub cpu_cores: usize,
    /// Total memory in MB
    pub total_memory_mb: u64,
}

impl MachineInfo {
    /// Gathers current machine information.
    #[must_use]
    pub fn current() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            cpu_cores: num_cpus::get(),
            total_memory_mb: 0, // Would need sys-info crate for this
        }
    }
}

impl Default for MachineInfo {
    fn default() -> Self {
        Self::current()
    }
}

impl PerformanceReport {
    /// Report version.
    pub const VERSION: u32 = 1;

    /// Creates a new performance report.
    #[must_use]
    pub fn new(regression_results: Vec<RegressionResult>) -> Self {
        let all_passed = !RegressionTest::any_regressions(&regression_results);

        Self {
            version: Self::VERSION,
            commit_hash: None,
            timestamp_ms: std::time::UNIX_EPOCH
                .elapsed()
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            regression_results,
            all_passed,
            machine_info: MachineInfo::current(),
        }
    }

    /// Saves the report to a JSON file.
    ///
    /// # Errors
    ///
    /// Returns `PerfError` if saving fails.
    pub fn save(&self, path: &PathBuf) -> Result<(), PerfError> {
        let content = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Loads a report from a JSON file.
    ///
    /// # Errors
    ///
    /// Returns `PerfError` if loading fails.
    pub fn load(path: &PathBuf) -> Result<Self, PerfError> {
        let content = std::fs::read_to_string(path)?;
        let report: Self = serde_json::from_str(&content)?;
        Ok(report)
    }

    /// Returns a markdown summary.
    #[must_use]
    pub fn markdown_summary(&self) -> String {
        let status = if self.all_passed { "PASSED" } else { "FAILED" };

        let mut md = format!("# Performance Report\n\n**Status**: {status}\n\n");

        md.push_str("## Regression Results\n\n");
        md.push_str("| Operation | Current FPS | Baseline FPS | Delta | Status |\n");
        md.push_str("|-----------|-------------|--------------|-------|--------|\n");

        for result in &self.regression_results {
            let status_icon = if result.passed { "OK" } else { "FAIL" };
            md.push_str(&format!(
                "| {} | {:.1} | {:.1} | {:+.1} | {} |\n",
                result.operation,
                result.current_fps,
                result.baseline_fps,
                result.delta_fps,
                status_icon
            ));
        }

        md.push_str(&format!(
            "\n## Machine Info\n\n- OS: {}\n- CPU Cores: {}\n",
            self.machine_info.os, self.machine_info.cpu_cores
        ));

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::perf::{
        benchmark::{BenchmarkConfig, NodeCount},
        fps::FpsReport,
        metrics::FrameSample,
    };

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

    #[test]
    fn test_regression_result_new() {
        let result = RegressionResult::new(Operation::Pan, 100.0, 120.0, 20.0);

        assert!(result.passed); // 20 FPS drop is exactly at threshold
        assert!((result.delta_fps - (-20.0)).abs() < 0.1);
        assert!((result.delta_percent - (-16.67)).abs() < 0.1);
    }

    #[test]
    fn test_regression_result_failed() {
        let result = RegressionResult::new(Operation::Pan, 90.0, 120.0, 20.0);

        assert!(!result.passed); // 30 FPS drop exceeds threshold
    }

    #[test]
    fn test_regression_result_summary() {
        let result = RegressionResult::new(Operation::Pan, 110.0, 120.0, 20.0);
        let summary = result.summary();

        assert!(summary.contains("PASS"));
        assert!(summary.contains("pan"));
        assert!(summary.contains("110"));
        assert!(summary.contains("120"));
    }

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

    #[test]
    fn test_regression_test_unknown_operation() {
        let baseline = make_test_baseline();
        let test = RegressionTest::from_baseline(baseline);

        let result = make_test_result("unknown", 100.0);
        let regression = test.compare(&result);

        assert!(regression.is_err());
    }

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

    #[test]
    fn test_performance_report_new() {
        let results: Vec<RegressionResult> =
            vec![RegressionResult::new(Operation::Pan, 120.0, 120.0, 20.0)];

        let report = PerformanceReport::new(results);
        assert!(report.all_passed);
        assert_eq!(report.regression_results.len(), 1);
    }

    #[test]
    fn test_performance_report_markdown() {
        let results: Vec<RegressionResult> =
            vec![RegressionResult::new(Operation::Pan, 120.0, 120.0, 20.0)];

        let report = PerformanceReport::new(results);
        let md = report.markdown_summary();

        assert!(md.contains("# Performance Report"));
        assert!(md.contains("PASSED"));
        assert!(md.contains("| pan |"));
    }

    #[test]
    fn test_performance_report_save_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("report.json");

        let results: Vec<RegressionResult> =
            vec![RegressionResult::new(Operation::Pan, 120.0, 120.0, 20.0)];

        let report = PerformanceReport::new(results);
        report.save(&path).unwrap();

        let loaded = PerformanceReport::load(&path).unwrap();
        assert_eq!(loaded.all_passed, report.all_passed);
        assert_eq!(
            loaded.regression_results.len(),
            report.regression_results.len()
        );
    }
}
