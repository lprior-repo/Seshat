//! Diagram projection module - deterministic document projection replayer
//!
//! This module provides deterministic replay of events to produce a consistent `DiagramProjection`.
//! The replay is deterministic: given the same input events, it always produces the same output.
//!
//! ## Module Structure
//!
//! - [`types`] - Core types (DiagramProjection, NodeId, EdgeId, etc.)
//! - [`replay`] - Event replay functions
//! - [`policy`] - Cycle policy enforcement
//! - [`ops`] - Operation modules (node, edge, z-order, group operations)

#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]
#![allow(dead_code)]
#![allow(unused_imports)]

pub mod ops;
pub mod policy;
pub mod replay;
pub mod tests;
pub mod types;

// Re-export main types for convenient access
pub use types::{
    document_to_projection, is_human_author, projection_to_document, CyclePolicy,
    DiagramProjection, EventRecord, ReplayError, SUPPORTED_VERSION,
};

// Re-export replay functions
pub use replay::{apply_event, apply_operation, replay_events, replay_events_from, replay_stream};

// Re-export policy functions
pub use policy::{apply_policy_op, enforce_cycle_policy, projection_hash};

// Re-export operation functions via ops module
pub use ops::{
    apply_bring_forward, apply_bring_to_front, apply_edge_connect, apply_edge_connect_checked,
    apply_edge_disconnect, apply_edge_disconnect_checked, apply_edge_op, apply_group,
    apply_group_op, apply_node_add, apply_node_delete, apply_node_move, apply_node_op,
    apply_node_restore, apply_send_backward, apply_send_to_back, apply_ungroup, apply_z_order_op,
    create_default_edge, create_default_node, verify_edge_tolerance,
};
