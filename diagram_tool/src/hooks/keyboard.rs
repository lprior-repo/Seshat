#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::history::History;
use crate::models::document::DiagramDocument;
use dioxus::prelude::*;

/// Global keyboard hook that handles ONLY undo/redo shortcuts:
/// - Ctrl+Z        → undo
/// - Ctrl+Shift+Z  → redo
/// - Ctrl+Y        → redo
///
/// Must be called inside a component that has `Signal<DiagramDocument>` and
/// `Signal<History>` in context (i.e. after the context providers in `App`).
pub fn use_global_keyboard() {
    let mut doc_signal = use_context::<Signal<DiagramDocument>>();
    let mut history_signal = use_context::<Signal<History>>();

    use_effect(move || {
        let mut eval = document::eval(
            r#"
            window.addEventListener('keydown', (e) => {
                if (e.ctrlKey && (e.key === 'z' || e.key === 'Z' || e.key === 'y' || e.key === 'Y')) {
                    dioxus.send({ type: 'keydown', key: e.key, ctrl: e.ctrlKey, shift: e.shiftKey });
                }
            });
        "#,
        );

        spawn(async move {
            while let Ok(json) = eval.recv::<serde_json::Value>().await {
                let key = json["key"].as_str().map_or("", |s| s);
                let ctrl = json["ctrl"].as_bool().is_some_and(|b| b);
                let shift = json["shift"].as_bool().is_some_and(|b| b);

                if ctrl {
                    match (shift, key) {
                        (true, "z" | "Z") => {
                            let (current, history) =
                                (doc_signal.read().clone(), history_signal.read().clone());
                            if let Some((doc, h)) = history.redo(current) {
                                *doc_signal.write() = doc;
                                *history_signal.write() = h;
                            }
                        }
                        (false, "z" | "Z") => {
                            let (current, history) =
                                (doc_signal.read().clone(), history_signal.read().clone());
                            if let Some((doc, h)) = history.undo(current) {
                                *doc_signal.write() = doc;
                                *history_signal.write() = h;
                            }
                        }
                        (_, "y" | "Y") => {
                            let (current, history) =
                                (doc_signal.read().clone(), history_signal.read().clone());
                            if let Some((doc, h)) = history.redo(current) {
                                *doc_signal.write() = doc;
                                *history_signal.write() = h;
                            }
                        }
                        _ => {}
                    }
                }
            }
        });
    });
}
