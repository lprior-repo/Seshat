use std::time::{Duration, Instant};

use crate::perf::{error::PerfError, metrics::FrameSample};

use super::report::FpsReport;

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
        self.stop_with_min_samples(target_fps, 10)
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

    #[cfg(kani)]
    #[kani::proof]
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

    #[cfg(kani)]
    #[kani::proof]
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

    #[cfg(kani)]
    #[kani::proof]
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

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_frame_sample_fps_calculation() {
        let sample = FrameSample::new(0, 16.67, 0.0);
        assert!((sample.fps() - 60.0).abs() < 0.1);

        let sample = FrameSample::new(0, 8.33, 0.0);
        assert!((sample.fps() - 120.0).abs() < 0.5);
    }
}
