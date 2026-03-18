//! Benchmark result types.

use serde::{Deserialize, Serialize};

use super::config::BenchmarkConfig;
use crate::perf::{error::PerfError, fps::FpsReport};

/// Result of a benchmark run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Configuration used for this benchmark
    pub config: BenchmarkConfig,
    /// FPS report
    pub fps_report: FpsReport,
    /// Whether the benchmark passed (met target)
    pub passed: bool,
    /// Delta from target FPS
    pub delta_fps: f64,
    /// Timestamp when benchmark was run
    pub timestamp_ms: u64,
}

impl BenchmarkResult {
    /// Creates a new benchmark result.
    #[must_use]
    pub fn new(config: BenchmarkConfig, fps_report: FpsReport, timestamp_ms: u64) -> Self {
        let delta_fps = fps_report.mean_fps - config.target_fps;
        let passed = fps_report.target_achieved;

        Self {
            config,
            fps_report,
            passed,
            delta_fps,
            timestamp_ms,
        }
    }

    /// Validates the result.
    ///
    /// # Errors
    ///
    /// Returns `PerfError` if validation fails.
    pub fn validate(&self) -> Result<(), PerfError> {
        self.fps_report.validate()?;

        if !self.delta_fps.is_finite() {
            return Err(PerfError::InvariantViolation {
                invariant: "INV-1",
                details: format!("delta_fps is not finite: {}", self.delta_fps),
            });
        }

        Ok(())
    }

    /// Returns whether this result represents a regression from a baseline.
    #[must_use]
    pub fn is_regression(&self, threshold_fps: f64) -> bool {
        self.delta_fps < -threshold_fps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::perf::metrics::FrameSample;

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_benchmark_result_is_regression() {
        let mut config = BenchmarkConfig::new("test");
        config.target_fps = 100.0;

        let fps_report = FpsReport::from_samples(
            (0..10)
                .map(|i| FrameSample::new(i, 10.0, i as f64 * 10.0))
                .collect(),
            100.0,
        );

        let result = BenchmarkResult::new(config.clone(), fps_report.clone(), 0);

        // At 10ms per frame, FPS is ~100, so delta should be ~0
        assert!(!result.is_regression(10.0));
    }
}
