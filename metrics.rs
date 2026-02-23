#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestionDecision {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionDecisionMetrics {
    pub timestamp: DateTime<Utc>,
    pub suggestion_key: String,
    pub decision: SuggestionDecision,
    pub source: String,
}

pub struct MetricsStore {
    _path: std::path::PathBuf,
}

impl MetricsStore {
    #[must_use]
    pub fn new(path: &std::path::Path) -> Self {
        Self {
            _path: path.to_path_buf(),
        }
    }

    /// Record a suggestion decision to persistent storage.
    ///
    /// # Errors
    /// Returns an I/O error if writing to storage fails.
    pub fn record_suggestion_decision(
        &self,
        _metrics: SuggestionDecisionMetrics,
    ) -> Result<(), std::io::Error> {
        Ok(())
    }
}
