use crate::app::types::DraggedIconPayload;
use crate::history::History;
use crate::ui::commands::ClipboardData;
use crate::ui::editor::ToolMode;
use crate::ui::mobile::SidebarUiState;
use crate::ui::toast::{AiConflictState, ToastQueue};
use crate::ui::toolbar::ToolbarStats;
use diagram_models::document::{ArrowType, DiagramDocument, EdgeStyle};
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, Copy)]
pub struct AppState {
    pub document: Signal<DiagramDocument>,
    pub history: Signal<History>,
    pub clipboard: Signal<Option<ClipboardData>>,
    pub tool_mode: Signal<ToolMode>,
    pub edge_style: Signal<EdgeStyle>,
    pub arrow_type: Signal<ArrowType>,
    pub dragging_icon: Signal<Option<DraggedIconPayload>>,
    pub sidebar: Signal<SidebarUiState>,
    pub toolbar_stats: Signal<ToolbarStats>,
    pub viewport_size: Signal<(f64, f64)>,
    pub ai_conflict: Signal<Option<AiConflictState>>,
    pub conflict_toast_shown: Signal<bool>,
    pub pending_ai_ops: Signal<HashSet<String>>,
    pub validate_trigger: Signal<u64>,
    pub toasts: Signal<ToastQueue>,
}

impl AppState {
    pub fn provide() -> Self {
        use_context_provider(|| Self {
            document: Signal::new(DiagramDocument::default()),
            history: Signal::new(History::new()),
            clipboard: Signal::new(None),
            tool_mode: Signal::new(ToolMode::Select),
            edge_style: Signal::new(EdgeStyle::Solid),
            arrow_type: Signal::new(ArrowType::Default),
            dragging_icon: Signal::new(None),
            sidebar: Signal::new(SidebarUiState::default()),
            toolbar_stats: Signal::new(ToolbarStats::default()),
            viewport_size: Signal::new((0.0_f64, 0.0_f64)),
            ai_conflict: Signal::new(None),
            conflict_toast_shown: Signal::new(false),
            pending_ai_ops: Signal::new(HashSet::new()),
            validate_trigger: Signal::new(0_u64),
            toasts: Signal::new(ToastQueue::default()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::prelude::*;

    #[test]
    fn test_app_state_provide() {
        #[component]
        fn TestComponent() -> Element {
            let state = AppState::provide();
            assert_eq!(state.tool_mode.read().clone(), ToolMode::Select);
            assert_eq!(state.edge_style.read().clone(), EdgeStyle::Solid);
            assert_eq!(state.arrow_type.read().clone(), ArrowType::Default);
            assert_eq!(state.sidebar.read().open, true);
            rsx! { div {} }
        }

        let mut vdom = VirtualDom::new(TestComponent);
        vdom.rebuild_in_place(); // Should not panic, meaning signals are provided correctly
    }
}
