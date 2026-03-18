use serde::{Deserialize, Serialize};

use super::Percentiles;

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

impl Default for Statistics {
    fn default() -> Self {
        Self {
            count: 0,
            mean: 0.0,
            std_dev: 0.0,
            variance: 0.0,
            percentiles: Percentiles::default(),
            ci95_lower: 0.0,
            ci95_upper: 0.0,
        }
    }
}

impl Statistics {
    /// Computes statistics from raw samples.
    #[must_use]
    pub fn from_samples(samples: &[f64]) -> Self {
        let count = samples.len();

        if count == 0 {
            return Self::default();
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
    pub const fn is_valid(&self) -> bool {
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
        if self.mean == 0.0 {
            0.0
        } else {
            self.std_dev / self.mean.abs()
        }
    }
}

extern crate alloc;

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_statistics_from_samples() {
        let samples: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = Statistics::from_samples(&samples);

        assert_eq!(stats.count, 5);
        assert!((stats.mean - 3.0).abs() < 0.001);
        assert!(stats.std_dev > 0.0);
        assert!(stats.is_valid());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_statistics_empty_samples() {
        let samples: Vec<f64> = vec![];
        let stats = Statistics::from_samples(&samples);

        assert_eq!(stats.count, 0);
        assert_eq!(stats.mean, 0.0);
    }

    #[cfg(kani)]
    #[kani::proof]
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

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_coefficient_of_variation() {
        let samples: Vec<f64> = vec![10.0, 20.0, 30.0];
        let stats = Statistics::from_samples(&samples);

        let cv = stats.coefficient_of_variation();
        assert!(cv > 0.0 && cv < 1.0);
    }
}
