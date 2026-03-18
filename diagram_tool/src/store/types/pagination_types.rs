use super::core_types::EventRecord;
use serde::{Deserialize, Serialize};

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
