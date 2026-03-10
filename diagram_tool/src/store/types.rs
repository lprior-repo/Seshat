use crate::store_async::AsyncStoreError as StoreError;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;

#[derive(Debug, Clone, PartialEq)]
pub struct ValidEvent {
    pub op_id: ValidOperationId,
    pub timestamp: ValidTimestamp,
    pub payload: ValidPayload,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoundedBatch<const MIN: usize, const MAX: usize>(Vec<ValidEvent>);

impl<const MIN: usize, const MAX: usize> TryFrom<Vec<ValidEvent>> for BoundedBatch<MIN, MAX> {
    type Error = StoreError;

    fn try_from(vec: Vec<ValidEvent>) -> Result<Self, Self::Error> {
        if vec.len() < MIN {
            return Err(StoreError::EmptyBatch);
        }
        if vec.len() > MAX {
            return Err(StoreError::BatchTooLarge);
        }
        Ok(Self(vec))
    }
}

impl<const MIN: usize, const MAX: usize> BoundedBatch<MIN, MAX> {
    #[must_use]
    pub fn into_inner(self) -> Vec<ValidEvent> {
        self.0
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidTimestamp(NonZeroU64);

impl ValidTimestamp {
    pub fn new(ts: u64) -> Result<Self, StoreError> {
        NonZeroU64::new(ts)
            .map(Self)
            .ok_or(StoreError::InvalidTimestamp)
    }

    #[must_use]
    pub const fn get(&self) -> u64 {
        self.0.get()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidOperationId(String);

impl ValidOperationId {
    pub fn new(id: String) -> Result<Self, StoreError> {
        if id.is_empty() || id.trim().is_empty() || id.contains('\0') {
            return Err(StoreError::InvalidOperationId);
        }
        if id.len() > 255 {
            return Err(StoreError::OperationIdTooLong);
        }
        Ok(Self(id))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidPayload(String);

impl ValidPayload {
    pub fn new(payload: String) -> Result<Self, StoreError> {
        if payload.len() > 100 * 1024 * 1024 {
            return Err(StoreError::PayloadTooLarge);
        }
        Ok(Self(payload))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Revision(i64);

impl Revision {
    pub fn new(rev: i64) -> Result<Self, StoreError> {
        if rev < 0 {
            return Err(StoreError::ValidationFailed(
                "Revision cannot be negative".to_string(),
            ));
        }
        Ok(Self(rev))
    }

    #[must_use]
    pub const fn get(&self) -> i64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppendOutcome {
    pub revision: i64,
    pub op_id: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventRecord {
    pub op_id: String,
    pub revision: i64,
    pub timestamp: i64,
    pub payload: String,
}

// =============================================================================
// Durable Operation Tracking - Restate-like Workflow
// =============================================================================

/// State of a durable operation (tracks multi-step AI workflows)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationState {
    Started,
    InProgress,
    Completed,
    Failed,
}

impl OperationState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "started" => Some(Self::Started),
            "in_progress" => Some(Self::InProgress),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

/// Record of a durable operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationRecord {
    pub operation_id: String,
    pub state: OperationState,
    pub current_step: u32,
    pub total_steps: u32,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub final_revision: Option<i64>,
    pub error_message: Option<String>,
    pub author_id: String,
    pub description: String,
}

// =============================================================================
// Step Journal - Tracks individual steps in multi-step operations
// =============================================================================

/// Status of a single step within an operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

impl StepStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "running" => Some(Self::Running),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "skipped" => Some(Self::Skipped),
            _ => None,
        }
    }
}

/// Record of a single step in the step journal
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepRecord {
    pub operation_id: String,
    pub step_index: u32,
    pub step_name: String,
    pub status: StepStatus,
    pub event_revision: Option<i64>,
    pub created_at: i64,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
    pub error_message: Option<String>,
}

// =============================================================================
// Outbox - Reliable side-effect delivery
// =============================================================================

/// Status of an outbox entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutboxStatus {
    Pending,
    Dispatched,
    Acknowledged,
    Failed,
}

impl OutboxStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Dispatched => "dispatched",
            Self::Acknowledged => "acknowledged",
            Self::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "dispatched" => Some(Self::Dispatched),
            "acknowledged" => Some(Self::Acknowledged),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

/// Type of side effect in outbox
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SideEffectType {
    Notify,
    Webhook,
    TriggerOperation,
    Custom,
}

impl SideEffectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Notify => "notify",
            Self::Webhook => "webhook",
            Self::TriggerOperation => "trigger_operation",
            Self::Custom => "custom",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "notify" => Some(Self::Notify),
            "webhook" => Some(Self::Webhook),
            "trigger_operation" => Some(Self::TriggerOperation),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }
}

/// Record in the outbox table
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutboxRecord {
    pub id: String,
    pub side_effect_type: SideEffectType,
    pub payload: String,
    pub event_revision: i64,
    pub status: OutboxStatus,
    pub retry_count: u32,
    pub max_retries: u32,
    pub created_at: i64,
    pub dispatched_at: Option<i64>,
    pub acknowledged_at: Option<i64>,
    pub last_error: Option<String>,
}

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

// =============================================================================
// Cursor-based Pagination
// =============================================================================

/// Cursor for paginating through events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventCursor {
    pub revision: i64,
    pub limit: u32,
}

impl EventCursor {
    pub fn new(revision: i64, limit: u32) -> Self {
        Self { revision, limit }
    }

    pub fn first(limit: u32) -> Self {
        Self { revision: 0, limit }
    }
}

/// Result of a cursor-based event fetch
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventPage {
    pub events: Vec<EventRecord>,
    pub next_cursor: Option<EventCursor>,
    pub has_more: bool,
}
