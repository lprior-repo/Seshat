#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::history::History;
use crate::ui::dispatch::dispatch_update_node_style;
use crate::ui::properties_helpers::{node_kind_str, node_style_str, parse_node_style};
use crate::ui::theme::{BG_BASE, BORDER, TEXT_MAIN, TEXT_MUTED};
use diagram_models::document::{DiagramDocument, Node, NodeId};
use diagram_models::envelope::EventEnvelope;
use dioxus::prelude::*;

use super::update::update_node_if_changed;

#[component]
#[allow(clippy::approx_constant, clippy::float_cmp)]
pub fn CorePropsPanel(id: NodeId, node: Node) -> Element {
    let mut doc_signal = use_context::<Signal<DiagramDocument>>();
    let mut history = use_context::<Signal<History>>();
    let db_tx = use_context::<Option<Coroutine<EventEnvelope>>>();

    let id_label = id.clone();
    let id_style = id.clone();

    rsx! {
        div {
            label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Label" }
            input {
                "data-testid": "node-label-input",
                style: "width: 100%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                value: "{node.label}",
                onchange: move |evt| {
                    let new_label = evt.value();
                    let lbl_clone = new_label.clone();
                    update_node_if_changed(
                        &mut doc_signal,
                        &mut history,
                        &id_label,
                        |n| n.label != lbl_clone,
                        |n| n.label = new_label,
                    );
                }
            }
        }

        div {
            label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Kind" }
            div {
                style: "margin-top: 3px; display: inline-block; border: 1px solid {BORDER}; border-radius: 999px; padding: 2px 8px; font-size: 11px; color: {TEXT_MAIN};",
                "{node_kind_str(&node.kind)}"
            }
        }

        div {
            label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Style" }
            select {
                style: "width: 100%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                value: "{node_style_str(&node.style)}",
                onchange: move |evt| {
                    let new_style = match parse_node_style(&evt.value()) {
                        Ok(style) => style,
                        Err(_) => return,
                    };
                    let style_clone = new_style.clone();
                    let nid = id_style.clone();

                    let changed = doc_signal.read().document.nodes.get(&nid).is_some_and(|n| n.style.as_ref() != Some(&style_clone));

                    if changed {
                        let current = doc_signal.read().clone();
                        let next_h = history.read().push(current);
                        *history.write() = next_h;
                        dispatch_update_node_style(&db_tx, nid.as_str(), style_clone.clone()).ok();
                        doc_signal.with_mut(|doc| {
                            if let Some(n) = doc.document.nodes.get_mut(&nid) {
                                n.style = Some(style_clone);
                                doc.revision = doc.revision.increment();
                            }
                        });
                    }
                },
                option { value: "box", "Box" }
                option { value: "cloud", "Cloud" }
                option { value: "cylinder", "Cylinder" }
                option { value: "dashed", "Dashed" }
            }
        }

        if !node.icon.is_empty() {
            div {
                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Icon" }
                p {
                    style: "margin: 3px 0 0 0; font-size: 11px; color: {TEXT_MAIN}; word-break: break-all;",
                    "{node.icon}"
                }
            }
        }
    }
}
