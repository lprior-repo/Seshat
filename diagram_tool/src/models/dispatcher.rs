//! Event Replay & Dispatch Engine
//!
//! This module provides the deterministic event replay functionality.
//! It handles event iteration, validation, and routing to domain operation handlers.
//!
//! ## Design by Contract
//!
//! ### Preconditions
//! - P1: `replay_events(events)` - events slice must not be null (can be empty)
//! - P2: `replay_events_from(initial_state, events)` - initial_state must be a valid DiagramProjection
//! - P3: `apply_event(state, event)` - event.revision must equal state.revision
//! - P4: `apply_event(state, event)` - event.op_id must not have been processed before (idempotency)
//! - P5: All events in sequence must have sequential revisions starting from initial state's revision
//!
//! ### Postconditions
//! - Q1: `replay_events(events)` returns a DiagramProjection with revision equal to number of events
//! - Q2: `replay_events_from(initial_state, events)` returns projection with revision = initial_state.revision + events.len()
//! - Q3: `apply_event` increments revision by exactly 1
//! - Q4: All operations in events are applied in order (deterministic)
//! - Q5: Author priority map is updated for each event with is_human flag
//!
//! ### Invariants
//! - I1: After replay, all edges reference valid source and target nodes
//! - I2: After replay, no duplicate node or edge IDs exist
//! - I3: Projection revision is always equal to initial_revision + number_of_events_processed

#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]
#![allow(dead_code)]

use crate::models::projection::{
    apply_operation, is_human_author, DiagramProjection, EventRecord, ReplayError,
};

/// Replay a sequence of events to produce a diagram projection
///
/// This is the main entry point for deterministic replay.
/// Each event is applied in order, with revision incrementing by exactly one.
///
/// # Errors
/// Returns `ReplayError` if:
/// - An event is invalid (`InvalidEvent`)
/// - An invariant is violated (`InvariantViolation`)
/// - Schema version is unsupported (`UnsupportedVersion`)
pub fn replay_events(events: &[EventRecord]) -> Result<DiagramProjection, ReplayError> {
    replay_events_from(DiagramProjection::empty(), events)
}

/// Replay events starting from a given initial projection state
///
/// This function validates that the event revisions form a continuous sequence
/// starting from the initial state's revision.
///
/// # Errors
/// Returns `ReplayError` if:
/// - An event is invalid (`InvalidEvent`)
/// - An invariant is violated (`InvariantViolation`)
/// - Schema version is unsupported (`UnsupportedVersion`)
pub fn replay_events_from(
    initial_state: DiagramProjection,
    events: &[EventRecord],
) -> Result<DiagramProjection, ReplayError> {
    // Validate revision sequence starts from initial state's revision
    let mut expected_revision = initial_state.revision;
    for event in events {
        if event.revision != expected_revision {
            return Err(ReplayError::InvariantViolation(format!(
                "revision gap: expected {}, found {}",
                expected_revision, event.revision
            )));
        }
        expected_revision += 1;
    }

    // Fold over events to produce final projection
    events.iter().try_fold(initial_state, apply_event)
}

/// Apply a single event to the projection, returning a new projection
///
/// This function is pure and deterministic. It validates the event,
/// applies the operation, and returns the updated projection with revision incremented by one.
///
/// # Errors
/// Returns `ReplayError` if:
/// - The event is invalid (`InvalidEvent`)
/// - The operation violates an invariant (`InvariantViolation`)
/// - The schema version is unsupported (`UnsupportedVersion`)
pub fn apply_event(
    state: DiagramProjection,
    event: &EventRecord,
) -> Result<DiagramProjection, ReplayError> {
    // Validate: event revision should match current state revision
    if event.revision != state.revision {
        return Err(ReplayError::InvariantViolation(format!(
            "revision mismatch: state has {}, event has {}",
            state.revision, event.revision
        )));
    }

    // Apply the domain operation
    let new_state = apply_operation(state, event)?;

    // Increment revision by exactly one
    let new_revision = new_state.revision + 1;

    // Update author priority map - clone to avoid mut
    let is_human = is_human_author(&event.author);
    let new_priority_map = new_state.author_priority.clone();
    let old_value = new_priority_map.get(&event.op_id).copied();

    // Verify idempotency: if this op_id was already processed, we should get the same result
    if old_value.is_some() {
        return Err(ReplayError::InvariantViolation(format!(
            "duplicate op_id: {}",
            event.op_id
        )));
    }

    // Insert the new author priority
    let mut updated_priority_map = new_priority_map;
    updated_priority_map.insert(event.op_id.clone(), is_human);

    Ok(DiagramProjection {
        version: new_state.version,
        revision: new_revision,
        nodes: new_state.nodes,
        edges: new_state.edges,
        author_priority: updated_priority_map,
        cycle_policy: new_state.cycle_policy,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::envelope::{Author, DomainOp};

    fn make_author(is_human: bool) -> Author {
        if is_human {
            Author {
                id: "human-1".to_string(),
                name: "Alice".to_string(),
                email: None,
            }
        } else {
            Author {
                id: "ai-1".to_string(),
                name: "AI Assistant".to_string(),
                email: None,
            }
        }
    }

    fn make_event(op_id: &str, revision: u64, operation: DomainOp, is_human: bool) -> EventRecord {
        EventRecord {
            op_id: op_id.to_string(),
            revision,
            operation,
            author: make_author(is_human),
            timestamp: 1700000000,
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_empty_events_when_replaying_then_returns_empty_projection() {
        let events: &[EventRecord] = &[];
        let result = replay_events(events);

        assert!(result.is_ok());
        let projection = result.unwrap();
        assert_eq!(projection.revision, 0);
        assert!(projection.nodes.is_empty());
        assert!(projection.edges.is_empty());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_single_node_add_when_replaying_then_includes_node() {
        let events = [make_event(
            "op-1",
            0,
            DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 100.0,
                y: 200.0,
                width: 80.0,
                height: 40.0,
                label: "Test Node".to_string(),
            },
            true,
        )];

        let result = replay_events(&events);

        assert!(result.is_ok(), "Error: {:?}", result.err());
        let projection = result.unwrap();
        assert_eq!(projection.revision, 1);
        assert_eq!(projection.nodes.len(), 1);
        assert!(projection
            .nodes
            .contains_key(&crate::models::document::NodeId::new("node-1".to_string())));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_multiple_events_when_replaying_then_increments_revision() {
        let events = [
            make_event(
                "op-1",
                0,
                DomainOp::NodeAdd {
                    id: "node-1".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "Node 1".to_string(),
                },
                true,
            ),
            make_event(
                "op-2",
                1,
                DomainOp::NodeAdd {
                    id: "node-2".to_string(),
                    x: 100.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "Node 2".to_string(),
                },
                true,
            ),
            make_event(
                "op-3",
                2,
                DomainOp::EdgeConnect {
                    id: "edge-1".to_string(),
                    source: "node-1".to_string(),
                    target: "node-2".to_string(),
                },
                true,
            ),
        ];

        let result = replay_events(&events);

        assert!(result.is_ok());
        let projection = result.unwrap();
        assert_eq!(projection.revision, 3);
        assert_eq!(projection.nodes.len(), 2);
        assert_eq!(projection.edges.len(), 1);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_revision_gap_when_replaying_then_returns_error() {
        let events = [
            make_event(
                "op-1",
                0,
                DomainOp::NodeAdd {
                    id: "node-1".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "Node 1".to_string(),
                },
                true,
            ),
            // Skip revision 1 - gap!
            make_event(
                "op-2",
                2,
                DomainOp::NodeAdd {
                    id: "node-2".to_string(),
                    x: 100.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "Node 2".to_string(),
                },
                true,
            ),
        ];

        let result = replay_events(&events);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ReplayError::InvariantViolation(_)));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_duplicate_node_id_when_replaying_then_returns_error() {
        let events = [
            make_event(
                "op-1",
                0,
                DomainOp::NodeAdd {
                    id: "node-1".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "Node 1".to_string(),
                },
                true,
            ),
            make_event(
                "op-2",
                1,
                DomainOp::NodeAdd {
                    id: "node-1".to_string(), // Duplicate!
                    x: 100.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "Node 1 Duplicate".to_string(),
                },
                true,
            ),
        ];

        let result = replay_events(&events);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ReplayError::InvariantViolation(_)));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_revision_mismatch_when_apply_event_then_returns_error() {
        let state = DiagramProjection::with_revision(5);
        let event = make_event(
            "op-1",
            3, // Mismatch! State is at 5
            DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 40.0,
                label: "Node 1".to_string(),
            },
            true,
        );

        let result = apply_event(state, &event);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ReplayError::InvariantViolation(_)));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_duplicate_op_id_when_apply_event_then_returns_error() {
        let mut state = DiagramProjection::empty();
        state.author_priority.insert("op-1".to_string(), true);
        let event = make_event(
            "op-1", // Duplicate!
            0,
            DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 40.0,
                label: "Node 1".to_string(),
            },
            true,
        );

        let result = apply_event(state, &event);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ReplayError::InvariantViolation(_)));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_ai_author_when_apply_event_then_sets_human_false() {
        let state = DiagramProjection::empty();
        let event = make_event(
            "op-1",
            0,
            DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 40.0,
                label: "Node 1".to_string(),
            },
            false, // AI author
        );

        let result = apply_event(state, &event);

        assert!(result.is_ok());
        let projection = result.unwrap();
        assert_eq!(projection.author_priority.get("op-1"), Some(&false));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_human_author_when_apply_event_then_sets_human_true() {
        let state = DiagramProjection::empty();
        let event = make_event(
            "op-1",
            0,
            DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 40.0,
                label: "Node 1".to_string(),
            },
            true, // Human author
        );

        let result = apply_event(state, &event);

        assert!(result.is_ok());
        let projection = result.unwrap();
        assert_eq!(projection.author_priority.get("op-1"), Some(&true));
    }
}
