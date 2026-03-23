//! Operations module for diagram projection
//!
//! This module re-exports all operation modules for convenient access.

pub mod edge_ops;
pub mod group_ops;
pub mod node_bounds;
pub mod node_ops;
pub mod node_resize;
pub mod z_order;

// Re-export commonly used types and functions
pub use edge_ops::{
    apply_edge_connect, apply_edge_connect_checked, apply_edge_disconnect,
    apply_edge_disconnect_checked, apply_edge_op, apply_update_edge_label, apply_update_edge_style,
    create_default_edge, verify_edge_tolerance,
};

pub use group_ops::{apply_group, apply_group_op, apply_ungroup};

pub use node_ops::{
    apply_node_add, apply_node_delete, apply_node_move, apply_node_op, apply_node_restore,
    apply_update_label, apply_update_node_style, create_default_node,
};

pub use node_resize::apply_node_resize;

pub use z_order::{
    apply_bring_forward, apply_bring_to_front, apply_send_backward, apply_send_to_back,
    apply_z_order_op,
};
