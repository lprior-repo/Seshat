//! Benchmark execution.

use std::time::{Duration, Instant, UNIX_EPOCH};

use super::{config::BenchmarkConfig, result::BenchmarkResult};
use crate::perf::{error::PerfError, fps::FpsMeasurement};

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
        let node_factor = f64::from(self.config.node_count.value()) / 3000.0;
        let work_time_ms = base_time_ms * node_factor;

        // Add small variance to simulate real-world conditions
        #[allow(clippy::cast_precision_loss)]
        let variance = (self.config.seed as f64 % 0.5) - 0.25; // -0.25 to +0.25 ms

        let total_ms = work_time_ms + variance;
        if total_ms > 0.0 {
            std::thread::sleep(std::time::Duration::from_secs_f64(total_ms / 1000.0));
        }

        total_ms
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

        #[allow(clippy::map_unwrap_or, clippy::cast_possible_truncation)]
        let timestamp_ms = UNIX_EPOCH
            .elapsed()
            // Cast u128 to u64 - would need ~340M years to overflow, truncation is acceptable
            .map(|d| d.as_millis() as u64)
            .unwrap_or_else(|_| 0);

        let fps_report = measurement.stop(self.config.target_fps)?;
        let result = BenchmarkResult::new(self.config.clone(), fps_report, timestamp_ms);
        result.validate()?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(kani)]
    #[kani::proof]
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
}
