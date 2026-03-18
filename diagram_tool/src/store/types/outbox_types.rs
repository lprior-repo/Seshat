use serde::{Deserialize, Serialize};

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

    #[allow(clippy::should_implement_trait)]
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

    #[allow(clippy::should_implement_trait)]
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
