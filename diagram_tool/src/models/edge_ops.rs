//! Edge operations module - isolated edge connectivity and tolerance checking
//!
//! This module provides pure functions for edge operations including connect,
//! disconnect, and tolerance verification. All functions follow the functional
//! pattern: input state + parameters -> Result<`new_state`, error>.
//!
//! ## Design by Contract
//!
//! ### Preconditions
//! - P1a: Edge ID must not exist before connect
//! - P1b: Source node must exist before connect
//! - P1c: Target node must exist before connect
//! - P2: Edge ID must exist before disconnect
//!
//! ### Postconditions
//! - Q1: New edge present in returned projection after connect
//! - Q2: Edge absent in returned projection after disconnect
//! - Q3: Tolerance check returns Ok for valid edges
//!
//! ### Invariants
//! - I1: Edge IDs unique within projection
//! - I2: All edges reference valid nodes
//! - I3: Edge geometry values finite

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(dead_code)]

use im::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::models::document::{
    DiagramDocument, DocumentData, Edge, EdgeId, Node, NodeId, OrderedFloat,
};
use crate::models::envelope::DomainOp;

/// Current supported schema version
const SUPPORTED_VERSION: u32 = 2;

/// Errors that can occur during edge operations
#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EdgeOpsError {
    #[error("invalid event: {0}")]
    InvalidEvent(String),
    #[error("invariant violation: {0}")]
    InvariantViolation(String),
    #[error("edge not found: {0}")]
    EdgeNotFound(String),
    #[error("duplicate edge: {0}")]
    DuplicateEdge(String),
    #[error("policy violation: {0}")]
    PolicyViolation(String),
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

/// Apply `EdgeConnect` operation
///
/// Creates a new edge connecting source node to target node.
///
/// # Errors
/// Returns `EdgeOpsError::InvariantViolation` if:
/// - Edge ID already exists (`DuplicateEdge`)
/// - Source node doesn't exist
/// - Target node doesn't exist
pub fn apply_edge_connect(
    state: DiagramProjection,
    id: &str,
    source: &str,
    target: &str,
) -> Result<DiagramProjection, EdgeOpsError> {
    let edge_id = EdgeId::new(id.to_string());
    let source_id = NodeId::new(source.to_string());
    let target_id = NodeId::new(target.to_string());

    // Check for duplicate edge ID
    if state.has_edge(&edge_id) {
        return Err(EdgeOpsError::InvariantViolation(format!(
            "duplicate edge ID: {id}"
        )));
    }

    // Validate source and target nodes exist
    if !state.has_node(&source_id) {
        return Err(EdgeOpsError::InvariantViolation(format!(
            "source node not found: {source}"
        )));
    }
    if !state.has_node(&target_id) {
        return Err(EdgeOpsError::InvariantViolation(format!(
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
    })
}

/// Apply `EdgeDisconnect` operation
///
/// Removes an edge by its ID.
///
/// # Errors
/// Returns `EdgeOpsError::InvariantViolation` if:
/// - Edge ID doesn't exist
pub fn apply_edge_disconnect(
    state: DiagramProjection,
    id: &str,
) -> Result<DiagramProjection, EdgeOpsError> {
    let edge_id = EdgeId::new(id.to_string());

    // Check edge exists
    if !state.has_edge(&edge_id) {
        return Err(EdgeOpsError::InvariantViolation(format!(
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

/// Apply `EdgeStyle` operation
///
/// Updates the style of an existing edge.
///
/// # Errors
/// Returns `EdgeOpsError::EdgeNotFound` if:
/// - Edge ID doesn't exist
pub fn apply_edge_style(
    state: DiagramProjection,
    id: &str,
    style: crate::models::document::EdgeStyle,
) -> Result<DiagramProjection, EdgeOpsError> {
    let edge_id = EdgeId::new(id.to_string());

    let edge = state
        .edges
        .get(&edge_id)
        .ok_or_else(|| EdgeOpsError::EdgeNotFound(id.to_string()))?;

    let updated_edge = Edge {
        style,
        ..edge.clone()
    };

    let new_edges = state.edges.update(edge_id, updated_edge);

    Ok(DiagramProjection {
        version: state.version,
        revision: state.revision,
        nodes: state.nodes,
        edges: new_edges,
        author_priority: state.author_priority,
    })
}

/// Apply an edge operation to the projection
///
/// This is the contract-specified entry point for applying edge operations.
/// It dispatches to the appropriate handler based on the operation type.
///
/// # Errors
/// - Returns `EdgeOpsError::EdgeNotFound` if the edge does not exist for disconnect operations
/// - Returns `EdgeOpsError::DuplicateEdge` if the edge already exists for connect operations
/// - Returns `EdgeOpsError::PolicyViolation` if the operation violates policy constraints
/// - Returns `EdgeOpsError::InvalidEvent` if the operation is not an edge operation
pub fn apply_edge_op(
    state: DiagramProjection,
    op: &DomainOp,
) -> Result<DiagramProjection, EdgeOpsError> {
    match op {
        DomainOp::EdgeConnect { id, source, target } => {
            apply_edge_connect_checked(state, id, source, target)
        }
        DomainOp::EdgeDisconnect { id } => apply_edge_disconnect_checked(state, id),
        _ => Err(EdgeOpsError::InvalidEvent(format!(
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
) -> Result<DiagramProjection, EdgeOpsError> {
    let edge_id = EdgeId::new(id.to_string());
    let source_id = NodeId::new(source.to_string());
    let target_id = NodeId::new(target.to_string());

    // Check for duplicate edge ID
    if state.has_edge(&edge_id) {
        return Err(EdgeOpsError::DuplicateEdge(id.to_string()));
    }

    // Validate source node exists
    if !state.has_node(&source_id) {
        return Err(EdgeOpsError::PolicyViolation(format!(
            "source node not found: {source}"
        )));
    }

    // Validate target node exists
    if !state.has_node(&target_id) {
        return Err(EdgeOpsError::PolicyViolation(format!(
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
    })
}

/// Apply `EdgeDisconnect` operation with contract-specified error types
fn apply_edge_disconnect_checked(
    state: DiagramProjection,
    id: &str,
) -> Result<DiagramProjection, EdgeOpsError> {
    let edge_id = EdgeId::new(id.to_string());

    // Check edge exists
    if !state.has_edge(&edge_id) {
        return Err(EdgeOpsError::EdgeNotFound(id.to_string()));
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

/// Verify edge tolerance constraints in the projection
///
/// This function validates that all edges in the projection satisfy
/// the defined tolerance boundaries:
/// - All edges reference existing source and target nodes
/// - No duplicate edge IDs exist
/// - All edges have valid geometry (finite coordinates)
///
/// # Errors
/// - Returns `EdgeOpsError::PolicyViolation` if any edge references a non-existent node
/// - Returns `EdgeOpsError::DuplicateEdge` if duplicate edge IDs are detected
/// - Returns `EdgeOpsError::InvariantViolation` if edge geometry is invalid
pub fn verify_edge_tolerance(state: &DiagramProjection) -> Result<(), EdgeOpsError> {
    // Track seen edge IDs to detect duplicates
    let mut seen_ids = std::collections::HashSet::new();

    for (edge_id, edge) in state.edges.iter() {
        // Check for duplicate IDs (should not happen with HashMap, but verify)
        let id_str = edge_id.to_string();
        if !seen_ids.insert(id_str.clone()) {
            return Err(EdgeOpsError::DuplicateEdge(id_str));
        }

        // Verify source node exists
        if !state.has_node(&edge.source) {
            return Err(EdgeOpsError::PolicyViolation(format!(
                "edge {} references non-existent source node: {}",
                edge_id, edge.source
            )));
        }

        // Verify target node exists
        if !state.has_node(&edge.target) {
            return Err(EdgeOpsError::PolicyViolation(format!(
                "edge {} references non-existent target node: {}",
                edge_id, edge.target
            )));
        }

        // Verify edge geometry is valid
        if !edge.label_offset_t.0.is_finite() {
            return Err(EdgeOpsError::InvariantViolation(format!(
                "edge {edge_id} has invalid label_offset_t"
            )));
        }
        if !edge.thickness.0.is_finite() {
            return Err(EdgeOpsError::InvariantViolation(format!(
                "edge {edge_id} has invalid thickness"
            )));
        }
    }

    Ok(())
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

    fn make_node(id: &str) -> Node {
        Node {
            kind: crate::models::document::NodeKind::Node,
            icon: String::new(),
            label: id.to_string(),
            x: OrderedFloat(0.0),
            y: OrderedFloat(0.0),
            width: OrderedFloat(80.0),
            height: OrderedFloat(40.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        }
    }

    fn make_projection_with_nodes_and_edge() -> DiagramProjection {
        let mut projection = DiagramProjection::empty();
        let _ = projection
            .nodes
            .insert(NodeId::new("node-a".to_string()), make_node("node-a"));
        let _ = projection
            .nodes
            .insert(NodeId::new("node-b".to_string()), make_node("node-b"));
        projection
    }

    // Happy Path Tests

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_apply_edge_connect_creates_edge_successfully() {
        let state = make_projection_with_nodes_and_edge();
        let result = apply_edge_connect(state, "edge-1", "node-a", "node-b");

        assert!(result.is_ok(), "Error: {:?}", result.err());
        let new_state = result.unwrap();
        assert!(new_state.has_edge(&EdgeId::new("edge-1".to_string())));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_apply_edge_disconnect_removes_edge_successfully() {
        let mut state = make_projection_with_nodes_and_edge();
        let edge = Edge {
            source: NodeId::new("node-a".to_string()),
            target: NodeId::new("node-b".to_string()),
            label: String::new(),
            style: crate::models::document::EdgeStyle::Solid,
            arrow_type: crate::models::document::ArrowType::Default,
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
        };
        let _ = state.edges.insert(EdgeId::new("edge-1".to_string()), edge);

        let result = apply_edge_disconnect(state, "edge-1");

        assert!(result.is_ok(), "Error: {:?}", result.err());
        let new_state = result.unwrap();
        assert!(!new_state.has_edge(&EdgeId::new("edge-1".to_string())));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_verify_edge_tolerance_returns_ok_for_valid_edges() {
        let state = make_projection_with_nodes_and_edge();
        let result = verify_edge_tolerance(&state);

        assert!(result.is_ok(), "Error: {:?}", result.err());
    }

    // Error Path Tests

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_apply_edge_connect_returns_error_for_duplicate_edge_id() {
        let mut state = make_projection_with_nodes_and_edge();
        let edge = Edge {
            source: NodeId::new("node-a".to_string()),
            target: NodeId::new("node-b".to_string()),
            label: String::new(),
            style: crate::models::document::EdgeStyle::Solid,
            arrow_type: crate::models::document::ArrowType::Default,
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
        };
        let _ = state.edges.insert(EdgeId::new("edge-1".to_string()), edge);

        let result = apply_edge_connect(state, "edge-1", "node-a", "node-b");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, EdgeOpsError::InvariantViolation(_)));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_apply_edge_connect_returns_error_for_nonexistent_source() {
        let state = make_projection_with_nodes_and_edge();
        let result = apply_edge_connect(state, "edge-new", "nonexistent", "node-b");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, EdgeOpsError::InvariantViolation(_)));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_apply_edge_connect_returns_error_for_nonexistent_target() {
        let state = make_projection_with_nodes_and_edge();
        let result = apply_edge_connect(state, "edge-new", "node-a", "nonexistent");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, EdgeOpsError::InvariantViolation(_)));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_apply_edge_disconnect_returns_error_for_nonexistent_edge() {
        let state = DiagramProjection::empty();
        let result = apply_edge_disconnect(state, "nonexistent-edge");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, EdgeOpsError::InvariantViolation(_)));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_apply_edge_op_returns_error_for_non_edge_operation() {
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
        let err = result.unwrap_err();
        assert!(matches!(err, EdgeOpsError::InvalidEvent(_)));
    }

    // Edge Case Tests

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_apply_edge_connect_with_self_loop() {
        let mut projection = DiagramProjection::empty();
        let _ = projection
            .nodes
            .insert(NodeId::new("node-a".to_string()), make_node("node-a"));

        let result = apply_edge_connect(projection, "edge-1", "node-a", "node-a");

        // Self-loops are allowed by default
        assert!(result.is_ok(), "Error: {:?}", result.err());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_apply_edge_disconnect_on_empty_projection() {
        let state = DiagramProjection::empty();
        let result = apply_edge_disconnect(state, "any-edge");

        assert!(result.is_err());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_verify_edge_tolerance_on_empty_projection() {
        let state = DiagramProjection::empty();
        let result = verify_edge_tolerance(&state);

        assert!(result.is_ok());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_apply_edge_connect_preserves_existing_edges() {
        let mut state = make_projection_with_nodes_and_edge();

        // Add first edge
        let edge1 = Edge {
            source: NodeId::new("node-a".to_string()),
            target: NodeId::new("node-b".to_string()),
            label: String::new(),
            style: crate::models::document::EdgeStyle::Solid,
            arrow_type: crate::models::document::ArrowType::Default,
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
        };
        let _ = state.edges.insert(EdgeId::new("edge-1".to_string()), edge1);

        // Add second edge to different nodes
        let _ = state
            .nodes
            .insert(NodeId::new("node-c".to_string()), make_node("node-c"));

        let result = apply_edge_connect(state, "edge-2", "node-a", "node-c");
        assert!(result.is_ok());

        let new_state = result.unwrap();
        assert!(new_state.has_edge(&EdgeId::new("edge-1".to_string())));
        assert!(new_state.has_edge(&EdgeId::new("edge-2".to_string())));
    }

    // Edge Style Tests

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn edg_026_test_default_edge_style_is_solid() {
        let state = make_projection_with_nodes_and_edge();
        let result = apply_edge_connect(state, "edge-1", "node-a", "node-b");
        assert!(result.is_ok());
        let new_state = result.unwrap();
        let edge = new_state
            .get_edge(&EdgeId::new("edge-1".to_string()))
            .unwrap();
        assert_eq!(edge.style, crate::models::document::EdgeStyle::Solid);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn edg_027_test_set_edge_style_to_dashed_updates_successfully() {
        let state = make_projection_with_nodes_and_edge();
        let state = apply_edge_connect(state, "edge-1", "node-a", "node-b").unwrap();

        let result = apply_edge_style(state, "edge-1", crate::models::document::EdgeStyle::Dashed);
        assert!(result.is_ok());
        let new_state = result.unwrap();
        let edge = new_state
            .get_edge(&EdgeId::new("edge-1".to_string()))
            .unwrap();
        assert_eq!(edge.style, crate::models::document::EdgeStyle::Dashed);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn edg_028_test_set_edge_style_to_dotted_updates_successfully() {
        let state = make_projection_with_nodes_and_edge();
        let state = apply_edge_connect(state, "edge-1", "node-a", "node-b").unwrap();

        let result = apply_edge_style(state, "edge-1", crate::models::document::EdgeStyle::Dotted);
        assert!(result.is_ok());
        let new_state = result.unwrap();
        let edge = new_state
            .get_edge(&EdgeId::new("edge-1".to_string()))
            .unwrap();
        assert_eq!(edge.style, crate::models::document::EdgeStyle::Dotted);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn edg_029_test_returns_error_when_setting_style_on_missing_edge() {
        let state = make_projection_with_nodes_and_edge();
        let result = apply_edge_style(
            state,
            "missing-edge",
            crate::models::document::EdgeStyle::Dashed,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            EdgeOpsError::EdgeNotFound(id) => assert_eq!(id, "missing-edge"),
            _ => panic!("Expected EdgeNotFound error"),
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn edg_030_test_set_edge_style_to_same_style_is_idempotent() {
        let state = make_projection_with_nodes_and_edge();
        let state = apply_edge_connect(state, "edge-1", "node-a", "node-b").unwrap();

        let result = apply_edge_style(
            state.clone(),
            "edge-1",
            crate::models::document::EdgeStyle::Solid,
        );
        assert!(result.is_ok());
        let new_state = result.unwrap();

        let edge_before = state.get_edge(&EdgeId::new("edge-1".to_string())).unwrap();
        let edge_after = new_state
            .get_edge(&EdgeId::new("edge-1".to_string()))
            .unwrap();
        assert_eq!(edge_before, edge_after);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_handles_applying_style_to_newly_connected_edge() {
        let state = make_projection_with_nodes_and_edge();
        let state_after_connect =
            apply_edge_connect(state, "edge-new", "node-a", "node-b").unwrap();
        let result = apply_edge_style(
            state_after_connect,
            "edge-new",
            crate::models::document::EdgeStyle::Dashed,
        );

        assert!(result.is_ok());
        let new_state = result.unwrap();
        let edge = new_state
            .get_edge(&EdgeId::new("edge-new".to_string()))
            .unwrap();
        assert_eq!(edge.style, crate::models::document::EdgeStyle::Dashed);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_style_persists_across_multiple_operations() {
        let state = make_projection_with_nodes_and_edge();
        let state = apply_edge_connect(state, "edge-1", "node-a", "node-b").unwrap();
        let state =
            apply_edge_style(state, "edge-1", crate::models::document::EdgeStyle::Dotted).unwrap();

        let mut state = state;
        let _ = state
            .nodes
            .insert(NodeId::new("node-c".to_string()), make_node("node-c"));

        let state = apply_edge_connect(state, "edge-2", "node-a", "node-c").unwrap();

        let edge1 = state.get_edge(&EdgeId::new("edge-1".to_string())).unwrap();
        assert_eq!(edge1.style, crate::models::document::EdgeStyle::Dotted);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_precondition_target_edge_must_exist() {
        let state = DiagramProjection::empty();
        let result = apply_edge_style(state, "edge-1", crate::models::document::EdgeStyle::Solid);
        assert!(matches!(result, Err(EdgeOpsError::EdgeNotFound(_))));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_postcondition_edge_style_is_updated() {
        let state = make_projection_with_nodes_and_edge();
        let state = apply_edge_connect(state, "edge-1", "node-a", "node-b").unwrap();

        let result =
            apply_edge_style(state, "edge-1", crate::models::document::EdgeStyle::Dashed).unwrap();
        let edge = result.get_edge(&EdgeId::new("edge-1".to_string())).unwrap();
        assert_eq!(edge.style, crate::models::document::EdgeStyle::Dashed);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_postcondition_other_properties_unchanged() {
        let state = make_projection_with_nodes_and_edge();
        let state = apply_edge_connect(state, "edge-1", "node-a", "node-b").unwrap();

        let original_edge = state
            .get_edge(&EdgeId::new("edge-1".to_string()))
            .unwrap()
            .clone();

        let result =
            apply_edge_style(state, "edge-1", crate::models::document::EdgeStyle::Dotted).unwrap();
        let updated_edge = result.get_edge(&EdgeId::new("edge-1".to_string())).unwrap();

        assert_eq!(original_edge.source, updated_edge.source);
        assert_eq!(original_edge.target, updated_edge.target);
        assert_eq!(original_edge.label, updated_edge.label);
        assert_eq!(original_edge.arrow_type, updated_edge.arrow_type);
        assert_eq!(original_edge.label_offset_t, updated_edge.label_offset_t);
        assert_eq!(original_edge.thickness, updated_edge.thickness);
        assert_eq!(original_edge.directed, updated_edge.directed);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_postcondition_other_edges_unchanged() {
        let mut state = make_projection_with_nodes_and_edge();
        let _ = state
            .nodes
            .insert(NodeId::new("node-c".to_string()), make_node("node-c"));

        let state = apply_edge_connect(state, "edge-1", "node-a", "node-b").unwrap();
        let state = apply_edge_connect(state, "edge-2", "node-b", "node-c").unwrap();

        let original_edge2 = state
            .get_edge(&EdgeId::new("edge-2".to_string()))
            .unwrap()
            .clone();

        let result =
            apply_edge_style(state, "edge-1", crate::models::document::EdgeStyle::Dashed).unwrap();
        let updated_edge2 = result.get_edge(&EdgeId::new("edge-2".to_string())).unwrap();

        assert_eq!(&original_edge2, updated_edge2);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_invariant_projection_structural_integrity_preserved() {
        let state = make_projection_with_nodes_and_edge();
        let state = apply_edge_connect(state, "edge-1", "node-a", "node-b").unwrap();

        let result =
            apply_edge_style(state, "edge-1", crate::models::document::EdgeStyle::Dashed).unwrap();

        // Nodes should be untouched
        assert!(result.has_node(&NodeId::new("node-a".to_string())));
        assert!(result.has_node(&NodeId::new("node-b".to_string())));

        // Tolerance check passes
        assert!(verify_edge_tolerance(&result).is_ok());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_edge_exists_violation_returns_not_found_error() {
        let state = make_projection_with_nodes_and_edge();
        let result = apply_edge_style(
            state,
            "missing-edge",
            crate::models::document::EdgeStyle::Dashed,
        );

        assert!(matches!(result, Err(EdgeOpsError::EdgeNotFound(id)) if id == "missing-edge"));
    }
}
