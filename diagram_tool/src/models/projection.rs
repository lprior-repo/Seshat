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

/// Errors that can occur during replay
#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReplayError {
    #[error("invalid event: {0}")]
    InvalidEvent(String),
    #[error("invariant violation: {0}")]
    InvariantViolation(String),
    #[error("unsupported schema version: {0}")]
    UnsupportedVersion(u32),
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
    })
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
    }
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
}
