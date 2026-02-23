#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

pub mod calc;
mod connectivity;
mod core;
mod execution;
pub mod execution_record;
pub mod execution_state;
mod metadata;
mod view;

pub mod expressions;
pub mod layout;
pub mod restate_types;
pub mod validation;
pub mod workflow_node;

pub use connectivity::{ConnectionError, ConnectionResult};
pub use execution_record::{ExecutionOverallStatus, ExecutionRecord, StepOutput, StepRecord};
pub use execution_state::ExecutionState;
pub use validation::{validate_workflow, ValidationIssue, ValidationResult, ValidationSeverity};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NodeId(pub Uuid);

impl NodeId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PortName(pub String);

impl<S: Into<String>> From<S> for PortName {
    fn from(s: S) -> Self {
        Self(s.into())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NodeCategory {
    Entry,
    Durable,
    State,
    Flow,
    Timing,
    Signal,
}

impl fmt::Display for NodeCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Entry => "entry",
            Self::Durable => "durable",
            Self::State => "state",
            Self::Flow => "flow",
            Self::Timing => "timing",
            Self::Signal => "signal",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Node {
    pub id: NodeId,
    pub name: String,
    pub description: String,
    pub node_type: String,
    pub category: NodeCategory,
    pub icon: String,
    pub x: f32,
    pub y: f32,
    pub config: serde_json::Value,
    pub last_output: Option<serde_json::Value>,
    pub selected: bool,
    pub executing: bool,
    pub skipped: bool,
    pub error: Option<String>,
    #[serde(default)]
    pub execution_state: ExecutionState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Connection {
    pub id: Uuid,
    pub source: NodeId,
    pub target: NodeId,
    pub source_port: PortName,
    pub target_port: PortName,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunRecord {
    pub id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub results: std::collections::HashMap<NodeId, serde_json::Value>,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Workflow {
    pub nodes: Vec<Node>,
    pub connections: Vec<Connection>,
    pub viewport: Viewport,
    pub execution_queue: Vec<NodeId>,
    pub current_step: usize,
    pub history: Vec<RunRecord>,
    #[serde(default)]
    pub execution_records: Vec<ExecutionRecord>,
}

impl Default for Workflow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{NodeCategory, NodeId, PortName};

    #[test]
    fn given_node_id_when_displayed_then_it_matches_inner_uuid() {
        let id = NodeId::new();
        assert_eq!(id.to_string(), id.0.to_string());
    }

    #[test]
    fn given_default_node_id_when_created_then_it_is_not_nil() {
        let id = NodeId::default();
        assert_ne!(id.0, uuid::Uuid::nil());
    }

    #[test]
    fn given_string_when_converted_to_port_name_then_value_is_preserved() {
        let port = PortName::from("main");
        assert_eq!(port.0, "main");
    }

    #[test]
    fn given_node_categories_when_displayed_then_lowercase_labels_are_returned() {
        assert_eq!(NodeCategory::Entry.to_string(), "entry");
        assert_eq!(NodeCategory::Durable.to_string(), "durable");
        assert_eq!(NodeCategory::State.to_string(), "state");
        assert_eq!(NodeCategory::Flow.to_string(), "flow");
        assert_eq!(NodeCategory::Timing.to_string(), "timing");
        assert_eq!(NodeCategory::Signal.to_string(), "signal");
    }
}
