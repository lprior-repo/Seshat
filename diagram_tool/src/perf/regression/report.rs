use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::perf::error::PerfError;

use super::{result::RegressionResult, test::RegressionTest};

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

#[allow(dead_code)]
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
            #[allow(clippy::map_unwrap_or, clippy::cast_possible_truncation)]
            timestamp_ms: std::time::UNIX_EPOCH
                .elapsed()
                // Cast u128 to u64 - would need ~340M years to overflow, truncation is acceptable
                .map(|d| d.as_millis() as u64)
                .unwrap_or_else(|_| 0),
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
        use std::fmt::Write;
        let status = if self.all_passed { "PASSED" } else { "FAILED" };

        let mut md = format!("# Performance Report\n\n**Status**: {status}\n\n");

        md.push_str("## Regression Results\n\n");
        md.push_str("| Operation | Current FPS | Baseline FPS | Delta | Status |\n");
        md.push_str("|-----------|-------------|--------------|-------|--------|\n");

        for result in &self.regression_results {
            let status_icon = if result.passed { "OK" } else { "FAIL" };
            let _ = writeln!(
                md,
                "| {} | {:.1} | {:.1} | {:+.1} | {} |",
                result.operation,
                result.current_fps,
                result.baseline_fps,
                result.delta_fps,
                status_icon
            );
        }

        let _ = writeln!(
            md,
            "\n## Machine Info\n\n- OS: {}\n- CPU Cores: {}",
            self.machine_info.os, self.machine_info.cpu_cores
        );

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::perf::Operation;

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_performance_report_new() {
        let results: Vec<RegressionResult> =
            vec![RegressionResult::new(Operation::Pan, 120.0, 120.0, 20.0)];

        let report = PerformanceReport::new(results);
        assert!(report.all_passed);
        assert_eq!(report.regression_results.len(), 1);
    }

    #[cfg(kani)]
    #[kani::proof]
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

    #[cfg(kani)]
    #[kani::proof]
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
