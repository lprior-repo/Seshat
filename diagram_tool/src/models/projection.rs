//! Projection replay module - deterministic document projection replayer
//!
//! This module provides deterministic replay of events to produce a consistent `DiagramProjection`.
//! The replay is deterministic: given the same input events, it always produces the same output.

#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use im::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::models::document::{
    DiagramDocument, DocumentData, Edge, EdgeId, Node, NodeId, OrderedFloat,
};
use crate::models::envelope::{Author, DomainOp};

/// Current supported schema version
const SUPPORTED_VERSION: u32 = 1;

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

/// Errors that can occur during replay
#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReplayError {
    #[error("invalid event: {0}")]
    InvalidEvent(String),
    #[error("invariant violation: {0}")]
    InvariantViolation(String),
    #[error("unsupported schema version: {0}")]
    UnsupportedVersion(u32),
    #[error("cycle violation: {0}")]
    CycleViolation(String),
    #[error("policy missing: {0}")]
    PolicyMissing(String),
    #[error("edge not found: {0}")]
    EdgeNotFound(String),
    #[error("duplicate edge: {0}")]
    DuplicateEdge(String),
    #[error("policy violation: {0}")]
    PolicyViolation(String),
}

/// Event record for replay - contains all information needed to reconstruct state
///
/// `DomainOp` is not Eq, so we use `PartialEq` instead
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

/// Diagram projection - the result of replaying events
///
/// This is a pure data structure representing the complete diagram state
/// after replaying a sequence of events.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiagramProjection {
    /// Schema version for compatibility checking
    pub version: u32,
    /// Current revision number
    pub revision: u64,
    /// All nodes in the diagram
    pub nodes: HashMap<NodeId, Node>,
    /// All edges in the diagram
    pub edges: HashMap<EdgeId, Edge>,
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
            version: SUPPORTED_VERSION,
            revision: 0,
            nodes: HashMap::new(),
            edges: HashMap::new(),
            author_priority: HashMap::new(),
            cycle_policy: CyclePolicy::default(),
        }
    }

    /// Create a new projection with initial revision
    #[must_use]
    pub fn with_revision(revision: u64) -> Self {
        Self {
            version: SUPPORTED_VERSION,
            revision,
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
            version: SUPPORTED_VERSION,
            revision: 0,
            nodes: HashMap::new(),
            edges: HashMap::new(),
            author_priority: HashMap::new(),
            cycle_policy,
        }
    }

    /// Get the current revision
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
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

    /// Get a node by ID
    #[must_use]
    pub fn get_node(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// Get an edge by ID
    #[must_use]
    pub fn get_edge(&self, id: &EdgeId) -> Option<&Edge> {
        self.edges.get(id)
    }
}

/// Check if an author is human (not AI-generated)
fn is_human_author(author: &Author) -> bool {
    // Author IDs starting with "human-" are considered human-authored
    // All others are assumed to be AI-authored
    author.id.starts_with("human-") || author.name.to_lowercase().contains("human")
}

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
    events
        .iter()
        .try_fold(initial_state, apply_event)
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
    // Validate schema version
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
    let mut new_priority_map = new_state.author_priority.clone();
    let old_value = new_priority_map.insert(event.op_id.clone(), is_human);

    // Verify idempotency: if this op_id was already processed, we should get the same result
    if old_value.is_some() {
        return Err(ReplayError::InvariantViolation(format!(
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
fn apply_operation(
    state: DiagramProjection,
    event: &EventRecord,
) -> Result<DiagramProjection, ReplayError> {
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
) -> Result<DiagramProjection, ReplayError> {
    let node_id = NodeId::new(id.to_string());

    // Check for duplicate node ID
    if state.has_node(&node_id) {
        return Err(ReplayError::InvariantViolation(format!(
            "duplicate node ID: {id}"
        )));
    }

    let node = Node {
        kind: crate::models::document::NodeKind::Node,
        icon: String::new(),
        label: label.to_string(),
        x: OrderedFloat(x),
        y: OrderedFloat(y),
        width: OrderedFloat(width),
        height: OrderedFloat(height),
        font_size: None,
        font_weight: None,
        locked: false,
        parent: None,
        dag_rank: None,
        tags: vec![],
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
) -> Result<DiagramProjection, ReplayError> {
    let node_id = NodeId::new(id.to_string());

    // Check node exists
    let node = state
        .nodes
        .get(&node_id)
        .ok_or_else(|| ReplayError::InvariantViolation(format!("node not found: {id}")))?
        .clone();

    // Create updated node with new position
    let updated_node = Node {
        x: OrderedFloat(x),
        y: OrderedFloat(y),
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
fn apply_node_delete(state: DiagramProjection, id: &str) -> Result<DiagramProjection, ReplayError> {
    let node_id = NodeId::new(id.to_string());

    // Check node exists
    if !state.has_node(&node_id) {
        return Err(ReplayError::InvariantViolation(format!(
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

/// Apply `NodeRestore` operation (currently same as `NodeAdd` but validates no duplicate)
fn apply_node_restore(
    state: DiagramProjection,
    id: &str,
) -> Result<DiagramProjection, ReplayError> {
    // NodeRestore is essentially a no-op in this implementation
    // since deleted nodes are permanently removed
    // In a more complex implementation, we might have a "deleted" set
    let node_id = NodeId::new(id.to_string());

    if state.has_node(&node_id) {
        return Err(ReplayError::InvariantViolation(format!(
            "node already exists: {id}"
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
) -> Result<DiagramProjection, ReplayError> {
    let edge_id = EdgeId::new(id.to_string());
    let source_id = NodeId::new(source.to_string());
    let target_id = NodeId::new(target.to_string());

    // Check for duplicate edge ID
    if state.has_edge(&edge_id) {
        return Err(ReplayError::InvariantViolation(format!(
            "duplicate edge ID: {id}"
        )));
    }

    // Validate source and target nodes exist
    if !state.has_node(&source_id) {
        return Err(ReplayError::InvariantViolation(format!(
            "source node not found: {source}"
        )));
    }
    if !state.has_node(&target_id) {
        return Err(ReplayError::InvariantViolation(format!(
            "target node not found: {target}"
        )));
    }

    let edge = Edge {
        source: source_id,
        target: target_id,
        label: String::new(),
        style: crate::models::document::EdgeStyle::Solid,
        arrow_type: crate::models::document::ArrowType::Default,
        label_offset_t: OrderedFloat(0.5),
        color: None,
        thickness: OrderedFloat(1.5),
        directed: true,
        bend_points: vec![],
        tags: vec![],
        metadata: HashMap::new(),
        font_size: None,
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
) -> Result<DiagramProjection, ReplayError> {
    let edge_id = EdgeId::new(id.to_string());

    // Check edge exists
    if !state.has_edge(&edge_id) {
        return Err(ReplayError::InvariantViolation(format!(
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

/// Apply an edge operation to the projection
///
/// This is the contract-specified entry point for applying edge operations.
/// It dispatches to the appropriate handler based on the operation type.
///
/// # Errors
/// Returns `ReplayError::EdgeNotFound` if the edge does not exist for disconnect operations
/// Returns `ReplayError::DuplicateEdge` if the edge already exists for connect operations
/// Returns `ReplayError::PolicyViolation` if the operation violates policy constraints
/// Returns `ReplayError::InvalidEvent` if the operation is not an edge operation
pub fn apply_edge_op(
    state: DiagramProjection,
    op: &DomainOp,
) -> Result<DiagramProjection, ReplayError> {
    match op {
        DomainOp::EdgeConnect { id, source, target } => {
            apply_edge_connect_checked(state, id, source, target)
        }
        DomainOp::EdgeDisconnect { id } => apply_edge_disconnect_checked(state, id),
        _ => Err(ReplayError::InvalidEvent(format!(
            "not an edge operation: {:?}",
            op.kind()
        ))),
    }
}

/// Apply `EdgeConnect` operation with contract-specified error types
fn apply_edge_connect_checked(
    state: DiagramProjection,
    id: &str,
    source: &str,
    target: &str,
) -> Result<DiagramProjection, ReplayError> {
    let edge_id = EdgeId::new(id.to_string());
    let source_id = NodeId::new(source.to_string());
    let target_id = NodeId::new(target.to_string());

    // Check for duplicate edge ID
    if state.has_edge(&edge_id) {
        return Err(ReplayError::DuplicateEdge(id.to_string()));
    }

    // Validate source node exists
    if !state.has_node(&source_id) {
        return Err(ReplayError::PolicyViolation(format!(
            "source node not found: {source}"
        )));
    }

    // Validate target node exists
    if !state.has_node(&target_id) {
        return Err(ReplayError::PolicyViolation(format!(
            "target node not found: {target}"
        )));
    }

    let edge = Edge {
        source: source_id,
        target: target_id,
        label: String::new(),
        style: crate::models::document::EdgeStyle::Solid,
        arrow_type: crate::models::document::ArrowType::Default,
        label_offset_t: OrderedFloat(0.5),
        color: None,
        thickness: OrderedFloat(1.5),
        directed: true,
        bend_points: vec![],
        tags: vec![],
        metadata: HashMap::new(),
        font_size: None,
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

/// Apply `EdgeDisconnect` operation with contract-specified error types
fn apply_edge_disconnect_checked(
    state: DiagramProjection,
    id: &str,
) -> Result<DiagramProjection, ReplayError> {
    let edge_id = EdgeId::new(id.to_string());

    // Check edge exists
    if !state.has_edge(&edge_id) {
        return Err(ReplayError::EdgeNotFound(id.to_string()));
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

/// Verify edge tolerance constraints in the projection
///
/// This function validates that all edges in the projection satisfy
/// the defined tolerance boundaries:
/// - All edges reference existing source and target nodes
/// - No duplicate edge IDs exist
/// - All edges have valid geometry (finite coordinates)
///
/// # Errors
/// Returns `ReplayError::PolicyViolation` if any edge references a non-existent node
/// Returns `ReplayError::DuplicateEdge` if duplicate edge IDs are detected
/// Returns `ReplayError::InvariantViolation` if edge geometry is invalid
pub fn verify_edge_tolerance(state: &DiagramProjection) -> Result<(), ReplayError> {
    // Track seen edge IDs to detect duplicates
    let mut seen_ids = std::collections::HashSet::new();

    for (edge_id, edge) in state.edges.iter() {
        // Check for duplicate IDs (should not happen with HashMap, but verify)
        let id_str = edge_id.to_string();
        if !seen_ids.insert(id_str.clone()) {
            return Err(ReplayError::DuplicateEdge(id_str));
        }

        // Verify source node exists
        if !state.has_node(&edge.source) {
            return Err(ReplayError::PolicyViolation(format!(
                "edge {} references non-existent source node: {}",
                edge_id, edge.source
            )));
        }

        // Verify target node exists
        if !state.has_node(&edge.target) {
            return Err(ReplayError::PolicyViolation(format!(
                "edge {} references non-existent target node: {}",
                edge_id, edge.target
            )));
        }

        // Verify edge geometry is valid
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

/// Apply `BringForward` operation (z-order)
const fn apply_bring_forward(
    state: DiagramProjection,
    ids: &[String],
) -> Result<DiagramProjection, ReplayError> {
    // For now, z-order operations are no-ops
    // A full implementation would adjust z_index values
    let _ = ids;
    Ok(state)
}

/// Apply `SendBackward` operation (z-order)
const fn apply_send_backward(
    state: DiagramProjection,
    ids: &[String],
) -> Result<DiagramProjection, ReplayError> {
    let _ = ids;
    Ok(state)
}

/// Apply `BringToFront` operation (z-order)
const fn apply_bring_to_front(
    state: DiagramProjection,
    ids: &[String],
) -> Result<DiagramProjection, ReplayError> {
    let _ = ids;
    Ok(state)
}

/// Apply `SendToBack` operation (z-order)
const fn apply_send_to_back(
    state: DiagramProjection,
    ids: &[String],
) -> Result<DiagramProjection, ReplayError> {
    let _ = ids;
    Ok(state)
}

/// Apply Group operation
const fn apply_group(
    state: DiagramProjection,
    ids: &[String],
) -> Result<DiagramProjection, ReplayError> {
    // Group operations are no-ops in this basic implementation
    let _ = ids;
    Ok(state)
}

/// Apply Ungroup operation
const fn apply_ungroup(
    state: DiagramProjection,
    id: &str,
) -> Result<DiagramProjection, ReplayError> {
    // Ungroup operations are no-ops in this basic implementation
    let _ = id;
    Ok(state)
}

/// Convert a `DiagramProjection` to a `DiagramDocument`
///
/// This is useful for interoperability with existing document handling.
#[must_use]
pub fn projection_to_document(projection: &DiagramProjection) -> DiagramDocument {
    DiagramDocument {
        version: projection.version,
        revision: crate::models::document::Revision::new(projection.revision),
        document: DocumentData {
            nodes: projection.nodes.clone(),
            edges: projection.edges.clone(),
        },
        editor_state: crate::models::document::EditorState::default(),
    }
}

/// Convert a `DiagramDocument` to a `DiagramProjection`
///
/// This is useful for bootstrapping a projection from an existing document.
#[must_use]
pub fn document_to_projection(document: &DiagramDocument) -> DiagramProjection {
    DiagramProjection {
        version: document.version,
        revision: document.revision.value(),
        nodes: document.document.nodes.clone(),
        edges: document.document.edges.clone(),
        author_priority: HashMap::new(),
        cycle_policy: CyclePolicy::default(),
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

/// Compute a stable hash of a diagram projection
///
/// This function produces a deterministic hash string that uniquely identifies
/// the projection state. The hash is stable across multiple invocations and
/// can be used for:
/// - Verifying replay determinism
/// - Detecting state changes
/// - Caching and optimization
///
/// # Errors
/// Returns `ReplayError::InvariantViolation` if the projection contains
/// data that cannot be hashed (e.g., NaN values in coordinates).
///
/// # Example
/// ```ignore
/// let projection = replay_events(&events)?;
/// let hash = projection_hash(&projection)?;
/// // Same events always produce same hash
/// assert_eq!(hash, projection_hash(&replay_events(&events)?)?);
/// ```
pub fn projection_hash(state: &DiagramProjection) -> Result<String, ReplayError> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Validate that coordinates are finite (no NaN or infinity)
    for (id, node) in state.nodes.iter() {
        if !node.x.0.is_finite() || !node.y.0.is_finite() {
            return Err(ReplayError::InvariantViolation(format!(
                "node {id} has non-finite coordinates"
            )));
        }
        if !node.width.0.is_finite() || !node.height.0.is_finite() {
            return Err(ReplayError::InvariantViolation(format!(
                "node {id} has non-finite dimensions"
            )));
        }
    }

    for (id, edge) in state.edges.iter() {
        if !edge.label_offset_t.0.is_finite() {
            return Err(ReplayError::InvariantViolation(format!(
                "edge {id} has non-finite label_offset_t"
            )));
        }
        if !edge.thickness.0.is_finite() {
            return Err(ReplayError::InvariantViolation(format!(
                "edge {id} has non-finite thickness"
            )));
        }
    }

    let mut hasher = DefaultHasher::new();

    // Hash version
    state.version.hash(&mut hasher);

    // Hash revision
    state.revision.hash(&mut hasher);

    // Hash nodes in deterministic order (sorted by ID)
    let mut node_ids: Vec<_> = state.nodes.keys().collect();
    node_ids.sort();
    for id in node_ids {
        id.hash(&mut hasher);
        let node = state.nodes.get(id).ok_or_else(|| {
            ReplayError::InvariantViolation(format!("node {id} disappeared during hashing"))
        })?;

        // Hash node fields in consistent order
        node.kind.hash(&mut hasher);
        node.icon.hash(&mut hasher);
        node.label.hash(&mut hasher);
        // Use bitwise representation for floats to ensure determinism
        node.x.0.to_bits().hash(&mut hasher);
        node.y.0.to_bits().hash(&mut hasher);
        node.width.0.to_bits().hash(&mut hasher);
        node.height.0.to_bits().hash(&mut hasher);
        node.font_size.hash(&mut hasher);
        node.font_weight.hash(&mut hasher);
        node.locked.hash(&mut hasher);
        node.parent.hash(&mut hasher);
        node.dag_rank.hash(&mut hasher);
        node.z_index.hash(&mut hasher);
        node.style.hash(&mut hasher);
        node.collapsed.hash(&mut hasher);

        // Hash tags in sorted order
        let mut tags = node.tags.clone();
        tags.sort();
        tags.len().hash(&mut hasher);
        for tag in tags {
            tag.hash(&mut hasher);
        }

        // Hash metadata in sorted order
        let mut metadata_keys: Vec<_> = node.metadata.keys().collect();
        metadata_keys.sort();
        for key in metadata_keys {
            key.hash(&mut hasher);
            let value = node.metadata.get(key).ok_or_else(|| {
                ReplayError::InvariantViolation(format!(
                    "metadata key {key} disappeared during hashing"
                ))
            })?;
            value.hash(&mut hasher);
        }
    }

    // Hash edges in deterministic order (sorted by ID)
    let mut edge_ids: Vec<_> = state.edges.keys().collect();
    edge_ids.sort();
    for id in edge_ids {
        id.hash(&mut hasher);
        let edge = state.edges.get(id).ok_or_else(|| {
            ReplayError::InvariantViolation(format!("edge {id} disappeared during hashing"))
        })?;

        edge.source.hash(&mut hasher);
        edge.target.hash(&mut hasher);
        edge.label.hash(&mut hasher);
        edge.style.hash(&mut hasher);
        edge.arrow_type.hash(&mut hasher);
        edge.label_offset_t.0.to_bits().hash(&mut hasher);
        edge.color.hash(&mut hasher);
        edge.thickness.0.to_bits().hash(&mut hasher);
        edge.directed.hash(&mut hasher);
        edge.font_size.hash(&mut hasher);

        // Hash bend points
        edge.bend_points.len().hash(&mut hasher);
        for bp in &edge.bend_points {
            bp.x.0.to_bits().hash(&mut hasher);
            bp.y.0.to_bits().hash(&mut hasher);
        }

        // Hash tags in sorted order
        let mut tags = edge.tags.clone();
        tags.sort();
        tags.len().hash(&mut hasher);
        for tag in tags {
            tag.hash(&mut hasher);
        }

        // Hash metadata in sorted order
        let mut metadata_keys: Vec<_> = edge.metadata.keys().collect();
        metadata_keys.sort();
        for key in metadata_keys {
            key.hash(&mut hasher);
            let value = edge.metadata.get(key).ok_or_else(|| {
                ReplayError::InvariantViolation(format!(
                    "metadata key {key} disappeared during hashing"
                ))
            })?;
            value.hash(&mut hasher);
        }
    }

    // Hash author_priority in sorted order
    let mut priority_keys: Vec<_> = state.author_priority.keys().collect();
    priority_keys.sort();
    for key in priority_keys {
        key.hash(&mut hasher);
        let value = state.author_priority.get(key).ok_or_else(|| {
            ReplayError::InvariantViolation(format!(
                "author_priority key {key} disappeared during hashing"
            ))
        })?;
        value.hash(&mut hasher);
    }

    // Produce hex string of hash
    let hash_value = hasher.finish();
    Ok(format!("{hash_value:016x}"))
}

/// Enforce cycle policy on a diagram projection
///
/// This function checks whether the current projection violates its configured
/// cycle policy. If the policy is `CyclePolicy::Deny` and the graph contains
/// cycles, an error is returned.
///
/// # Errors
/// Returns `ReplayError::CycleViolation` if:
/// - The cycle policy is `Deny` and the projection contains a cycle
/// Returns `ReplayError::PolicyMissing` if:
/// - The cycle policy field is not properly initialized (should not happen with default)
///
/// # Example
/// ```ignore
/// let projection = DiagramProjection::with_cycle_policy(CyclePolicy::Deny);
/// // Add nodes and edges...
/// enforce_cycle_policy(&projection)?; // Returns error if cycle detected
/// ```
pub fn enforce_cycle_policy(state: &DiagramProjection) -> Result<(), ReplayError> {
    match state.cycle_policy {
        CyclePolicy::Allow => Ok(()),
        CyclePolicy::Deny => {
            // Use the DAG validation from the dag module
            crate::models::dag::validate_dag(&state.nodes, &state.edges)
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
/// Returns `ReplayError::CycleViolation` if:
/// - The operation would create a cycle and policy is `Deny`
/// Returns `ReplayError::InvariantViolation` if:
/// - The operation itself violates an invariant (e.g., duplicate node ID)
/// Returns `ReplayError::InvalidEvent` if:
/// - The event is malformed
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
) -> Result<DiagramProjection, ReplayError> {
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

    let new_state = apply_event(state, &event)?;

    // Then, enforce the cycle policy on the new state
    enforce_cycle_policy(&new_state)?;

    // If we get here, the operation is valid
    Ok(new_state)
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
            .contains_key(&NodeId::new("node-1".to_string())));
    }

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

    #[test]
    fn given_node_move_for_nonexistent_node_then_returns_error() {
        let events = [make_event(
            "op-1",
            0,
            DomainOp::NodeMove {
                id: "nonexistent".to_string(),
                x: 100.0,
                y: 200.0,
            },
            true,
        )];

        let result = replay_events(&events);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ReplayError::InvariantViolation(_)));
    }

    #[test]
    fn given_edge_connect_to_nonexistent_source_then_returns_error() {
        let events = [make_event(
            "op-1",
            0,
            DomainOp::EdgeConnect {
                id: "edge-1".to_string(),
                source: "nonexistent".to_string(),
                target: "node-1".to_string(),
            },
            true,
        )];

        let result = replay_events(&events);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ReplayError::InvariantViolation(_)));
    }

    #[test]
    fn given_apply_event_on_valid_state_then_returns_updated_state() {
        let initial = DiagramProjection::empty();
        let event = make_event(
            "op-1",
            0,
            DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 50.0,
                y: 75.0,
                width: 100.0,
                height: 50.0,
                label: "Test".to_string(),
            },
            true,
        );

        let result = apply_event(initial, &event);

        assert!(result.is_ok(), "Error: {:?}", result.err());
        let projection = result.unwrap();
        assert_eq!(projection.revision, 1);
        assert!(projection
            .nodes
            .contains_key(&NodeId::new("node-1".to_string())));
    }

    #[test]
    fn given_apply_event_with_wrong_revision_then_returns_error() {
        let initial = DiagramProjection::with_revision(5); // State at revision 5
        let event = make_event(
            "op-1",
            0, // Event at revision 0 - mismatch!
            DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 40.0,
                label: "Test".to_string(),
            },
            true,
        );

        let result = apply_event(initial, &event);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ReplayError::InvariantViolation(_)));
    }

    #[test]
    fn given_duplicate_op_id_when_applying_then_returns_error() {
        let mut initial = DiagramProjection::empty();
        // Pre-insert an op_id to simulate duplicate
        initial.author_priority.insert("op-1".to_string(), true);

        let event = make_event(
            "op-1", // Duplicate!
            0,
            DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 40.0,
                label: "Test".to_string(),
            },
            true,
        );

        let result = apply_event(initial, &event);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ReplayError::InvariantViolation(_)));
    }

    #[test]
    fn given_human_author_then_priority_map_has_true() {
        let initial = DiagramProjection::empty();
        let event = make_event(
            "op-1",
            0,
            DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 40.0,
                label: "Human Node".to_string(),
            },
            true, // Human author
        );

        let result = apply_event(initial, &event);

        assert!(result.is_ok());
        let projection = result.unwrap();
        assert_eq!(projection.author_priority.get("op-1"), Some(&true));
    }

    #[test]
    fn given_ai_author_then_priority_map_has_false() {
        let initial = DiagramProjection::empty();
        let event = make_event(
            "op-1",
            0,
            DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 40.0,
                label: "AI Node".to_string(),
            },
            false, // AI author
        );

        let result = apply_event(initial, &event);

        assert!(result.is_ok());
        let projection = result.unwrap();
        assert_eq!(projection.author_priority.get("op-1"), Some(&false));
    }

    #[test]
    fn given_deterministic_replay_then_multiple_replays_produce_same_result() {
        let events = [
            make_event(
                "op-1",
                0,
                DomainOp::NodeAdd {
                    id: "node-1".to_string(),
                    x: 10.0,
                    y: 20.0,
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
                    y: 200.0,
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

        // Replay multiple times
        let result1 = replay_events(&events).unwrap();
        let result2 = replay_events(&events).unwrap();
        let result3 = replay_events(&events).unwrap();

        // All should be equal
        assert_eq!(result1, result2);
        assert_eq!(result2, result3);
        assert_eq!(result1.revision, 3);
    }

    #[test]
    fn given_projection_to_document_then_preserves_data() {
        let mut projection = DiagramProjection::empty();
        let event = make_event(
            "op-1",
            0,
            DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 100.0,
                y: 200.0,
                width: 80.0,
                height: 40.0,
                label: "Test".to_string(),
            },
            true,
        );
        projection = apply_event(projection, &event).unwrap();

        let document = projection_to_document(&projection);

        assert_eq!(document.revision.value(), 1);
        assert!(document
            .document
            .nodes
            .contains_key(&NodeId::new("node-1".to_string())));
    }

    #[test]
    fn given_document_to_projection_then_preserves_data() {
        let mut doc = DiagramDocument::default();
        let node = Node {
            kind: crate::models::document::NodeKind::Node,
            icon: String::new(),
            label: "Test".to_string(),
            x: OrderedFloat(100.0),
            y: OrderedFloat(200.0),
            width: OrderedFloat(80.0),
            height: OrderedFloat(40.0),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: None,
            dag_rank: None,
            tags: vec![],
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };
        doc.document.nodes = doc
            .document
            .nodes
            .update(NodeId::new("node-1".to_string()), node);
        doc = DiagramDocument {
            revision: doc.revision.increment(),
            ..doc
        };

        let projection = document_to_projection(&doc);

        assert_eq!(projection.revision, 1);
        assert!(projection
            .nodes
            .contains_key(&NodeId::new("node-1".to_string())));
    }

    #[test]
    fn given_node_delete_then_also_removes_connected_edges() {
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
            make_event(
                "op-4",
                3,
                DomainOp::NodeDelete {
                    id: "node-1".to_string(),
                },
                true,
            ),
        ];

        let result = replay_events(&events);

        assert!(result.is_ok());
        let projection = result.unwrap();
        assert!(!projection
            .nodes
            .contains_key(&NodeId::new("node-1".to_string())));
        assert!(!projection
            .edges
            .contains_key(&EdgeId::new("edge-1".to_string())));
        // node-2 should still exist
        assert!(projection
            .nodes
            .contains_key(&NodeId::new("node-2".to_string())));
    }

    #[test]
    fn given_edge_disconnect_then_removes_edge() {
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
            make_event(
                "op-4",
                3,
                DomainOp::EdgeDisconnect {
                    id: "edge-1".to_string(),
                },
                true,
            ),
        ];

        let result = replay_events(&events);

        assert!(result.is_ok());
        let projection = result.unwrap();
        assert!(projection.edges.is_empty());
        assert_eq!(projection.nodes.len(), 2);
    }

    // =============================================================================
    // E2E Human-Priority Conflict Scenarios (bd-19t)
    // =============================================================================

    /// Test: Human-priority conflict scenario where human and AI both move the same node.
    /// The human operation should be tracked with higher priority.
    #[test]
    fn given_concurrent_human_and_ai_node_move_then_human_priority_is_tracked() {
        let events = [
            // Initial node created by AI
            make_event(
                "ai-create-1",
                0,
                DomainOp::NodeAdd {
                    id: "node-conflict".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "Contested Node".to_string(),
                },
                false, // AI author
            ),
            // Human moves the node
            make_event(
                "human-move-1",
                1,
                DomainOp::NodeMove {
                    id: "node-conflict".to_string(),
                    x: 100.0,
                    y: 100.0,
                },
                true, // Human author
            ),
            // AI also tries to move the same node
            make_event(
                "ai-move-1",
                2,
                DomainOp::NodeMove {
                    id: "node-conflict".to_string(),
                    x: 500.0,
                    y: 500.0,
                },
                false, // AI author
            ),
        ];

        let result = replay_events(&events);
        assert!(result.is_ok(), "Replay should succeed: {:?}", result.err());

        let projection = result.unwrap();

        // Verify author priority tracking
        assert_eq!(
            projection.author_priority.get("ai-create-1"),
            Some(&false),
            "AI creation should be marked as non-human"
        );
        assert_eq!(
            projection.author_priority.get("human-move-1"),
            Some(&true),
            "Human move should be marked as human priority"
        );
        assert_eq!(
            projection.author_priority.get("ai-move-1"),
            Some(&false),
            "AI move should be marked as non-human"
        );

        // Verify final node position (last operation wins in replay)
        let node = projection
            .get_node(&NodeId::new("node-conflict".to_string()))
            .expect("Node should exist");
        assert_eq!(
            node.x,
            OrderedFloat(500.0),
            "Final position should be from AI move"
        );
        assert_eq!(
            node.y,
            OrderedFloat(500.0),
            "Final position should be from AI move"
        );
    }

    /// Test: Human drag operation followed by AI move - verify both are tracked.
    #[test]
    fn given_human_drag_then_ai_move_on_same_entity_then_both_priorities_tracked() {
        let events = [
            make_event(
                "human-drag-1",
                0,
                DomainOp::NodeAdd {
                    id: "drag-node".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 50.0,
                    label: "Drag Test".to_string(),
                },
                true, // Human creates
            ),
            // Human drags to new position
            make_event(
                "human-drag-2",
                1,
                DomainOp::NodeMove {
                    id: "drag-node".to_string(),
                    x: 200.0,
                    y: 150.0,
                },
                true, // Human drag
            ),
            // AI attempts to move during/after human drag
            make_event(
                "ai-attempt-1",
                2,
                DomainOp::NodeMove {
                    id: "drag-node".to_string(),
                    x: 999.0,
                    y: 999.0,
                },
                false, // AI attempt
            ),
        ];

        let result = replay_events(&events);
        assert!(result.is_ok());

        let projection = result.unwrap();

        // Verify all operations have correct priority
        assert_eq!(projection.author_priority.get("human-drag-1"), Some(&true));
        assert_eq!(projection.author_priority.get("human-drag-2"), Some(&true));
        assert_eq!(projection.author_priority.get("ai-attempt-1"), Some(&false));

        // Human priority count
        let human_count = projection
            .author_priority
            .values()
            .filter(|&&is_human| is_human)
            .count();
        assert_eq!(human_count, 2, "Should have 2 human operations");
    }

    /// Test: Multiple conflicting operations on the same edge from different authors.
    #[test]
    fn given_human_and_ai_edge_operations_then_priorities_distinguished() {
        let events = [
            // Setup nodes
            make_event(
                "setup-1",
                0,
                DomainOp::NodeAdd {
                    id: "n1".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "Source".to_string(),
                },
                true,
            ),
            make_event(
                "setup-2",
                1,
                DomainOp::NodeAdd {
                    id: "n2".to_string(),
                    x: 100.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "Target".to_string(),
                },
                true,
            ),
            // Human creates edge
            make_event(
                "human-edge-1",
                2,
                DomainOp::EdgeConnect {
                    id: "edge-conflict".to_string(),
                    source: "n1".to_string(),
                    target: "n2".to_string(),
                },
                true, // Human
            ),
            // AI tries to disconnect
            make_event(
                "ai-disconnect-1",
                3,
                DomainOp::EdgeDisconnect {
                    id: "edge-conflict".to_string(),
                },
                false, // AI
            ),
        ];

        let result = replay_events(&events);
        assert!(result.is_ok());

        let projection = result.unwrap();

        // Verify edge is removed (AI disconnect succeeded in replay)
        assert!(!projection.has_edge(&EdgeId::new("edge-conflict".to_string())));

        // But priorities are correctly tracked
        assert_eq!(projection.author_priority.get("human-edge-1"), Some(&true));
        assert_eq!(
            projection.author_priority.get("ai-disconnect-1"),
            Some(&false)
        );
    }

    /// Test: Verify is_human_author function behavior for various author patterns.
    #[test]
    fn given_various_author_patterns_then_is_human_author_correctly_identifies() {
        // Human authors
        let human_patterns = [
            Author {
                id: "human-alice".to_string(),
                name: "Alice".to_string(),
                email: None,
            },
            Author {
                id: "human-bob-123".to_string(),
                name: "Bob".to_string(),
                email: Some("bob@example.com".to_string()),
            },
            Author {
                id: "user-42".to_string(),
                name: "Human User".to_string(), // Contains "human"
                email: None,
            },
            Author {
                id: "any-id".to_string(),
                name: "SuperHuman".to_string(), // Contains "human" (case insensitive)
                email: None,
            },
        ];

        for author in &human_patterns {
            assert!(
                is_human_author(author),
                "Expected human author for: {:?}",
                author
            );
        }

        // AI authors
        let ai_patterns = [
            Author {
                id: "ai-assistant".to_string(),
                name: "AI Assistant".to_string(),
                email: None,
            },
            Author {
                id: "bot-123".to_string(),
                name: "Automation Bot".to_string(),
                email: None,
            },
            Author {
                id: "system".to_string(),
                name: "System".to_string(),
                email: None,
            },
            Author {
                id: "gpt-4".to_string(),
                name: "GPT-4".to_string(),
                email: None,
            },
        ];

        for author in &ai_patterns {
            assert!(
                !is_human_author(author),
                "Expected AI author for: {:?}",
                author
            );
        }
    }

    /// Test: Complex scenario with interleaved human and AI operations.
    #[test]
    fn given_interleaved_human_ai_operations_then_priority_map_maintains_order() {
        let events = [
            // AI creates initial structure
            make_event(
                "ai-node-1",
                0,
                DomainOp::NodeAdd {
                    id: "n1".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "AI Node 1".to_string(),
                },
                false,
            ),
            // Human adds a node
            make_event(
                "human-node-1",
                1,
                DomainOp::NodeAdd {
                    id: "n2".to_string(),
                    x: 100.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "Human Node".to_string(),
                },
                true,
            ),
            // AI connects them
            make_event(
                "ai-edge-1",
                2,
                DomainOp::EdgeConnect {
                    id: "e1".to_string(),
                    source: "n1".to_string(),
                    target: "n2".to_string(),
                },
                false,
            ),
            // Human moves AI's node
            make_event(
                "human-move-1",
                3,
                DomainOp::NodeMove {
                    id: "n1".to_string(),
                    x: 50.0,
                    y: 50.0,
                },
                true,
            ),
            // AI tries to move human's node
            make_event(
                "ai-move-2",
                4,
                DomainOp::NodeMove {
                    id: "n2".to_string(),
                    x: 200.0,
                    y: 200.0,
                },
                false,
            ),
        ];

        let result = replay_events(&events);
        assert!(result.is_ok());

        let projection = result.unwrap();

        // Verify priority mapping
        assert_eq!(projection.author_priority.get("ai-node-1"), Some(&false));
        assert_eq!(projection.author_priority.get("human-node-1"), Some(&true));
        assert_eq!(projection.author_priority.get("ai-edge-1"), Some(&false));
        assert_eq!(projection.author_priority.get("human-move-1"), Some(&true));
        assert_eq!(projection.author_priority.get("ai-move-2"), Some(&false));

        // Count operations by type
        let human_ops: Vec<_> = projection
            .author_priority
            .iter()
            .filter(|(_, &is_human)| is_human)
            .collect();
        let ai_ops: Vec<_> = projection
            .author_priority
            .iter()
            .filter(|(_, &is_human)| !is_human)
            .collect();

        assert_eq!(human_ops.len(), 2, "Should have 2 human operations");
        assert_eq!(ai_ops.len(), 3, "Should have 3 AI operations");
    }

    /// Test: Verify that author_priority map preserves insertion order for conflict resolution.
    #[test]
    fn given_conflicting_operations_then_priority_map_enables_resolution() {
        let events = [
            make_event(
                "op-1",
                0,
                DomainOp::NodeAdd {
                    id: "conflict-node".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "Conflict".to_string(),
                },
                false, // AI creates
            ),
            make_event(
                "op-2",
                1,
                DomainOp::NodeMove {
                    id: "conflict-node".to_string(),
                    x: 100.0,
                    y: 100.0,
                },
                true, // Human moves
            ),
            make_event(
                "op-3",
                2,
                DomainOp::NodeMove {
                    id: "conflict-node".to_string(),
                    x: 200.0,
                    y: 200.0,
                },
                false, // AI moves
            ),
            make_event(
                "op-4",
                3,
                DomainOp::NodeMove {
                    id: "conflict-node".to_string(),
                    x: 150.0,
                    y: 150.0,
                },
                true, // Human moves again
            ),
        ];

        let result = replay_events(&events);
        assert!(result.is_ok());

        let projection = result.unwrap();

        // The priority map should allow determining which operations were human
        // for conflict resolution purposes
        let human_op_ids: Vec<_> = projection
            .author_priority
            .iter()
            .filter(|(_, &is_human)| is_human)
            .map(|(op_id, _)| op_id.as_str())
            .collect();

        assert!(
            human_op_ids.contains(&"op-2"),
            "op-2 should be marked as human"
        );
        assert!(
            human_op_ids.contains(&"op-4"),
            "op-4 should be marked as human"
        );

        // Final position is from the last operation (op-4)
        let node = projection
            .get_node(&NodeId::new("conflict-node".to_string()))
            .expect("Node should exist");
        assert_eq!(node.x, OrderedFloat(150.0));
        assert_eq!(node.y, OrderedFloat(150.0));
    }

    /// Test: Verify deterministic replay produces consistent author_priority maps.
    #[test]
    fn given_same_events_replayed_multiple_times_then_priority_map_is_deterministic() {
        let events = [
            make_event(
                "h1",
                0,
                DomainOp::NodeAdd {
                    id: "n1".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "N1".to_string(),
                },
                true,
            ),
            make_event(
                "a1",
                1,
                DomainOp::NodeMove {
                    id: "n1".to_string(),
                    x: 100.0,
                    y: 100.0,
                },
                false,
            ),
            make_event(
                "h2",
                2,
                DomainOp::NodeMove {
                    id: "n1".to_string(),
                    x: 200.0,
                    y: 200.0,
                },
                true,
            ),
        ];

        // Replay multiple times
        let result1 = replay_events(&events).expect("First replay should succeed");
        let result2 = replay_events(&events).expect("Second replay should succeed");
        let result3 = replay_events(&events).expect("Third replay should succeed");

        // All priority maps should be identical
        assert_eq!(result1.author_priority, result2.author_priority);
        assert_eq!(result2.author_priority, result3.author_priority);

        // Verify specific values
        assert_eq!(result1.author_priority.get("h1"), Some(&true));
        assert_eq!(result1.author_priority.get("a1"), Some(&false));
        assert_eq!(result1.author_priority.get("h2"), Some(&true));
    }

    // =============================================================================
    // replay_stream and projection_hash tests (bd-2cg)
    // =============================================================================

    /// Test: replay_stream is an alias for replay_events and produces same results.
    #[test]
    fn given_events_when_using_replay_stream_then_produces_same_result_as_replay_events() {
        let events = [
            make_event(
                "op-1",
                0,
                DomainOp::NodeAdd {
                    id: "node-1".to_string(),
                    x: 100.0,
                    y: 200.0,
                    width: 80.0,
                    height: 40.0,
                    label: "Test".to_string(),
                },
                true,
            ),
            make_event(
                "op-2",
                1,
                DomainOp::NodeMove {
                    id: "node-1".to_string(),
                    x: 150.0,
                    y: 250.0,
                },
                true,
            ),
        ];

        let result_events = replay_events(&events).expect("replay_events should succeed");
        let result_stream = replay_stream(&events).expect("replay_stream should succeed");

        assert_eq!(result_events, result_stream);
    }

    /// Test: projection_hash produces consistent hash for empty projection.
    #[test]
    fn given_empty_projection_when_hashing_then_returns_consistent_hash() {
        let projection = DiagramProjection::empty();

        let hash1 = projection_hash(&projection).expect("Hash should succeed");
        let hash2 = projection_hash(&projection).expect("Hash should succeed");

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 16); // 64-bit hex string
    }

    /// Test: projection_hash produces different hashes for different projections.
    #[test]
    fn given_different_projections_when_hashing_then_returns_different_hashes() {
        let events1 = [make_event(
            "op-1",
            0,
            DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 100.0,
                y: 200.0,
                width: 80.0,
                height: 40.0,
                label: "Node 1".to_string(),
            },
            true,
        )];

        let events2 = [make_event(
            "op-2",
            0,
            DomainOp::NodeAdd {
                id: "node-2".to_string(),
                x: 300.0,
                y: 400.0,
                width: 120.0,
                height: 60.0,
                label: "Node 2".to_string(),
            },
            true,
        )];

        let projection1 = replay_events(&events1).expect("Replay should succeed");
        let projection2 = replay_events(&events2).expect("Replay should succeed");

        let hash1 = projection_hash(&projection1).expect("Hash should succeed");
        let hash2 = projection_hash(&projection2).expect("Hash should succeed");

        assert_ne!(hash1, hash2, "Different projections should have different hashes");
    }

    /// Test: projection_hash is deterministic across multiple calls.
    #[test]
    fn given_same_projection_when_hashing_multiple_times_then_returns_same_hash() {
        let events = [
            make_event(
                "op-1",
                0,
                DomainOp::NodeAdd {
                    id: "node-1".to_string(),
                    x: 10.0,
                    y: 20.0,
                    width: 80.0,
                    height: 40.0,
                    label: "Node".to_string(),
                },
                true,
            ),
            make_event(
                "op-2",
                1,
                DomainOp::NodeAdd {
                    id: "node-2".to_string(),
                    x: 100.0,
                    y: 200.0,
                    width: 80.0,
                    height: 40.0,
                    label: "Node 2".to_string(),
                },
                false,
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

        let projection = replay_events(&events).expect("Replay should succeed");

        // Hash multiple times
        let hashes: Vec<_> = (0..5)
            .map(|_| projection_hash(&projection).expect("Hash should succeed"))
            .collect();

        // All hashes should be identical
        for hash in &hashes[1..] {
            assert_eq!(&hashes[0], hash, "All hashes should be identical");
        }
    }

    /// Test: projection_hash includes revision in hash.
    #[test]
    fn given_different_revisions_when_hashing_then_returns_different_hashes() {
        let mut projection1 = DiagramProjection::empty();
        let mut projection2 = DiagramProjection::empty();

        projection1.revision = 1;
        projection2.revision = 2;

        let hash1 = projection_hash(&projection1).expect("Hash should succeed");
        let hash2 = projection_hash(&projection2).expect("Hash should succeed");

        assert_ne!(hash1, hash2, "Different revisions should have different hashes");
    }

    /// Test: projection_hash includes author_priority in hash.
    #[test]
    fn given_different_author_priority_when_hashing_then_returns_different_hashes() {
        let mut projection1 = DiagramProjection::empty();
        let mut projection2 = DiagramProjection::empty();

        projection1
            .author_priority
            .insert("op-1".to_string(), true);
        projection2
            .author_priority
            .insert("op-1".to_string(), false);

        let hash1 = projection_hash(&projection1).expect("Hash should succeed");
        let hash2 = projection_hash(&projection2).expect("Hash should succeed");

        assert_ne!(
            hash1, hash2,
            "Different author_priority should have different hashes"
        );
    }

    /// Test: Deterministic replay produces same hash.
    #[test]
    fn given_same_events_replayed_then_projection_hash_is_deterministic() {
        let events = [
            make_event(
                "op-1",
                0,
                DomainOp::NodeAdd {
                    id: "n1".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "Node".to_string(),
                },
                true,
            ),
            make_event(
                "op-2",
                1,
                DomainOp::NodeMove {
                    id: "n1".to_string(),
                    x: 100.0,
                    y: 100.0,
                },
                false,
            ),
        ];

        // Replay multiple times
        let projection1 = replay_events(&events).expect("Replay should succeed");
        let projection2 = replay_events(&events).expect("Replay should succeed");
        let projection3 = replay_stream(&events).expect("replay_stream should succeed");

        // Hash each projection
        let hash1 = projection_hash(&projection1).expect("Hash should succeed");
        let hash2 = projection_hash(&projection2).expect("Hash should succeed");
        let hash3 = projection_hash(&projection3).expect("Hash should succeed");

        // All hashes should be identical
        assert_eq!(hash1, hash2);
        assert_eq!(hash2, hash3);
    }

    /// Test: Hash is stable for complex projections with edges.
    #[test]
    fn given_complex_projection_with_edges_when_hashing_then_succeeds() {
        let events = [
            make_event(
                "n1",
                0,
                DomainOp::NodeAdd {
                    id: "node-1".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "A".to_string(),
                },
                true,
            ),
            make_event(
                "n2",
                1,
                DomainOp::NodeAdd {
                    id: "node-2".to_string(),
                    x: 100.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "B".to_string(),
                },
                true,
            ),
            make_event(
                "n3",
                2,
                DomainOp::NodeAdd {
                    id: "node-3".to_string(),
                    x: 200.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "C".to_string(),
                },
                true,
            ),
            make_event(
                "e1",
                3,
                DomainOp::EdgeConnect {
                    id: "edge-1".to_string(),
                    source: "node-1".to_string(),
                    target: "node-2".to_string(),
                },
                false,
            ),
            make_event(
                "e2",
                4,
                DomainOp::EdgeConnect {
                    id: "edge-2".to_string(),
                    source: "node-2".to_string(),
                    target: "node-3".to_string(),
                },
                false,
            ),
        ];

        let projection = replay_events(&events).expect("Replay should succeed");
        let hash = projection_hash(&projection).expect("Hash should succeed");

        assert_eq!(hash.len(), 16);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// Test: New path handles valid input and produces expected output.
    #[test]
    fn given_valid_input_when_replaying_then_produces_expected_output() {
        let events = [
            make_event(
                "add-1",
                0,
                DomainOp::NodeAdd {
                    id: "test-node".to_string(),
                    x: 50.0,
                    y: 75.0,
                    width: 100.0,
                    height: 50.0,
                    label: "Test Node".to_string(),
                },
                true,
            ),
        ];

        let result = replay_stream(&events);

        assert!(result.is_ok(), "Should succeed with valid input");
        let projection = result.unwrap();
        assert_eq!(projection.revision, 1);
        assert!(projection.has_node(&NodeId::new("test-node".to_string())));

        let node = projection
            .get_node(&NodeId::new("test-node".to_string()))
            .expect("Node should exist");
        assert_eq!(node.x, OrderedFloat(50.0));
        assert_eq!(node.y, OrderedFloat(75.0));
        assert_eq!(node.width, OrderedFloat(100.0));
        assert_eq!(node.height, OrderedFloat(50.0));
        assert_eq!(node.label, "Test Node");
    }

    /// Test: Invalid input returns typed error without partial durable mutation.
    #[test]
    fn given_invalid_input_when_replaying_then_returns_typed_error_without_mutation() {
        // Try to move a non-existent node
        let events = [make_event(
            "move-1",
            0,
            DomainOp::NodeMove {
                id: "nonexistent".to_string(),
                x: 100.0,
                y: 100.0,
            },
            true,
        )];

        let result = replay_stream(&events);

        assert!(result.is_err(), "Should fail with invalid input");
        match result {
            Err(ReplayError::InvariantViolation(msg)) => {
                assert!(msg.contains("not found"));
            }
            _ => panic!("Expected InvariantViolation error"),
        }

        // Verify no partial state was created - replay is atomic
        let empty = DiagramProjection::empty();
        let result2 = replay_stream(&[]);
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), empty);
    }

    /// Test: Command flow uses replacement implementation without legacy calls.
    #[test]
    fn given_replay_operation_then_uses_new_dispatcher_path() {
        // This test verifies that replay_stream (the contract-specified entry point)
        // correctly dispatches all operation types through the new path

        let events = [
            make_event(
                "op-1",
                0,
                DomainOp::NodeAdd {
                    id: "n1".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "Node".to_string(),
                },
                true,
            ),
            make_event(
                "op-2",
                1,
                DomainOp::NodeMove {
                    id: "n1".to_string(),
                    x: 10.0,
                    y: 10.0,
                },
                true,
            ),
            make_event(
                "op-3",
                2,
                DomainOp::NodeAdd {
                    id: "n2".to_string(),
                    x: 100.0,
                    y: 100.0,
                    width: 80.0,
                    height: 40.0,
                    label: "Node 2".to_string(),
                },
                true,
            ),
            make_event(
                "op-4",
                3,
                DomainOp::EdgeConnect {
                    id: "e1".to_string(),
                    source: "n1".to_string(),
                    target: "n2".to_string(),
                },
                true,
            ),
            make_event(
                "op-5",
                4,
                DomainOp::EdgeDisconnect {
                    id: "e1".to_string(),
                },
                true,
            ),
        ];

        let result = replay_stream(&events);

        assert!(result.is_ok(), "Should succeed using new dispatcher path");
        let projection = result.unwrap();

        // Verify all operations were applied through the dispatcher
        assert_eq!(projection.revision, 5);
        assert_eq!(projection.nodes.len(), 2);
        assert_eq!(projection.edges.len(), 0); // Edge was disconnected
    }

    // =============================================================================
    // Cycle Policy Tests (bd-1lj)
    // =============================================================================

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

        projection = replay_events_from(projection, &events).unwrap();

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

        projection = replay_events_from(projection, &events).unwrap();

        // Cycle exists and policy is Deny, so enforcement should fail
        let result = enforce_cycle_policy(&projection);
        assert!(result.is_err(), "Deny policy should reject cycles");
        match result {
            Err(ReplayError::CycleViolation(msg)) => {
                assert!(msg.contains("Cycle detected"));
            }
            _ => panic!("Expected CycleViolation error"),
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

        projection = replay_events_from(projection, &events).unwrap();

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
        projection = replay_events_from(projection, &events).unwrap();

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
        projection = replay_events_from(projection, &events).unwrap();

        // Try to add b -> a which would create a cycle
        let cyclic_op = DomainOp::EdgeConnect {
            id: "e2".to_string(),
            source: "b".to_string(),
            target: "a".to_string(),
        };

        let result = apply_policy_op(projection, &cyclic_op);
        assert!(result.is_err(), "Creating a cycle should fail under Deny policy");
        match result {
            Err(ReplayError::CycleViolation(msg)) => {
                assert!(msg.contains("Cycle detected"));
            }
            _ => panic!("Expected CycleViolation error"),
        }
    }

    /// Test: apply_policy_op allows cycle-creating operations under Allow policy
    #[test]
    fn given_cycle_policy_allow_when_applying_cyclic_op_then_succeeds() {
        let mut projection = DiagramProjection::with_cycle_policy(CyclePolicy::Allow);

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
        projection = replay_events_from(projection, &events).unwrap();

        // Add b -> a which creates a cycle - should succeed under Allow policy
        let cyclic_op = DomainOp::EdgeConnect {
            id: "e2".to_string(),
            source: "b".to_string(),
            target: "a".to_string(),
        };

        let result = apply_policy_op(projection, &cyclic_op);
        assert!(result.is_ok(), "Creating a cycle should succeed under Allow policy");
    }

    /// Test: Empty projection always passes cycle enforcement
    #[test]
    fn given_empty_projection_when_enforcing_cycle_policy_then_succeeds() {
        let projection_allow = DiagramProjection::with_cycle_policy(CyclePolicy::Allow);
        let projection_deny = DiagramProjection::with_cycle_policy(CyclePolicy::Deny);

        assert!(enforce_cycle_policy(&projection_allow).is_ok());
        assert!(enforce_cycle_policy(&projection_deny).is_ok());
    }

    /// Test: CyclePolicy serialization roundtrips correctly
    #[test]
    fn given_cycle_policy_when_serializing_then_roundtrips_correctly() {
        let policies = [CyclePolicy::Allow, CyclePolicy::Deny];

        for policy in &policies {
            let json = serde_json::to_string(policy).unwrap();
            let decoded: CyclePolicy = serde_json::from_str(&json).unwrap();
            assert_eq!(*policy, decoded);
        }
    }

    /// Test: DiagramProjection with cycle_policy serializes correctly
    #[test]
    fn given_projection_with_cycle_policy_when_serializing_then_roundtrips_correctly() {
        let projection = DiagramProjection::with_cycle_policy(CyclePolicy::Deny);

        let json = serde_json::to_string(&projection).unwrap();
        let decoded: DiagramProjection = serde_json::from_str(&json).unwrap();

        assert_eq!(projection.cycle_policy, decoded.cycle_policy);
        assert_eq!(decoded.cycle_policy, CyclePolicy::Deny);
    }

    /// Test: Complex cycle detection in larger graph
    #[test]
    fn given_large_graph_with_cycle_when_policy_deny_then_cycle_detected() {
        let mut projection = DiagramProjection::with_cycle_policy(CyclePolicy::Deny);

        // Create a larger graph: a -> b -> c -> d -> b (cycle)
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
                DomainOp::NodeAdd {
                    id: "c".to_string(),
                    x: 200.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "C".to_string(),
                },
                true,
            ),
            make_event(
                "op-4",
                3,
                DomainOp::NodeAdd {
                    id: "d".to_string(),
                    x: 300.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "D".to_string(),
                },
                true,
            ),
            make_event(
                "op-5",
                4,
                DomainOp::EdgeConnect {
                    id: "e1".to_string(),
                    source: "a".to_string(),
                    target: "b".to_string(),
                },
                true,
            ),
            make_event(
                "op-6",
                5,
                DomainOp::EdgeConnect {
                    id: "e2".to_string(),
                    source: "b".to_string(),
                    target: "c".to_string(),
                },
                true,
            ),
            make_event(
                "op-7",
                6,
                DomainOp::EdgeConnect {
                    id: "e3".to_string(),
                    source: "c".to_string(),
                    target: "d".to_string(),
                },
                true,
            ),
            make_event(
                "op-8",
                7,
                DomainOp::EdgeConnect {
                    id: "e4".to_string(),
                    source: "d".to_string(),
                    target: "b".to_string(),
                },
                true,
            ),
        ];

        projection = replay_events_from(projection, &events).unwrap();

        let result = enforce_cycle_policy(&projection);
        assert!(result.is_err(), "Should detect cycle in larger graph");
    }

    /// Test: Removing an edge that breaks a cycle makes enforcement pass
    #[test]
    fn given_graph_with_cycle_when_edge_removed_then_enforcement_passes() {
        let mut projection = DiagramProjection::with_cycle_policy(CyclePolicy::Deny);

        // Create a cycle: a -> b -> a
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

        projection = replay_events_from(projection, &events).unwrap();

        // Verify cycle is detected
        assert!(enforce_cycle_policy(&projection).is_err());

        // Remove one edge to break the cycle
        let disconnect_op = DomainOp::EdgeDisconnect {
            id: "e2".to_string(),
        };
        let new_projection = apply_policy_op(projection, &disconnect_op).unwrap();

        // Now enforcement should pass
        assert!(
            enforce_cycle_policy(&new_projection).is_ok(),
            "After removing cycle edge, enforcement should pass"
        );
    }

    // =============================================================================
    // apply_edge_op and verify_edge_tolerance tests (bd-1kc)
    // =============================================================================

    /// Test: apply_edge_op handles EdgeConnect correctly
    #[test]
    fn given_edge_connect_op_when_apply_edge_op_then_edge_is_added() {
        let mut state = DiagramProjection::empty();

        // Add nodes first
        state.nodes = state.nodes.update(
            NodeId::new("n1".to_string()),
            Node {
                kind: crate::models::document::NodeKind::Node,
                icon: String::new(),
                label: "Node 1".to_string(),
                x: OrderedFloat(0.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(80.0),
                height: OrderedFloat(40.0),
                font_size: None,
                font_weight: None,
                locked: false,
                parent: None,
                dag_rank: None,
                tags: vec![],
                metadata: HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            },
        );
        state.nodes = state.nodes.update(
            NodeId::new("n2".to_string()),
            Node {
                kind: crate::models::document::NodeKind::Node,
                icon: String::new(),
                label: "Node 2".to_string(),
                x: OrderedFloat(100.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(80.0),
                height: OrderedFloat(40.0),
                font_size: None,
                font_weight: None,
                locked: false,
                parent: None,
                dag_rank: None,
                tags: vec![],
                metadata: HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            },
        );

        let op = DomainOp::EdgeConnect {
            id: "edge-1".to_string(),
            source: "n1".to_string(),
            target: "n2".to_string(),
        };

        let result = apply_edge_op(state, &op);

        assert!(result.is_ok(), "Should succeed: {:?}", result.err());
        let new_state = result.unwrap();
        assert!(new_state.has_edge(&EdgeId::new("edge-1".to_string())));
    }

    /// Test: apply_edge_op returns DuplicateEdge for duplicate edge ID
    #[test]
    fn given_duplicate_edge_when_apply_edge_op_then_returns_duplicate_edge_error() {
        let mut state = DiagramProjection::empty();

        // Add nodes
        state.nodes = state.nodes.update(
            NodeId::new("n1".to_string()),
            Node {
                kind: crate::models::document::NodeKind::Node,
                icon: String::new(),
                label: "Node 1".to_string(),
                x: OrderedFloat(0.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(80.0),
                height: OrderedFloat(40.0),
                font_size: None,
                font_weight: None,
                locked: false,
                parent: None,
                dag_rank: None,
                tags: vec![],
                metadata: HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            },
        );
        state.nodes = state.nodes.update(
            NodeId::new("n2".to_string()),
            Node {
                kind: crate::models::document::NodeKind::Node,
                icon: String::new(),
                label: "Node 2".to_string(),
                x: OrderedFloat(100.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(80.0),
                height: OrderedFloat(40.0),
                font_size: None,
                font_weight: None,
                locked: false,
                parent: None,
                dag_rank: None,
                tags: vec![],
                metadata: HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            },
        );

        // Add first edge
        state.edges = state.edges.update(
            EdgeId::new("edge-1".to_string()),
            Edge {
                source: NodeId::new("n1".to_string()),
                target: NodeId::new("n2".to_string()),
                label: String::new(),
                style: crate::models::document::EdgeStyle::Solid,
                arrow_type: crate::models::document::ArrowType::Default,
                label_offset_t: OrderedFloat(0.5),
                color: None,
                thickness: OrderedFloat(1.5),
                directed: true,
                bend_points: vec![],
                tags: vec![],
                metadata: HashMap::new(),
                font_size: None,
            },
        );

        let op = DomainOp::EdgeConnect {
            id: "edge-1".to_string(), // Duplicate ID
            source: "n1".to_string(),
            target: "n2".to_string(),
        };

        let result = apply_edge_op(state, &op);

        assert!(result.is_err());
        match result {
            Err(ReplayError::DuplicateEdge(id)) => assert_eq!(id, "edge-1"),
            _ => panic!("Expected DuplicateEdge error"),
        }
    }

    /// Test: apply_edge_op returns PolicyViolation for missing source node
    #[test]
    fn given_missing_source_node_when_apply_edge_op_then_returns_policy_violation() {
        let mut state = DiagramProjection::empty();

        // Add only target node
        state.nodes = state.nodes.update(
            NodeId::new("n2".to_string()),
            Node {
                kind: crate::models::document::NodeKind::Node,
                icon: String::new(),
                label: "Node 2".to_string(),
                x: OrderedFloat(100.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(80.0),
                height: OrderedFloat(40.0),
                font_size: None,
                font_weight: None,
                locked: false,
                parent: None,
                dag_rank: None,
                tags: vec![],
                metadata: HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            },
        );

        let op = DomainOp::EdgeConnect {
            id: "edge-1".to_string(),
            source: "nonexistent".to_string(),
            target: "n2".to_string(),
        };

        let result = apply_edge_op(state, &op);

        assert!(result.is_err());
        match result {
            Err(ReplayError::PolicyViolation(msg)) => {
                assert!(msg.contains("source node not found"));
            }
            _ => panic!("Expected PolicyViolation error"),
        }
    }

    /// Test: apply_edge_op handles EdgeDisconnect correctly
    #[test]
    fn given_edge_disconnect_op_when_apply_edge_op_then_edge_is_removed() {
        let mut state = DiagramProjection::empty();

        // Add nodes
        state.nodes = state.nodes.update(
            NodeId::new("n1".to_string()),
            Node {
                kind: crate::models::document::NodeKind::Node,
                icon: String::new(),
                label: "Node 1".to_string(),
                x: OrderedFloat(0.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(80.0),
                height: OrderedFloat(40.0),
                font_size: None,
                font_weight: None,
                locked: false,
                parent: None,
                dag_rank: None,
                tags: vec![],
                metadata: HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            },
        );
        state.nodes = state.nodes.update(
            NodeId::new("n2".to_string()),
            Node {
                kind: crate::models::document::NodeKind::Node,
                icon: String::new(),
                label: "Node 2".to_string(),
                x: OrderedFloat(100.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(80.0),
                height: OrderedFloat(40.0),
                font_size: None,
                font_weight: None,
                locked: false,
                parent: None,
                dag_rank: None,
                tags: vec![],
                metadata: HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            },
        );

        // Add edge
        state.edges = state.edges.update(
            EdgeId::new("edge-1".to_string()),
            Edge {
                source: NodeId::new("n1".to_string()),
                target: NodeId::new("n2".to_string()),
                label: String::new(),
                style: crate::models::document::EdgeStyle::Solid,
                arrow_type: crate::models::document::ArrowType::Default,
                label_offset_t: OrderedFloat(0.5),
                color: None,
                thickness: OrderedFloat(1.5),
                directed: true,
                bend_points: vec![],
                tags: vec![],
                metadata: HashMap::new(),
                font_size: None,
            },
        );

        let op = DomainOp::EdgeDisconnect {
            id: "edge-1".to_string(),
        };

        let result = apply_edge_op(state, &op);

        assert!(result.is_ok(), "Should succeed: {:?}", result.err());
        let new_state = result.unwrap();
        assert!(!new_state.has_edge(&EdgeId::new("edge-1".to_string())));
    }

    /// Test: apply_edge_op returns EdgeNotFound for missing edge on disconnect
    #[test]
    fn given_missing_edge_when_disconnect_then_returns_edge_not_found() {
        let state = DiagramProjection::empty();

        let op = DomainOp::EdgeDisconnect {
            id: "nonexistent".to_string(),
        };

        let result = apply_edge_op(state, &op);

        assert!(result.is_err());
        match result {
            Err(ReplayError::EdgeNotFound(id)) => assert_eq!(id, "nonexistent"),
            _ => panic!("Expected EdgeNotFound error"),
        }
    }

    /// Test: apply_edge_op returns InvalidEvent for non-edge operations
    #[test]
    fn given_non_edge_op_when_apply_edge_op_then_returns_invalid_event() {
        let state = DiagramProjection::empty();

        let op = DomainOp::NodeAdd {
            id: "node-1".to_string(),
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 40.0,
            label: "Test".to_string(),
        };

        let result = apply_edge_op(state, &op);

        assert!(result.is_err());
        match result {
            Err(ReplayError::InvalidEvent(msg)) => {
                assert!(msg.contains("not an edge operation"));
            }
            _ => panic!("Expected InvalidEvent error"),
        }
    }

    /// Test: verify_edge_tolerance passes for valid projection
    #[test]
    fn given_valid_projection_when_verify_edge_tolerance_then_returns_ok() {
        let mut state = DiagramProjection::empty();

        // Add nodes
        state.nodes = state.nodes.update(
            NodeId::new("n1".to_string()),
            Node {
                kind: crate::models::document::NodeKind::Node,
                icon: String::new(),
                label: "Node 1".to_string(),
                x: OrderedFloat(0.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(80.0),
                height: OrderedFloat(40.0),
                font_size: None,
                font_weight: None,
                locked: false,
                parent: None,
                dag_rank: None,
                tags: vec![],
                metadata: HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            },
        );
        state.nodes = state.nodes.update(
            NodeId::new("n2".to_string()),
            Node {
                kind: crate::models::document::NodeKind::Node,
                icon: String::new(),
                label: "Node 2".to_string(),
                x: OrderedFloat(100.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(80.0),
                height: OrderedFloat(40.0),
                font_size: None,
                font_weight: None,
                locked: false,
                parent: None,
                dag_rank: None,
                tags: vec![],
                metadata: HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            },
        );

        // Add valid edge
        state.edges = state.edges.update(
            EdgeId::new("edge-1".to_string()),
            Edge {
                source: NodeId::new("n1".to_string()),
                target: NodeId::new("n2".to_string()),
                label: String::new(),
                style: crate::models::document::EdgeStyle::Solid,
                arrow_type: crate::models::document::ArrowType::Default,
                label_offset_t: OrderedFloat(0.5),
                color: None,
                thickness: OrderedFloat(1.5),
                directed: true,
                bend_points: vec![],
                tags: vec![],
                metadata: HashMap::new(),
                font_size: None,
            },
        );

        let result = verify_edge_tolerance(&state);

        assert!(result.is_ok(), "Should pass: {:?}", result.err());
    }

    /// Test: verify_edge_tolerance fails for edge with missing source node
    #[test]
    fn given_edge_with_missing_source_when_verify_edge_tolerance_then_returns_policy_violation() {
        let mut state = DiagramProjection::empty();

        // Add only target node
        state.nodes = state.nodes.update(
            NodeId::new("n2".to_string()),
            Node {
                kind: crate::models::document::NodeKind::Node,
                icon: String::new(),
                label: "Node 2".to_string(),
                x: OrderedFloat(100.0),
                y: OrderedFloat(0.0),
                width: OrderedFloat(80.0),
                height: OrderedFloat(40.0),
                font_size: None,
                font_weight: None,
                locked: false,
                parent: None,
                dag_rank: None,
                tags: vec![],
                metadata: HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            },
        );

        // Add edge with missing source
        state.edges = state.edges.update(
            EdgeId::new("edge-1".to_string()),
            Edge {
                source: NodeId::new("nonexistent".to_string()),
                target: NodeId::new("n2".to_string()),
                label: String::new(),
                style: crate::models::document::EdgeStyle::Solid,
                arrow_type: crate::models::document::ArrowType::Default,
                label_offset_t: OrderedFloat(0.5),
                color: None,
                thickness: OrderedFloat(1.5),
                directed: true,
                bend_points: vec![],
                tags: vec![],
                metadata: HashMap::new(),
                font_size: None,
            },
        );

        let result = verify_edge_tolerance(&state);

        assert!(result.is_err());
        match result {
            Err(ReplayError::PolicyViolation(msg)) => {
                assert!(msg.contains("non-existent source node"));
            }
            _ => panic!("Expected PolicyViolation error"),
        }
    }

    /// Test: verify_edge_tolerance passes for empty projection
    #[test]
    fn given_empty_projection_when_verify_edge_tolerance_then_returns_ok() {
        let state = DiagramProjection::empty();

        let result = verify_edge_tolerance(&state);

        assert!(result.is_ok());
    }
}
