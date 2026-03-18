use serde::{Deserialize, Serialize};

use crate::perf::{
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

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(kani)]
    #[kani::proof]
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

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_fps_report_validate_success() {
        let samples: Vec<FrameSample> = (0..10)
            .map(|i| FrameSample::new(i, 8.33, i as f64 * 8.33))
            .collect();

        let report = FpsReport::from_samples(samples, 120.0);
        assert!(report.validate().is_ok());
    }

    #[cfg(kani)]
    #[kani::proof]
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
}
