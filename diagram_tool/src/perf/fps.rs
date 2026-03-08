//! FPS measurement utilities.

use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use super::{
    error::PerfError,
    metrics::{FrameSample, Statistics},
};

/// FPS measurement result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FpsReport {
    /// Mean FPS across all samples
    pub mean_fps: f64,
    /// Standard deviation of FPS
    pub std_dev_fps: f64,
    /// Number of samples collected
    pub sample_count: usize,
    /// Statistics for frame time (ms)
    pub frame_time_stats: Statistics,
    /// Statistics for FPS
    pub fps_stats: Statistics,
    /// Raw frame samples
    pub samples: Vec<FrameSample>,
    /// Total measurement duration in milliseconds
    pub duration_ms: f64,
    /// Whether target FPS was achieved
    pub target_achieved: bool,
}

impl FpsReport {
    /// Creates a new FPS report from samples.
    #[must_use]
    pub fn from_samples(samples: Vec<FrameSample>, target_fps: f64) -> Self {
        let frame_times: Vec<f64> = samples.iter().map(|s| s.frame_time_ms).collect();
        let fps_values: Vec<f64> = samples.iter().map(FrameSample::fps).collect();

        let frame_time_stats = Statistics::from_samples(&frame_times);
        let fps_stats = Statistics::from_samples(&fps_values);

        let duration_ms = samples.last().map_or(0.0, |s| s.timestamp_ms);

        let target_achieved = fps_stats.mean >= target_fps;

        Self {
            mean_fps: fps_stats.mean,
            std_dev_fps: fps_stats.std_dev,
            sample_count: samples.len(),
            frame_time_stats,
            fps_stats,
            samples,
            duration_ms,
            target_achieved,
        }
    }

    /// Validates all invariants.
    ///
    /// # Errors
    ///
    /// Returns `PerfError::InvariantViolation` if any invariant is broken.
    pub fn validate(&self) -> Result<(), PerfError> {
        // INV-1: No NaN/Infinity in measurements
        if !self.mean_fps.is_finite() {
            return Err(PerfError::invariant_violation(
                "INV-1",
                format!("mean_fps is not finite: {}", self.mean_fps),
            ));
        }
        if !self.std_dev_fps.is_finite() {
            return Err(PerfError::invariant_violation(
                "INV-1",
                format!("std_dev_fps is not finite: {}", self.std_dev_fps),
            ));
        }

        // INV-4: Sample count matches
        if self.sample_count != self.samples.len() {
            return Err(PerfError::invariant_violation(
                "INV-4",
                format!(
                    "sample_count {} != samples.len() {}",
                    self.sample_count,
                    self.samples.len()
                ),
            ));
        }

        // INV-5: Percentiles ordered (via Statistics validation)
        if !self.fps_stats.is_valid() {
            return Err(PerfError::invariant_violation(
                "INV-5",
                "fps_stats validation failed".to_string(),
            ));
        }
        if !self.frame_time_stats.is_valid() {
            return Err(PerfError::invariant_violation(
                "INV-5",
                "frame_time_stats validation failed".to_string(),
            ));
        }

        // INV-2: Monotonic timestamps
        for window in self.samples.windows(2) {
            if window[0].timestamp_ms > window[1].timestamp_ms {
                return Err(PerfError::invariant_violation(
                    "INV-2",
                    format!(
                        "non-monotonic timestamps: {} > {}",
                        window[0].timestamp_ms, window[1].timestamp_ms
                    ),
                ));
            }
        }

        // INV-3: Frame time and FPS consistency
        for sample in &self.samples {
            let expected_fps = if sample.frame_time_ms > 0.0 {
                1000.0 / sample.frame_time_ms
            } else {
                0.0
            };
            if (sample.fps() - expected_fps).abs() > 0.01 {
                return Err(PerfError::invariant_violation(
                    "INV-3",
                    format!(
                        "frame time {}ms gives {} fps, but stored as {}",
                        sample.frame_time_ms,
                        expected_fps,
                        sample.fps()
                    ),
                ));
            }
        }

        Ok(())
    }
}

/// FPS measurement utility.
#[derive(Debug)]
pub struct FpsMeasurement {
    samples: Vec<FrameSample>,
    start_time: Option<Instant>,
    last_frame_time: Option<Instant>,
    frame_count: u64,
}

impl FpsMeasurement {
    /// Creates a new FPS measurement context.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            samples: Vec::new(),
            start_time: None,
            last_frame_time: None,
            frame_count: 0,
        }
    }

    /// Starts measurement.
    pub fn start(&mut self) {
        self.samples.clear();
        self.frame_count = 0;
        self.start_time = Some(Instant::now());
        self.last_frame_time = self.start_time;
    }

    /// Records a frame.
    pub fn record_frame(&mut self) {
        let now = Instant::now();

        let (frame_time_ms, timestamp_ms) =
            if let (Some(start), Some(last)) = (self.start_time, self.last_frame_time) {
                #[allow(clippy::cast_precision_loss)]
                let frame_time = now.duration_since(last).as_nanos() as f64 / 1_000_000.0;
                #[allow(clippy::cast_precision_loss)]
                let timestamp = now.duration_since(start).as_nanos() as f64 / 1_000_000.0;
                (frame_time, timestamp)
            } else {
                (0.0, 0.0)
            };

        self.samples.push(FrameSample::new(
            self.frame_count,
            frame_time_ms,
            timestamp_ms,
        ));
        self.last_frame_time = Some(now);
        self.frame_count += 1;
    }

    /// Stops measurement and returns the report.
    ///
    /// # Errors
    ///
    /// Returns `PerfError::InsufficientSamples` if fewer than 10 samples collected.
    pub fn stop(self, target_fps: f64) -> Result<FpsReport, PerfError> {
        if self.samples.len() < 10 {
            return Err(PerfError::InsufficientSamples {
                got: self.samples.len(),
                need: 10,
            });
        }

        let report = FpsReport::from_samples(self.samples, target_fps);
        report.validate()?;
        Ok(report)
    }

    /// Stops measurement with a minimum sample requirement.
    ///
    /// # Errors
    ///
    /// Returns `PerfError::InsufficientSamples` if fewer than `min_samples` collected.
    pub fn stop_with_min_samples(
        self,
        target_fps: f64,
        min_samples: usize,
    ) -> Result<FpsReport, PerfError> {
        if self.samples.len() < min_samples {
            return Err(PerfError::InsufficientSamples {
                got: self.samples.len(),
                need: min_samples,
            });
        }

        let report = FpsReport::from_samples(self.samples, target_fps);
        report.validate()?;
        Ok(report)
    }

    /// Returns the current sample count.
    #[must_use]
    pub const fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Returns the elapsed time since start.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.start_time.map_or(Duration::ZERO, |s| s.elapsed())
    }
}

impl Default for FpsMeasurement {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fps_report_from_samples() {
        let samples: Vec<FrameSample> = (0..100)
            .map(|i| FrameSample::new(i, 8.33, i as f64 * 8.33))
            .collect();

        let report = FpsReport::from_samples(samples, 120.0);

        assert_eq!(report.sample_count, 100);
        assert!(report.mean_fps > 0.0);
        assert!(report.std_dev_fps >= 0.0);
    }

    #[test]
    fn test_fps_report_validate_success() {
        let samples: Vec<FrameSample> = (0..10)
            .map(|i| FrameSample::new(i, 8.33, i as f64 * 8.33))
            .collect();

        let report = FpsReport::from_samples(samples, 120.0);
        assert!(report.validate().is_ok());
    }

    #[test]
    fn test_fps_report_validate_sample_count_mismatch() {
        let mut report = FpsReport::from_samples(
            (0..10)
                .map(|i| FrameSample::new(i, 8.33, i as f64 * 8.33))
                .collect(),
            120.0,
        );
        report.sample_count = 5; // Mismatch

        let result = report.validate();
        assert!(result.is_err());
        if let Err(PerfError::InvariantViolation { invariant, .. }) = result {
            assert_eq!(invariant, "INV-4");
        } else {
            panic!("Expected INV-4 violation");
        }
    }

    #[test]
    fn test_fps_measurement_basic() {
        let mut measurement = FpsMeasurement::new();
        measurement.start();

        for _ in 0..15 {
            measurement.record_frame();
        }

        let report = measurement.stop(120.0);
        assert!(report.is_ok());
        let report = report.unwrap();
        assert!(report.sample_count >= 10);
    }

    #[test]
    fn test_fps_measurement_insufficient_samples() {
        let mut measurement = FpsMeasurement::new();
        measurement.start();

        for _ in 0..5 {
            measurement.record_frame();
        }

        let result = measurement.stop(120.0);
        assert!(result.is_err());
        if let Err(PerfError::InsufficientSamples { got, need }) = result {
            assert_eq!(got, 5);
            assert_eq!(need, 10);
        } else {
            panic!("Expected InsufficientSamples error");
        }
    }

    #[test]
    fn test_fps_measurement_custom_min_samples() {
        let mut measurement = FpsMeasurement::new();
        measurement.start();

        for _ in 0..15 {
            measurement.record_frame();
        }

        let result = measurement.stop_with_min_samples(120.0, 20);
        assert!(result.is_err());
        if let Err(PerfError::InsufficientSamples { got, need }) = result {
            assert_eq!(got, 15);
            assert_eq!(need, 20);
        } else {
            panic!("Expected InsufficientSamples error");
        }
    }

    #[test]
    fn test_frame_sample_fps_calculation() {
        let sample = FrameSample::new(0, 16.67, 0.0);
        assert!((sample.fps() - 60.0).abs() < 0.1);

        let sample = FrameSample::new(0, 8.33, 0.0);
        assert!((sample.fps() - 120.0).abs() < 0.5);
    }
}
