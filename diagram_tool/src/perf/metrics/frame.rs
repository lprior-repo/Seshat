use serde::{Deserialize, Serialize};

/// A single frame sample from measurement.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FrameSample {
    /// Frame number in sequence
    pub frame: u64,
    /// Frame time in milliseconds
    pub frame_time_ms: f64,
    /// Timestamp when frame started (milliseconds since benchmark start)
    pub timestamp_ms: f64,
}

impl FrameSample {
    /// Creates a new frame sample.
    #[must_use]
    pub const fn new(frame: u64, frame_time_ms: f64, timestamp_ms: f64) -> Self {
        Self {
            frame,
            frame_time_ms,
            timestamp_ms,
        }
    }

    /// Returns the FPS for this frame.
    #[must_use]
    pub fn fps(&self) -> f64 {
        if self.frame_time_ms > 0.0 {
            1000.0 / self.frame_time_ms
        } else {
            0.0
        }
    }

    /// Validates this sample (INV-1: no NaN/Infinity).
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.frame_time_ms.is_finite() && self.timestamp_ms.is_finite()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_frame_sample_fps() {
        let sample = FrameSample::new(1, 8.33, 0.0);
        let fps = sample.fps();
        assert!((fps - 120.0).abs() < 0.5);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_frame_sample_is_valid() {
        let valid = FrameSample::new(1, 8.33, 0.0);
        assert!(valid.is_valid());

        let invalid = FrameSample::new(1, f64::NAN, 0.0);
        assert!(!invalid.is_valid());
    }
}
