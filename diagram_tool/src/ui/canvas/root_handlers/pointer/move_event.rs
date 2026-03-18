use crate::ui::grid::snap_point;
use canvas_domain::interaction_reducer::InteractionMode;
use canvas_domain::perf::to_canvas_coords;
use dioxus::prelude::*;

use super::types::PointerDeps;

pub fn handle_pointer_move(
    deps: &mut PointerDeps,
    json: &serde_json::Value,
    local_x: f64,
    local_y: f64,
) {
    if *deps.multi_touch_active.read() {
        return;
    }

    let move_pointer_id = json["pointerId"].as_u64().map_or(0_u32, |v| v as u32);
    let captured_id = *deps.captured_pointer.read();

    if captured_id != Some(move_pointer_id) {
        return;
    }

    deps.interaction_mode.with_mut(|mode| match mode {
        InteractionMode::DrawingEdge { current_pos, .. } => {
            let doc = deps.doc_signal.read();
            let canvas_domain::CanvasCoord(px, py) = to_canvas_coords(
                canvas_domain::ScreenCoord(local_x, local_y),
                canvas_domain::CanvasCoord(
                    doc.editor_state.camera_x.0,
                    doc.editor_state.camera_y.0,
                ),
                doc.editor_state.zoom.0,
            );
            *current_pos = (px, py);
        }
        InteractionMode::RubberBand { current, .. }
        | InteractionMode::DrawingSubgraph { current, .. } => {
            let doc = deps.doc_signal.read();
            let raw = to_canvas_coords(
                canvas_domain::ScreenCoord(local_x, local_y),
                canvas_domain::CanvasCoord(
                    doc.editor_state.camera_x.0,
                    doc.editor_state.camera_y.0,
                ),
                doc.editor_state.zoom.0,
            );
            *current = snap_point(
                (raw.0, raw.1),
                doc.editor_state.snap_to_grid,
                doc.editor_state.grid_size,
            );
        }
        InteractionMode::DraggingSelection { .. }
        | InteractionMode::ResizingSelection { .. }
        | InteractionMode::Panning { .. } => {
            deps.pending_pointer_sample.set(Some((local_x, local_y)));
        }
        InteractionMode::Select => {}
    });
}
