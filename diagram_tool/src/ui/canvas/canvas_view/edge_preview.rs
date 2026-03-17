#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use dioxus::prelude::*;

use crate::ui::theme::ACCENT;
use canvas_domain::interaction_reducer::InteractionMode;
use diagram_models::document::DiagramDocument;

use super::geometry::rect_ray_intersection;

pub(crate) fn edge_preview_overlay(
    mode: &InteractionMode,
    doc: &DiagramDocument,
    to_screen_coords: impl Fn(
        canvas_domain::CanvasCoord,
        canvas_domain::CanvasCoord,
        f64,
    ) -> canvas_domain::ScreenCoord,
) -> Element {
    let s = &doc.editor_state;
    if let InteractionMode::DrawingEdge {
        from_node,
        current_pos,
    } = mode
    {
        doc.document.nodes.get(from_node).map_or_else(
            || rsx! {},
            |src| {
                let scx = src.x.0 + src.width.0 / 2.0;
                let scy = src.y.0 + src.height.0 / 2.0;
                let (edge_x, edge_y) = rect_ray_intersection(scx, scy, src.width.0, src.height.0, current_pos.0, current_pos.1);

                let canvas_domain::ScreenCoord(sx, sy) = to_screen_coords(canvas_domain::CanvasCoord(edge_x, edge_y), canvas_domain::CanvasCoord(s.camera_x.0, s.camera_y.0), s.zoom.0);
                let canvas_domain::ScreenCoord(tx, ty) = to_screen_coords(canvas_domain::CanvasCoord(current_pos.0, current_pos.1), canvas_domain::CanvasCoord(s.camera_x.0, s.camera_y.0), s.zoom.0);
                rsx! {
                    line {
                        x1: "{sx}", y1: "{sy}", x2: "{tx}", y2: "{ty}",
                        stroke: "{ACCENT}", stroke_width: "1.8", stroke_dasharray: "5,5", marker_end: "url(#arrow-pending)"
                    }
                }
            },
        )
    } else {
        rsx! {}
    }
}
