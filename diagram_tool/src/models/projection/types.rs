//! Core types for diagram projection
//!
//! This module defines the fundamental types used in diagram projection:
//! - `DiagramProjection`: The main state container
//! - `CyclePolicy`: Policy for cycle handling
//! - `EventRecord`: Events for replay
//! - `ReplayError`: Error types

#![allow(dead_code)]
#![allow(unused_imports)]

use im::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::models::document::{DiagramDocument, DocumentData, Edge, EdgeId, Node, NodeId};
use crate::models::envelope::{Author, DomainOp};

/// Current supported schema version
pub const SUPPORTED_VERSION: u32 = 2;

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
    #[error("no nodes specified for z-order operation")]
    NoNodesSpecified,
    #[error("all nodes invalid or not found: {0}")]
    AllNodesInvalid(String),
    #[error("z-index overflow")]
    ZIndexOverflow,
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
pub fn is_human_author(author: &Author) -> bool {
    // Author IDs starting with "human-" are considered human-authored
    // All others are assumed to be AI-authored
    author.id.starts_with("human-") || author.name.to_lowercase().contains("human")
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
