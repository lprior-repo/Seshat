//! Metrics and statistics for performance measurement.

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

/// Percentile statistics for measurements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Percentiles {
    /// 50th percentile (median)
    pub p50: f64,
    /// 90th percentile
    pub p90: f64,
    /// 95th percentile
    pub p95: f64,
    /// 99th percentile
    pub p99: f64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
}

impl Percentiles {
    /// Creates percentile statistics from sorted samples.
    #[must_use]
    pub fn from_sorted(sorted: &[f64]) -> Self {
        if sorted.is_empty() {
            return Self {
                p50: 0.0,
                p90: 0.0,
                p95: 0.0,
                p99: 0.0,
                min: 0.0,
                max: 0.0,
            };
        }

        let len = sorted.len();
        let percentile = |p: f64| -> f64 {
            #[allow(clippy::cast_precision_loss)]
            let idx = ((len - 1) as f64 * p).round() as usize;
            sorted[idx.min(len - 1)]
        };

        Self {
            p50: percentile(0.50),
            p90: percentile(0.90),
            p95: percentile(0.95),
            p99: percentile(0.99),
            min: sorted[0],
            max: sorted[len - 1],
        }
    }

    /// Validates that percentiles are ordered (INV-5).
    #[must_use]
    pub const fn is_ordered(&self) -> bool {
        self.p50 <= self.p90
            && self.p90 <= self.p95
            && self.p95 <= self.p99
            && self.min <= self.p50
            && self.p99 <= self.max
    }
}

/// Statistical summary of measurements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Statistics {
    /// Number of samples
    pub count: usize,
    /// Mean value
    pub mean: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Variance
    pub variance: f64,
    /// Percentile breakdowns
    pub percentiles: Percentiles,
    /// 95% confidence interval (lower bound)
    pub ci95_lower: f64,
    /// 95% confidence interval (upper bound)
    pub ci95_upper: f64,
}

impl Statistics {
    /// Computes statistics from raw samples.
    #[must_use]
    pub fn from_samples(samples: &[f64]) -> Self {
        let count = samples.len();

        if count == 0 {
            return Self {
                count: 0,
                mean: 0.0,
                std_dev: 0.0,
                variance: 0.0,
                percentiles: Percentiles {
                    p50: 0.0,
                    p90: 0.0,
                    p95: 0.0,
                    p99: 0.0,
                    min: 0.0,
                    max: 0.0,
                },
                ci95_lower: 0.0,
                ci95_upper: 0.0,
            };
        }

        // Compute mean
        #[allow(clippy::cast_precision_loss)]
        let mean = samples.iter().sum::<f64>() / count as f64;

        // Compute variance
        #[allow(clippy::cast_precision_loss)]
        let variance = if count > 1 {
            samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (count - 1) as f64
        } else {
            0.0
        };

        let std_dev = variance.sqrt();

        // Sort for percentiles
        let mut sorted = samples.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let percentiles = Percentiles::from_sorted(&sorted);

        // 95% confidence interval using t-distribution approximation
        // For large samples, t ≈ 1.96
        let t_value = if count >= 30 { 1.96 } else { 2.0 };
        #[allow(clippy::cast_precision_loss)]
        let margin = t_value * std_dev / (count as f64).sqrt();
        let ci95_lower = mean - margin;
        let ci95_upper = mean + margin;

        Self {
            count,
            mean,
            std_dev,
            variance,
            percentiles,
            ci95_lower,
            ci95_upper,
        }
    }

    /// Validates all invariants.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        // INV-1: No NaN/Infinity
        let values_finite = self.mean.is_finite()
            && self.std_dev.is_finite()
            && self.variance.is_finite()
            && self.ci95_lower.is_finite()
            && self.ci95_upper.is_finite();

        // INV-4: Sample count matches
        let count_valid = self.count > 0;

        // INV-5: Percentiles ordered
        let percentiles_valid = self.percentiles.is_ordered();

        values_finite && count_valid && percentiles_valid
    }

    /// Returns the coefficient of variation (CV).
    #[must_use]
    pub fn coefficient_of_variation(&self) -> f64 {
        if self.mean != 0.0 {
            self.std_dev / self.mean.abs()
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    #[test]
    fn test_frame_sample_fps() {
        let sample = FrameSample::new(1, 8.33, 0.0);
        let fps = sample.fps();
        assert!((fps - 120.0).abs() < 0.5);
    }

    #[test]
    fn test_frame_sample_is_valid() {
        let valid = FrameSample::new(1, 8.33, 0.0);
        assert!(valid.is_valid());

        let invalid = FrameSample::new(1, f64::NAN, 0.0);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_percentiles_from_sorted() {
        let sorted: Vec<f64> = (1..=100).map(f64::from).collect();
        let p = Percentiles::from_sorted(&sorted);

        // With 100 elements, percentiles are calculated using index = (len-1) * p
        // p50: index 49.5 -> rounds to 50 -> value 51
        // p90: index 89.1 -> rounds to 89 -> value 90
        // p95: index 94.05 -> rounds to 94 -> value 95
        // p99: index 98.01 -> rounds to 98 -> value 99
        assert!((p.p50 - 51.0).abs() < 1.0);
        assert!((p.p90 - 90.0).abs() < 1.0);
        assert!((p.p95 - 95.0).abs() < 1.0);
        assert!((p.p99 - 99.0).abs() < 1.0);
        assert_eq!(p.min, 1.0);
        assert_eq!(p.max, 100.0);
    }

    #[test]
    fn test_percentiles_is_ordered() {
        let p = Percentiles {
            p50: 5.0,
            p90: 9.0,
            p95: 9.5,
            p99: 9.9,
            min: 1.0,
            max: 10.0,
        };
        assert!(p.is_ordered());

        let invalid = Percentiles {
            p50: 10.0,
            p90: 5.0, // Out of order
            p95: 9.5,
            p99: 9.9,
            min: 1.0,
            max: 10.0,
        };
        assert!(!invalid.is_ordered());
    }

    #[test]
    fn test_statistics_from_samples() {
        let samples: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = Statistics::from_samples(&samples);

        assert_eq!(stats.count, 5);
        assert!((stats.mean - 3.0).abs() < 0.001);
        assert!(stats.std_dev > 0.0);
        assert!(stats.is_valid());
    }

    #[test]
    fn test_statistics_empty_samples() {
        let samples: Vec<f64> = vec![];
        let stats = Statistics::from_samples(&samples);

        assert_eq!(stats.count, 0);
        assert_eq!(stats.mean, 0.0);
    }

    #[test]
    fn test_statistics_no_nan_with_finite_input() {
        let samples: Vec<f64> = (0..100).map(|i| f64::from(i) + 0.5).collect();
        let stats = Statistics::from_samples(&samples);

        assert!(stats.mean.is_finite());
        assert!(stats.std_dev.is_finite());
        assert!(stats.variance.is_finite());
        assert!(stats.ci95_lower.is_finite());
        assert!(stats.ci95_upper.is_finite());
    }

    #[test]
    fn test_coefficient_of_variation() {
        let samples: Vec<f64> = vec![10.0, 20.0, 30.0];
        let stats = Statistics::from_samples(&samples);

        let cv = stats.coefficient_of_variation();
        assert!(cv > 0.0 && cv < 1.0);
    }
}

extern crate alloc;
