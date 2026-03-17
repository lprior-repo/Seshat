use dioxus::prelude::*;
use crate::ui::theme::{EDGE_DEFAULT, EDGE_SELECTED};

#[component]
pub fn MarkerDefs() -> Element {
    rsx! {
        defs {
            marker { id: "arrowhead", marker_width: "10", marker_height: "7", ref_x: "9", ref_y: "3.5", orient: "auto", polygon { points: "0 0, 10 3.5, 0 7", fill: "{EDGE_DEFAULT}" } }
            marker { id: "arrowhead-selected", marker_width: "10", marker_height: "7", ref_x: "9", ref_y: "3.5", orient: "auto", polygon { points: "0 0, 10 3.5, 0 7", fill: "{EDGE_SELECTED}" } }
            marker { id: "arrow-pending", marker_width: "10", marker_height: "7", ref_x: "9", ref_y: "3.5", orient: "auto", polygon { points: "0 0, 10 3.5, 0 7", fill: "{EDGE_SELECTED}", opacity: "0.5" } }
        }
    }
}
