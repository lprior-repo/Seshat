use crate::ui::theme::GRID_DOT;
use diagram_models::document::DiagramDocument;
use dioxus::prelude::*;

#[component]
pub fn GridLayer(
    doc_signal: Signal<DiagramDocument>,
    viewport_size: Signal<(f64, f64)>,
) -> Element {
    let doc = doc_signal.read();
    let s = &doc.editor_state;
    let (vw, vh) = *viewport_size.read();
    let pattern_step = (s.grid_size.0.max(8.0) * s.zoom.0).max(4.0);
    let pattern_x = (-s.camera_x.0 * s.zoom.0).rem_euclid(pattern_step);
    let pattern_y = (-s.camera_y.0 * s.zoom.0).rem_euclid(pattern_step);
    let dot_r = if s.zoom.0 >= 0.75 { 1.0 } else { 0.8 };

    if s.show_grid && s.zoom.0 >= 0.3 {
        rsx! {
            defs {
                pattern {
                    id: "canvas-grid-dot-pattern",
                    pattern_units: "userSpaceOnUse",
                    x: "{pattern_x}",
                    y: "{pattern_y}",
                    width: "{pattern_step}",
                    height: "{pattern_step}",
                    circle {
                        cx: "{pattern_step / 2.0}",
                        cy: "{pattern_step / 2.0}",
                        r: "{dot_r}",
                        fill: "{GRID_DOT}",
                    }
                }
            }
            rect {
                x: "0",
                y: "0",
                width: "{vw.max(1.0)}",
                height: "{vh.max(1.0)}",
                fill: "url(#canvas-grid-dot-pattern)",
            }
        }
    } else {
        rsx! {}
    }
}
