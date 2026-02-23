#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use dioxus::prelude::*;
use crate::models::document::DiagramDocument;
use crate::layout::grid::calculate_grid_layout;
use crate::history::History;
use crate::export::png::export_png;
use crate::export::svg::generate_svg_string;
use std::fs::File;
use std::io::Write;

#[component]
pub fn Toolbar() -> Element {
    let mut doc_signal = use_context::<Signal<DiagramDocument>>();
    let mut history_signal = use_context::<Signal<History>>();

    let handle_auto_layout = move |_| {
        let current_doc = doc_signal.read().clone();
        let history = history_signal.read().clone();
        *history_signal.write() = history.push(current_doc);
        doc_signal.with_mut(|doc| {
            let next_doc = calculate_grid_layout(doc, 200.0);
            *doc = next_doc;
            doc.revision = doc.revision.increment();
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
        }
    }
}
