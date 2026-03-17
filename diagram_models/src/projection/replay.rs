//! Event replay functions for diagram projection
//!
//! This module provides deterministic replay of events to produce a consistent `DiagramProjection`.

#![allow(dead_code)]
#![allow(unused_imports)]

use crate::envelope::{DomainOp, LabelTargetId, LabelTargetType};

use crate::projection::ops::{
    apply_bring_forward, apply_bring_to_front, apply_edge_connect, apply_edge_disconnect,
    apply_group, apply_node_add, apply_node_delete, apply_node_move, apply_node_resize,
    apply_node_restore, apply_send_backward, apply_send_to_back, apply_ungroup,
    apply_update_edge_label, apply_update_edge_style, apply_update_label, apply_update_node_style,
};
use crate::projection::types::{DiagramProjection, EventRecord, ReplayError};

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
    validate_revision_sequence(initial_state.revision, events)?;

    // Fold over events to produce final projection
    events.iter().try_fold(initial_state, apply_event)
}

/// Validate that event revisions form a continuous sequence
fn validate_revision_sequence(
    initial_revision: u64,
    events: &[EventRecord],
) -> Result<(), ReplayError> {
    let mut expected_revision = initial_revision;
    for event in events {
        if event.revision != expected_revision {
            return Err(ReplayError::InvariantViolation(format!(
                "revision gap: expected {}, found {}",
                expected_revision, event.revision
            )));
        }
        expected_revision += 1;
    }
    Ok(())
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
    validate_event_revision(&state, event)?;

    // Apply the domain operation
    let new_state = apply_operation(state, event)?;

    // Build final projection with incremented revision and updated priority map
    build_event_result(new_state, event)
}

/// Validate event revision matches state revision
fn validate_event_revision(
    state: &DiagramProjection,
    event: &EventRecord,
) -> Result<(), ReplayError> {
    if event.revision != state.revision {
        return Err(ReplayError::InvariantViolation(format!(
            "revision mismatch: state has {}, event has {}",
            state.revision, event.revision
        )));
    }
    Ok(())
}

/// Build result after applying event
fn build_event_result(
    new_state: DiagramProjection,
    event: &EventRecord,
) -> Result<DiagramProjection, ReplayError> {
    let new_revision = new_state.revision + 1;
    let new_priority_map = update_priority_map(&new_state, event)?;
    build_projection_result(new_state, new_revision, new_priority_map)
}

/// Update priority map with event author
fn update_priority_map(
    state: &DiagramProjection,
    event: &EventRecord,
) -> Result<im::HashMap<String, bool>, ReplayError> {
    let is_human = crate::projection::types::is_human_author(&event.author);
    let mut new_priority_map = state.author_priority.clone();
    let old_value = new_priority_map.insert(event.op_id.clone(), is_human);

    if old_value.is_some() {
        return Err(ReplayError::InvariantViolation(format!(
            "duplicate op_id: {}",
            event.op_id
        )));
    }
    Ok(new_priority_map)
}

/// Build projection result with new revision and priority map
fn build_projection_result(
    state: DiagramProjection,
    new_revision: u64,
    new_priority_map: im::HashMap<String, bool>,
) -> Result<DiagramProjection, ReplayError> {
    Ok(DiagramProjection {
        version: state.version,
        revision: new_revision,
        nodes: state.nodes,
        edges: state.edges,
        author_priority: new_priority_map,
        cycle_policy: state.cycle_policy,
    })
}

/// Apply a domain operation to the projection
pub fn apply_operation(
    state: DiagramProjection,
    event: &EventRecord,
) -> Result<DiagramProjection, ReplayError> {
    dispatch_operation(state, &event.operation)
}

/// Dispatch operation to the appropriate handler
fn dispatch_operation(
    state: DiagramProjection,
    op: &DomainOp,
) -> Result<DiagramProjection, ReplayError> {
    match op {
        DomainOp::NodeAdd {
            id,
            x,
            y,
            width,
            height,
            label,
        } => apply_node_add(state, id.as_str(), *x, *y, *width, *height, label),
        DomainOp::NodeMove { id, x, y } => apply_node_move(state, id.as_str(), *x, *y),
        DomainOp::NodeDelete { id } => apply_node_delete(state, id.as_str()),
        DomainOp::NodeRestore { id } => apply_node_restore(state, id.as_str()),
        DomainOp::NodeResize {
            id,
            x,
            y,
            width,
            height,
            ..
        } => apply_node_resize(state, id, *x, *y, *width, *height).map_err(
            |e: crate::projection::types::ProjectionError| ReplayError::InvalidEvent(e.to_string()),
        ),
        DomainOp::UpdateLabel {
            target_id,
            target_type,
            old_label: _,
            new_label,
        } => match target_type {
            LabelTargetType::Node => {
                if let LabelTargetId::Node(node_id) = target_id {
                    apply_update_label(state, node_id.as_str(), new_label)
                } else {
                    Err(ReplayError::InvalidEvent("Expected Node target".into()))
                }
            }
            LabelTargetType::Edge => {
                if let LabelTargetId::Edge(edge_id) = target_id {
                    apply_update_edge_label(state, edge_id.as_str(), new_label)
                } else {
                    Err(ReplayError::InvalidEvent("Expected Edge target".into()))
                }
            }
        },
        DomainOp::UpdateNodeStyle { id, style } => {
            apply_update_node_style(state, id.as_str(), style.clone())
        }
        DomainOp::EdgeConnect { id, source, target } => {
            apply_edge_connect(state, id.as_str(), source.as_str(), target.as_str())
        }
        DomainOp::EdgeDisconnect { id } => apply_edge_disconnect(state, id.as_str()),
        DomainOp::UpdateEdgeStyle { id, style } => {
            apply_update_edge_style(state, id.as_str(), *style)
        }
        DomainOp::BringForward { ids } => apply_bring_forward(state, ids),
        DomainOp::SendBackward { ids } => apply_send_backward(state, ids),
        DomainOp::BringToFront { ids } => apply_bring_to_front(state, ids),
        DomainOp::SendToBack { ids } => apply_send_to_back(state, ids),
        DomainOp::Group { id, ids } => apply_group(state, id, ids),
        DomainOp::Ungroup { id } => apply_ungroup(state, id),
    }
}

/// Replay a stream of events to produce a diagram projection
///
/// This is the contract-specified entry point for deterministic replay.
/// It is an alias for `replay_events` to match the contract signature:
/// `fn replay_stream(events: &[EventRecord]) -> Result<DiagramProjection, ReplayError>`
///
/// # Errors
/// Returns `ReplayError` if:
/// - An event is invalid (`InvalidEvent`)
/// - An invariant is violated (`InvariantViolation`)
/// - Schema version is unsupported (`UnsupportedVersion`)
pub fn replay_stream(events: &[EventRecord]) -> Result<DiagramProjection, ReplayError> {
    replay_events(events)
}
