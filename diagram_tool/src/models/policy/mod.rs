//! Policy types
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

#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PolicyError {
    #[error("cycle violation: {0}")] CycleViolation(String),
    #[error("policy missing: {0}")] PolicyMissing(String),
    #[error("policy violation: {0}")] PolicyViolation(String),
    #[error("invalid event: {0}")] InvalidEvent(String),
    #[error("invariant violation: {0}")] InvariantViolation(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CyclePolicy { #[default] Allow, Deny }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiagramProjection {
    pub version: u32,
    pub revision: u64,
    pub nodes: HashMap<NodeId, crate::models::document::Node>,
    pub edges: HashMap<EdgeId, crate::models::document::Edge>,
    #[serde(default)] pub author_priority: HashMap<String, bool>,
    #[serde(default)] pub cycle_policy: CyclePolicy,
}

impl Default for DiagramProjection { fn default() -> Self { Self::empty() } }

impl DiagramProjection {
    #[must_use] pub fn empty() -> Self { Self { version: 2, revision: 0, nodes: HashMap::new(), edges: HashMap::new(), author_priority: HashMap::new(), cycle_policy: CyclePolicy::default() } }
    #[must_use] pub fn with_cycle_policy(cycle_policy: CyclePolicy) -> Self { Self { version: 2, revision: 0, nodes: HashMap::new(), edges: HashMap::new(), author_priority: HashMap::new(), cycle_policy } }
    #[must_use] pub fn has_node(&self, id: &NodeId) -> bool { self.nodes.contains_key(id) }
    #[must_use] pub fn has_edge(&self, id: &EdgeId) -> bool { self.edges.contains_key(id) }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventRecord { pub op_id: String, pub revision: u64, pub operation: DomainOp, pub author: Author, pub timestamp: i64 }

pub mod validation;
pub use validation::*;
