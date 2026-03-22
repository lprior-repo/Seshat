//! Command operations module
//!
//! This module re-exports all command operations for easier importing.
//! The commands are organized into submodules by functionality:
//! - `clipboard` - Clipboard operations (copy, paste, duplicate)
//! - `zorder` - Z-order operations (bring forward, send backward, etc.)
//! - `selection` - Selection operations (select all, clear, delete, group, ungroup, nudge)
//! - `alignment` - Alignment operations
//! - `distribution` - Distribution operations
//! - `zoom` - Zoom and undo/redo operations

pub mod alignment;
pub mod clipboard;
pub mod distribution;
pub mod selection;
pub mod zoom;
pub mod zorder;

// Re-export all public types and functions for backwards compatibility
pub use alignment::{apply_align_selection, AlignmentAxis, AlignmentMode};
pub use clipboard::{
    apply_copy_selection, apply_duplicate_selection, apply_paste_selection, clipboard_has_content,
    ClipboardData,
};
pub use distribution::{apply_distribute_selection, DistributionAxis};
pub use selection::{
    apply_clear_selection, apply_delete_selected, apply_group_selection, apply_nudge_selection,
    apply_select_all, apply_toggle_edge_direction, apply_ungroup_selection,
};
pub use zoom::{apply_redo, apply_undo, apply_zoom_in, apply_zoom_out, apply_zoom_reset};
pub use zorder::{
    apply_bring_forward, apply_bring_to_front, apply_send_backward, apply_send_to_back, ZOrderOp,
};

#[cfg(test)]
mod tests_selection;
