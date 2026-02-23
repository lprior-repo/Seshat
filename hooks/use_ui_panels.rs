#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

use crate::graph::NodeId;
use dioxus::prelude::*;

const fn toggle_palette_state(current_open: bool) -> (bool, bool) {
    let next_open = !current_open;
    let should_clear_query = next_open;
    (next_open, should_clear_query)
}

fn toggle_inline_target(current: Option<NodeId>, node_id: NodeId) -> Option<NodeId> {
    if current == Some(node_id) {
        None
    } else {
        Some(node_id)
    }
}

/// Context menu state
#[derive(Clone, Debug, PartialEq, Store)]
pub struct ContextMenuState {
    pub open: bool,
    pub x: f32,
    pub y: f32,
}

impl Default for ContextMenuState {
    fn default() -> Self {
        Self {
            open: false,
            x: 0.0,
            y: 0.0,
        }
    }
}

/// UI panels hook - manages modal and overlay state.
///
/// Follows functional reactive pattern with methods for panel control.
#[derive(Clone, Copy, PartialEq)]
pub struct UiPanels {
    settings_open: Signal<bool>,
    palette_open: Signal<bool>,
    palette_query: Signal<String>,
    context_menu: Store<ContextMenuState>,
    context_menu_state: Memo<ContextMenuState>,
    inline_panel_node_id: Signal<Option<NodeId>>,
}

#[allow(dead_code)]
impl UiPanels {
    /// Read-only access to settings panel state
    #[must_use]
    pub fn settings_open(&self) -> ReadSignal<bool> {
        self.settings_open.into()
    }

    /// Read-only access to command palette state
    #[must_use]
    pub fn palette_open(&self) -> ReadSignal<bool> {
        self.palette_open.into()
    }

    /// Read-only access to palette query string
    #[must_use]
    pub fn palette_query(&self) -> ReadSignal<String> {
        self.palette_query.into()
    }

    /// Read-only access to context menu state
    #[must_use]
    pub fn context_menu(&self) -> ReadSignal<ContextMenuState> {
        self.context_menu_state.into()
    }

    // === Settings panel ===

    /// Toggle settings panel
    pub fn toggle_settings(mut self) {
        let current = *self.settings_open.read();
        self.settings_open.set(!current);
    }

    /// Open settings panel
    pub fn open_settings(mut self) {
        self.settings_open.set(true);
    }

    /// Close settings panel
    pub fn close_settings(mut self) {
        self.settings_open.set(false);
    }

    // === Command palette ===

    /// Toggle command palette
    pub fn toggle_palette(mut self) {
        let current = *self.palette_open.read();
        let (next_open, should_clear_query) = toggle_palette_state(current);
        self.palette_open.set(next_open);
        if should_clear_query {
            self.palette_query.set(String::new());
        }
    }

    /// Open command palette
    pub fn open_palette(mut self) {
        self.palette_open.set(true);
        self.palette_query.set(String::new());
    }

    /// Close command palette
    pub fn close_palette(mut self) {
        self.palette_open.set(false);
    }

    /// Set palette query string
    pub fn set_palette_query(mut self, query: String) {
        self.palette_query.set(query);
    }

    /// Clear palette query
    pub fn clear_palette_query(mut self) {
        self.palette_query.set(String::new());
    }

    // === Context menu ===

    /// Show context menu at position
    pub fn show_context_menu(mut self, x: f32, y: f32) {
        self.context_menu.set(ContextMenuState { open: true, x, y });
    }

    /// Close context menu
    pub fn close_context_menu(mut self) {
        self.context_menu.set(ContextMenuState::default());
    }

    /// Check if context menu is open
    #[must_use]
    pub fn is_context_menu_open(&self) -> bool {
        self.context_menu_state.read().open
    }

    // === Bulk operations ===

    /// Close all panels
    pub fn close_all(mut self) {
        self.palette_open.set(false);
        self.settings_open.set(false);
        self.context_menu.set(ContextMenuState::default());
        self.inline_panel_node_id.set(None);
    }

    /// Check if any panel is open
    #[must_use]
    pub fn any_open(&self) -> bool {
        *self.settings_open.read()
            || *self.palette_open.read()
            || self.context_menu_state.read().open
            || self.inline_panel_node_id.read().is_some()
    }

    // === Inline panel ===

    #[must_use]
    pub fn inline_panel_node_id(&self) -> ReadSignal<Option<NodeId>> {
        self.inline_panel_node_id.into()
    }

    pub fn open_inline_panel(mut self, node_id: NodeId) {
        self.inline_panel_node_id.set(Some(node_id));
    }

    pub fn close_inline_panel(mut self) {
        self.inline_panel_node_id.set(None);
    }

    pub fn toggle_inline_panel(mut self, node_id: NodeId) {
        let current = *self.inline_panel_node_id.read();
        self.inline_panel_node_id
            .set(toggle_inline_target(current, node_id));
    }

    #[must_use]
    pub fn is_inline_panel_open(&self, node_id: NodeId) -> bool {
        *self.inline_panel_node_id.read() == Some(node_id)
    }
}

pub fn use_ui_panels() -> UiPanels {
    let settings_open = use_signal(|| false);
    let palette_open = use_signal(|| false);
    let palette_query = use_signal(String::new);
    let context_menu = use_store(ContextMenuState::default);
    let context_menu_state = use_memo(move || context_menu.cloned());
    let inline_panel_node_id = use_signal(|| None);

    UiPanels {
        settings_open,
        palette_open,
        palette_query,
        context_menu,
        context_menu_state,
        inline_panel_node_id,
    }
}

#[cfg(test)]
mod tests {
    use super::{toggle_inline_target, toggle_palette_state};
    use crate::graph::NodeId;

    #[test]
    fn given_palette_closed_when_toggling_then_it_opens_and_query_should_clear() {
        let (open, clear_query) = toggle_palette_state(false);
        assert!(open);
        assert!(clear_query);
    }

    #[test]
    fn given_palette_open_when_toggling_then_it_closes_and_query_not_forced_clear() {
        let (open, clear_query) = toggle_palette_state(true);
        assert!(!open);
        assert!(!clear_query);
    }

    #[test]
    fn given_same_node_when_toggling_inline_panel_then_panel_closes() {
        let id = NodeId::new();
        let next = toggle_inline_target(Some(id), id);
        assert_eq!(next, None);
    }
}
