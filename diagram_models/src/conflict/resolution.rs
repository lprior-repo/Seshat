//! Conflict resolution
#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use super::{ConflictDecision, ConflictError, ProjectionState};
use crate::envelope::{Author, DomainOp, EventEnvelope, LabelTargetType};
use crate::projection::DiagramProjection;

fn is_human_author(author: &Author) -> bool {
    author.id.starts_with("human-") || author.name.to_lowercase().contains("human")
}

fn extract_affected_entities(op: &DomainOp) -> Vec<String> {
    match op {
        DomainOp::NodeAdd { id, .. }
        | DomainOp::NodeMove { id, .. }
        | DomainOp::NodeDelete { id }
        | DomainOp::NodeRestore { id }
        | DomainOp::UpdateNodeStyle { id, .. } => vec![format!("node:{}", id)],
        DomainOp::UpdateLabel {
            target_id,
            target_type,
            ..
        } => match target_type {
            LabelTargetType::Node => vec![format!("node:{}", target_id)],
            LabelTargetType::Edge => vec![format!("edge:{}", target_id)],
        },
        DomainOp::NodeResize { id, .. } => vec![format!("node:{}", id.as_str())],
        DomainOp::EdgeConnect { id, source, target } => vec![
            format!("edge:{}", id),
            format!("node:{}", source),
            format!("node:{}", target),
        ],
        DomainOp::EdgeDisconnect { id } | DomainOp::UpdateEdgeStyle { id, .. } => {
            vec![format!("edge:{}", id)]
        }
        DomainOp::BringForward { ids }
        | DomainOp::SendBackward { ids }
        | DomainOp::BringToFront { ids }
        | DomainOp::SendToBack { ids } => ids.iter().map(|id| format!("node:{}", id)).collect(),
        DomainOp::Group { id, ids } => {
            let mut entities: Vec<String> = ids.iter().map(|id| format!("node:{}", id)).collect();
            entities.push(format!("node:{}", id));
            entities
        }
        DomainOp::Ungroup { id } => vec![format!("group:{}", id)],
    }
}

pub fn evaluate_human_priority(
    op: &EventEnvelope,
    state: &ProjectionState,
) -> Result<ConflictDecision, ConflictError> {
    if is_human_author(&op.author) {
        return Ok(ConflictDecision::Allow);
    }
    if state.is_processed(&op.op_id) {
        return Ok(ConflictDecision::Allow);
    }
    let affected_entities = extract_affected_entities(&op.operation);
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

pub fn evaluate_human_priority_with_projection(
    op: &EventEnvelope,
    state: &ProjectionState,
    projection: &DiagramProjection,
) -> Result<ConflictDecision, ConflictError> {
    let decision = evaluate_human_priority(op, state)?;
    if decision == ConflictDecision::Allow {
        match &op.operation {
            DomainOp::NodeMove { id, .. }
            | DomainOp::NodeDelete { id }
            | DomainOp::NodeRestore { id } => {
                if !projection.has_node(id) {
                    return Err(ConflictError::MissingEntity(format!("node:{}", id)));
                }
            }
            DomainOp::EdgeDisconnect { id } => {
                if !projection.has_edge(id) {
                    return Err(ConflictError::MissingEntity(format!("edge:{}", id)));
                }
            }
            DomainOp::EdgeConnect { source, target, .. } => {
                if !projection.has_node(source) {
                    return Err(ConflictError::MissingEntity(format!("node:{}", source)));
                }
                if !projection.has_node(target) {
                    return Err(ConflictError::MissingEntity(format!("node:{}", target)));
                }
            }
            _ => {}
        }
    }
    Ok(decision)
}

pub fn record_conflict_rejection(
    state: &mut ProjectionState,
    op: &EventEnvelope,
    _decision: &ConflictDecision,
) {
    state.mark_processed(&op.op_id);
    if is_human_author(&op.author) {
        let entities = extract_affected_entities(&op.operation);
        for entity in entities {
            state.register_human_edit(&entity, &op.author.id);
        }
    }
    state.cleanup_expired();
}
