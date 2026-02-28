#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! E2E test reset hook.
//!
//! Registers `window.__seshatResetDocument()` which, when called from a
//! Playwright test, sends a message back to the Dioxus runtime.  The hook
//! receives the message and resets **every** context signal to its default
//! value, giving each test a guaranteed-clean starting state without a full
//! page reload.
//!
//! The hook also exposes `window.__seshatE2eReady` as a boolean that tests
//! can poll to know when the WASM app has fully initialised its signals.

use crate::app::DraggedIconPayload;
use crate::history::History;
use crate::models::document::{ArrowType, DiagramDocument, EdgeStyle};
use crate::ui::editor::ToolMode;
use crate::ui::mobile::SidebarUiState;
use crate::ui::panels::PanelVisibility;
use crate::ui::toast::ToastQueue;
use crate::ui::toolbar::ToolbarStats;
use dioxus::prelude::*;

/// Must be called inside the `App` component after all `use_context_provider`
/// calls so that every signal is already present in context.
pub fn use_e2e_reset_hook() {
    let mut doc_signal = use_context::<Signal<DiagramDocument>>();
    let mut dragging_icon = use_context::<Signal<Option<DraggedIconPayload>>>();
    let mut history_signal = use_context::<Signal<History>>();
    let mut tool_mode = use_context::<Signal<ToolMode>>();
    let mut edge_style = use_context::<Signal<EdgeStyle>>();
    let mut arrow_type = use_context::<Signal<ArrowType>>();
    let mut toast_queue = use_context::<Signal<ToastQueue>>();
    let mut panel_vis = use_context::<Signal<PanelVisibility>>();
    let mut toolbar_stats = use_context::<Signal<ToolbarStats>>();
    let mut sidebar_ui = use_context::<Signal<SidebarUiState>>();
    let mut viewport_size = use_context::<Signal<(f64, f64)>>();
    let mut validate_trigger = use_context::<Signal<u64>>();

    use_effect(move || {
        let mut eval = document::eval(
            r#"
            window.__seshatE2eReady = true;

            if (window.__seshat_e2e_reset_cleanup) {
                window.__seshat_e2e_reset_cleanup();
            }

            const handler = () => {
                dioxus.send({ type: "reset" });
            };

            const isWebDriver = navigator.webdriver === true;
            if (isWebDriver) {
                try { localStorage.clear(); } catch (_) {}
                try { sessionStorage.clear(); } catch (_) {}
            }

            window.__seshatResetDocument = () => {
                return new Promise((resolve) => {
                    window.__seshat_e2e_reset_resolve = resolve;
                    handler();
                });
            };

            window.__seshat_e2e_reset_cleanup = () => {
                delete window.__seshatResetDocument;
                delete window.__seshatE2eReady;
                delete window.__seshat_e2e_reset_resolve;
                delete window.__seshat_e2e_reset_cleanup;
            };

            if (isWebDriver) {
                queueMicrotask(handler);
            }
            "#,
        );

        spawn(async move {
            while let Ok(msg) = eval.recv::<serde_json::Value>().await {
                let msg_type = msg["type"].as_str().map_or("", |s| s);
                if msg_type == "reset" {
                    doc_signal.set(DiagramDocument::default());
                    dragging_icon.set(None);
                    history_signal.set(History::new());
                    tool_mode.set(ToolMode::Select);
                    edge_style.set(EdgeStyle::Solid);
                    arrow_type.set(ArrowType::Default);
                    toast_queue.set(ToastQueue::default());
                    panel_vis.set(PanelVisibility::default());
                    toolbar_stats.set(ToolbarStats::default());
                    sidebar_ui.set(SidebarUiState::default());
                    viewport_size.set((1200.0, 800.0));
                    validate_trigger.set(0);

                    // Signal completion back to the JS Promise.
                    let _ = document::eval(
                        r"
                        if (window.__seshat_e2e_reset_resolve) {
                            window.__seshat_e2e_reset_resolve();
                            delete window.__seshat_e2e_reset_resolve;
                        }
                        ",
                    );
                }
            }
        });
    });

    use_drop(move || {
        let _ = document::eval(
            r"
            if (window.__seshat_e2e_reset_cleanup) {
                window.__seshat_e2e_reset_cleanup();
            }
            ",
        );
    });
}
