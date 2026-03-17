#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::ui::properties_helpers::{
    arrow_type_str, edge_style_str, parse_arrow_type, parse_edge_style,
};
use crate::ui::theme::{BG_BASE, BORDER, TEXT_DIM, TEXT_MAIN, TEXT_MUTED};
use diagram_models::document::{ArrowType, EdgeStyle};
use dioxus::prelude::*;

#[component]
pub fn DefaultPanel() -> Element {
    let mut edge_style_default = use_context::<Signal<EdgeStyle>>();
    let mut arrow_type_default = use_context::<Signal<ArrowType>>();

    let edge_default_value = {
        let v = *edge_style_default.read();
        edge_style_str(v)
    };
    let arrow_default_value = {
        let v = *arrow_type_default.read();
        arrow_type_str(v)
    };

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 10px;",
            div { style: "color: {TEXT_DIM}; font-style: italic;", "Select a node or edge to view its properties" }

            div {
                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Default Edge Style" }
                select {
                    style: "width: 100%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                    value: "{edge_default_value}",
                    onchange: move |evt| edge_style_default.set(parse_edge_style(&evt.value())),
                    option { value: "solid", "Solid" }
                    option { value: "dashed", "Dashed" }
                    option { value: "dotted", "Dotted" }
                }
            }

            div {
                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Default Arrow Type" }
                select {
                    style: "width: 100%; padding: 6px 8px; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                    value: "{arrow_default_value}",
                    onchange: move |evt| arrow_type_default.set(parse_arrow_type(&evt.value())),
                    option { value: "default", "Default" }
                    option { value: "straight", "Straight" }
                    option { value: "curved", "Curved" }
                    option { value: "step", "Step" }
                    option { value: "sharp", "Sharp" }
                }
            }
        }
    }
}
