//! AI Event Drop Detection Module
//!
//! This module provides functionality to detect dropped AI events in the WAL poller.
//! It follows the Data->Calc->Actions pattern with pure calculation functions.

use crate::store_async::EventRecord;
use crate::ui::toast::AiConflictState;
use std::collections::HashSet;
use thiserror::Error;

/// Type alias for pending operation IDs
type PendingOps = HashSet<String>;

/// Error types for AI event detection
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DetectError {
    /// Store bridge not available or pool not initialized
    #[error("Store unavailable: {0}")]
    StoreUnavailable(String),
    /// Failed to fetch events from WAL
    #[error("Fetch failed: {0}")]
    FetchFailed(String),
}

/// Default conflict message for dropped AI operations
pub const DEFAULT_CONFLICT_MESSAGE: &str = "AI operation rejected - human has active edit";

/// Checks if a pending `op_id` was not found in the fetched events (i.e., it was dropped).
///
/// This is a pure calculation - no side effects.
///
/// # Parameters
/// - `pending_ops`: Set of pending AI operation `op_id`s
/// - `fetched_events`: Vector of `EventRecord`s fetched from WAL
///
/// # Returns
/// - Vector of `op_id`s that were in pending but not in fetched events
#[must_use]
pub fn find_dropped_op_ids(
    pending_ops: &PendingOps,
    fetched_events: &[EventRecord],
) -> Vec<String> {
    let fetched_op_ids: HashSet<String> = fetched_events.iter().map(|e| e.op_id.clone()).collect();

    pending_ops
        .iter()
        .filter(|op_id| !fetched_op_ids.contains(*op_id))
        .cloned()
        .collect()
}

/// Generates the conflict message for dropped AI operations.
///
/// # Parameters
/// - `dropped_ops`: Vector of dropped `op_id`s
///
/// # Returns
/// - The conflict message string
#[must_use]
pub fn generate_conflict_message(dropped_ops: &[String]) -> String {
    match dropped_ops.len() {
        0 => String::new(),
        1 => format!(
            "{} - operation {} was rejected",
            DEFAULT_CONFLICT_MESSAGE, dropped_ops[0]
        ),
        _ => format!(
            "{} - {} operations were rejected",
            DEFAULT_CONFLICT_MESSAGE,
            dropped_ops.len()
        ),
    }
}

/// Result type for detection operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropDetectionResult {
    /// The `op_id`s that were detected as dropped
    pub dropped_op_ids: Vec<String>,
    /// Whether conflict state should be updated
    pub has_conflict: bool,
    /// The conflict state if there's a conflict
    pub conflict_state: Option<AiConflictState>,
}

impl DropDetectionResult {
    /// Creates a new result with no drops detected
    #[must_use]
    pub const fn no_drops() -> Self {
        Self {
            dropped_op_ids: Vec::new(),
            has_conflict: false,
            conflict_state: None,
        }
    }

    /// Creates a new result with drops detected
    #[must_use]
    pub fn with_drops(dropped_op_ids: Vec<String>) -> Self {
        let has_conflict = !dropped_op_ids.is_empty();
        let conflict_state = has_conflict.then(|| {
            AiConflictState::new(Some(generate_conflict_message(&dropped_op_ids)), Vec::new())
        });
        Self {
            dropped_op_ids,
            has_conflict,
            conflict_state,
        }
    }
}

/// Detects dropped AI events by comparing pending operations against fetched events.
///
/// This is a pure calculation function that takes all inputs as parameters
/// and returns a result with no side effects.
///
/// # Parameters
/// - `pending_ops`: Set of pending AI operation `op_id`s
/// - `fetched_events`: Vector of `EventRecord`s fetched from WAL
/// - `current_conflict_state`: Current value of the conflict state signal (to avoid overwriting)
///
/// # Returns
/// - `DropDetectionResult` containing dropped `op_id`s and conflict information
#[must_use]
pub fn detect_dropped_ai_events(
    pending_ops: &PendingOps,
    fetched_events: &[EventRecord],
    current_conflict_state: &Option<AiConflictState>,
) -> DropDetectionResult {
    // If no pending ops, no drops possible
    if pending_ops.is_empty() {
        return DropDetectionResult::no_drops();
    }

    // Find dropped operations
    let dropped_op_ids = find_dropped_op_ids(pending_ops, fetched_events);

    // If no drops, nothing to report
    if dropped_op_ids.is_empty() {
        return DropDetectionResult::no_drops();
    }

    // Only update conflict if there's no existing conflict state (avoid overwriting)
    let has_conflict = current_conflict_state.is_none();

    if has_conflict {
        DropDetectionResult::with_drops(dropped_op_ids)
    } else {
        // Conflict already exists - still report the drops (for removal from pending)
        // but don't update the conflict state
        DropDetectionResult {
            dropped_op_ids,
            has_conflict: false,
            conflict_state: None,
        }
    }
}

/// Filters pending ops to remove the dropped ones.
///
/// This is a pure calculation that returns a new set.
///
/// # Parameters
/// - `pending_ops`: Original set of pending ops
/// - `dropped_op_ids`: Vector of `op_id`s to remove
///
/// # Returns
/// - New `HashSet` with dropped ops removed
#[must_use]
pub fn remove_dropped_ops(pending_ops: &PendingOps, dropped_op_ids: &[String]) -> PendingOps {
    pending_ops
        .iter()
        .filter(|op_id| !dropped_op_ids.contains(op_id))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests;
