//! Edge operations for diagram projection
//!
//! This module provides functions for applying edge-related operations
//! to a diagram projection.

#![allow(dead_code)]
#![allow(unused_imports)]

use im::HashMap;
use std::collections::HashSet;

use crate::document::{ArrowType, Edge, EdgeId, EdgeStyle, NodeId, OrderedFloat};
use crate::envelope::DomainOp;
use crate::projection::types::{DiagramProjection, ReplayError};

/// Create a default edge with standard settings
pub fn create_default_edge(source: NodeId, target: NodeId) -> Edge {
    Edge {
        source,
        target,
        label: String::new(),
        style: EdgeStyle::Solid,
        arrow_type: ArrowType::Default,
        label_offset_t: OrderedFloat(0.5),
        color: None,
        thickness: OrderedFloat(1.5),
        directed: true,
        bend_points: im::Vector::new(),
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        font_size: None,
        source_port: None,
        target_port: None,
    }
}

/// Core function to perform edge updates by mutating an existing edge
fn update_edge(
    mut state: DiagramProjection,
    id: &str,
    mut f: impl FnMut(&mut Edge),
) -> Result<DiagramProjection, ReplayError> {
    let edge_id = EdgeId::new(id.to_string());
    let mut edge = state
        .edges
        .get(&edge_id)
        .cloned()
        .ok_or_else(|| ReplayError::InvariantViolation(format!("edge not found: {id}")))?;

    f(&mut edge);
    state.edges = state.edges.update(edge_id, edge);
    Ok(state)
}

pub fn apply_update_edge_label(
    state: DiagramProjection,
    id: &str,
    label: &str,
) -> Result<DiagramProjection, ReplayError> {
    update_edge(state, id, |e| e.label = label.to_owned())
}

pub fn apply_update_edge_style(
    state: DiagramProjection,
    id: &str,
    style: EdgeStyle,
) -> Result<DiagramProjection, ReplayError> {
    update_edge(state, id, |e| e.style = style)
}

/// Determines the strictness of validation for edge connections/disconnections
enum Strictness {
    Invariant, // Basic unverified operations
    Policy,    // Operations verified against policy and contracts
}

fn apply_connect(
    mut state: DiagramProjection,
    id: &str,
    source: &str,
    target: &str,
    strictness: Strictness,
) -> Result<DiagramProjection, ReplayError> {
    let edge_id = EdgeId::new(id.to_string());
    let source_id = NodeId::new(source.to_string());
    let target_id = NodeId::new(target.to_string());

    if state.has_edge(&edge_id) {
        return Err(match strictness {
            Strictness::Invariant => {
                ReplayError::InvariantViolation(format!("duplicate edge ID: {id}"))
            }
            Strictness::Policy => ReplayError::DuplicateEdge(id.to_string()),
        });
    }

    for (node_id, kind, raw_id) in [
        (&source_id, "source", source),
        (&target_id, "target", target),
    ] {
        if !state.has_node(node_id) {
            let msg = format!("{kind} node not found: {raw_id}");
            return Err(match strictness {
                Strictness::Invariant => ReplayError::InvariantViolation(msg),
                Strictness::Policy => ReplayError::PolicyViolation(msg),
            });
        }
    }

    state.edges = state
        .edges
        .update(edge_id, create_default_edge(source_id, target_id));
    Ok(state)
}

pub fn apply_edge_connect(
    state: DiagramProjection,
    id: &str,
    source: &str,
    target: &str,
) -> Result<DiagramProjection, ReplayError> {
    apply_connect(state, id, source, target, Strictness::Invariant)
}

pub fn apply_edge_connect_checked(
    state: DiagramProjection,
    id: &str,
    source: &str,
    target: &str,
) -> Result<DiagramProjection, ReplayError> {
    apply_connect(state, id, source, target, Strictness::Policy)
}

fn apply_disconnect(
    mut state: DiagramProjection,
    id: &str,
    strictness: Strictness,
) -> Result<DiagramProjection, ReplayError> {
    let edge_id = EdgeId::new(id.to_string());

    if !state.has_edge(&edge_id) {
        return Err(match strictness {
            Strictness::Invariant => {
                ReplayError::InvariantViolation(format!("edge not found: {id}"))
            }
            Strictness::Policy => ReplayError::EdgeNotFound(id.to_string()),
        });
    }

    state.edges = state.edges.without(&edge_id);
    Ok(state)
}

pub fn apply_edge_disconnect(
    state: DiagramProjection,
    id: &str,
) -> Result<DiagramProjection, ReplayError> {
    apply_disconnect(state, id, Strictness::Invariant)
}

pub fn apply_edge_disconnect_checked(
    state: DiagramProjection,
    id: &str,
) -> Result<DiagramProjection, ReplayError> {
    apply_disconnect(state, id, Strictness::Policy)
}

pub fn apply_edge_op(
    state: DiagramProjection,
    op: &DomainOp,
) -> Result<DiagramProjection, ReplayError> {
    match op {
        DomainOp::EdgeConnect { id, source, target } => {
            apply_edge_connect_checked(state, id.as_str(), source.as_str(), target.as_str())
        }
        DomainOp::EdgeDisconnect { id } => apply_edge_disconnect_checked(state, id.as_str()),
        _ => Err(ReplayError::InvalidEvent(format!(
            "not an edge operation: {:?}",
            op.kind()
        ))),
    }
}

pub fn verify_edge_tolerance(state: &DiagramProjection) -> Result<(), ReplayError> {
    let mut seen_ids = HashSet::new();

    for (edge_id, edge) in state.edges.iter() {
        if !seen_ids.insert(edge_id.to_string()) {
            return Err(ReplayError::DuplicateEdge(edge_id.to_string()));
        }

        for (node_id, kind) in [(&edge.source, "source"), (&edge.target, "target")] {
            if !state.has_node(node_id) {
                return Err(ReplayError::PolicyViolation(format!(
                    "edge {} references non-existent {} node: {}",
                    edge_id, kind, node_id
                )));
            }
        }

        if !edge.label_offset_t.0.is_finite() {
            return Err(ReplayError::InvariantViolation(format!(
                "edge {} has invalid label_offset_t",
                edge_id
            )));
        }
        if !edge.thickness.0.is_finite() {
            return Err(ReplayError::InvariantViolation(format!(
                "edge {} has invalid thickness",
                edge_id
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "edge_ops_tests.rs"]
mod tests;
