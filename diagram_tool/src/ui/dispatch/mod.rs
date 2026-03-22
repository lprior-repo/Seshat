//! Module for dispatch submodules

pub mod create;
pub mod errors;
pub mod helpers;
pub mod send;
pub mod validators;

#[cfg(test)]
pub mod create_tests;

pub use errors::{DispatchError, DispatchResult};
pub use validators::{edge_preserves_dag, validate_coordinates, validate_dimensions};

// Re-export create functions
pub use create::{
    create_bring_forward_envelope, create_bring_to_front_envelope, create_edge_connect_envelope,
    create_edge_disconnect_envelope, create_group_envelope, create_node_add_envelope,
    create_node_delete_envelope, create_node_resize_envelope, create_send_backward_envelope,
    create_send_to_back_envelope, create_ungroup_envelope, create_update_edge_style_envelope,
    create_update_label_envelope, create_update_node_style_envelope,
};

// Re-export send functions
pub use send::{
    dispatch_bring_forward, dispatch_bring_to_front, dispatch_edge_connect,
    dispatch_edge_disconnect, dispatch_group, dispatch_node_add, dispatch_node_delete,
    dispatch_node_delete_batch, dispatch_node_resize, dispatch_send_backward,
    dispatch_send_to_back, dispatch_ungroup, dispatch_update_edge_style, dispatch_update_label,
    dispatch_update_node_style, handle_edge_drawing_complete, validate_edge_connect_preconditions,
    ResizeBounds,
};
