#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]
#![allow(dead_code)]

mod async_sync;
mod autosave_hooks;
mod types;
mod validation;

pub use types::DraggedIconPayload;

use crate::history::History;
use crate::hooks::e2e_reset::use_e2e_reset_hook;
use crate::hooks::keyboard::use_global_keyboard;
use crate::ui::canvas::Canvas;
use crate::ui::commands::ClipboardData;
use crate::ui::editor::ToolMode;
use crate::ui::theme_provider::ThemeProvider;
use crate::ui::toast::{ToastQueue, Toaster};
use crate::ui::toolbar::{Toolbar, ToolbarStats};
use crate::ui::sidebar::Sidebar;
use crate::ui::mobile::SidebarUiState;
use diagram_models::document::{ArrowType, DiagramDocument, EdgeStyle};

use dioxus::prelude::*;

#[allow(non_snake_case)]
#[allow(clippy::too_many_lines)]
pub fn App() -> Element {
    use_context_provider(|| Signal::new(DiagramDocument::default()));
    let _dragging_icon = use_context_provider(|| Signal::new(Option::<DraggedIconPayload>::None));
    use_context_provider(|| Signal::new(History::new()));
    use_context_provider(|| Signal::new(Option::<ClipboardData>::None));
    use_context_provider(|| Signal::new(ToolMode::Select));
    use_context_provider(|| Signal::new(EdgeStyle::Solid));
    use_context_provider(|| Signal::new(ArrowType::Default));
    use_context_provider(|| Signal::new(ToastQueue::default()));
    use_context_provider(|| Signal::new(ToolbarStats::default()));
    use_context_provider(|| Signal::new(SidebarUiState::default()));
    use_context_provider(|| Signal::new((1200.0_f64, 800.0_f64)));
    use_context_provider(|| Signal::new(0_u64));
    use_context_provider(|| Signal::new(Option::<crate::ui::toast::AiConflictState>::None));
    use_context_provider(|| Signal::new(false));
    use_context_provider(|| Signal::new(std::collections::HashSet::<String>::new()));

    let doc_signal = use_context::<Signal<DiagramDocument>>();
    let mut toolbar_stats = use_context::<Signal<ToolbarStats>>();

    let keyboard_db_tx = async_sync::provide_db_event_context();
    use_global_keyboard(keyboard_db_tx);
    use_e2e_reset_hook();

    async_sync::use_conflict_toast_effect();

    #[cfg(all(feature = "async-db", not(target_arch = "wasm32")))]
    async_sync::use_store_sync_loop(doc_signal);

    #[cfg(target_arch = "wasm32")]
    autosave_hooks::use_auto_save(doc_signal);

    use_effect(move || {
        let doc = doc_signal.read();
        let next = ToolbarStats {
            selected_count: doc.editor_state.selected_items.len(),
            node_count: doc.document.nodes.len(),
            edge_count: doc.document.edges.len(),
            revision: doc.revision.value(),
        };
        if *toolbar_stats.read() != next {
            toolbar_stats.set(next);
        }
    });

    rsx! {
        ThemeProvider {
            div {
                display: "flex",
                flex_direction: "column",
                width: "100vw",
                height: "100vh",
                overflow: "hidden",
                Toolbar {}
                Toaster {}
                div {
                    display: "flex",
                    flex: "1",
                    position: "relative",
                    min_width: "0",
                    Sidebar {}
                    Canvas {}
                }
            }
        }
    }
}