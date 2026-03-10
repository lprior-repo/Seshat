#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::history::History;
use crate::models::document::DiagramDocument;
use crate::ui::commands::{
    apply_copy_selection, apply_duplicate_selection, apply_group_selection, apply_paste_selection,
    apply_redo, apply_select_all, apply_undo, apply_ungroup_selection, ClipboardData,
};
use dioxus::prelude::*;

/// Global keyboard hook that handles document-wide modifier shortcuts.
///
/// Must be called inside a component that has `Signal<DiagramDocument>` and
/// `Signal<History>` in context (i.e. after the context providers in `App`).
pub fn use_global_keyboard() {
    let doc_signal = use_context::<Signal<DiagramDocument>>();
    let history_signal = use_context::<Signal<History>>();
    let clipboard_signal = use_context::<Signal<Option<ClipboardData>>>();

    use_effect(move || {
        let mut eval = document::eval(
            r"
            if (window.__seshat_global_keyboard_cleanup) {
                window.__seshat_global_keyboard_cleanup();
            }

            const onKeyDown = (e) => {
                const active = document.activeElement;
                const editing = active && (
                    active.tagName === 'INPUT' ||
                    active.tagName === 'TEXTAREA' ||
                    active.isContentEditable
                );
                if (editing) return;
                const modifier = e.ctrlKey || e.metaKey;
                const key = e.key.length === 1 ? e.key.toLowerCase() : e.key;
                const handled = modifier && (
                    key === 'z' ||
                    key === 'y' ||
                    key === 'a' ||
                    key === 'c' ||
                    key === 'v' ||
                    key === 'd' ||
                    key === 'g'
                );
                if (handled) {
                    e.preventDefault();
                    dioxus.send({ type: 'keydown', key: e.key, modifier, shift: e.shiftKey });
                }
            };

            window.addEventListener('keydown', onKeyDown);

            window.__seshat_global_keyboard_cleanup = () => {
                window.removeEventListener('keydown', onKeyDown);
            };
        ",
        );

        spawn(async move {
            while let Ok(json) = eval.recv::<serde_json::Value>().await {
                let key = json["key"].as_str().map_or("", |s| s);
                let modifier = json["modifier"].as_bool().is_some_and(|b| b);
                let shift = json["shift"].as_bool().is_some_and(|b| b);
                let lowered = key.to_ascii_lowercase();

                if modifier {
                    match (shift, lowered.as_str()) {
                        (true, "z") | (_, "y") => {
                            apply_redo(doc_signal, history_signal);
                        }
                        (false, "z") => {
                            apply_undo(doc_signal, history_signal);
                        }
                        (_, "a") => {
                            apply_select_all(doc_signal);
                        }
                        (_, "c") => {
                            let _ = apply_copy_selection(doc_signal, clipboard_signal);
                        }
                        (_, "v") => {
                            let _ =
                                apply_paste_selection(doc_signal, clipboard_signal, history_signal);
                        }
                        (_, "d") => {
                            let _ = apply_duplicate_selection(
                                doc_signal,
                                clipboard_signal,
                                history_signal,
                            );
                        }
                        (true, "g") => {
                            let _ = apply_ungroup_selection(doc_signal, history_signal);
                        }
                        (false, "g") => {
                            let _ = apply_group_selection(doc_signal, history_signal);
                        }
                        _ => {}
                    }
                }
            }
        });
    });

    use_drop(move || {
        let _ = document::eval(
            r"
                if (window.__seshat_global_keyboard_cleanup) {
                    window.__seshat_global_keyboard_cleanup();
                    window.__seshat_global_keyboard_cleanup = null;
                }
            ",
        );
    });
}
