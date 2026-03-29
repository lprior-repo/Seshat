#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![allow(clippy::volatile_composites)]
#![forbid(unsafe_code)]
#![allow(dead_code)]

mod async_sync;
mod autosave_hooks;
mod state;
mod types;
mod validation;

#[cfg(test)]
mod tests;

pub use state::AppState;
pub use types::DraggedIconPayload;

use crate::hooks::e2e_reset::use_e2e_reset_hook;
use crate::hooks::keyboard::use_global_keyboard;
use crate::ui::canvas::Canvas;
use crate::ui::sidebar::Sidebar;
use crate::ui::theme_provider::ThemeProvider;
use crate::ui::toast::Toaster;
use crate::ui::toolbar::{Toolbar, ToolbarStats};

use dioxus::prelude::*;

#[allow(non_snake_case)]
#[allow(clippy::too_many_lines)]
pub fn App() -> Element {
    let state = AppState::provide();

    let doc_signal = state.document;
    let mut toolbar_stats = state.toolbar_stats;

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
            document::Stylesheet {
                href: asset!("/assets/tailwind.css")
            }
            div {
                class: "flex flex-col w-screen h-screen overflow-hidden",
                Toolbar {}
                Toaster {}
                div {
                    class: "flex flex-1 relative min-w-0 min-h-0",
                    Sidebar {}
                    Canvas {}
                }
            }
        }
    }
}
