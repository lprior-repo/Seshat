#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::history::History;
use crate::ui::theme::{BG_BASE, BORDER, TEXT_MAIN, TEXT_MUTED};
use diagram_models::document::{DiagramDocument, Node, NodeId, OrderedFloat};
use dioxus::prelude::*;

use super::update::update_node_if_changed;

#[component]
#[allow(clippy::approx_constant, clippy::float_cmp)]
pub fn LayoutPropsPanel(id: NodeId, node: Node) -> Element {
    let mut doc_signal = use_context::<Signal<DiagramDocument>>();
    let mut history = use_context::<Signal<History>>();

    let id_x = id.clone();
    let id_y = id.clone();
    let id_w = id.clone();
    let id_h = id.clone();
    let id_font = id.clone();

    rsx! {
        div {
            label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Position" }
            div { style: "display: flex; gap: 5px;",
                input {
                    style: "width: 50%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                    r#type: "number",
                    value: "{node.x}",
                    onchange: move |evt| {
                        if let Ok(val) = evt.value().parse::<f64>() {
                            update_node_if_changed(
                                &mut doc_signal,
                                &mut history,
                                &id_x,
                                |n| n.x.0 != val,
                                |n| n.x = OrderedFloat(val),
                            );
                        }
                    }
                }
                input {
                    style: "width: 50%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                    r#type: "number",
                    value: "{node.y}",
                    onchange: move |evt| {
                        if let Ok(val) = evt.value().parse::<f64>() {
                            update_node_if_changed(
                                &mut doc_signal,
                                &mut history,
                                &id_y,
                                |n| n.y.0 != val,
                                |n| n.y = OrderedFloat(val),
                            );
                        }
                    }
                }
            }
        }

        div {
            label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Size" }
            div { style: "display: flex; gap: 5px;",
                input {
                    style: "width: 50%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                    r#type: "number",
                    value: "{node.width}",
                    onchange: move |evt| {
                        if let Ok(val) = evt.value().parse::<f64>() {
                            let clamped_val = val.max(24.0);
                            update_node_if_changed(
                                &mut doc_signal,
                                &mut history,
                                &id_w,
                                |n| n.width.0 != clamped_val,
                                |n| n.width = OrderedFloat(clamped_val),
                            );
                        }
                    }
                }
                input {
                    style: "width: 50%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                    r#type: "number",
                    value: "{node.height}",
                    onchange: move |evt| {
                        if let Ok(val) = evt.value().parse::<f64>() {
                            let clamped_val = val.max(24.0);
                            update_node_if_changed(
                                &mut doc_signal,
                                &mut history,
                                &id_h,
                                |n| n.height.0 != clamped_val,
                                |n| n.height = OrderedFloat(clamped_val),
                            );
                        }
                    }
                }
            }
        }

        div {
            label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Font Size" }
            input {
                style: "width: 100%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                r#type: "number",
                value: "{node.font_size.map_or(11.0, |v| v.0)}",
                onchange: move |evt| {
                    if let Ok(val) = evt.value().parse::<f64>() {
                        let clamped_val = val.clamp(8.0, 72.0);
                        update_node_if_changed(
                            &mut doc_signal,
                            &mut history,
                            &id_font,
                            |n| n.font_size.map_or(11.0, |fs| fs.0) != clamped_val,
                            |n| n.font_size = Some(OrderedFloat(clamped_val)),
                        );
                    }
                }
            }
        }
    }
}
