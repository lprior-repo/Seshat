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
        self.human_edit_windows
            .retain(|_, w| w.is_active());
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
        | DomainOp::Group { ids } => {
            ids.iter().map(|id| format!("node:{}", id)).collect()
        }
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
        op.op_id,
        op.author.id,
        reason
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
        let parsed: ConflictDecision =
            serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(decision, parsed);
    }

    #[test]
    fn given_conflict_error_serialization_then_roundtrips() {
        let error = ConflictError::HumanPriorityBlock("test error".to_string());

        let json = serde_json::to_string(&error).expect("Should serialize");
        let parsed: ConflictError = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(error, parsed);
    }
}
