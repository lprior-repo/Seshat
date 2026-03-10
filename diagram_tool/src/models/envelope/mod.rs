//! Event envelope types
#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContractError {
    #[error("invalid JSON: {0}")] InvalidJson(String),
    #[error("missing required field: {0}")] MissingField(&'static str),
    #[error("invalid author: {0}")] InvalidAuthor(String),
    #[error("unknown op_type type: {0}")] UnknownOpType(String),
    #[error("invalid payload: {0}")] InvalidPayload(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Author { pub id: String, pub name: String, pub email: Option<String> }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OpKind { Node, Edge, Composite, ZOrder }

impl OpKind { #[must_use] pub const fn as_str(&self) -> &'static str { match self { Self::Node => "node", Self::Edge => "edge", Self::Composite => "composite", Self::ZOrder => "z_order" } } }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op_type", rename_all = "snake_case")]
pub enum DomainOp {
    NodeAdd { id: String, x: f64, y: f64, width: f64, height: f64, label: String },
    NodeMove { id: String, x: f64, y: f64 },
    NodeDelete { id: String },
    NodeRestore { id: String },
    EdgeConnect { id: String, source: String, target: String },
    EdgeDisconnect { id: String },
    BringForward { ids: Vec<String> },
    SendBackward { ids: Vec<String> },
    BringToFront { ids: Vec<String> },
    SendToBack { ids: Vec<String> },
    Group { ids: Vec<String> },
    Ungroup { id: String },
}

impl DomainOp { #[must_use] pub const fn kind(&self) -> OpKind { match self { Self::NodeAdd { .. } | Self::NodeMove { .. } | Self::NodeDelete { .. } | Self::NodeRestore { .. } => OpKind::Node, Self::EdgeConnect { .. } | Self::EdgeDisconnect { .. } => OpKind::Edge, Self::BringForward { .. } | Self::SendBackward { .. } | Self::BringToFront { .. } | Self::SendToBack { .. } => OpKind::ZOrder, Self::Group { .. } | Self::Ungroup { .. } => OpKind::Composite } } }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventEnvelope { #[serde(rename = "op_id")] pub op_id: String, pub operation: DomainOp, pub author: Author, pub timestamp: i64 }

pub mod codec;
pub use codec::*;
