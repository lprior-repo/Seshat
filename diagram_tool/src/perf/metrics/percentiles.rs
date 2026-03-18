use serde::{Deserialize, Serialize};

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

impl Default for Percentiles {
    fn default() -> Self {
        Self {
            p50: 0.0,
            p90: 0.0,
            p95: 0.0,
            p99: 0.0,
            min: 0.0,
            max: 0.0,
        }
    }
}

impl Percentiles {
    /// Creates percentile statistics from sorted samples.
    #[must_use]
    pub fn from_sorted(sorted: &[f64]) -> Self {
        if sorted.is_empty() {
            return Self::default();
        }

        let len = sorted.len();
        let percentile = |p: f64| -> f64 {
            #[allow(
                clippy::cast_precision_loss,
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss
            )]
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

extern crate alloc;

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_percentiles_from_sorted() {
        let sorted: Vec<f64> = (1..=100).map(f64::from).collect();
        let p = Percentiles::from_sorted(&sorted);

        assert!((p.p50 - 51.0).abs() < 1.0);
        assert!((p.p90 - 90.0).abs() < 1.0);
        assert!((p.p95 - 95.0).abs() < 1.0);
        assert!((p.p99 - 99.0).abs() < 1.0);
        assert_eq!(p.min, 1.0);
        assert_eq!(p.max, 100.0);
    }

    #[cfg(kani)]
    #[kani::proof]
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
}
