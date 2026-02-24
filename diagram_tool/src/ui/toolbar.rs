#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::export::png::export_png;
use crate::export::svg::generate_svg_string;
use crate::history::History;
use crate::layout::dag::{dag_layout, DagLayoutSettings};
use crate::models::document::{DiagramDocument, Revision};
use dioxus::prelude::*;
use rfd::FileDialog;
use std::fs;
use std::fs::File;
use std::io::Write;

#[component]
pub fn Toolbar() -> Element {
    let mut doc_signal = use_context::<Signal<DiagramDocument>>();
    let mut history_signal = use_context::<Signal<History>>();
    let mut validate_trigger = use_context::<Signal<u64>>();
    let mut toolbar_message: Signal<Option<String>> = use_signal(|| None);

    let handle_auto_layout = move |_| {
        let current_doc = doc_signal.read().clone();
        let history = history_signal.read().clone();
        *history_signal.write() = history.push(current_doc);
        doc_signal.with_mut(|doc| {
            let next_doc = dag_layout(doc, &DagLayoutSettings::default());
            *doc = next_doc;
        });
    };

    let handle_undo = move |_| {
        let current = doc_signal.read().clone();
        let history = history_signal.read().clone();
        if let Some((doc, h)) = history.undo(current) {
            *doc_signal.write() = doc;
            *history_signal.write() = h;
        }
    };

    let handle_redo = move |_| {
        let current = doc_signal.read().clone();
        let history = history_signal.read().clone();
        if let Some((doc, h)) = history.redo(current) {
            *doc_signal.write() = doc;
            *history_signal.write() = h;
        }
    };

    let handle_save = move |_| {
        let doc_snapshot = doc_signal.read().clone();
        let mut msg = toolbar_message;
        spawn(async move {
            let path = FileDialog::new()
                .add_filter("Seshat Diagram", &["json"])
                .set_file_name("diagram.json")
                .save_file();
            match path {
                None => {}
                Some(p) => match serde_json::to_string_pretty(&doc_snapshot) {
                    Ok(json_str) => match fs::write(&p, json_str.as_bytes()) {
                        Ok(()) => msg.set(Some(format!("Saved to {}", p.display()))),
                        Err(e) => msg.set(Some(format!("Save error: {e}"))),
                    },
                    Err(e) => msg.set(Some(format!("Serialize error: {e}"))),
                },
            }
        });
    };

    let handle_open = move |_| {
        let mut doc_sig = doc_signal;
        let mut hist_sig = history_signal;
        let mut msg = toolbar_message;
        spawn(async move {
            let path = FileDialog::new()
                .add_filter("Seshat Diagram", &["json"])
                .pick_file();
            match path {
                None => {}
                Some(p) => match fs::read_to_string(&p) {
                    Err(e) => msg.set(Some(format!("Read error: {e}"))),
                    Ok(contents) => match serde_json::from_str::<DiagramDocument>(&contents) {
                        Err(e) => msg.set(Some(format!("Parse error: {e}"))),
                        Ok(mut loaded_doc) => {
                            loaded_doc.revision = Revision::INITIAL;
                            *doc_sig.write() = loaded_doc;
                            *hist_sig.write() = History::new();
                            msg.set(Some(format!("Loaded from {}", p.display())));
                        }
                    },
                },
            }
        });
    };

    rsx! {
        div {
            class: "toolbar",
            style: "height: 50px; background: #333; color: white; display: flex; align-items: center; padding: 0 20px; gap: 10px;",

            button {
                style: "padding: 5px 10px; cursor: pointer;",
                onclick: handle_auto_layout,
                "Auto-Arrange"
            }

            div { style: "width: 1px; height: 20px; background: #666;" }

            button {
                style: "padding: 5px 10px; cursor: pointer;",
                onclick: handle_undo,
                "Undo"
            }
            button {
                style: "padding: 5px 10px; cursor: pointer;",
                onclick: handle_redo,
                "Redo"
            }

            div { style: "width: 1px; height: 20px; background: #666;" }

            button {
                style: "padding: 5px 10px; cursor: pointer; background: #1d4ed8; border: none; border-radius: 4px; color: white;",
                onclick: move |_| {
                    validate_trigger.with_mut(|t| *t = t.saturating_add(1));
                },
                "Validate"
            }

            div { style: "width: 1px; height: 20px; background: #666;" }

            button {
                style: "padding: 5px 10px; cursor: pointer;",
                onclick: handle_save,
                "Save"
            }
            button {
                style: "padding: 5px 10px; cursor: pointer;",
                onclick: handle_open,
                "Open"
            }

            div { style: "flex: 1;" }

            button {
                style: "padding: 5px 10px; cursor: pointer;",
                onclick: move |_| {
                    let doc = doc_signal.read();
                    let _ = export_png(&doc, "diagram.png");
                },
                "Export PNG"
            }
            button {
                style: "padding: 5px 10px; cursor: pointer;",
                onclick: move |_| {
                    let doc = doc_signal.read();
                    let svg = generate_svg_string(&doc);
                    if let Ok(mut file) = File::create("diagram.svg") {
                        let _ = file.write_all(svg.as_bytes());
                    }
                },
                "Export SVG"
            }

            if let Some(msg) = toolbar_message.read().as_deref() {
                span {
                    style: "font-size: 12px; color: #aaa; margin-left: 8px;",
                    "{msg}"
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use crate::models::document::DiagramDocument;

    #[test]
    fn given_document_when_serialized_then_round_trips() {
        let doc = DiagramDocument::default();
        let json = serde_json::to_string_pretty(&doc).unwrap();
        let loaded: DiagramDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(doc.revision, loaded.revision);
    }
}
