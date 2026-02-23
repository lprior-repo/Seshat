#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use dioxus::prelude::*;
use crate::models::document::{DiagramDocument, NodeId, OrderedFloat};
use crate::history::History;

#[component]
pub fn PropertiesPanel() -> Element {
    let mut doc_signal = use_context::<Signal<DiagramDocument>>();
    let mut history = use_context::<Signal<History>>();
    
    let current_selection = use_memo(move || {
        doc_signal.read().editor_state.selected_items.iter().next().and_then(|id_str| {
            let id = NodeId::new(id_str.clone());
            doc_signal.read().document.nodes.get(&id).map(|n| (id, n.clone()))
        })
    });

    let selection = current_selection.read().clone();

    rsx! {
        div {
            class: "properties-panel",
            style: "width: 250px; background: #f0f0f0; padding: 10px; border-left: 1px solid #ccc; display: flex; flex-direction: column; gap: 10px;",
            
            if let Some((id, node)) = selection {
                {
                    let (id_label, id_x, id_y) = (id.clone(), id.clone(), id.clone());
                    rsx! {
                        div { key: "{id}", style: "display: flex; flex-direction: column; gap: 10px;",
                            div {
                                label { style: "display: block; font-size: 12px; color: #666;", "Label" }
                                input {
                                    style: "width: 100%; padding: 5px;",
                                    value: "{node.label}",
                                    onfocus: move |_| {
                                        let current = doc_signal.read().clone();
                                        let next_h = history.read().push(current);
                                        *history.write() = next_h;
                                    },
                                    oninput: move |evt| {
                                        let id = id_label.clone();
                                        doc_signal.with_mut(|doc| {
                                            if let Some(n) = doc.document.nodes.get_mut(&id) {
                                                n.label = evt.value();
                                            }
                                        });
                                    }
                                }
                            }
                            div {
                                label { style: "display: block; font-size: 12px; color: #666;", "Position" }
                                div { style: "display: flex; gap: 5px;",
                                    input {
                                        style: "width: 50%; padding: 5px;",
                                        type: "number",
                                        value: "{node.x}",
                                        onfocus: move |_| {
                                            let current = doc_signal.read().clone();
                                            let next_h = history.read().push(current);
                                            *history.write() = next_h;
                                        },
                                        oninput: move |evt| {
                                            let id = id_x.clone();
                                            if let Ok(val) = evt.value().parse::<f64>() {
                                                doc_signal.with_mut(|doc| {
                                                    if let Some(n) = doc.document.nodes.get_mut(&id) {
                                                        n.x = OrderedFloat(val);
                                                    }
                                                });
                                            }
                                        }
                                    }
                                    input {
                                        style: "width: 50%; padding: 5px;",
                                        type: "number",
                                        value: "{node.y}",
                                        onfocus: move |_| {
                                            let current = doc_signal.read().clone();
                                            let next_h = history.read().push(current);
                                            *history.write() = next_h;
                                        },
                                        oninput: move |evt| {
                                            let id = id_y.clone();
                                            if let Ok(val) = evt.value().parse::<f64>() {
                                                doc_signal.with_mut(|doc| {
                                                    if let Some(n) = doc.document.nodes.get_mut(&id) {
                                                        n.y = OrderedFloat(val);
                                                    }
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                            div {
                                label { style: "display: block; font-size: 12px; color: #666;", "Icon" }
                                div { style: "font-family: monospace; font-size: 11px; word-break: break-all;", "{node.icon}" }
                            }
                        }
                    }
                }
            } else {
                div { style: "color: #999; font-style: italic; text-align: center; margin-top: 20px;", "Select a node to edit properties" }
            }
        }
    }
}
