#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

use crate::graph::NodeId;
use crate::ui::edges::Position as FlowPosition;
use dioxus::prelude::*;

fn drag_mode_from_selection(selected_ids: Vec<NodeId>) -> Option<InteractionMode> {
    if selected_ids.is_empty() {
        None
    } else {
        Some(InteractionMode::Dragging {
            node_ids: selected_ids,
        })
    }
}

fn update_marquee_mode(mode: &InteractionMode, pos: (f32, f32)) -> InteractionMode {
    match mode {
        InteractionMode::Marquee { start, .. } => InteractionMode::Marquee {
            start: *start,
            current: pos,
        },
        _ => mode.clone(),
    }
}

const fn cursor_class_for(mode: &InteractionMode, is_space_hand: bool) -> &'static str {
    match mode {
        InteractionMode::Panning => "cursor-grabbing",
        InteractionMode::Idle if is_space_hand => "cursor-grab",
        _ => "cursor-default",
    }
}

/// Interaction mode state machine - ensures illegal states are unrepresentable.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum InteractionMode {
    #[default]
    Idle,
    Panning,
    Dragging {
        node_ids: Vec<NodeId>,
    },
    Connecting {
        from: NodeId,
        handle: String,
    },
    Marquee {
        start: (f32, f32),
        current: (f32, f32),
    },
}

/// Canvas interaction hook - manages all canvas interaction state.
///
/// Uses a state machine (`InteractionMode`) to ensure consistent state.
/// Follows functional reactive pattern with methods for state transitions.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CanvasInteraction {
    mode: Signal<InteractionMode>,
    is_space_hand: Signal<bool>,
    mouse_pos: Signal<(f32, f32)>,
    canvas_origin: Signal<(f32, f32)>,
    temp_edge_to: Signal<Option<(FlowPosition, FlowPosition)>>,
    hovered_handle: Signal<Option<(NodeId, String)>>,
    drag_anchor: Signal<Option<(f32, f32)>>,
}

#[allow(dead_code)]
impl CanvasInteraction {
    /// Read-only access to current interaction mode
    #[must_use]
    pub fn mode(&self) -> ReadSignal<InteractionMode> {
        self.mode.into()
    }

    /// Read-only access to space-hand tool state
    #[must_use]
    pub fn is_space_hand(&self) -> ReadSignal<bool> {
        self.is_space_hand.into()
    }

    /// Read-only access to mouse position
    #[must_use]
    pub fn mouse_pos(&self) -> ReadSignal<(f32, f32)> {
        self.mouse_pos.into()
    }

    /// Read-only access to canvas origin (for coordinate transforms)
    #[must_use]
    pub fn canvas_origin(&self) -> ReadSignal<(f32, f32)> {
        self.canvas_origin.into()
    }

    /// Read-only access to temporary edge target position
    #[must_use]
    pub fn temp_edge_to(&self) -> ReadSignal<Option<(FlowPosition, FlowPosition)>> {
        self.temp_edge_to.into()
    }

    /// Read-only access to currently hovered handle
    #[must_use]
    pub fn hovered_handle(&self) -> ReadSignal<Option<(NodeId, String)>> {
        self.hovered_handle.into()
    }

    // === State transitions ===
    // Note: Methods take `self` (not `&self`) because Signal::set requires mutable access.
    // Since CanvasInteraction is Copy, consuming `self` is cheap and allows mutation.

    /// Start panning mode
    pub fn start_pan(mut self) {
        self.mode.set(InteractionMode::Panning);
    }

    /// Start dragging nodes
    pub fn start_drag(mut self, _node_id: NodeId, selected_ids: Vec<NodeId>) {
        if let Some(next_mode) = drag_mode_from_selection(selected_ids) {
            self.mode.set(next_mode);
        }
    }

    /// Start connecting from a handle
    pub fn start_connect(mut self, node_id: NodeId, handle: String) {
        self.hovered_handle.set(Some((node_id, handle.clone())));
        self.mode.set(InteractionMode::Connecting {
            from: node_id,
            handle,
        });
    }

    /// Start marquee selection
    pub fn start_marquee(mut self, pos: (f32, f32)) {
        self.mode.set(InteractionMode::Marquee {
            start: pos,
            current: pos,
        });
    }

    /// Update marquee current position
    pub fn update_marquee(mut self, pos: (f32, f32)) {
        let mode = self.mode.read().clone();
        self.mode.set(update_marquee_mode(&mode, pos));
    }

    /// Update mouse position
    pub fn update_mouse(mut self, pos: (f32, f32)) {
        self.mouse_pos.set(pos);
    }

    /// Set canvas origin for coordinate transforms
    pub fn set_origin(mut self, origin: (f32, f32)) {
        self.canvas_origin.set(origin);
    }

    /// Set temporary edge target position
    pub fn set_temp_edge(mut self, pos: Option<(FlowPosition, FlowPosition)>) {
        self.temp_edge_to.set(pos);
    }

    /// Set hovered handle
    pub fn set_hovered_handle(mut self, handle: Option<(NodeId, String)>) {
        self.hovered_handle.set(handle);
    }

    /// Start drag anchor for thresholding
    pub fn start_drag_anchor(mut self, pos: (f32, f32)) {
        self.drag_anchor.set(Some(pos));
    }

    /// Clear drag anchor
    pub fn clear_drag_anchor(mut self) {
        self.drag_anchor.set(None);
    }

    /// Enable space-hand tool mode
    pub fn enable_space_hand(mut self) {
        self.is_space_hand.set(true);
    }

    /// Disable space-hand tool mode
    pub fn disable_space_hand(mut self) {
        self.is_space_hand.set(false);
    }

    /// End current interaction (return to idle)
    pub fn end_interaction(mut self) {
        self.mode.set(InteractionMode::Idle);
        self.temp_edge_to.set(None);
        self.hovered_handle.set(None);
        self.drag_anchor.set(None);
    }

    /// Cancel current interaction and reset state
    pub fn cancel_interaction(mut self) {
        self.mode.set(InteractionMode::Idle);
        self.temp_edge_to.set(None);
        self.hovered_handle.set(None);
        self.is_space_hand.set(false);
        self.drag_anchor.set(None);
    }

    // === Query methods ===

    /// Check if currently in dragging mode
    #[must_use]
    pub fn is_dragging(&self) -> bool {
        matches!(*self.mode.read(), InteractionMode::Dragging { .. })
    }

    /// Check if currently in connecting mode
    #[must_use]
    pub fn is_connecting(&self) -> bool {
        matches!(*self.mode.read(), InteractionMode::Connecting { .. })
    }

    /// Check if currently in marquee selection mode
    #[must_use]
    pub fn is_marquee(&self) -> bool {
        matches!(*self.mode.read(), InteractionMode::Marquee { .. })
    }

    /// Check if currently in panning mode
    #[must_use]
    pub fn is_panning(&self) -> bool {
        matches!(*self.mode.read(), InteractionMode::Panning)
    }

    /// Check if idle
    #[must_use]
    pub fn is_idle(&self) -> bool {
        matches!(*self.mode.read(), InteractionMode::Idle)
    }

    /// Get cursor CSS class based on current mode
    #[must_use]
    pub fn cursor_class(&self) -> &'static str {
        let mode = self.mode.read().clone();
        cursor_class_for(&mode, *self.is_space_hand.read())
    }

    /// Get dragging node IDs if in dragging mode
    #[must_use]
    pub fn dragging_node_ids(&self) -> Option<Vec<NodeId>> {
        match &*self.mode.read() {
            InteractionMode::Dragging { node_ids } => Some(node_ids.clone()),
            _ => None,
        }
    }

    /// Get drag anchor position if set
    #[must_use]
    pub fn drag_anchor(&self) -> Option<(f32, f32)> {
        *self.drag_anchor.read()
    }

    /// Get connection source if in connecting mode
    #[must_use]
    pub fn connecting_from(&self) -> Option<(NodeId, String)> {
        match &*self.mode.read() {
            InteractionMode::Connecting { from, handle } => Some((*from, handle.clone())),
            _ => None,
        }
    }

    /// Get marquee rect if in marquee mode
    #[must_use]
    pub fn marquee_rect(&self) -> Option<((f32, f32), (f32, f32))> {
        match &*self.mode.read() {
            InteractionMode::Marquee { start, current } => Some((*start, *current)),
            _ => None,
        }
    }
}

pub fn use_canvas_interaction() -> CanvasInteraction {
    let mode = use_signal(InteractionMode::default);
    let is_space_hand = use_signal(|| false);
    let mouse_pos = use_signal(|| (0.0_f32, 0.0_f32));
    let canvas_origin = use_signal(|| (0.0_f32, 0.0_f32));
    let temp_edge_to = use_signal(|| None::<(FlowPosition, FlowPosition)>);
    let hovered_handle = use_signal(|| None::<(NodeId, String)>);
    let drag_anchor = use_signal(|| None::<(f32, f32)>);

    CanvasInteraction {
        mode,
        is_space_hand,
        mouse_pos,
        canvas_origin,
        temp_edge_to,
        hovered_handle,
        drag_anchor,
    }
}

#[cfg(test)]
mod tests {
    use super::{cursor_class_for, drag_mode_from_selection, update_marquee_mode, InteractionMode};
    use crate::graph::NodeId;

    #[test]
    fn given_empty_selected_ids_when_starting_drag_then_mode_remains_unchanged() {
        let next = drag_mode_from_selection(Vec::new());
        assert_eq!(next, None);
    }

    #[test]
    fn given_marquee_mode_when_updating_then_start_is_preserved_and_current_updates() {
        let mode = InteractionMode::Marquee {
            start: (1.0, 2.0),
            current: (3.0, 4.0),
        };

        let next = update_marquee_mode(&mode, (9.0, 10.0));
        assert_eq!(
            next,
            InteractionMode::Marquee {
                start: (1.0, 2.0),
                current: (9.0, 10.0)
            }
        );
    }

    #[test]
    fn given_space_hand_enabled_and_idle_when_getting_cursor_class_then_cursor_grab_is_returned() {
        let class = cursor_class_for(&InteractionMode::Idle, true);
        assert_eq!(class, "cursor-grab");
    }

    #[test]
    fn given_non_empty_selected_ids_when_starting_drag_then_dragging_mode_is_returned() {
        let id = NodeId::new();
        let next = drag_mode_from_selection(vec![id]);
        assert_eq!(next, Some(InteractionMode::Dragging { node_ids: vec![id] }));
    }
}
