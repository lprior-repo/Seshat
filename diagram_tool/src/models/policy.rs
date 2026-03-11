//! Policy module - Graph cycle policy enforcement
//!
//! This module isolates the graph traversal algorithm and policy enforcement rules
//! from the projection replay logic.
//!
//! ## Design by Contract
//!
//! ### Preconditions
//! - P1: `enforce_cycle_policy` requires a valid `DiagramProjection` with initialized `cycle_policy` field
//! - P2: `apply_policy_op` requires valid state and non-null operation
//!
//! ### Postconditions
//! - Q1: `enforce_cycle_policy` returns `Ok(())` when policy is `Allow` regardless of graph state
//! - Q2: `enforce_cycle_policy` returns `Err(CycleViolation)` when policy is `Deny` and cycle detected
//! - Q3: `apply_policy_op` returns new state when operation succeeds
//! - Q4: `apply_policy_op` returns error when operation would violate policy
//!
//! ### Invariants
//! - I1: `CyclePolicy` is always either `Allow` or `Deny` (no invalid state)
//! - I2: `enforce_cycle_policy` does not modify the projection state

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use im::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::models::document::{EdgeId, NodeId};
use crate::models::envelope::{Author, DomainOp};

/// Errors that can occur during policy enforcement
#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PolicyError {
    #[error("cycle violation: {0}")]
    CycleViolation(String),
    #[error("policy missing: {0}")]
    PolicyMissing(String),
    #[error("policy violation: {0}")]
    PolicyViolation(String),
    #[error("invalid event: {0}")]
    InvalidEvent(String),
    #[error("invariant violation: {0}")]
    InvariantViolation(String),
}

/// Cycle policy for a diagram
///
/// This enum defines whether a diagram allows or denies cycles in its edge graph.
/// When set to `Deny`, any operation that would create a cycle is rejected.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CyclePolicy {
    /// Cycles are allowed in the diagram (default)
    #[default]
    Allow,
    /// Cycles are denied - operations creating cycles are rejected
    Deny,
}

/// Diagram projection - the result of replaying events
///
/// This is a pure data structure representing the complete diagram state
/// after replaying a sequence of events.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiagramProjection {
    /// Schema version for compatibility checking
    pub version: u32,
    /// Current revision number
    pub revision: u64,
    /// All nodes in the diagram
    pub nodes: HashMap<NodeId, crate::models::document::Node>,
    /// All edges in the diagram
    pub edges: HashMap<EdgeId, crate::models::document::Edge>,
    /// Author priority map: `op_id` -> `is_human`
    /// Human-authored operations take priority over AI in conflicts
    #[serde(default)]
    pub author_priority: HashMap<String, bool>,
    /// Cycle policy for the diagram - whether cycles are allowed or denied
    #[serde(default)]
    pub cycle_policy: CyclePolicy,
}

impl Default for DiagramProjection {
    fn default() -> Self {
        Self::empty()
    }
}

impl DiagramProjection {
    /// Create an empty projection
    #[must_use]
    pub fn empty() -> Self {
        Self {
            version: 2,
            revision: 0,
            nodes: HashMap::new(),
            edges: HashMap::new(),
            author_priority: HashMap::new(),
            cycle_policy: CyclePolicy::default(),
        }
    }

    /// Create a new projection with a specific cycle policy
    #[must_use]
    pub fn with_cycle_policy(cycle_policy: CyclePolicy) -> Self {
        Self {
            version: 2,
            revision: 0,
            nodes: HashMap::new(),
            edges: HashMap::new(),
            author_priority: HashMap::new(),
            cycle_policy,
        }
    }

    /// Check if a node exists
    #[must_use]
    pub fn has_node(&self, id: &NodeId) -> bool {
        self.nodes.contains_key(id)
    }

    /// Check if an edge exists
    #[must_use]
    pub fn has_edge(&self, id: &EdgeId) -> bool {
        self.edges.contains_key(id)
    }
}

/// Event record for replay - contains all information needed to reconstruct state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventRecord {
    /// Unique operation identifier (for idempotency)
    pub op_id: String,
    /// Revision number of this event
    pub revision: u64,
    /// The domain operation to apply
    pub operation: DomainOp,
    /// Author who created this event
    pub author: Author,
    /// Timestamp of the event (Unix timestamp)
    pub timestamp: i64,
}

/// Enforce cycle policy on a diagram projection
///
/// This function checks whether the current projection violates its configured
/// cycle policy. If the policy is `CyclePolicy::Deny` and the graph contains
/// cycles, an error is returned.
///
/// # Errors
/// - Returns `PolicyError::CycleViolation` if:
///   - The cycle policy is `Deny` and the projection contains a cycle
/// - Returns `PolicyError::PolicyMissing` if:
///   - The cycle policy field is not properly initialized (should not happen with default)
///
/// # Example
/// ```ignore
/// let projection = DiagramProjection::with_cycle_policy(CyclePolicy::Deny);
/// // Add nodes and edges...
/// enforce_cycle_policy(&projection)?; // Returns error if cycle detected
/// ```
pub fn enforce_cycle_policy(state: &DiagramProjection) -> Result<(), PolicyError> {
    match state.cycle_policy {
        CyclePolicy::Allow => Ok(()),
        CyclePolicy::Deny => {
            // Use the DAG validation from the dag module
            crate::models::dag::validate_dag(&state.nodes, &state.edges)
                .map_err(|e| PolicyError::CycleViolation(e.to_string()))
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
/// - Returns `PolicyError::CycleViolation` if:
///   - The operation would create a cycle and policy is `Deny`
/// - Returns `PolicyError::InvariantViolation` if:
///   - The operation itself violates an invariant (e.g., duplicate node ID)
/// - Returns `PolicyError::InvalidEvent` if:
///   - The event is malformed
///
/// # Example
/// ```ignore
/// let projection = DiagramProjection::with_cycle_policy(CyclePolicy::Deny);
/// let op = DomainOp::EdgeConnect {
///     id: "e1".to_string(),
///     source: "a".to_string(),
///     target: "b".to_string(),
/// };
/// let new_projection = apply_policy_op(projection, &op)?;
/// ```
pub fn apply_policy_op(
    state: DiagramProjection,
    op: &DomainOp,
) -> Result<DiagramProjection, PolicyError> {
    // First, apply the operation to get a tentative new state
    let event = EventRecord {
        op_id: format!("policy-op-{}", state.revision),
        revision: state.revision,
        operation: op.clone(),
        author: crate::models::envelope::Author {
            id: "system".to_string(),
            name: "Policy Enforcer".to_string(),
            email: None,
        },
        timestamp: 0,
    };

    let new_state = apply_event_internal(state, &event)?;

    // Then, enforce the cycle policy on the new state
    enforce_cycle_policy(&new_state)?;

    // If we get here, the operation is valid
    Ok(new_state)
}

/// Internal event application - applies a single event to the projection
fn apply_event_internal(
    state: DiagramProjection,
    event: &EventRecord,
) -> Result<DiagramProjection, PolicyError> {
    // Validate: event revision should match current state revision
    if event.revision != state.revision {
        return Err(PolicyError::InvariantViolation(format!(
            "revision mismatch: state has {}, event has {}",
            state.revision, event.revision
        )));
    }

    // Apply the domain operation
    let new_state = apply_operation_internal(state, event)?;

    // Increment revision by exactly one
    let new_revision = new_state.revision + 1;

    // Update author priority map - clone to avoid mut
    let is_human =
        event.author.id.starts_with("human-") || event.author.name.to_lowercase().contains("human");
    let mut new_priority_map = new_state.author_priority.clone();
    let old_value = new_priority_map.insert(event.op_id.clone(), is_human);

    // Verify idempotency: if this op_id was already processed, we should get the same result
    if old_value.is_some() {
        return Err(PolicyError::InvariantViolation(format!(
            "duplicate op_id: {}",
            event.op_id
        )));
    }

    Ok(DiagramProjection {
        version: new_state.version,
        revision: new_revision,
        nodes: new_state.nodes,
        edges: new_state.edges,
        author_priority: new_priority_map,
        cycle_policy: new_state.cycle_policy,
    })
}

/// Apply a domain operation to the projection
fn apply_operation_internal(
    state: DiagramProjection,
    event: &EventRecord,
) -> Result<DiagramProjection, PolicyError> {
    match &event.operation {
        // Node operations
        DomainOp::NodeAdd {
            id,
            x,
            y,
            width,
            height,
            label,
        } => apply_node_add(state, id, *x, *y, *width, *height, label),
        DomainOp::NodeMove { id, x, y } => apply_node_move(state, id, *x, *y),
        DomainOp::NodeDelete { id } => apply_node_delete(state, id),
        DomainOp::NodeRestore { id } => apply_node_restore(state, id),
        // Edge operations
        DomainOp::EdgeConnect { id, source, target } => {
            apply_edge_connect(state, id, source, target)
        }
        DomainOp::EdgeDisconnect { id } => apply_edge_disconnect(state, id),
        // Z-order operations
        DomainOp::BringForward { ids } => apply_bring_forward(state, ids),
        DomainOp::SendBackward { ids } => apply_send_backward(state, ids),
        DomainOp::BringToFront { ids } => apply_bring_to_front(state, ids),
        DomainOp::SendToBack { ids } => apply_send_to_back(state, ids),
        // Composite operations
        DomainOp::Group { ids } => apply_group(state, ids),
        DomainOp::Ungroup { id } => apply_ungroup(state, id),
    }
}

/// Apply `NodeAdd` operation
fn apply_node_add(
    state: DiagramProjection,
    id: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    label: &str,
) -> Result<DiagramProjection, PolicyError> {
    let node_id = NodeId::new(id.to_string());

    // Check for duplicate node ID
    if state.has_node(&node_id) {
        return Err(PolicyError::InvariantViolation(format!(
            "duplicate node ID: {id}"
        )));
    }

    let node = crate::models::document::Node {
        kind: crate::models::document::NodeKind::Node,
        icon: String::new(),
        label: label.to_string(),
        x: crate::models::document::OrderedFloat(x),
        y: crate::models::document::OrderedFloat(y),
        width: crate::models::document::OrderedFloat(width),
        height: crate::models::document::OrderedFloat(height),
        font_size: None,
        font_weight: None,
        locked: false,
        parent: None,
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        z_index: 0,
        style: None,
        collapsed: None,
    };

    let new_nodes = state.nodes.update(node_id, node);

    Ok(DiagramProjection {
        version: state.version,
        revision: state.revision,
        nodes: new_nodes,
        edges: state.edges,
        author_priority: state.author_priority,
        cycle_policy: state.cycle_policy,
    })
}

/// Apply `NodeMove` operation
fn apply_node_move(
    state: DiagramProjection,
    id: &str,
    x: f64,
    y: f64,
) -> Result<DiagramProjection, PolicyError> {
    let node_id = NodeId::new(id.to_string());

    // Check node exists
    let node = state
        .nodes
        .get(&node_id)
        .ok_or_else(|| PolicyError::InvariantViolation(format!("node not found: {id}")))?
        .clone();

    // Create updated node with new position
    let updated_node = crate::models::document::Node {
        x: crate::models::document::OrderedFloat(x),
        y: crate::models::document::OrderedFloat(y),
        ..node
    };

    let new_nodes = state.nodes.update(node_id, updated_node);

    Ok(DiagramProjection {
        version: state.version,
        revision: state.revision,
        nodes: new_nodes,
        edges: state.edges,
        author_priority: state.author_priority,
        cycle_policy: state.cycle_policy,
    })
}

/// Apply `NodeDelete` operation
fn apply_node_delete(state: DiagramProjection, id: &str) -> Result<DiagramProjection, PolicyError> {
    let node_id = NodeId::new(id.to_string());

    // Check node exists
    if !state.has_node(&node_id) {
        return Err(PolicyError::InvariantViolation(format!(
            "node not found: {id}"
        )));
    }

    // Also remove edges connected to this node
    let edges_to_remove: Vec<EdgeId> = state
        .edges
        .iter()
        .filter(|(_, edge)| edge.source == node_id || edge.target == node_id)
        .map(|(id, _)| id.clone())
        .collect();

    // Use without() which returns HashMap, not Option
    let new_edges = edges_to_remove
        .into_iter()
        .fold(state.edges.clone(), |acc, eid| acc.without(&eid));

    let new_nodes = state.nodes.without(&node_id);

    Ok(DiagramProjection {
        version: state.version,
        revision: state.revision,
        nodes: new_nodes,
        edges: new_edges,
        author_priority: state.author_priority,
        cycle_policy: state.cycle_policy,
    })
}

/// Apply `NodeRestore` operation
fn apply_node_restore(
    state: DiagramProjection,
    id: &str,
) -> Result<DiagramProjection, PolicyError> {
    let node_id = NodeId::new(id.to_string());

    if !state.has_node(&node_id) {
        return Err(PolicyError::InvariantViolation(format!(
            "node not found for restore: {id}"
        )));
    }

    Ok(state)
}

/// Apply `EdgeConnect` operation
fn apply_edge_connect(
    state: DiagramProjection,
    id: &str,
    source: &str,
    target: &str,
) -> Result<DiagramProjection, PolicyError> {
    let edge_id = EdgeId::new(id.to_string());
    let source_id = NodeId::new(source.to_string());
    let target_id = NodeId::new(target.to_string());

    // Check for duplicate edge ID
    if state.has_edge(&edge_id) {
        return Err(PolicyError::InvariantViolation(format!(
            "duplicate edge ID: {id}"
        )));
    }

    // Validate source and target nodes exist
    if !state.has_node(&source_id) {
        return Err(PolicyError::PolicyViolation(format!(
            "source node not found: {source}"
        )));
    }
    if !state.has_node(&target_id) {
        return Err(PolicyError::PolicyViolation(format!(
            "target node not found: {target}"
        )));
    }

    let edge = crate::models::document::Edge {
        source: source_id,
        target: target_id,
        label: String::new(),
        style: crate::models::document::EdgeStyle::Solid,
        arrow_type: crate::models::document::ArrowType::Default,
        label_offset_t: crate::models::document::OrderedFloat(0.5),
        color: None,
        thickness: crate::models::document::OrderedFloat(1.5),
        directed: true,
        bend_points: im::Vector::new(),
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        font_size: None,
            source_port: None,
            target_port: None,
    };

    let new_edges = state.edges.update(edge_id, edge);

    Ok(DiagramProjection {
        version: state.version,
        revision: state.revision,
        nodes: state.nodes,
        edges: new_edges,
        author_priority: state.author_priority,
        cycle_policy: state.cycle_policy,
    })
}

/// Apply `EdgeDisconnect` operation
fn apply_edge_disconnect(
    state: DiagramProjection,
    id: &str,
) -> Result<DiagramProjection, PolicyError> {
    let edge_id = EdgeId::new(id.to_string());

    // Check edge exists
    if !state.has_edge(&edge_id) {
        return Err(PolicyError::PolicyViolation(format!(
            "edge not found: {id}"
        )));
    }

    let new_edges = state.edges.without(&edge_id);

    Ok(DiagramProjection {
        version: state.version,
        revision: state.revision,
        nodes: state.nodes,
        edges: new_edges,
        author_priority: state.author_priority,
        cycle_policy: state.cycle_policy,
    })
}

/// Apply `BringForward` operation (z-order)
fn apply_bring_forward(
    state: DiagramProjection,
    ids: &[String],
) -> Result<DiagramProjection, PolicyError> {
    if ids.is_empty() {
        return Err(PolicyError::InvalidEvent(
            "no nodes specified for z-order operation".to_string(),
        ));
    }

    let selected: std::collections::BTreeSet<NodeId> = ids
        .iter()
        .map(|s| NodeId::new(s.clone()))
        .filter(|id| state.has_node(id))
        .collect();

    if selected.is_empty() {
        let invalid_ids = ids.join(", ");
        return Err(PolicyError::InvariantViolation(format!(
            "all nodes invalid or not found: {invalid_ids}"
        )));
    }

    let mut node_ids: Vec<NodeId> = state.nodes.keys().cloned().collect();
    node_ids.sort_by(|a, b| {
        let z_a = state.nodes.get(a).map_or(0, |n| n.z_index);
        let z_b = state.nodes.get(b).map_or(0, |n| n.z_index);
        z_a.cmp(&z_b)
    });

    for idx in (0..node_ids.len() - 1).rev() {
        let current_selected = selected.contains(&node_ids[idx]);
        let next_selected = selected.contains(&node_ids[idx + 1]);
        if current_selected && !next_selected {
            node_ids.swap(idx, idx + 1);
        }
    }

    let min_z = node_ids
        .iter()
        .filter_map(|id| state.nodes.get(id).map(|n| n.z_index))
        .min()
        .unwrap_or(0);

    let max_idx = node_ids.len().saturating_sub(1);
    let _ = i64::try_from(max_idx)
        .map_err(|_| PolicyError::InvariantViolation("z-index overflow".to_string()))?;

    let new_nodes = node_ids
        .iter()
        .enumerate()
        .fold(state.nodes, |acc, (idx, id)| {
            let Some(node) = acc.get(id) else {
                return acc;
            };
            let new_z = min_z.saturating_add(idx as i64);
            let mut new_node = node.clone();
            new_node.z_index = new_z;
            acc.update(id.clone(), new_node)
        });

    Ok(DiagramProjection {
        version: state.version,
        revision: state.revision,
        nodes: new_nodes,
        edges: state.edges,
        author_priority: state.author_priority,
        cycle_policy: state.cycle_policy,
    })
}

/// Apply `SendBackward` operation (z-order)
fn apply_send_backward(
    state: DiagramProjection,
    ids: &[String],
) -> Result<DiagramProjection, PolicyError> {
    if ids.is_empty() {
        return Err(PolicyError::InvalidEvent(
            "no nodes specified for z-order operation".to_string(),
        ));
    }

    let selected: std::collections::BTreeSet<NodeId> = ids
        .iter()
        .map(|s| NodeId::new(s.clone()))
        .filter(|id| state.has_node(id))
        .collect();

    if selected.is_empty() {
        let invalid_ids = ids.join(", ");
        return Err(PolicyError::InvariantViolation(format!(
            "all nodes invalid or not found: {invalid_ids}"
        )));
    }

    let mut node_ids: Vec<NodeId> = state.nodes.keys().cloned().collect();
    node_ids.sort_by(|a, b| {
        let z_a = state.nodes.get(a).map_or(0, |n| n.z_index);
        let z_b = state.nodes.get(b).map_or(0, |n| n.z_index);
        z_a.cmp(&z_b)
    });

    for idx in 1..node_ids.len() {
        let current_selected = selected.contains(&node_ids[idx]);
        let previous_selected = selected.contains(&node_ids[idx - 1]);
        if current_selected && !previous_selected {
            node_ids.swap(idx - 1, idx);
        }
    }

    let min_z = node_ids
        .iter()
        .filter_map(|id| state.nodes.get(id).map(|n| n.z_index))
        .min()
        .unwrap_or(0);

    let max_idx = node_ids.len().saturating_sub(1);
    let _ = i64::try_from(max_idx)
        .map_err(|_| PolicyError::InvariantViolation("z-index overflow".to_string()))?;

    let new_nodes = node_ids
        .iter()
        .enumerate()
        .fold(state.nodes, |acc, (idx, id)| {
            let Some(node) = acc.get(id) else {
                return acc;
            };
            let new_z = min_z.saturating_add(idx as i64);
            let mut new_node = node.clone();
            new_node.z_index = new_z;
            acc.update(id.clone(), new_node)
        });

    Ok(DiagramProjection {
        version: state.version,
        revision: state.revision,
        nodes: new_nodes,
        edges: state.edges,
        author_priority: state.author_priority,
        cycle_policy: state.cycle_policy,
    })
}

/// Apply `BringToFront` operation (z-order)
fn apply_bring_to_front(
    state: DiagramProjection,
    ids: &[String],
) -> Result<DiagramProjection, PolicyError> {
    if ids.is_empty() {
        return Err(PolicyError::InvalidEvent(
            "no nodes specified for z-order operation".to_string(),
        ));
    }

    let selected: std::collections::BTreeSet<NodeId> = ids
        .iter()
        .map(|s| NodeId::new(s.clone()))
        .filter(|id| state.has_node(id))
        .collect();

    if selected.is_empty() {
        let invalid_ids = ids.join(", ");
        return Err(PolicyError::InvariantViolation(format!(
            "all nodes invalid or not found: {invalid_ids}"
        )));
    }

    let mut node_ids: Vec<NodeId> = state.nodes.keys().cloned().collect();
    node_ids.sort_by(|a, b| {
        let z_a = state.nodes.get(a).map_or(0, |n| n.z_index);
        let z_b = state.nodes.get(b).map_or(0, |n| n.z_index);
        z_a.cmp(&z_b)
    });

    let mut reordered: Vec<NodeId> = node_ids
        .iter()
        .filter(|id| !selected.contains(*id))
        .cloned()
        .collect();
    reordered.extend(node_ids.iter().filter(|id| selected.contains(*id)).cloned());
    node_ids = reordered;

    let min_z = node_ids
        .iter()
        .filter_map(|id| state.nodes.get(id).map(|n| n.z_index))
        .min()
        .unwrap_or(0);

    let max_idx = node_ids.len().saturating_sub(1);
    let _ = i64::try_from(max_idx)
        .map_err(|_| PolicyError::InvariantViolation("z-index overflow".to_string()))?;

    let new_nodes = node_ids
        .iter()
        .enumerate()
        .fold(state.nodes, |acc, (idx, id)| {
            let Some(node) = acc.get(id) else {
                return acc;
            };
            let new_z = min_z.saturating_add(idx as i64);
            let mut new_node = node.clone();
            new_node.z_index = new_z;
            acc.update(id.clone(), new_node)
        });

    Ok(DiagramProjection {
        version: state.version,
        revision: state.revision,
        nodes: new_nodes,
        edges: state.edges,
        author_priority: state.author_priority,
        cycle_policy: state.cycle_policy,
    })
}

/// Apply `SendToBack` operation (z-order)
fn apply_send_to_back(
    state: DiagramProjection,
    ids: &[String],
) -> Result<DiagramProjection, PolicyError> {
    if ids.is_empty() {
        return Err(PolicyError::InvalidEvent(
            "no nodes specified for z-order operation".to_string(),
        ));
    }

    let selected: std::collections::BTreeSet<NodeId> = ids
        .iter()
        .map(|s| NodeId::new(s.clone()))
        .filter(|id| state.has_node(id))
        .collect();

    if selected.is_empty() {
        let invalid_ids = ids.join(", ");
        return Err(PolicyError::InvariantViolation(format!(
            "all nodes invalid or not found: {invalid_ids}"
        )));
    }

    let mut node_ids: Vec<NodeId> = state.nodes.keys().cloned().collect();
    node_ids.sort_by(|a, b| {
        let z_a = state.nodes.get(a).map_or(0, |n| n.z_index);
        let z_b = state.nodes.get(b).map_or(0, |n| n.z_index);
        z_a.cmp(&z_b)
    });

    let mut reordered: Vec<NodeId> = node_ids
        .iter()
        .filter(|id| selected.contains(*id))
        .cloned()
        .collect();
    reordered.extend(
        node_ids
            .iter()
            .filter(|id| !selected.contains(*id))
            .cloned(),
    );
    node_ids = reordered;

    let min_z = node_ids
        .iter()
        .filter_map(|id| state.nodes.get(id).map(|n| n.z_index))
        .min()
        .unwrap_or(0);

    let max_idx = node_ids.len().saturating_sub(1);
    let _ = i64::try_from(max_idx)
        .map_err(|_| PolicyError::InvariantViolation("z-index overflow".to_string()))?;

    let new_nodes = node_ids
        .iter()
        .enumerate()
        .fold(state.nodes, |acc, (idx, id)| {
            let Some(node) = acc.get(id) else {
                return acc;
            };
            let new_z = min_z.saturating_add(idx as i64);
            let mut new_node = node.clone();
            new_node.z_index = new_z;
            acc.update(id.clone(), new_node)
        });

    Ok(DiagramProjection {
        version: state.version,
        revision: state.revision,
        nodes: new_nodes,
        edges: state.edges,
        author_priority: state.author_priority,
        cycle_policy: state.cycle_policy,
    })
}

/// Apply Group operation - creates a subgraph and assigns all specified nodes as children
fn apply_group(state: DiagramProjection, ids: &[String]) -> Result<DiagramProjection, PolicyError> {
    if ids.is_empty() {
        return Err(PolicyError::InvalidEvent(
            "no nodes specified for group operation".to_string(),
        ));
    }

    let node_ids: Vec<NodeId> = ids.iter().map(|s| NodeId::new(s.clone())).collect();

    let valid_ids: Vec<NodeId> = node_ids
        .iter()
        .filter(|id| state.has_node(id))
        .cloned()
        .collect();

    if valid_ids.len() < 2 {
        let invalid_ids = ids.join(", ");
        return Err(PolicyError::InvariantViolation(format!(
            "all nodes invalid or not found: {invalid_ids}"
        )));
    }

    let (min_x, min_y, max_x, max_y) = {
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for id in &valid_ids {
            if let Some(node) = state.nodes.get(id) {
                min_x = min_x.min(node.x.0);
                min_y = min_y.min(node.y.0);
                max_x = max_x.max(node.x.0 + node.width.0);
                max_y = max_y.max(node.y.0 + node.height.0);
            }
        }

        (min_x, min_y, max_x, max_y)
    };

    if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
        return Err(PolicyError::InvariantViolation(
            "invalid node coordinates for grouping".to_string(),
        ));
    }

    use uuid::Uuid;
    let group_id = NodeId::new(format!("group-{}", Uuid::new_v4()));
    let padding = 24.0;

    let subgraph = crate::models::document::Node {
        kind: crate::models::document::NodeKind::Subgraph,
        icon: String::new(),
        label: "Group".to_string(),
        x: crate::models::document::OrderedFloat(min_x - padding),
        y: crate::models::document::OrderedFloat(min_y - padding),
        width: crate::models::document::OrderedFloat((max_x - min_x) + (padding * 2.0)),
        height: crate::models::document::OrderedFloat((max_y - min_y) + (padding * 2.0)),
        font_size: None,
        font_weight: None,
        locked: true,
        parent: None,
        dag_rank: None,
        tags: im::vector![],
        metadata: HashMap::new(),
        z_index: -1,
        style: Some(crate::models::document::NodeStyle::Box),
        collapsed: Some(false),
    };

    let mut new_nodes = state.nodes.clone();
    let _ = new_nodes.insert(group_id.clone(), subgraph);

    let new_nodes = valid_ids.iter().fold(new_nodes, |acc, id| {
        if let Some(node) = acc.get(id) {
            let mut updated_node = node.clone();
            updated_node.parent = Some(group_id.clone());
            acc.update(id.clone(), updated_node)
        } else {
            acc
        }
    });

    Ok(DiagramProjection {
        version: state.version,
        revision: state.revision,
        nodes: new_nodes,
        edges: state.edges,
        author_priority: state.author_priority,
        cycle_policy: state.cycle_policy,
    })
}

/// Apply Ungroup operation - removes the subgraph node and clears parent on all children
fn apply_ungroup(state: DiagramProjection, id: &str) -> Result<DiagramProjection, PolicyError> {
    let subgraph_id = NodeId::new(id.to_string());

    if !state.has_node(&subgraph_id) {
        return Err(PolicyError::InvariantViolation(format!(
            "subgraph not found: {id}"
        )));
    }

    let subgraph = state.nodes.get(&subgraph_id).cloned();
    let _subgraph = match subgraph {
        Some(s) if s.kind == crate::models::document::NodeKind::Subgraph => s,
        _ => {
            return Err(PolicyError::InvariantViolation(format!(
                "node is not a subgraph: {id}"
            )))
        }
    };

    let children_to_unparent: Vec<NodeId> = state
        .nodes
        .iter()
        .filter(|(_, node)| node.parent.as_ref() == Some(&subgraph_id))
        .map(|(id, _)| id.clone())
        .collect();

    let new_nodes = children_to_unparent
        .iter()
        .fold(state.nodes.clone(), |acc, child_id| {
            if let Some(child) = acc.get(child_id) {
                let mut updated_child = child.clone();
                updated_child.parent = None;
                acc.update(child_id.clone(), updated_child)
            } else {
                acc
            }
        })
        .without(&subgraph_id);

    Ok(DiagramProjection {
        version: state.version,
        revision: state.revision,
        nodes: new_nodes,
        edges: state.edges,
        author_priority: state.author_priority,
        cycle_policy: state.cycle_policy,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::envelope::Author;

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

    /// Test: CyclePolicy default is Allow
    #[test]
    fn given_no_cycle_policy_specified_then_default_is_allow() {
        let projection = DiagramProjection::empty();
        assert_eq!(projection.cycle_policy, CyclePolicy::Allow);
    }

    /// Test: CyclePolicy::Allow permits cycles
    #[test]
    fn given_cycle_policy_allow_when_cycle_exists_then_enforce_succeeds() {
        let mut projection = DiagramProjection::with_cycle_policy(CyclePolicy::Allow);

        // Add two nodes
        let events = [
            make_event(
                "op-1",
                0,
                DomainOp::NodeAdd {
                    id: "a".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "A".to_string(),
                },
                true,
            ),
            make_event(
                "op-2",
                1,
                DomainOp::NodeAdd {
                    id: "b".to_string(),
                    x: 100.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "B".to_string(),
                },
                true,
            ),
            make_event(
                "op-3",
                2,
                DomainOp::EdgeConnect {
                    id: "e1".to_string(),
                    source: "a".to_string(),
                    target: "b".to_string(),
                },
                true,
            ),
            make_event(
                "op-4",
                3,
                DomainOp::EdgeConnect {
                    id: "e2".to_string(),
                    source: "b".to_string(),
                    target: "a".to_string(),
                },
                true,
            ),
        ];

        for event in &events {
            projection = apply_event_internal(projection, event).unwrap();
        }

        // Cycle exists but policy is Allow, so enforcement should succeed
        let result = enforce_cycle_policy(&projection);
        assert!(result.is_ok(), "Allow policy should permit cycles");
    }

    /// Test: CyclePolicy::Deny rejects cycles
    #[test]
    fn given_cycle_policy_deny_when_cycle_exists_then_enforce_fails() {
        let mut projection = DiagramProjection::with_cycle_policy(CyclePolicy::Deny);

        // Add two nodes with a cycle
        let events = [
            make_event(
                "op-1",
                0,
                DomainOp::NodeAdd {
                    id: "a".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "A".to_string(),
                },
                true,
            ),
            make_event(
                "op-2",
                1,
                DomainOp::NodeAdd {
                    id: "b".to_string(),
                    x: 100.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "B".to_string(),
                },
                true,
            ),
            make_event(
                "op-3",
                2,
                DomainOp::EdgeConnect {
                    id: "e1".to_string(),
                    source: "a".to_string(),
                    target: "b".to_string(),
                },
                true,
            ),
            make_event(
                "op-4",
                3,
                DomainOp::EdgeConnect {
                    id: "e2".to_string(),
                    source: "b".to_string(),
                    target: "a".to_string(),
                },
                true,
            ),
        ];

        for event in &events {
            projection = apply_event_internal(projection, event).unwrap();
        }

        // Cycle exists and policy is Deny, so enforcement should fail
        let result = enforce_cycle_policy(&projection);
        assert!(result.is_err(), "Deny policy should reject cycles");
        match result {
            Err(PolicyError::CycleViolation(msg)) => {
                assert!(msg.contains("Cycle") || msg.contains("cycle"));
            }
            _ => panic!("Expected PolicyError::CycleViolation error"),
        }
    }

    /// Test: CyclePolicy::Deny allows acyclic graphs
    #[test]
    fn given_cycle_policy_deny_when_no_cycle_exists_then_enforce_succeeds() {
        let mut projection = DiagramProjection::with_cycle_policy(CyclePolicy::Deny);

        // Add nodes without a cycle (linear chain)
        let events = [
            make_event(
                "op-1",
                0,
                DomainOp::NodeAdd {
                    id: "a".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "A".to_string(),
                },
                true,
            ),
            make_event(
                "op-2",
                1,
                DomainOp::NodeAdd {
                    id: "b".to_string(),
                    x: 100.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "B".to_string(),
                },
                true,
            ),
            make_event(
                "op-3",
                2,
                DomainOp::EdgeConnect {
                    id: "e1".to_string(),
                    source: "a".to_string(),
                    target: "b".to_string(),
                },
                true,
            ),
        ];

        for event in &events {
            projection = apply_event_internal(projection, event).unwrap();
        }

        // No cycle, so enforcement should succeed
        let result = enforce_cycle_policy(&projection);
        assert!(result.is_ok(), "Deny policy should allow acyclic graphs");
    }

    /// Test: apply_policy_op allows acyclic operations under Deny policy
    #[test]
    fn given_cycle_policy_deny_when_applying_acyclic_op_then_succeeds() {
        let mut projection = DiagramProjection::with_cycle_policy(CyclePolicy::Deny);

        // Add initial node
        let events = [make_event(
            "op-1",
            0,
            DomainOp::NodeAdd {
                id: "a".to_string(),
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 40.0,
                label: "A".to_string(),
            },
            true,
        )];
        for event in &events {
            projection = apply_event_internal(projection, event).unwrap();
        }

        // Apply another node add via apply_policy_op
        let op = DomainOp::NodeAdd {
            id: "b".to_string(),
            x: 100.0,
            y: 0.0,
            width: 80.0,
            height: 40.0,
            label: "B".to_string(),
        };

        let result = apply_policy_op(projection, &op);
        assert!(result.is_ok(), "Adding a node should not create a cycle");
    }

    /// Test: apply_policy_op rejects cycle-creating operations under Deny policy
    #[test]
    fn given_cycle_policy_deny_when_applying_cyclic_op_then_fails() {
        let mut projection = DiagramProjection::with_cycle_policy(CyclePolicy::Deny);

        // Set up: a -> b
        let events = [
            make_event(
                "op-1",
                0,
                DomainOp::NodeAdd {
                    id: "a".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "A".to_string(),
                },
                true,
            ),
            make_event(
                "op-2",
                1,
                DomainOp::NodeAdd {
                    id: "b".to_string(),
                    x: 100.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "B".to_string(),
                },
                true,
            ),
            make_event(
                "op-3",
                2,
                DomainOp::EdgeConnect {
                    id: "e1".to_string(),
                    source: "a".to_string(),
                    target: "b".to_string(),
                },
                true,
            ),
        ];

        for event in &events {
            projection = apply_event_internal(projection, event).unwrap();
        }

        // Try to create a cycle: b -> a
        let op = DomainOp::EdgeConnect {
            id: "e2".to_string(),
            source: "b".to_string(),
            target: "a".to_string(),
        };

        let result = apply_policy_op(projection, &op);
        assert!(
            result.is_err(),
            "Creating a cycle should fail under Deny policy"
        );
        match result {
            Err(PolicyError::CycleViolation(_)) => {}
            _ => panic!("Expected PolicyError::CycleViolation"),
        }
    }
}
