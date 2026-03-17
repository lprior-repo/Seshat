//! Cycle policy enforcement for diagram projection
//!
//! This module provides cycle policy enforcement for the diagram projection.

#![allow(dead_code)]
#![allow(unused_imports)]

use crate::envelope::DomainOp;

use crate::projection::replay::apply_event;
use crate::projection::types::{CyclePolicy, DiagramProjection, EventRecord, ReplayError};

/// Enforce cycle policy on a diagram projection
///
/// This function checks whether the current projection violates its configured
/// cycle policy. If the policy is `CyclePolicy::Deny` and the graph contains
/// cycles, an error is returned.
///
/// # Errors
/// - Returns `ReplayError::CycleViolation` if:
///   - The cycle policy is `Deny` and the projection contains a cycle
/// - Returns `ReplayError::PolicyMissing` if:
///   - The cycle policy field is not properly initialized (should not happen with default)
pub fn enforce_cycle_policy(state: &DiagramProjection) -> Result<(), ReplayError> {
    match state.cycle_policy {
        CyclePolicy::Allow => Ok(()),
        CyclePolicy::Deny => {
            // Use the DAG validation from the dag module
            crate::dag::validate_dag(&state.nodes, &state.edges)
                .map_err(|e| ReplayError::CycleViolation(e.to_string()))
        }
    }
}

/// Apply a domain operation with cycle policy enforcement
///
/// This function applies an operation to the projection while respecting
/// the configured cycle policy. If the operation would create a cycle and
/// the policy is `Deny`, the operation is rejected.
///
/// # Errors
/// - Returns `ReplayError::CycleViolation` if:
///   - The operation would create a cycle and policy is `Deny`
/// - Returns `ReplayError::InvariantViolation` if:
///   - The operation itself violates an invariant (e.g., duplicate node ID)
/// - Returns `ReplayError::InvalidEvent` if:
///   - The event is malformed
pub fn apply_policy_op(
    state: DiagramProjection,
    op: &DomainOp,
) -> Result<DiagramProjection, ReplayError> {
    // First, apply the operation to get a tentative new state
    let event = EventRecord {
        op_id: format!("policy-op-{}", state.revision),
        revision: state.revision,
        operation: op.clone(),
        author: crate::envelope::Author {
            id: "system".to_string(),
            name: "Policy Enforcer".to_string(),
            email: None,
        },
        timestamp: 0,
    };

    let new_state = apply_event(state, &event)?;

    // Then, enforce the cycle policy on the new state
    enforce_cycle_policy(&new_state)?;

    // If we get here, the operation is valid
    Ok(new_state)
}
