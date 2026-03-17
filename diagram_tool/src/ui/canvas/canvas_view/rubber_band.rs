#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use dioxus::prelude::*;

use crate::ui::theme::{SELECTION_RECT_FILL, SELECTION_RECT_STROKE};
use canvas_domain::interaction_reducer::InteractionMode;
use diagram_models::document::DiagramDocument;

pub(crate) fn rubber_band_overlay(
    mode: &InteractionMode,
    doc: &DiagramDocument,
    to_screen_coords: impl Fn(
        canvas_domain::CanvasCoord,
        canvas_domain::CanvasCoord,
        f64,
    ) -> canvas_domain::ScreenCoord,
) -> Element {
    if let InteractionMode::RubberBand { start, current } = mode {
        let s = &doc.editor_state;
        let canvas_domain::ScreenCoord(rx, ry) = to_screen_coords(
            canvas_domain::CanvasCoord(start.0.min(current.0), start.1.min(current.1)),
            canvas_domain::CanvasCoord(s.camera_x.0, s.camera_y.0),
            s.zoom.0,
        );
        let rw = (start.0 - current.0).abs() * s.zoom.0;
        let rh = (start.1 - current.1).abs() * s.zoom.0;
        rsx! {
            rect {
                x: "{rx}", y: "{ry}", width: "{rw}", height: "{rh}",
                fill: "{SELECTION_RECT_FILL}", stroke: "{SELECTION_RECT_STROKE}", stroke_width: "1", stroke_dasharray: "4,2"
            }
        }
    } else {
        rsx! {}
    }
}
