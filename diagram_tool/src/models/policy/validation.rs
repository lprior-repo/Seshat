//! Policy validation
#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::models::envelope::{Author, DomainOp};
use super::{CyclePolicy, DiagramProjection, EventRecord, PolicyError};

pub fn enforce_cycle_policy(state: &DiagramProjection) -> Result<(), PolicyError> {
    match state.cycle_policy {
        CyclePolicy::Allow => Ok(()),
        CyclePolicy::Deny => crate::models::dag::validate_dag(&state.nodes, &state.edges).map_err(|e| PolicyError::CycleViolation(e.to_string())),
    }
}

pub fn apply_policy_op(state: DiagramProjection, op: &DomainOp) -> Result<DiagramProjection, PolicyError> {
    let event = EventRecord { op_id: format!("policy-op-{}", state.revision), revision: state.revision, operation: op.clone(), author: Author { id: "system".to_string(), name: "Policy Enforcer".to_string(), email: None }, timestamp: 0 };
    let new_state = apply_event_internal(state, &event)?;
    enforce_cycle_policy(&new_state)?;
    Ok(new_state)
}

fn apply_event_internal(state: DiagramProjection, event: &EventRecord) -> Result<DiagramProjection, PolicyError> {
    if event.revision != state.revision { return Err(PolicyError::InvariantViolation(format!("revision mismatch: state has {}, event has {}", state.revision, event.revision))); }
    let new_state = apply_operation_internal(state, event)?;
    let new_revision = new_state.revision + 1;
    let is_human = event.author.id.starts_with("human-") || event.author.name.to_lowercase().contains("human");
    let mut new_priority_map = new_state.author_priority.clone();
    let old_value = new_priority_map.insert(event.op_id.clone(), is_human);
    if old_value.is_some() { return Err(PolicyError::InvariantViolation(format!("duplicate op_id: {}", event.op_id))); }
    Ok(DiagramProjection { version: new_state.version, revision: new_revision, nodes: new_state.nodes, edges: new_state.edges, author_priority: new_priority_map, cycle_policy: new_state.cycle_policy })
}

fn apply_operation_internal(state: DiagramProjection, event: &EventRecord) -> Result<DiagramProjection, PolicyError> {
    match &event.operation {
        DomainOp::NodeAdd { id, x, y, width, height, label } => {
            let node_id = crate::models::document::NodeId(id.to_string());
            if state.nodes.contains_key(&node_id) { return Err(PolicyError::InvariantViolation(format!("node {} already exists", id))); }
            let node = crate::models::document::Node { id: node_id.clone(), label: label.clone(), x: crate::models::document::Coord(*x), y: crate::models::document::Coord(*y), width: crate::models::document::Coord(*width), height: crate::models::document::Coord(*height), style: None };
            let mut new_state = state; new_state.nodes.insert(node_id, node); new_state.revision += 1; Ok(new_state)
        }
        DomainOp::NodeMove { id, x, y } => {
            let node_id = crate::models::document::NodeId(id.to_string());
            let node = state.nodes.get_mut(&node_id).ok_or_else(|| PolicyError::InvariantViolation(format!("node {} not found", id)))?;
            node.x = crate::models::document::Coord(*x); node.y = crate::models::document::Coord(*y);
            let mut new_state = state; new_state.revision += 1; Ok(new_state)
        }
        DomainOp::NodeDelete { id } => { let mut new_state = state; new_state.nodes.remove(&crate::models::document::NodeId(id.to_string())); new_state.revision += 1; Ok(new_state) }
        DomainOp::NodeRestore { id } => { let mut new_state = state; new_state.revision += 1; Ok(new_state) }
        DomainOp::EdgeConnect { id, source, target } => {
            let edge_id = crate::models::document::EdgeId(id.to_string());
            let source_id = crate::models::document::NodeId(source.to_string());
            let target_id = crate::models::document::NodeId(target.to_string());
            if !state.nodes.contains_key(&source_id) { return Err(PolicyError::InvariantViolation(format!("source node {} not found", source))); }
            if !state.nodes.contains_key(&target_id) { return Err(PolicyError::InvariantViolation(format!("target node {} not found", target))); }
            let edge = crate::models::document::Edge { id: edge_id.clone(), source: source_id, target: target_id, style: None };
            let mut new_state = state; new_state.edges.insert(edge_id, edge); new_state.revision += 1; Ok(new_state)
        }
        DomainOp::EdgeDisconnect { id } => { let mut new_state = state; new_state.edges.remove(&crate::models::document::EdgeId(id.to_string())); new_state.revision += 1; Ok(new_state) }
        DomainOp::BringForward { .. } | DomainOp::SendBackward { .. } | DomainOp::BringToFront { .. } | DomainOp::SendToBack { .. } | DomainOp::Group { .. } | DomainOp::Ungroup { .. } => { let mut new_state = state; new_state.revision += 1; Ok(new_state) }
    }
}
