use crate::app::types::DraggedIconPayload;
use crate::history::History;
use crate::ui::editor::ToolMode;
use crate::ui::mobile::SidebarUiState;
use crate::ui::panels::PanelVisibility;
use crate::ui::toast::{AiConflictState, ToastQueue};
use crate::ui::toolbar::ToolbarStats;
use diagram_models::clipboard::ClipboardData;
use diagram_models::document::{ArrowType, DiagramDocument, DocumentSession, EdgeStyle};
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, Copy)]
pub struct AppState {
    pub document: Signal<DiagramDocument>,
    pub session: Signal<DocumentSession>,
    pub history: Signal<History>,
    pub clipboard: Signal<Option<ClipboardData>>,
    pub tool_mode: Signal<ToolMode>,
    pub edge_style: Signal<EdgeStyle>,
    pub arrow_type: Signal<ArrowType>,
    pub dragging_icon: Signal<Option<DraggedIconPayload>>,
    pub sidebar: Signal<SidebarUiState>,
    pub panels: Signal<PanelVisibility>,
    pub toolbar_stats: Signal<ToolbarStats>,
    pub viewport_size: Signal<(f64, f64)>,
    pub ai_conflict: Signal<Option<AiConflictState>>,
    pub conflict_toast_shown: Signal<bool>,
    pub pending_ai_ops: Signal<HashSet<String>>,
    pub validate_trigger: Signal<u64>,
    pub canvas_reset_trigger: Signal<u64>,
    pub toasts: Signal<ToastQueue>,
}

impl AppState {
    pub fn provide() -> Self {
        let doc = DiagramDocument::default();
        let session = DocumentSession::new(doc.clone());
        use_context_provider(|| Self {
            document: Signal::new(doc),
            session: Signal::new(session),
            history: Signal::new(History::new()),
            clipboard: Signal::new(None),
            tool_mode: Signal::new(ToolMode::Select),
            edge_style: Signal::new(EdgeStyle::Solid),
            arrow_type: Signal::new(ArrowType::Default),
            dragging_icon: Signal::new(None),
            sidebar: Signal::new(SidebarUiState::default()),
            panels: Signal::new(PanelVisibility::default()),
            toolbar_stats: Signal::new(ToolbarStats::default()),
            viewport_size: Signal::new((0.0_f64, 0.0_f64)),
            ai_conflict: Signal::new(None),
            conflict_toast_shown: Signal::new(false),
            pending_ai_ops: Signal::new(HashSet::new()),
            validate_trigger: Signal::new(0_u64),
            canvas_reset_trigger: Signal::new(0_u64),
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
            assert!(!state.panels.read().properties);
            assert_eq!(*state.canvas_reset_trigger.read(), 0);
            assert!(!state.session.read().is_dirty());
            assert!(state.session.read().file_path().is_none());
            rsx! { div {} }
        }

        let mut vdom = VirtualDom::new(TestComponent);
        vdom.rebuild_in_place(); // Should not panic, meaning signals are provided correctly
    }
}
