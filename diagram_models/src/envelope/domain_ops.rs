//! Domain operations for diagram editor
//!
//! This module defines all diagram operations as domain types.

#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::document::{EdgeId, EdgeStyle, NodeId, NodeStyle};
use crate::envelope::types::{LabelTargetId, LabelTargetType, OpKind};

/// Domain operation representing a diagram editor operation
///
/// Uses `op_type` as the tag to avoid conflicts with `EventRecord.operation` field.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op_type", rename_all = "snake_case")]
pub enum DomainOp {
    // Node operations
    NodeAdd {
        id: NodeId,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        label: String,
    },
    NodeMove {
        id: NodeId,
        x: f64,
        y: f64,
    },
    NodeDelete {
        id: NodeId,
    },
    NodeRestore {
        id: NodeId,
    },
    NodeResize {
        id: NodeId,
        original_x: f64,
        original_y: f64,
        original_width: f64,
        original_height: f64,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
    UpdateLabel {
        target_id: LabelTargetId,
        target_type: LabelTargetType,
        old_label: String,
        new_label: String,
    },
    UpdateNodeStyle {
        id: NodeId,
        style: NodeStyle,
    },
    // Edge operations
    EdgeConnect {
        id: EdgeId,
        source: NodeId,
        target: NodeId,
    },
    EdgeDisconnect {
        id: EdgeId,
    },
    UpdateEdgeStyle {
        id: EdgeId,
        style: EdgeStyle,
    },
    // Z-order op_types
    BringForward {
        ids: Vec<NodeId>,
    },
    SendBackward {
        ids: Vec<NodeId>,
    },
    BringToFront {
        ids: Vec<NodeId>,
    },
    SendToBack {
        ids: Vec<NodeId>,
    },
    // Composite op_types
    Group {
        id: NodeId,
        ids: Vec<NodeId>,
    },
    Ungroup {
        id: NodeId,
    },
}

impl DomainOp {
    /// Get the `op_type` kind for this domain `op_type`
    #[must_use]
    pub const fn kind(&self) -> OpKind {
        match self {
            Self::NodeAdd { .. }
            | Self::NodeMove { .. }
            | Self::NodeDelete { .. }
            | Self::NodeRestore { .. }
            | Self::NodeResize { .. }
            | Self::UpdateLabel { .. }
            | Self::UpdateNodeStyle { .. } => OpKind::Node,
            Self::EdgeConnect { .. }
            | Self::EdgeDisconnect { .. }
            | Self::UpdateEdgeStyle { .. } => OpKind::Edge,
            Self::BringForward { .. }
            | Self::SendBackward { .. }
            | Self::BringToFront { .. }
            | Self::SendToBack { .. } => OpKind::ZOrder,
            Self::Group { .. } | Self::Ungroup { .. } => OpKind::Composite,
        }
    }
}

/// Get the `op_type` kind for a domain `op_type`
///
/// This is a convenience function that delegates to `DomainOp::kind()`
#[must_use]
pub const fn domain_op_kind(op: &DomainOp) -> OpKind {
    op.kind()
}
