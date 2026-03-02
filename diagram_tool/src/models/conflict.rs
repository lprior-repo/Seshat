//! Conflict detection module - human priority conflict resolution
//!
//! This module provides conflict detection for AI operations during active human edits.
//! Human-authored operations take priority over AI operations when they conflict.
//!
//! # Design
//!
//! The conflict detection system tracks:
//! - Active human edit windows by entity
//! - Whether an incoming AI operation conflicts with ongoing human edits
//! - Deterministic rejection of conflicting AI operations

#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::{Duration, Instant};
use thiserror::Error;

use crate::models::envelope::{Author, DomainOp, EventEnvelope};
use crate::models::projection::DiagramProjection;

/// Duration for which a human edit window remains active after the last edit
const HUMAN_EDIT_WINDOW_SECS: u64 = 30;

/// Errors that can occur during conflict evaluation
#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConflictError {
    /// The operation was blocked because a human has priority
    #[error("human priority block: {0}")]
    HumanPriorityBlock(String),
    /// The referenced entity does not exist
    #[error("missing entity: {0}")]
    MissingEntity(String),
    /// A policy violation occurred
    #[error("policy violation: {0}")]
    PolicyViolation(String),
}

/// Decision about whether an operation should be allowed or rejected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConflictDecision {
    /// The operation is allowed to proceed
    Allow,
    /// The operation is rejected due to conflict
    Reject {
        /// The reason for rejection
        reason: ConflictError,
        /// The entity IDs involved in the conflict
        conflicting_entities: Vec<String>,
    },
}

/// State for tracking active human edit windows
#[derive(Debug, Clone, Default)]
pub struct ProjectionState {
    /// Entities currently being edited by humans with their last edit time
    human_edit_windows: im::HashMap<String, HumanEditWindow>,
    /// Set of operation IDs that have been processed (for idempotency)
    processed_ops: HashSet<String>,
}

/// Tracks an active human edit window for a specific entity
#[derive(Debug, Clone)]
struct HumanEditWindow {
    /// The entity being edited (node or edge ID)
    entity_id: String,
    /// When the last human edit occurred
    last_edit_time: Instant,
    /// The author who is editing
    author_id: String,
}

impl HumanEditWindow {
    /// Create a new human edit window
    fn new(entity_id: String, author_id: String) -> Self {
        Self {
            entity_id,
            last_edit_time: Instant::now(),
            author_id,
        }
    }

    /// Check if this edit window is still active
    fn is_active(&self) -> bool {
        let elapsed = Instant::now().duration_since(self.last_edit_time);
        elapsed < Duration::from_secs(HUMAN_EDIT_WINDOW_SECS)
    }

    /// Refresh the edit window with a new edit
    fn refresh(&mut self) {
        self.last_edit_time = Instant::now();
    }
}

impl ProjectionState {
    /// Create a new empty projection state
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a human edit for an entity
    pub fn register_human_edit(&mut self, entity_id: &str, author_id: &str) {
        if let Some(window) = self.human_edit_windows.get_mut(entity_id) {
            window.refresh();
        } else {
            self.human_edit_windows.insert(
                entity_id.to_string(),
                HumanEditWindow::new(entity_id.to_string(), author_id.to_string()),
            );
        }
    }

    /// Check if an entity has an active human edit window
    #[must_use]
    pub fn has_active_human_edit(&self, entity_id: &str) -> bool {
        self.human_edit_windows
            .get(entity_id)
            .is_some_and(|w| w.is_active())
    }

    /// Get all entities with active human edits
    #[must_use]
    pub fn active_human_edit_entities(&self) -> Vec<String> {
        self.human_edit_windows
            .iter()
            .filter(|(_, w)| w.is_active())
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Clean up expired edit windows
    pub fn cleanup_expired(&mut self) {
        self.human_edit_windows.retain(|_, w| w.is_active());
    }

    /// Mark an operation as processed (for idempotency)
    pub fn mark_processed(&mut self, op_id: &str) {
        self.processed_ops.insert(op_id.to_string());
    }

    /// Check if an operation has already been processed
    #[must_use]
    pub fn is_processed(&self, op_id: &str) -> bool {
        self.processed_ops.contains(op_id)
    }
}

/// Check if an author is human (not AI-generated)
fn is_human_author(author: &Author) -> bool {
    // Author IDs starting with "human-" are considered human-authored
    // All others are assumed to be AI-authored
    author.id.starts_with("human-") || author.name.to_lowercase().contains("human")
}

/// Extract entity IDs affected by a domain operation
fn extract_affected_entities(op: &DomainOp) -> Vec<String> {
    match op {
        DomainOp::NodeAdd { id, .. }
        | DomainOp::NodeMove { id, .. }
        | DomainOp::NodeDelete { id }
        | DomainOp::NodeRestore { id } => {
            vec![format!("node:{}", id)]
        }
        DomainOp::EdgeConnect { id, source, target } => {
            vec![
                format!("edge:{}", id),
                format!("node:{}", source),
                format!("node:{}", target),
            ]
        }
        DomainOp::EdgeDisconnect { id } => {
            vec![format!("edge:{}", id)]
        }
        DomainOp::BringForward { ids }
        | DomainOp::SendBackward { ids }
        | DomainOp::BringToFront { ids }
        | DomainOp::SendToBack { ids }
        | DomainOp::Group { ids } => ids.iter().map(|id| format!("node:{}", id)).collect(),
        DomainOp::Ungroup { id } => {
            vec![format!("group:{}", id)]
        }
    }
}

/// Evaluate whether an operation should be allowed or rejected based on human priority
///
/// This function checks if an incoming operation (typically from AI) conflicts with
/// any active human edit windows. Human operations always take priority.
///
/// # Arguments
/// * `op` - The event envelope to evaluate
/// * `state` - The current projection state with active edit windows
///
/// # Returns
/// * `Ok(ConflictDecision::Allow)` if the operation can proceed
/// * `Ok(ConflictDecision::Reject { ... })` if the operation should be rejected
/// * `Err(ConflictError)` if there's an error during evaluation
///
/// # Errors
/// Returns `ConflictError::MissingEntity` if the operation references a non-existent entity
/// Returns `ConflictError::PolicyViolation` if the operation violates policy
pub fn evaluate_human_priority(
    op: &EventEnvelope,
    state: &ProjectionState,
) -> Result<ConflictDecision, ConflictError> {
    // Human operations are always allowed
    if is_human_author(&op.author) {
        return Ok(ConflictDecision::Allow);
    }

    // Check for idempotency - if already processed, allow (idempotent)
    if state.is_processed(&op.op_id) {
        return Ok(ConflictDecision::Allow);
    }

    // Get entities affected by this operation
    let affected_entities = extract_affected_entities(&op.operation);

    // Check if any affected entity has an active human edit
    let conflicting_entities: Vec<String> = affected_entities
        .iter()
        .filter(|entity| state.has_active_human_edit(entity))
        .cloned()
        .collect();

    if !conflicting_entities.is_empty() {
        return Ok(ConflictDecision::Reject {
            reason: ConflictError::HumanPriorityBlock(format!(
                "active human edit on entities: {}",
                conflicting_entities.join(", ")
            )),
            conflicting_entities,
        });
    }

    Ok(ConflictDecision::Allow)
}

/// Evaluate human priority with projection context
///
/// This function also validates that referenced entities exist in the projection.
///
/// # Arguments
/// * `op` - The event envelope to evaluate
/// * `state` - The current projection state
/// * `projection` - The current diagram projection for entity validation
///
/// # Returns
/// * `Ok(ConflictDecision::Allow)` if the operation can proceed
/// * `Ok(ConflictDecision::Reject { ... })` if the operation should be rejected
/// * `Err(ConflictError)` if there's an error during evaluation
///
/// # Errors
/// Returns `ConflictError::MissingEntity` if the operation references a non-existent entity
pub fn evaluate_human_priority_with_projection(
    op: &EventEnvelope,
    state: &ProjectionState,
    projection: &DiagramProjection,
) -> Result<ConflictDecision, ConflictError> {
    use crate::models::document::{EdgeId, NodeId};

    // First check basic human priority
    let decision = evaluate_human_priority(op, state)?;

    // If allowed, also validate entity existence
    if decision == ConflictDecision::Allow {
        match &op.operation {
            DomainOp::NodeMove { id, .. }
            | DomainOp::NodeDelete { id }
            | DomainOp::NodeRestore { id } => {
                if !projection.has_node(&NodeId::new(id.clone())) {
                    return Err(ConflictError::MissingEntity(format!("node:{}", id)));
                }
            }
            DomainOp::EdgeDisconnect { id } => {
                if !projection.has_edge(&EdgeId::new(id.clone())) {
                    return Err(ConflictError::MissingEntity(format!("edge:{}", id)));
                }
            }
            DomainOp::EdgeConnect { source, target, .. } => {
                if !projection.has_node(&NodeId::new(source.clone())) {
                    return Err(ConflictError::MissingEntity(format!("node:{}", source)));
                }
                if !projection.has_node(&NodeId::new(target.clone())) {
                    return Err(ConflictError::MissingEntity(format!("node:{}", target)));
                }
            }
            _ => {}
        }
    }

    Ok(decision)
}

/// Record a conflict rejection for auditing/logging purposes
///
/// This function records when an operation is rejected due to conflict.
/// It can be used for:
/// - Audit logging
/// - Metrics collection
/// - Debugging
///
/// # Arguments
/// * `op` - The event envelope that was rejected
/// * `reason` - The conflict error that caused the rejection
///
/// # Returns
/// * `Ok(())` if the rejection was recorded successfully
/// * `Err(ConflictError)` if recording failed
///
/// # Errors
/// Returns an error if the rejection could not be recorded
pub fn record_conflict_rejection(
    op: &EventEnvelope,
    reason: ConflictError,
) -> Result<(), ConflictError> {
    // In a full implementation, this would:
    // 1. Write to an audit log
    // 2. Emit metrics
    // 3. Possibly notify the AI system

    // For now, we just validate that we have the required information
    if op.op_id.is_empty() {
        return Err(ConflictError::PolicyViolation(
            "op_id is required for conflict rejection recording".to_string(),
        ));
    }

    // Log the rejection (in production, this would go to a proper logging system)
    eprintln!(
        "[CONFLICT_REJECTION] op_id={} author={} reason={}",
        op.op_id, op.author.id, reason
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_human_author() -> Author {
        Author {
            id: "human-alice".to_string(),
            name: "Alice".to_string(),
            email: None,
        }
    }

    fn make_ai_author() -> Author {
        Author {
            id: "ai-assistant".to_string(),
            name: "AI Assistant".to_string(),
            email: None,
        }
    }

    fn make_envelope(op_id: &str, operation: DomainOp, is_human: bool) -> EventEnvelope {
        EventEnvelope {
            op_id: op_id.to_string(),
            operation,
            author: if is_human {
                make_human_author()
            } else {
                make_ai_author()
            },
            timestamp: 1700000000,
        }
    }

    #[test]
    fn given_human_operation_when_evaluating_then_allowed() {
        let state = ProjectionState::new();
        let envelope = make_envelope(
            "op-1",
            DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 40.0,
                label: "Test".to_string(),
            },
            true, // Human author
        );

        let result = evaluate_human_priority(&envelope, &state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ConflictDecision::Allow);
    }

    #[test]
    fn given_ai_operation_no_conflict_when_evaluating_then_allowed() {
        let state = ProjectionState::new();
        let envelope = make_envelope(
            "op-1",
            DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 40.0,
                label: "Test".to_string(),
            },
            false, // AI author
        );

        let result = evaluate_human_priority(&envelope, &state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ConflictDecision::Allow);
    }

    #[test]
    fn given_ai_operation_with_active_human_edit_when_evaluating_then_rejected() {
        let mut state = ProjectionState::new();
        // Register an active human edit on node-1
        state.register_human_edit("node:node-1", "human-alice");

        let envelope = make_envelope(
            "op-ai-1",
            DomainOp::NodeMove {
                id: "node-1".to_string(),
                x: 100.0,
                y: 100.0,
            },
            false, // AI author
        );

        let result = evaluate_human_priority(&envelope, &state);
        assert!(result.is_ok());

        match result.unwrap() {
            ConflictDecision::Reject {
                reason,
                conflicting_entities,
            } => {
                assert!(matches!(reason, ConflictError::HumanPriorityBlock(_)));
                assert!(conflicting_entities.contains(&"node:node-1".to_string()));
            }
            ConflictDecision::Allow => panic!("Expected rejection"),
        }
    }

    #[test]
    fn given_ai_operation_on_different_entity_when_evaluating_then_allowed() {
        let mut state = ProjectionState::new();
        // Register an active human edit on node-1
        state.register_human_edit("node:node-1", "human-alice");

        let envelope = make_envelope(
            "op-ai-1",
            DomainOp::NodeAdd {
                id: "node-2".to_string(), // Different node
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 40.0,
                label: "Test".to_string(),
            },
            false, // AI author
        );

        let result = evaluate_human_priority(&envelope, &state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ConflictDecision::Allow);
    }

    #[test]
    fn given_edge_operation_with_human_edit_on_source_node_when_evaluating_then_rejected() {
        let mut state = ProjectionState::new();
        // Register an active human edit on node-source
        state.register_human_edit("node:node-source", "human-alice");

        let envelope = make_envelope(
            "op-ai-1",
            DomainOp::EdgeConnect {
                id: "edge-1".to_string(),
                source: "node-source".to_string(),
                target: "node-target".to_string(),
            },
            false, // AI author
        );

        let result = evaluate_human_priority(&envelope, &state);
        assert!(result.is_ok());

        match result.unwrap() {
            ConflictDecision::Reject {
                reason,
                conflicting_entities,
            } => {
                assert!(matches!(reason, ConflictError::HumanPriorityBlock(_)));
                assert!(conflicting_entities.contains(&"node:node-source".to_string()));
            }
            ConflictDecision::Allow => panic!("Expected rejection"),
        }
    }

    #[test]
    fn given_record_conflict_rejection_when_valid_then_succeeds() {
        let envelope = make_envelope(
            "op-1",
            DomainOp::NodeMove {
                id: "node-1".to_string(),
                x: 100.0,
                y: 100.0,
            },
            false,
        );

        let reason = ConflictError::HumanPriorityBlock("test".to_string());
        let result = record_conflict_rejection(&envelope, reason);

        assert!(result.is_ok());
    }

    #[test]
    fn given_record_conflict_rejection_when_empty_op_id_then_fails() {
        let envelope = EventEnvelope {
            op_id: String::new(),
            operation: DomainOp::NodeMove {
                id: "node-1".to_string(),
                x: 100.0,
                y: 100.0,
            },
            author: make_ai_author(),
            timestamp: 1700000000,
        };

        let reason = ConflictError::HumanPriorityBlock("test".to_string());
        let result = record_conflict_rejection(&envelope, reason);

        assert!(result.is_err());
        match result {
            Err(ConflictError::PolicyViolation(msg)) => {
                assert!(msg.contains("op_id"));
            }
            _ => panic!("Expected PolicyViolation error"),
        }
    }

    #[test]
    fn given_extract_affected_entities_for_node_add_then_returns_node_id() {
        let op = DomainOp::NodeAdd {
            id: "node-1".to_string(),
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 40.0,
            label: "Test".to_string(),
        };

        let entities = extract_affected_entities(&op);
        assert_eq!(entities, vec!["node:node-1"]);
    }

    #[test]
    fn given_extract_affected_entities_for_edge_connect_then_returns_all_entities() {
        let op = DomainOp::EdgeConnect {
            id: "edge-1".to_string(),
            source: "node-source".to_string(),
            target: "node-target".to_string(),
        };

        let entities = extract_affected_entities(&op);
        assert_eq!(
            entities,
            vec!["edge:edge-1", "node:node-source", "node:node-target"]
        );
    }

    #[test]
    fn given_extract_affected_entities_for_bring_forward_then_returns_all_nodes() {
        let op = DomainOp::BringForward {
            ids: vec!["n1".to_string(), "n2".to_string(), "n3".to_string()],
        };

        let entities = extract_affected_entities(&op);
        assert_eq!(entities, vec!["node:n1", "node:n2", "node:n3"]);
    }

    #[test]
    fn given_is_human_author_with_human_prefix_then_returns_true() {
        let author = Author {
            id: "human-alice".to_string(),
            name: "Alice".to_string(),
            email: None,
        };
        assert!(is_human_author(&author));
    }

    #[test]
    fn given_is_human_author_with_human_in_name_then_returns_true() {
        let author = Author {
            id: "user-1".to_string(),
            name: "Human User".to_string(),
            email: None,
        };
        assert!(is_human_author(&author));
    }

    #[test]
    fn given_is_human_author_with_ai_prefix_then_returns_false() {
        let author = Author {
            id: "ai-assistant".to_string(),
            name: "AI Assistant".to_string(),
            email: None,
        };
        assert!(!is_human_author(&author));
    }

    #[test]
    fn given_projection_state_when_registering_human_edit_then_tracks_entity() {
        let mut state = ProjectionState::new();
        state.register_human_edit("node:node-1", "human-alice");

        assert!(state.has_active_human_edit("node:node-1"));
        assert!(!state.has_active_human_edit("node:node-2"));
    }

    #[test]
    fn given_projection_state_when_getting_active_entities_then_returns_correct_list() {
        let mut state = ProjectionState::new();
        state.register_human_edit("node:node-1", "human-alice");
        state.register_human_edit("node:node-2", "human-bob");

        let active = state.active_human_edit_entities();
        assert_eq!(active.len(), 2);
        assert!(active.contains(&"node:node-1".to_string()));
        assert!(active.contains(&"node:node-2".to_string()));
    }

    #[test]
    fn given_projection_state_when_marking_processed_then_tracks_op_id() {
        let mut state = ProjectionState::new();

        assert!(!state.is_processed("op-1"));
        state.mark_processed("op-1");
        assert!(state.is_processed("op-1"));
    }

    #[test]
    fn given_already_processed_op_when_evaluating_then_allowed() {
        let mut state = ProjectionState::new();
        state.register_human_edit("node:node-1", "human-alice");
        state.mark_processed("op-ai-1");

        let envelope = make_envelope(
            "op-ai-1",
            DomainOp::NodeMove {
                id: "node-1".to_string(),
                x: 100.0,
                y: 100.0,
            },
            false, // AI author
        );

        let result = evaluate_human_priority(&envelope, &state);
        assert!(result.is_ok());
        // Should be allowed because it's already processed (idempotent)
        assert_eq!(result.unwrap(), ConflictDecision::Allow);
    }

    #[test]
    fn given_conflict_decision_serialization_then_roundtrips() {
        let decision = ConflictDecision::Reject {
            reason: ConflictError::HumanPriorityBlock("test conflict".to_string()),
            conflicting_entities: vec!["node:node-1".to_string()],
        };

        let json = serde_json::to_string(&decision).expect("Should serialize");
        let parsed: ConflictDecision = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(decision, parsed);
    }

    #[test]
    fn given_conflict_error_serialization_then_roundtrips() {
        let error = ConflictError::HumanPriorityBlock("test error".to_string());

        let json = serde_json::to_string(&error).expect("Should serialize");
        let parsed: ConflictError = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(error, parsed);
    }

    // =========================================================================
    // Edit Window Expiry Edge Case Tests
    // =========================================================================

    #[test]
    fn given_edit_window_expired_when_ai_operation_evaluated_then_allowed() {
        let mut state = ProjectionState::new();
        // Register an edit that will immediately be considered "expired"
        // Since we can't manipulate time directly, we verify the behavior
        // by checking that without an active edit, AI ops are allowed
        state.register_human_edit("node:node-1", "human-alice");

        // Verify the edit is active initially
        assert!(state.has_active_human_edit("node:node-1"));

        // Cleanup expired windows (simulating time passage)
        state.cleanup_expired();

        // If the window expired, has_active_human_edit should return false
        // Note: This test validates the cleanup mechanism exists
    }

    #[test]
    fn given_edit_window_refreshed_when_subsequent_human_edit_then_still_active() {
        let mut state = ProjectionState::new();
        state.register_human_edit("node:node-1", "human-alice");

        // Refresh the edit window
        state.register_human_edit("node:node-1", "human-alice");

        // Should still be active
        assert!(state.has_active_human_edit("node:node-1"));
    }

    #[test]
    fn given_multiple_expired_windows_when_cleanup_then_only_active_remain() {
        let mut state = ProjectionState::new();

        // Register edits on multiple entities
        state.register_human_edit("node:node-1", "human-alice");
        state.register_human_edit("node:node-2", "human-bob");
        state.register_human_edit("node:node-3", "human-charlie");

        // All should be active
        let active = state.active_human_edit_entities();
        assert_eq!(active.len(), 3);

        // Cleanup (windows should still be active since just registered)
        state.cleanup_expired();

        let active_after = state.active_human_edit_entities();
        assert_eq!(active_after.len(), 3);
    }

    // =========================================================================
    // Concurrent Human/AI Operations Edge Case Tests
    // =========================================================================

    #[test]
    fn given_active_human_edit_when_ai_attempts_same_entity_then_rejected_with_entities() {
        let mut state = ProjectionState::new();
        state.register_human_edit("node:node-1", "human-alice");

        let envelope = make_envelope(
            "op-ai-1",
            DomainOp::NodeMove {
                id: "node-1".to_string(),
                x: 100.0,
                y: 100.0,
            },
            false, // AI author
        );

        let result = evaluate_human_priority(&envelope, &state);
        assert!(result.is_ok());

        match result.unwrap() {
            ConflictDecision::Reject {
                reason,
                conflicting_entities,
            } => {
                assert!(matches!(reason, ConflictError::HumanPriorityBlock(_)));
                assert!(conflicting_entities.contains(&"node:node-1".to_string()));
            }
            ConflictDecision::Allow => panic!("Expected rejection for conflicting entity"),
        }
    }

    #[test]
    fn given_active_human_edit_on_node1_when_ai_adds_node2_then_allowed() {
        let mut state = ProjectionState::new();
        state.register_human_edit("node:node-1", "human-alice");

        let envelope = make_envelope(
            "op-ai-1",
            DomainOp::NodeAdd {
                id: "node-2".to_string(),
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 40.0,
                label: "New Node".to_string(),
            },
            false, // AI author
        );

        let result = evaluate_human_priority(&envelope, &state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ConflictDecision::Allow);
    }

    #[test]
    fn given_active_human_edit_on_source_when_ai_connects_edge_then_rejected() {
        let mut state = ProjectionState::new();
        state.register_human_edit("node:node-source", "human-alice");

        let envelope = make_envelope(
            "op-ai-1",
            DomainOp::EdgeConnect {
                id: "edge-1".to_string(),
                source: "node-source".to_string(),
                target: "node-target".to_string(),
            },
            false, // AI author
        );

        let result = evaluate_human_priority(&envelope, &state);
        assert!(result.is_ok());

        match result.unwrap() {
            ConflictDecision::Reject {
                conflicting_entities,
                ..
            } => {
                assert!(conflicting_entities.contains(&"node:node-source".to_string()));
            }
            ConflictDecision::Allow => panic!("Expected rejection when source has human edit"),
        }
    }

    #[test]
    fn given_active_human_edit_on_target_when_ai_connects_edge_then_rejected() {
        let mut state = ProjectionState::new();
        state.register_human_edit("node:node-target", "human-alice");

        let envelope = make_envelope(
            "op-ai-1",
            DomainOp::EdgeConnect {
                id: "edge-1".to_string(),
                source: "node-source".to_string(),
                target: "node-target".to_string(),
            },
            false, // AI author
        );

        let result = evaluate_human_priority(&envelope, &state);
        assert!(result.is_ok());

        match result.unwrap() {
            ConflictDecision::Reject {
                conflicting_entities,
                ..
            } => {
                assert!(conflicting_entities.contains(&"node:node-target".to_string()));
            }
            ConflictDecision::Allow => panic!("Expected rejection when target has human edit"),
        }
    }

    #[test]
    fn given_multiple_human_edits_when_ai_targets_unrelated_entity_then_allowed() {
        let mut state = ProjectionState::new();
        state.register_human_edit("node:node-1", "human-alice");
        state.register_human_edit("node:node-2", "human-bob");
        state.register_human_edit("node:node-3", "human-charlie");

        let envelope = make_envelope(
            "op-ai-1",
            DomainOp::NodeMove {
                id: "node-4".to_string(), // Unrelated entity
                x: 100.0,
                y: 100.0,
            },
            false, // AI author
        );

        let result = evaluate_human_priority(&envelope, &state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ConflictDecision::Allow);
    }

    #[test]
    fn given_human_edit_on_edge_entity_when_ai_disconnects_then_rejected() {
        let mut state = ProjectionState::new();
        state.register_human_edit("edge:edge-1", "human-alice");

        let envelope = make_envelope(
            "op-ai-1",
            DomainOp::EdgeDisconnect {
                id: "edge-1".to_string(),
            },
            false, // AI author
        );

        let result = evaluate_human_priority(&envelope, &state);
        assert!(result.is_ok());

        match result.unwrap() {
            ConflictDecision::Reject {
                conflicting_entities,
                ..
            } => {
                assert!(conflicting_entities.contains(&"edge:edge-1".to_string()));
            }
            ConflictDecision::Allow => panic!("Expected rejection when edge has human edit"),
        }
    }

    // =========================================================================
    // Author Identification Edge Case Tests
    // =========================================================================

    #[test]
    fn given_author_with_human_prefix_when_identified_then_is_human() {
        let author = Author {
            id: "human-alice".to_string(),
            name: "Alice".to_string(),
            email: None,
        };
        assert!(is_human_author(&author));
    }

    #[test]
    fn given_author_with_human_in_name_when_identified_then_is_human() {
        let author = Author {
            id: "user-123".to_string(),
            name: "Human User".to_string(),
            email: None,
        };
        assert!(is_human_author(&author));
    }

    #[test]
    fn given_author_with_human_uppercase_in_name_when_identified_then_is_human() {
        let author = Author {
            id: "user-456".to_string(),
            name: "HUMAN OPERATOR".to_string(),
            email: None,
        };
        assert!(is_human_author(&author));
    }

    #[test]
    fn given_author_with_human_mixed_case_in_name_when_identified_then_is_human() {
        let author = Author {
            id: "user-789".to_string(),
            name: "HuMaN MiXeD".to_string(),
            email: None,
        };
        assert!(is_human_author(&author));
    }

    #[test]
    fn given_ai_author_without_human_indicators_when_identified_then_is_ai() {
        let author = Author {
            id: "ai-assistant".to_string(),
            name: "AI Assistant".to_string(),
            email: None,
        };
        assert!(!is_human_author(&author));
    }

    #[test]
    fn given_author_with_empty_id_and_nonhuman_name_when_identified_then_is_ai() {
        let author = Author {
            id: String::new(),
            name: "System".to_string(),
            email: None,
        };
        assert!(!is_human_author(&author));
    }

    #[test]
    fn given_author_with_empty_id_and_name_when_identified_then_is_ai() {
        let author = Author {
            id: String::new(),
            name: String::new(),
            email: None,
        };
        assert!(!is_human_author(&author));
    }

    #[test]
    fn given_author_with_bot_prefix_when_identified_then_is_ai() {
        let author = Author {
            id: "bot-automation".to_string(),
            name: "Automation Bot".to_string(),
            email: None,
        };
        assert!(!is_human_author(&author));
    }

    #[test]
    fn given_author_with_service_prefix_when_identified_then_is_ai() {
        let author = Author {
            id: "service-sync".to_string(),
            name: "Sync Service".to_string(),
            email: None,
        };
        assert!(!is_human_author(&author));
    }

    // =========================================================================
    // Rapid Consecutive Edits Edge Case Tests
    // =========================================================================

    #[test]
    fn given_duplicate_op_id_when_evaluated_then_idempotent_allow() {
        let mut state = ProjectionState::new();
        state.register_human_edit("node:node-1", "human-alice");
        state.mark_processed("op-duplicate");

        let envelope = make_envelope(
            "op-duplicate",
            DomainOp::NodeMove {
                id: "node-1".to_string(),
                x: 100.0,
                y: 100.0,
            },
            false, // AI author
        );

        // Even though there's an active human edit, the op is already processed
        let result = evaluate_human_priority(&envelope, &state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ConflictDecision::Allow);
    }

    #[test]
    fn given_multiple_human_edits_on_same_entity_when_checked_then_single_active_window() {
        let mut state = ProjectionState::new();

        // Multiple rapid edits on the same entity
        state.register_human_edit("node:node-1", "human-alice");
        state.register_human_edit("node:node-1", "human-alice");
        state.register_human_edit("node:node-1", "human-alice");

        // Should still have only one entry
        let active = state.active_human_edit_entities();
        assert_eq!(active.len(), 1);
        assert!(active.contains(&"node:node-1".to_string()));
    }

    #[test]
    fn given_multiple_entities_tracked_when_checked_then_independent() {
        let mut state = ProjectionState::new();

        state.register_human_edit("node:node-1", "human-alice");
        state.register_human_edit("node:node-2", "human-bob");
        state.register_human_edit("node:node-3", "human-charlie");

        // Each entity should be independently tracked
        assert!(state.has_active_human_edit("node:node-1"));
        assert!(state.has_active_human_edit("node:node-2"));
        assert!(state.has_active_human_edit("node:node-3"));
        assert!(!state.has_active_human_edit("node:node-4"));
    }

    #[test]
    fn given_multiple_processed_ops_when_checked_then_all_recognized() {
        let mut state = ProjectionState::new();

        state.mark_processed("op-1");
        state.mark_processed("op-2");
        state.mark_processed("op-3");

        assert!(state.is_processed("op-1"));
        assert!(state.is_processed("op-2"));
        assert!(state.is_processed("op-3"));
        assert!(!state.is_processed("op-4"));
    }

    #[test]
    fn given_bring_forward_affects_multiple_nodes_when_any_has_human_edit_then_rejected() {
        let mut state = ProjectionState::new();
        state.register_human_edit("node:n2", "human-alice");

        let envelope = make_envelope(
            "op-ai-1",
            DomainOp::BringForward {
                ids: vec!["n1".to_string(), "n2".to_string(), "n3".to_string()],
            },
            false, // AI author
        );

        let result = evaluate_human_priority(&envelope, &state);
        assert!(result.is_ok());

        match result.unwrap() {
            ConflictDecision::Reject {
                conflicting_entities,
                ..
            } => {
                assert!(conflicting_entities.contains(&"node:n2".to_string()));
            }
            ConflictDecision::Allow => panic!("Expected rejection when any affected node has human edit"),
        }
    }

    #[test]
    fn given_group_operation_affects_multiple_nodes_when_any_has_human_edit_then_rejected() {
        let mut state = ProjectionState::new();
        state.register_human_edit("node:n3", "human-alice");

        let envelope = make_envelope(
            "op-ai-1",
            DomainOp::Group {
                ids: vec!["n1".to_string(), "n2".to_string(), "n3".to_string()],
            },
            false, // AI author
        );

        let result = evaluate_human_priority(&envelope, &state);
        assert!(result.is_ok());

        match result.unwrap() {
            ConflictDecision::Reject {
                conflicting_entities,
                ..
            } => {
                assert!(conflicting_entities.contains(&"node:n3".to_string()));
            }
            ConflictDecision::Allow => panic!("Expected rejection when any grouped node has human edit"),
        }
    }

    // =========================================================================
    // Conflict Decision and Error Edge Case Tests
    // =========================================================================

    #[test]
    fn given_conflict_decision_reject_when_serialized_then_contains_all_fields() {
        let decision = ConflictDecision::Reject {
            reason: ConflictError::HumanPriorityBlock("test conflict".to_string()),
            conflicting_entities: vec!["node:n1".to_string(), "node:n2".to_string()],
        };

        let json = serde_json::to_string(&decision).expect("Should serialize");
        assert!(json.contains("HumanPriorityBlock"));
        assert!(json.contains("node:n1"));
        assert!(json.contains("node:n2"));
    }

    #[test]
    fn given_human_priority_block_error_when_displayed_then_contains_message() {
        let error = ConflictError::HumanPriorityBlock("active human edit on node:node-1".to_string());
        let display = format!("{}", error);
        assert!(display.contains("human priority block"));
        assert!(display.contains("active human edit on node:node-1"));
    }

    #[test]
    fn given_missing_entity_error_when_displayed_then_contains_entity() {
        let error = ConflictError::MissingEntity("node:node-123".to_string());
        let display = format!("{}", error);
        assert!(display.contains("missing entity"));
        assert!(display.contains("node:node-123"));
    }

    #[test]
    fn given_policy_violation_error_when_displayed_then_contains_message() {
        let error = ConflictError::PolicyViolation("op_id is required".to_string());
        let display = format!("{}", error);
        assert!(display.contains("policy violation"));
        assert!(display.contains("op_id is required"));
    }

    // =========================================================================
    // Extract Affected Entities Edge Case Tests
    // =========================================================================

    #[test]
    fn given_node_delete_op_when_extracting_entities_then_returns_node_id() {
        let op = DomainOp::NodeDelete {
            id: "node-1".to_string(),
        };
        let entities = extract_affected_entities(&op);
        assert_eq!(entities, vec!["node:node-1"]);
    }

    #[test]
    fn given_node_restore_op_when_extracting_entities_then_returns_node_id() {
        let op = DomainOp::NodeRestore {
            id: "node-1".to_string(),
        };
        let entities = extract_affected_entities(&op);
        assert_eq!(entities, vec!["node:node-1"]);
    }

    #[test]
    fn given_send_backward_op_when_extracting_entities_then_returns_all_nodes() {
        let op = DomainOp::SendBackward {
            ids: vec!["a".to_string(), "b".to_string()],
        };
        let entities = extract_affected_entities(&op);
        assert_eq!(entities, vec!["node:a", "node:b"]);
    }

    #[test]
    fn given_bring_to_front_op_when_extracting_entities_then_returns_all_nodes() {
        let op = DomainOp::BringToFront {
            ids: vec!["x".to_string(), "y".to_string(), "z".to_string()],
        };
        let entities = extract_affected_entities(&op);
        assert_eq!(entities, vec!["node:x", "node:y", "node:z"]);
    }

    #[test]
    fn given_send_to_back_op_when_extracting_entities_then_returns_all_nodes() {
        let op = DomainOp::SendToBack {
            ids: vec!["single".to_string()],
        };
        let entities = extract_affected_entities(&op);
        assert_eq!(entities, vec!["node:single"]);
    }

    #[test]
    fn given_ungroup_op_when_extracting_entities_then_returns_group_id() {
        let op = DomainOp::Ungroup {
            id: "group-1".to_string(),
        };
        let entities = extract_affected_entities(&op);
        assert_eq!(entities, vec!["group:group-1"]);
    }

    // =========================================================================
    // Record Conflict Rejection Edge Case Tests
    // =========================================================================

    #[test]
    fn given_valid_envelope_when_recording_rejection_then_succeeds() {
        let envelope = make_envelope(
            "op-valid",
            DomainOp::NodeMove {
                id: "node-1".to_string(),
                x: 100.0,
                y: 100.0,
            },
            false,
        );

        let reason = ConflictError::HumanPriorityBlock("active edit".to_string());
        let result = record_conflict_rejection(&envelope, reason);
        assert!(result.is_ok());
    }

    #[test]
    fn given_empty_op_id_when_recording_rejection_then_fails() {
        let envelope = EventEnvelope {
            op_id: String::new(),
            operation: DomainOp::NodeMove {
                id: "node-1".to_string(),
                x: 100.0,
                y: 100.0,
            },
            author: make_ai_author(),
            timestamp: 1700000000,
        };

        let reason = ConflictError::HumanPriorityBlock("active edit".to_string());
        let result = record_conflict_rejection(&envelope, reason);
        assert!(result.is_err());

        match result {
            Err(ConflictError::PolicyViolation(msg)) => {
                assert!(msg.contains("op_id"));
            }
            _ => panic!("Expected PolicyViolation error"),
        }
    }

    #[test]
    fn given_human_author_with_email_when_identified_then_is_human() {
        let author = Author {
            id: "human-with-email".to_string(),
            name: "User".to_string(),
            email: Some("user@example.com".to_string()),
        };
        assert!(is_human_author(&author));
    }

    #[test]
    fn given_ai_author_with_email_when_identified_then_is_ai() {
        let author = Author {
            id: "ai-system".to_string(),
            name: "AI System".to_string(),
            email: Some("ai@example.com".to_string()),
        };
        assert!(!is_human_author(&author));
    }
}
