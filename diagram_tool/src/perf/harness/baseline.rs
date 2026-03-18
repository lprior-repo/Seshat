//! Baseline performance data definition and persistence.

use std::{collections::HashMap, path::PathBuf, time::UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::perf::{benchmark::BenchmarkResult, error::PerfError};

use super::operation::Operation;

/// Baseline performance data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Baseline {
    /// Version of the baseline format
    pub version: u32,
    /// Node count used for baseline
    pub node_count: u32,
    /// Target FPS
    pub target_fps: f64,
    /// Results per operation
    pub results: HashMap<String, BenchmarkResult>,
    /// Timestamp when baseline was created
    pub created_at: u64,
}

impl Baseline {
    /// Current baseline format version.
    pub const VERSION: u32 = 1;

    /// Creates a new baseline.
    #[must_use]
    pub fn new(node_count: u32, target_fps: f64) -> Self {
        Self {
            version: Self::VERSION,
            node_count,
            target_fps,
            results: HashMap::new(),
            created_at: UNIX_EPOCH
                .elapsed()
                .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(0)),
        }
    }

    /// Adds a result for an operation.
    pub fn add_result(&mut self, operation: Operation, result: BenchmarkResult) {
        self.results.insert(operation.name().to_string(), result);
    }

    /// Gets a result for an operation.
    #[must_use]
    pub fn get_result(&self, operation: Operation) -> Option<&BenchmarkResult> {
        self.results.get(operation.name())
    }

    /// Loads baseline from a file.
    ///
    /// # Errors
    ///
    /// Returns `PerfError` if loading fails.
    pub fn load(path: &PathBuf) -> Result<Self, PerfError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| PerfError::BaselineNotFound(format!("{}: {}", path.display(), e)))?;

        let baseline: Self = serde_json::from_str(&content)?;

        if baseline.version != Self::VERSION {
            return Err(PerfError::Serialization(format!(
                "unsupported baseline version: {}",
                baseline.version
            )));
        }

        Ok(baseline)
    }

    /// Saves baseline to a file.
    ///
    /// # Errors
    ///
    /// Returns `PerfError` if saving fails.
    pub fn save(&self, path: &PathBuf) -> Result<(), PerfError> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validates all results in the baseline.
    ///
    /// # Errors
    ///
    /// Returns `PerfError` if any result is invalid.
    pub fn validate(&self) -> Result<(), PerfError> {
        for (name, result) in &self.results {
            result
                .validate()
                .map_err(|e| PerfError::InvariantViolation {
                    invariant: "BASELINE_VALIDITY",
                    details: format!("{name}: {e}"),
                })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::perf::benchmark::BenchmarkConfig;

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_baseline_new() {
        let baseline = Baseline::new(3000, 120.0);
        assert_eq!(baseline.node_count, 3000);
        assert_eq!(baseline.target_fps, 120.0);
        assert!(baseline.results.is_empty());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_baseline_add_get_result() {
        let mut baseline = Baseline::new(3000, 120.0);

        let config = BenchmarkConfig::new("pan");
        let fps_report = crate::perf::fps::FpsReport::from_samples(
            (0..10)
                .map(|i| crate::perf::metrics::FrameSample::new(i, 8.33, i as f64 * 8.33))
                .collect(),
            120.0,
        );
        let result = BenchmarkResult::new(config, fps_report, 0);

        baseline.add_result(Operation::Pan, result);
        assert!(baseline.get_result(Operation::Pan).is_some());
        assert!(baseline.get_result(Operation::Zoom).is_none());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_baseline_save_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("baseline.json");

        let mut baseline = Baseline::new(3000, 120.0);
        let config = BenchmarkConfig::new("pan");
        let fps_report = crate::perf::fps::FpsReport::from_samples(
            (0..10)
                .map(|i| crate::perf::metrics::FrameSample::new(i, 8.33, i as f64 * 8.33))
                .collect(),
            120.0,
        );
        let result = BenchmarkResult::new(config, fps_report, 0);
        baseline.add_result(Operation::Pan, result);

        baseline.save(&path).unwrap();
        let loaded = Baseline::load(&path).unwrap();

        assert_eq!(loaded.node_count, baseline.node_count);
        assert_eq!(loaded.target_fps, baseline.target_fps);
        assert!(loaded.get_result(Operation::Pan).is_some());
    }
}
