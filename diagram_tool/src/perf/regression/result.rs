use serde::{Deserialize, Serialize};

use crate::perf::Operation;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_regression_result_new() {
        let result = RegressionResult::new(Operation::Pan, 100.0, 120.0, 20.0);

        assert!(result.passed); // 20 FPS drop is exactly at threshold
        assert!((result.delta_fps - (-20.0)).abs() < 0.1);
        assert!((result.delta_percent - (-16.67)).abs() < 0.1);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_regression_result_failed() {
        let result = RegressionResult::new(Operation::Pan, 90.0, 120.0, 20.0);

        assert!(!result.passed); // 30 FPS drop exceeds threshold
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_regression_result_summary() {
        let result = RegressionResult::new(Operation::Pan, 110.0, 120.0, 20.0);
        let summary = result.summary();

        assert!(summary.contains("PASS"));
        assert!(summary.contains("pan"));
        assert!(summary.contains("110"));
        assert!(summary.contains("120"));
    }
}
