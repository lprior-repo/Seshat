#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use dioxus::prelude::*;

use crate::ui::theme::{SUBGRAPH_PREVIEW_FILL, SUBGRAPH_PREVIEW_STROKE};
use canvas_domain::interaction_reducer::InteractionMode;
use diagram_models::document::DiagramDocument;

pub fn subgraph_preview_overlay(
    mode: &InteractionMode,
    doc: &DiagramDocument,
    to_screen_coords: impl Fn(
        canvas_domain::CanvasCoord,
        canvas_domain::CanvasCoord,
        f64,
    ) -> canvas_domain::ScreenCoord,
) -> Element {
    if let InteractionMode::DrawingSubgraph { start, current } = mode {
        let editor = &doc.editor_state;
        let min_x = start.0.min(current.0);
        let min_y = start.1.min(current.1);
        let width = (start.0 - current.0).abs();
        let height = (start.1 - current.1).abs();
        let canvas_domain::ScreenCoord(screen_x, screen_y) = to_screen_coords(
            canvas_domain::CanvasCoord(min_x, min_y),
            canvas_domain::CanvasCoord(editor.camera_x.0, editor.camera_y.0),
            editor.zoom.0,
        );
        rsx! {
            rect {
                x: "{screen_x}", y: "{screen_y}", width: "{width * editor.zoom.0}", height: "{height * editor.zoom.0}",
                fill: "{SUBGRAPH_PREVIEW_FILL}", stroke: "{SUBGRAPH_PREVIEW_STROKE}", stroke_width: "1.2", stroke_dasharray: "6,3"
            }
        }
    } else {
        rsx! {}
    }
}
