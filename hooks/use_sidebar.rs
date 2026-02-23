#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

use dioxus::prelude::*;

fn pending_drop_after_pickup(node_type: &'static str) -> String {
    node_type.to_string()
}

const fn pending_drop_after_clear() -> Option<String> {
    None
}

/// Sidebar state hook - manages sidebar search and drag-drop state.
///
/// Follows functional reactive pattern with methods for state mutations.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SidebarState {
    search: Signal<String>,
    pending_drop: Signal<Option<String>>,
}

#[allow(dead_code)]
impl SidebarState {
    /// Read-only access to search query
    #[must_use]
    pub fn search(&self) -> ReadSignal<String> {
        self.search.into()
    }

    /// Read-only access to pending drop node type
    #[must_use]
    pub fn pending_drop(&self) -> ReadSignal<Option<String>> {
        self.pending_drop.into()
    }

    /// Set search query
    pub fn set_search(mut self, query: String) {
        self.search.set(query);
    }

    /// Set pending drop node type
    pub fn set_pending_drop(mut self, node_type: Option<String>) {
        self.pending_drop.set(node_type);
    }

    /// Start dragging a node type from sidebar
    pub fn pickup_node(mut self, node_type: &'static str) {
        self.pending_drop
            .set(Some(pending_drop_after_pickup(node_type)));
    }

    /// Clear pending drop (after drop or cancel)
    pub fn clear_pending_drop(mut self) {
        self.pending_drop.set(pending_drop_after_clear());
    }

    /// Check if there's a pending drop
    #[must_use]
    pub fn has_pending_drop(&self) -> bool {
        self.pending_drop.read().is_some()
    }
}

pub fn use_sidebar() -> SidebarState {
    let search = use_signal(String::new);
    let pending_drop = use_signal(|| None::<String>);

    SidebarState {
        search,
        pending_drop,
    }
}

#[cfg(test)]
mod tests {
    use super::{pending_drop_after_clear, pending_drop_after_pickup};

    #[test]
    fn given_node_type_when_picking_up_then_pending_drop_is_set() {
        assert_eq!(pending_drop_after_pickup("run"), "run".to_string());
    }

    #[test]
    fn given_pending_drop_when_clearing_then_pending_drop_is_none() {
        assert_eq!(pending_drop_after_clear(), None);
    }
}
