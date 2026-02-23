#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

use crate::graph::NodeId;
use dioxus::prelude::*;

fn toggle_selection_ids(current: &[NodeId], id: NodeId) -> (Vec<NodeId>, Option<NodeId>) {
    let mut next = current.to_vec();
    if let Some(pos) = next.iter().position(|&item| item == id) {
        next.remove(pos);
    } else {
        next.push(id);
    }

    let selected_id = if next.contains(&id) {
        Some(id)
    } else {
        next.first().copied()
    };

    (next, selected_id)
}

fn add_unique_selection(current: &[NodeId], id: NodeId) -> Vec<NodeId> {
    if current.contains(&id) {
        return current.to_vec();
    }
    let mut next = current.to_vec();
    next.push(id);
    next
}

/// Selection state hook - manages which nodes are selected.
///
/// Follows functional reactive pattern:
/// - State stored in Copy signals
/// - Methods for mutations
/// - `ReadSignal` accessors for derived views
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SelectionState {
    selected_id: Signal<Option<NodeId>>,
    selected_ids: Signal<Vec<NodeId>>,
    pending_drag: Signal<Option<Vec<NodeId>>>,
}

#[allow(dead_code)]
impl SelectionState {
    /// Access to the primary selected node ID signal
    #[must_use]
    pub const fn selected_id(&self) -> Signal<Option<NodeId>> {
        self.selected_id
    }

    /// Access to all selected node IDs signal
    #[must_use]
    pub const fn selected_ids(&self) -> Signal<Vec<NodeId>> {
        self.selected_ids
    }

    // === Mutation methods (take `self` since Copy) ===

    /// Select a single node (clears multi-selection)
    pub fn select_single(mut self, id: NodeId) {
        self.selected_id.set(Some(id));
        self.selected_ids.set(vec![id]);
    }

    /// Toggle a node in the selection
    pub fn toggle(mut self, id: NodeId) {
        let ids = self.selected_ids.read().clone();
        let (new_ids, selected_id) = toggle_selection_ids(&ids, id);
        self.selected_id.set(selected_id);
        self.selected_ids.set(new_ids);
    }

    /// Add a node to selection without clearing
    pub fn add_to_selection(mut self, id: NodeId) {
        let ids = self.selected_ids.read().clone();
        self.selected_ids.set(add_unique_selection(&ids, id));
    }

    /// Set multiple selected nodes at once
    pub fn set_multiple(mut self, ids: Vec<NodeId>) {
        self.selected_id.set(ids.first().copied());
        self.selected_ids.set(ids);
    }

    /// Clear all selection
    pub fn clear(mut self) {
        self.selected_id.set(None);
        self.selected_ids.set(Vec::new());
    }

    /// Set pending drag targets
    pub fn set_pending_drag(mut self, ids: Vec<NodeId>) {
        if let Ok(mut pending_drag) = self.pending_drag.try_write() {
            *pending_drag = Some(ids);
        }
    }

    /// Clear pending drag targets
    pub fn clear_pending_drag(mut self) {
        if let Ok(mut pending_drag) = self.pending_drag.try_write() {
            *pending_drag = None;
        }
    }

    /// Read and clear pending drag targets atomically if present.
    #[must_use]
    pub fn take_pending_drag(mut self) -> Option<Vec<NodeId>> {
        self.pending_drag
            .try_write()
            .ok()
            .and_then(|mut pending_drag| pending_drag.take())
    }

    /// Access pending drag targets
    #[must_use]
    pub fn pending_drag(&self) -> ReadSignal<Option<Vec<NodeId>>> {
        self.pending_drag.into()
    }

    /// Check if a node is selected
    #[must_use]
    pub fn is_selected(&self, id: NodeId) -> bool {
        self.selected_ids.read().contains(&id)
    }

    /// Get the number of selected nodes
    #[must_use]
    pub fn count(&self) -> usize {
        self.selected_ids.read().len()
    }

    /// Check if anything is selected
    #[must_use]
    pub fn has_selection(&self) -> bool {
        !self.selected_ids.read().is_empty()
    }
}

pub fn use_selection() -> SelectionState {
    let selected_id = use_signal(|| None::<NodeId>);
    let selected_ids = use_signal(Vec::<NodeId>::new);
    let pending_drag = use_signal(|| None::<Vec<NodeId>>);

    SelectionState {
        selected_id,
        selected_ids,
        pending_drag,
    }
}

#[cfg(test)]
mod tests {
    use super::{add_unique_selection, toggle_selection_ids};
    use crate::graph::NodeId;

    #[test]
    fn given_selected_node_when_toggling_existing_then_node_is_removed_and_primary_updates() {
        let a = NodeId::new();
        let b = NodeId::new();
        let current = vec![a, b];

        let (next, selected) = toggle_selection_ids(&current, a);

        assert_eq!(next, vec![b]);
        assert_eq!(selected, Some(b));
    }

    #[test]
    fn given_duplicate_add_when_adding_to_selection_then_selection_remains_unique() {
        let a = NodeId::new();
        let current = vec![a];

        let next = add_unique_selection(&current, a);

        assert_eq!(next, vec![a]);
    }

    #[test]
    fn given_missing_node_when_toggling_then_node_is_added_and_selected() {
        let a = NodeId::new();
        let b = NodeId::new();
        let current = vec![a];

        let (next, selected) = toggle_selection_ids(&current, b);

        assert_eq!(next, vec![a, b]);
        assert_eq!(selected, Some(b));
    }
}
