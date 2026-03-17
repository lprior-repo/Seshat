//! Domain types for event envelope
//!
//! This module provides the core domain types used in event envelopes.

#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::document::{EdgeId, NodeId};

/// Error types for domain `op_types`
#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContractError {
    #[error("invalid JSON: {0}")]
    InvalidJson(String),
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("invalid author: {0}")]
    InvalidAuthor(String),
    #[error("unknown op_type type: {0}")]
    UnknownOpType(String),
    #[error("invalid node id: {0}")]
    InvalidNodeId(String),
    #[error("invalid edge id: {0}")]
    InvalidEdgeId(String),
    #[error("invalid payload: {0}")]
    InvalidPayload(String),
}

/// Author metadata for events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Author {
    /// Unique identifier for the author
    pub id: String,
    /// Display name of the author
    pub name: String,
    /// Optional email for the author
    pub email: Option<String>,
}

/// Operation types for events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OpType {
    Create,
    Update,
    Delete,
    Migrate,
}

impl OpType {
    /// Parse `OpType` from string, returning `UnknownOpType` error for invalid values
    fn from_str(s: &str) -> Result<Self, ContractError> {
        match s {
            "create" => Ok(Self::Create),
            "update" => Ok(Self::Update),
            "delete" => Ok(Self::Delete),
            "migrate" => Ok(Self::Migrate),
            _ => Err(ContractError::UnknownOpType(s.to_string())),
        }
    }
}

/// Kind of domain `op_type`
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OpKind {
    /// Operation on a node
    Node,
    /// Operation on an edge
    Edge,
    /// Composite `op_type` involving multiple entities
    Composite,
    /// Z-order `op_type` for layering
    ZOrder,
}

impl OpKind {
    /// Returns the name of this `op_type` kind as a string
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Node => "node",
            Self::Edge => "edge",
            Self::Composite => "composite",
            Self::ZOrder => "z_order",
        }
    }
}

/// Type of label target (node or edge)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LabelTargetType {
    /// Target is a node
    Node,
    /// Target is an edge
    Edge,
}

/// Target identifier for label operations (can be node or edge)
///
/// This is a domain type that makes illegal states unrepresentable -
/// a label target must be either a NodeId or an EdgeId, not a raw string.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum LabelTargetId {
    /// Target is a node
    Node(NodeId),
    /// Target is an edge
    Edge(EdgeId),
}

impl LabelTargetId {
    /// Create a `LabelTargetId::Node` from a string
    #[must_use]
    pub fn node(id: NodeId) -> Self {
        Self::Node(id)
    }

    /// Create a `LabelTargetId::Edge` from a string
    #[must_use]
    pub fn edge(id: EdgeId) -> Self {
        Self::Edge(id)
    }

    /// Get the target type for this label target
    #[must_use]
    pub const fn target_type(&self) -> LabelTargetType {
        match self {
            Self::Node(_) => LabelTargetType::Node,
            Self::Edge(_) => LabelTargetType::Edge,
        }
    }
}

impl From<NodeId> for LabelTargetId {
    fn from(id: NodeId) -> Self {
        Self::Node(id)
    }
}

impl From<EdgeId> for LabelTargetId {
    fn from(id: EdgeId) -> Self {
        Self::Edge(id)
    }
}

impl std::fmt::Display for LabelTargetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Node(id) => write!(f, "{}", id),
            Self::Edge(id) => write!(f, "{}", id),
        }
    }
}
