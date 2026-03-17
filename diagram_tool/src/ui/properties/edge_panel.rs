#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::history::History;
use crate::ui::dispatch::dispatch_update_edge_style;
use crate::ui::properties_helpers::{
    arrow_type_str, edge_style_str, parse_arrow_type, parse_edge_style,
};
use crate::ui::theme::{BG_BASE, BORDER, BORDER_SUBTLE, TEXT_DIM, TEXT_MAIN, TEXT_MUTED};
use crate::ui::toast::use_toast;
use diagram_models::document::{DiagramDocument, Edge, EdgeId, OrderedFloat};
use diagram_models::envelope::EventEnvelope;
use dioxus::prelude::*;

#[component]
#[allow(clippy::approx_constant, clippy::float_cmp)]
pub fn EdgePanel(id: EdgeId, edge: Edge, source_label: String, target_label: String) -> Element {
    let mut doc_signal = use_context::<Signal<DiagramDocument>>();
    let mut history = use_context::<Signal<History>>();
    let db_tx = use_context::<Option<Coroutine<EventEnvelope>>>();
    let toast = use_toast();

    let eid_label = id.clone();
    let eid_style = id.clone();
    let eid_arrow = id.clone();
    let eid_font = id.clone();

    rsx! {
        div {
            key: "{id}",
            style: "display: flex; flex-direction: column; gap: 10px;",
            div {
                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Edge Label" }
                input {
                    style: "width: 100%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                    value: "{edge.label}",
                    oninput: move |evt| {
                        let eid = eid_label.clone();
                        doc_signal.with_mut(|doc| {
                            if let Some(e) = doc.document.edges.get_mut(&eid) {
                                e.label = evt.value();
                                doc.revision = doc.revision.increment();
                            }
                        });
                    }
                }
            }

            div {
                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Line Style" }
                select {
                    style: "width: 100%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                    value: "{edge_style_str(edge.style)}",
                    onchange: move |evt| {
                        let eid = eid_style.clone();
                        let new_style = parse_edge_style(&evt.value());
                        let has_changes = doc_signal.read()
                            .document
                            .edges
                            .get(&eid)
                            .is_some_and(|e| e.style != new_style);
                        if has_changes {
                            let current = doc_signal.read().clone();
                            let next_h = history.read().push(current);
                            *history.write() = next_h;
                            if let Err(e) = dispatch_update_edge_style(&db_tx, eid.as_str(), new_style) {
                                let _ = toast.error("Failed to save", Some(format!("{e:?}")));
                            }
                        }
                        doc_signal.with_mut(|doc| {
                            if let Some(e) = doc.document.edges.get_mut(&eid) {
                                e.style = new_style;
                                doc.revision = doc.revision.increment();
                            }
                        });
                    },
                    option { value: "solid", "Solid" }
                    option { value: "dashed", "Dashed" }
                    option { value: "dotted", "Dotted" }
                }
            }

            div {
                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Arrow Type" }
                select {
                    style: "width: 100%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                    value: "{arrow_type_str(edge.arrow_type)}",
                    onchange: move |evt| {
                        let eid = eid_arrow.clone();
                        let arrow = parse_arrow_type(&evt.value());
                        doc_signal.with_mut(|doc| {
                            if let Some(e) = doc.document.edges.get_mut(&eid) {
                                e.arrow_type = arrow;
                                doc.revision = doc.revision.increment();
                            }
                        });
                    },
                    option { value: "default", "Default" }
                    option { value: "straight", "Straight" }
                    option { value: "curved", "Curved" }
                    option { value: "step", "Step" }
                    option { value: "sharp", "Sharp" }
                }
            }

            div {
                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Font Size" }
                input {
                    style: "width: 100%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                    r#type: "number",
                    value: "{edge.font_size.map_or(10.0, |v| v.0)}",
                    oninput: move |evt| {
                        let eid = eid_font.clone();
                        if let Ok(val) = evt.value().parse::<f64>() {
                            doc_signal.with_mut(|doc| {
                                if let Some(e) = doc.document.edges.get_mut(&eid) {
                                    e.font_size = Some(OrderedFloat(val.clamp(8.0, 72.0)));
                                    doc.revision = doc.revision.increment();
                                }
                            });
                        }
                    }
                }
            }

            div { style: "height: 1px; background: {BORDER_SUBTLE};" }

            div {
                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Source" }
                p {
                    style: "margin: 3px 0 0 0; font-size: 11px; color: {TEXT_MAIN}; word-break: break-all;",
                    "{source_label}"
                }
            }

            div {
                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Target" }
                p {
                    style: "margin: 3px 0 0 0; font-size: 11px; color: {TEXT_MAIN}; word-break: break-all;",
                    "{target_label}"
                }
            }

            div {
                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "ID" }
                p {
                    style: "margin: 3px 0 0 0; font-size: 10px; color: {TEXT_DIM}; word-break: break-all;",
                    "{id}"
                }
            }
        }
    }
}
