//! Canonical clipboard data type for diagram operations
//!
//! This module provides the single source of truth for clipboard data structures
//! used across the diagram_tool application.

use crate::document::{Edge, Node, NodeId};

/// Pure clipboard data type - immutable state for clipboard operations.
///
/// This replaces the mutable `thread_local` RefCell-based clipboard with
/// a pure functional approach where clipboard state is passed explicitly.
///
/// # Design Rationale
///
/// - `nodes` preserves `NodeId` tuples to maintain identity during copy/paste
/// - `edges` contains only `Edge` values (no IDs) since pasting generates new edge IDs
/// - `paste_serial` tracks paste operations for spatial offset calculation
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClipboardData {
    /// The nodes that were copied to the clipboard, preserving their original IDs
    pub nodes: Vec<(NodeId, Node)>,
    /// The edges that were copied to the clipboard
    /// Note: Edge IDs are NOT preserved since new IDs are generated on paste
    pub edges: Vec<Edge>,
    /// Serial number for tracking paste operations (for offset calculation)
    pub paste_serial: u32,
}

impl ClipboardData {
    /// Creates a new empty clipboard
    #[must_use]
    pub const fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            paste_serial: 0,
        }
    }

    /// Returns true if the clipboard has content that can be pasted
    #[must_use]
    pub const fn has_content(&self) -> bool {
        !self.nodes.is_empty()
    }

    /// Prepares the clipboard for a paste operation by incrementing the serial
    #[must_use]
    pub const fn prepare_paste(mut self) -> Self {
        self.paste_serial = self.paste_serial.saturating_add(1);
        self
    }
}

impl Default for ClipboardData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_clipboard_is_empty() {
        let clipboard = ClipboardData::new();
        assert!(!clipboard.has_content());
        assert_eq!(clipboard.paste_serial, 0);
    }

    #[test]
    fn test_default_is_empty() {
        let clipboard = ClipboardData::default();
        assert!(!clipboard.has_content());
    }

    #[test]
    fn test_prepare_paste_increments_serial() {
        let clipboard = ClipboardData::new();
        assert_eq!(clipboard.paste_serial, 0);

        let prepared = clipboard.prepare_paste();
        assert_eq!(prepared.paste_serial, 1);

        let prepared_again = prepared.prepare_paste();
        assert_eq!(prepared_again.paste_serial, 2);
    }

    #[test]
    fn test_has_content_returns_true_when_nodes_present() {
        let mut clipboard = ClipboardData::new();
        assert!(!clipboard.has_content());

        clipboard.nodes = vec![(NodeId::new("test".to_string()), Node::default())];
        assert!(clipboard.has_content());
    }
}
