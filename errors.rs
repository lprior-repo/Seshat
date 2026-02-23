#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

use crate::graph::NodeId;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum WorkflowError {
    #[error("Node {0} not found")]
    NodeNotFound(NodeId),

    #[error("Connection would create a cycle")]
    CycleDetected,

    #[error("Connection already exists")]
    DuplicateConnection,

    #[error("Invalid connection: {0}")]
    InvalidConnection(String),

    #[error("Port not found: {0}")]
    PortNotFound(String),

    #[error("Cannot connect node to itself")]
    SelfConnection,
}

pub type WorkflowResult<T> = Result<T, WorkflowError>;
