//! Benchmark configuration and execution.

use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::error::PerfError;
use super::fps::FpsReport;
use super::{
    DEFAULT_BENCHMARK_DURATION_MS, DEFAULT_WARMUP_ITERATIONS, MAX_NODE_COUNT, MIN_DURATION_MS,
    MIN_NODE_COUNT, TARGET_FPS,
};

/// Validated node count (P1: 1-10000).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeCount(u32);

impl NodeCount {
    /// Creates a new validated node count.
    ///
    /// # Errors
    ///
    /// Returns `PerfError::InvalidNodeCount` if count is outside [1, 10000].
    pub fn new(count: u32) -> Result<Self, PerfError> {
        if count < MIN_NODE_COUNT || count > MAX_NODE_COUNT {
            return Err(PerfError::InvalidNodeCount(count));
        }
        Ok(Self(count))
    }

    /// Returns the raw count value.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

impl Default for NodeCount {
    fn default() -> Self {
        Self(super::BASELINE_NODE_COUNT)
    }
}

/// Validated benchmark duration (P2: >= 100ms).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurationMs(u64);

impl DurationMs {
    /// Creates a new validated duration.
    ///
    /// # Errors
    ///
    /// Returns `PerfError::InvalidDuration` if duration < 100ms.
    pub fn new(ms: u64) -> Result<Self, PerfError> {
        if ms < MIN_DURATION_MS {
            return Err(PerfError::InvalidDuration(ms));
        }
        Ok(Self(ms))
    }

    /// Returns the raw duration value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }

    /// Converts to a `std::time::Duration`.
    #[must_use]
    pub const fn to_duration(self) -> Duration {
        Duration::from_millis(self.0)
    }
}

impl Default for DurationMs {
    fn default() -> Self {
        Self(DEFAULT_BENCHMARK_DURATION_MS)
    }
}

/// Warm-up configuration (P3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WarmupConfig {
    /// Number of warm-up iterations
    pub iterations: u32,
    /// Duration of each warm-up iteration in ms
    pub duration_ms: u64,
}

impl WarmupConfig {
    /// Creates a new warm-up configuration.
    #[must_use]
    pub const fn new(iterations: u32, duration_ms: u64) -> Self {
        Self {
            iterations,
            duration_ms,
        }
    }

    /// Returns whether warm-up is complete.
    #[must_use]
    pub const fn is_complete(self, completed_iterations: u32) -> bool {
        completed_iterations >= self.iterations
    }
}

impl Default for WarmupConfig {
    fn default() -> Self {
        Self::new(DEFAULT_WARMUP_ITERATIONS, 500)
    }
}

/// Benchmark configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Number of nodes in the test scene
    pub node_count: NodeCount,
    /// Benchmark duration in milliseconds
    pub duration_ms: DurationMs,
    /// Warm-up configuration
    pub warmup: WarmupConfig,
    /// Target FPS
    pub target_fps: f64,
    /// Random seed for reproducibility
    pub seed: u64,
    /// Operation being benchmarked
    pub operation: String,
}

impl BenchmarkConfig {
    /// Creates a new benchmark configuration with defaults.
    #[must_use]
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            node_count: NodeCount::default(),
            duration_ms: DurationMs::default(),
            warmup: WarmupConfig::default(),
            target_fps: TARGET_FPS,
            seed: 42,
            operation: operation.into(),
        }
    }

    /// Sets the node count.
    ///
    /// # Errors
    ///
    /// Returns `PerfError::InvalidNodeCount` if count is invalid.
    pub fn with_node_count(mut self, count: u32) -> Result<Self, PerfError> {
        self.node_count = NodeCount::new(count)?;
        Ok(self)
    }

    /// Sets the duration.
    ///
    /// # Errors
    ///
    /// Returns `PerfError::InvalidDuration` if duration is invalid.
    pub fn with_duration_ms(mut self, ms: u64) -> Result<Self, PerfError> {
        self.duration_ms = DurationMs::new(ms)?;
        Ok(self)
    }

    /// Sets the target FPS.
    #[must_use]
    pub fn with_target_fps(mut self, fps: f64) -> Self {
        self.target_fps = fps;
        self
    }

    /// Sets the random seed.
    #[must_use]
    pub const fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Sets the warm-up configuration.
    #[must_use]
    pub const fn with_warmup(mut self, warmup: WarmupConfig) -> Self {
        self.warmup = warmup;
        self
    }

    /// Validates the configuration.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.target_fps > 0.0 && self.target_fps.is_finite()
    }
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

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

/// Benchmark runner.
#[derive(Debug)]
pub struct Benchmark {
    config: BenchmarkConfig,
}

impl Benchmark {
    /// Creates a new benchmark with the given configuration.
    #[must_use]
    pub const fn new(config: BenchmarkConfig) -> Self {
        Self { config }
    }

    /// Returns the configuration.
    #[must_use]
    pub const fn config(&self) -> &BenchmarkConfig {
        &self.config
    }

    /// Simulates a frame for the configured operation.
    /// This is a placeholder that simulates work based on node count.
    fn simulate_frame(&self) -> f64 {
        // Base frame time (target 8.33ms for 120 FPS)
        let base_time_ms = 1000.0 / self.config.target_fps;

        // Add simulated work proportional to node count
        // For 3000 nodes at 120 FPS target, this should complete in ~8.33ms
        let node_factor = self.config.node_count.value() as f64 / 3000.0;
        let work_time_ms = base_time_ms * node_factor;

        // Add small variance to simulate real-world conditions
        let variance = (self.config.seed as f64 % 0.5) - 0.25; // -0.25 to +0.25 ms

        work_time_ms + variance
    }

    /// Runs the benchmark and returns the result.
    ///
    /// # Errors
    ///
    /// Returns `PerfError` if the benchmark fails.
    pub fn run(&self) -> Result<BenchmarkResult, PerfError> {
        if !self.config.is_valid() {
            return Err(PerfError::MeasurementFailed(
                "invalid benchmark configuration".to_string(),
            ));
        }

        use super::fps::FpsMeasurement;
        use std::time::{Instant, UNIX_EPOCH};

        let mut measurement = FpsMeasurement::new();

        // Warm-up phase (P3)
        for _ in 0..self.config.warmup.iterations {
            let warmup_start = Instant::now();
            let warmup_duration = Duration::from_millis(self.config.warmup.duration_ms);
            while warmup_start.elapsed() < warmup_duration {
                let _ = self.simulate_frame();
            }
        }

        // Measurement phase
        measurement.start();
        let benchmark_start = Instant::now();
        let duration = self.config.duration_ms.to_duration();

        while benchmark_start.elapsed() < duration {
            let _ = self.simulate_frame();
            measurement.record_frame();
        }

        let timestamp_ms = UNIX_EPOCH
            .elapsed()
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let fps_report = measurement.stop(self.config.target_fps)?;
        let result = BenchmarkResult::new(self.config.clone(), fps_report, timestamp_ms);
        result.validate()?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_count_valid() {
        assert!(NodeCount::new(1).is_ok());
        assert!(NodeCount::new(3000).is_ok());
        assert!(NodeCount::new(10000).is_ok());
    }

    #[test]
    fn test_node_count_invalid() {
        assert!(NodeCount::new(0).is_err());
        assert!(NodeCount::new(10001).is_err());
    }

    #[test]
    fn test_duration_ms_valid() {
        assert!(DurationMs::new(100).is_ok());
        assert!(DurationMs::new(5000).is_ok());
    }

    #[test]
    fn test_duration_ms_invalid() {
        assert!(DurationMs::new(0).is_err());
        assert!(DurationMs::new(50).is_err());
        assert!(DurationMs::new(99).is_err());
    }

    #[test]
    fn test_benchmark_config_builder() {
        let config = BenchmarkConfig::new("pan")
            .with_node_count(3000)
            .unwrap()
            .with_duration_ms(1000)
            .unwrap()
            .with_target_fps(120.0)
            .with_seed(12345);

        assert_eq!(config.node_count.value(), 3000);
        assert_eq!(config.duration_ms.value(), 1000);
        assert_eq!(config.target_fps, 120.0);
        assert_eq!(config.seed, 12345);
        assert_eq!(config.operation, "pan");
    }

    #[test]
    fn test_warmup_config() {
        let warmup = WarmupConfig::new(5, 1000);
        assert_eq!(warmup.iterations, 5);
        assert_eq!(warmup.duration_ms, 1000);
        assert!(warmup.is_complete(5));
        assert!(warmup.is_complete(6));
        assert!(!warmup.is_complete(4));
    }

    #[test]
    fn test_benchmark_run() {
        let config = BenchmarkConfig::new("test")
            .with_node_count(100)
            .unwrap()
            .with_duration_ms(200)
            .unwrap();

        let benchmark = Benchmark::new(config);
        let result = benchmark.run();

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.fps_report.sample_count > 0);
        assert!(result.validate().is_ok());
    }

    #[test]
    fn test_benchmark_result_is_regression() {
        let mut config = BenchmarkConfig::new("test");
        config.target_fps = 100.0;

        let fps_report = super::super::fps::FpsReport::from_samples(
            (0..10)
                .map(|i| super::super::metrics::FrameSample::new(i, 10.0, i as f64 * 10.0))
                .collect(),
            100.0,
        );

        let result = BenchmarkResult::new(config.clone(), fps_report.clone(), 0);

        // At 10ms per frame, FPS is ~100, so delta should be ~0
        assert!(!result.is_regression(10.0));
    }

    #[test]
    fn test_benchmark_config_is_valid() {
        let valid_config = BenchmarkConfig::new("test").with_target_fps(120.0);
        assert!(valid_config.is_valid());

        let invalid_config = BenchmarkConfig::new("test").with_target_fps(0.0);
        assert!(!invalid_config.is_valid());

        let nan_config = BenchmarkConfig::new("test").with_target_fps(f64::NAN);
        assert!(!nan_config.is_valid());
    }
}
