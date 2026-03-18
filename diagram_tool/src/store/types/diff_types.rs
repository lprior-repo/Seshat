use serde::{Deserialize, Serialize};

// =============================================================================
// Conflict Diff - Rich diff on conditional append rejection
// =============================================================================

/// Domain operation for diff representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DiffDomainOp {
    NodeAdd {
        id: String,
    },
    NodeMove {
        id: String,
        x: f64,
        y: f64,
    },
    NodeDelete {
        id: String,
    },
    EdgeConnect {
        id: String,
        source: String,
        target: String,
    },
    EdgeDisconnect {
        id: String,
    },
    #[serde(other)]
    Other,
}

/// Rich diff returned when conditional append is rejected
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConflictDiff {
    pub assumed_revision: i64,
    pub actual_revision: i64,
    pub changes: Vec<DiffDomainOp>,
    pub first_change_timestamp: i64,
    pub first_change_author: String,
}
