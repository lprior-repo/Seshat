//! Error taxonomy for performance measurement module.
//!
//! Every failure mode has a corresponding variant per contract-spec.md.

use std::fmt;

/// Comprehensive error type for performance operations.
#[derive(Debug, Clone, PartialEq)]
pub enum PerfError {
    /// Node count outside valid range [1, 10000]
    InvalidNodeCount(u32),

    /// Duration below minimum threshold (100ms)
    InvalidDuration(u64),

    /// Measurement failed due to internal error
    MeasurementFailed(String),

    /// Benchmark exceeded timeout limit
    Timeout { ms: u64 },

    /// Collected fewer samples than required
    InsufficientSamples { got: usize, need: usize },

    /// Baseline file not found at specified path
    BaselineNotFound(String),

    /// Performance regression detected
    RegressionDetected { delta: f64, threshold: f64 },

    /// I/O operation failed
    Io(String),

    /// Serialization/deserialization failed
    Serialization(String),

    /// Environment not suitable for benchmarking
    Environment(String),

    /// Internal invariant was violated
    InvariantViolation {
        invariant: &'static str,
        details: String,
    },
}

impl fmt::Display for PerfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNodeCount(count) => {
                write!(f, "invalid node count: {count} (must be 1-10000)")
            }
            Self::InvalidDuration(ms) => {
                write!(f, "invalid duration: {ms}ms (must be >= 100ms)")
            }
            Self::MeasurementFailed(msg) => {
                write!(f, "measurement failed: {msg}")
            }
            Self::Timeout { ms } => {
                write!(f, "benchmark timeout after {ms}ms")
            }
            Self::InsufficientSamples { got, need } => {
                write!(f, "insufficient samples: got {got}, need {need}")
            }
            Self::BaselineNotFound(path) => {
                write!(f, "baseline not found: {path}")
            }
            Self::RegressionDetected { delta, threshold } => {
                write!(
                    f,
                    "regression detected: {delta:.2} FPS drop (threshold: {threshold:.2})"
                )
            }
            Self::Io(msg) => {
                write!(f, "IO error: {msg}")
            }
            Self::Serialization(msg) => {
                write!(f, "serialization error: {msg}")
            }
            Self::Environment(msg) => {
                write!(f, "environment error: {msg}")
            }
            Self::InvariantViolation { invariant, details } => {
                write!(f, "invariant violation: {invariant} - {details}")
            }
        }
    }
}

impl std::error::Error for PerfError {}

impl From<std::io::Error> for PerfError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<serde_json::Error> for PerfError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl PerfError {
    /// Creates an invariant violation error.
    #[must_use]
    pub const fn invariant_violation(invariant: &'static str, details: String) -> Self {
        Self::InvariantViolation { invariant, details }
    }

    /// Returns true if this error indicates a regression.
    #[must_use]
    pub const fn is_regression(&self) -> bool {
        matches!(self, Self::RegressionDetected { .. })
    }

    /// Returns true if this error is recoverable (can retry).
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Timeout { .. } | Self::InsufficientSamples { .. } | Self::Environment(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_invalid_node_count_display() {
        let err = PerfError::InvalidNodeCount(0);
        let msg = format!("{err}");
        assert!(msg.contains("0"));
        assert!(msg.contains("1-10000"));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_invalid_duration_display() {
        let err = PerfError::InvalidDuration(50);
        let msg = format!("{err}");
        assert!(msg.contains("50ms"));
        assert!(msg.contains("100ms"));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_regression_detected_display() {
        let err = PerfError::RegressionDetected {
            delta: 15.5,
            threshold: 10.0,
        };
        let msg = format!("{err}");
        assert!(msg.contains("15.50"));
        assert!(msg.contains("10.00"));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_is_regression() {
        assert!(PerfError::RegressionDetected {
            delta: 5.0,
            threshold: 10.0
        }
        .is_regression());
        assert!(!PerfError::InvalidNodeCount(0).is_regression());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_is_recoverable() {
        assert!(PerfError::Timeout { ms: 1000 }.is_recoverable());
        assert!(PerfError::InsufficientSamples { got: 5, need: 10 }.is_recoverable());
        assert!(!PerfError::InvalidNodeCount(0).is_recoverable());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_invariant_violation_constructor() {
        let err =
            PerfError::invariant_violation("INV-1", "NaN detected in measurements".to_string());
        assert!(matches!(
            err,
            PerfError::InvariantViolation {
                invariant: "INV-1",
                ..
            }
        ));
    }
}
