use crate::ui::canvas::document_ops::sync_canvas_origin;
use crate::ui::canvas::state::CanvasState;
use crate::ui::grid::snap_point;
use canvas_domain::interaction_reducer::InteractionMode;
use canvas_domain::perf::to_canvas_coords;
use dioxus::prelude::*;

pub fn handle_mouse_move(state: CanvasState, evt: Event<dioxus::prelude::MouseData>) {
    let mut interaction_mode = state.interaction_mode;
    let doc_signal = state.doc_signal;
    let mut pending_pointer_sample = state.pending_pointer_sample;
    let canvas_origin = state.canvas_origin;

    let coords = evt.data.coordinates().client();
    let origin = sync_canvas_origin().unwrap_or_else(|| *canvas_origin.read());
    let local_x = coords.x - origin.0;
    let local_y = coords.y - origin.1;

    // Avoid dirtying the interaction_mode signal if we don't need to mutate it
    if *interaction_mode.read() == InteractionMode::Select {
        return;
    }

    interaction_mode.with_mut(|mode| match mode {
        InteractionMode::DrawingEdge { current_pos, .. } => {
            let doc = doc_signal.read();
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
            let doc = doc_signal.read();
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
        | InteractionMode::Panning { .. }
        | InteractionMode::DraggingBendPoint { .. } => {
            pending_pointer_sample.set(Some((local_x, local_y)));
        }
        InteractionMode::Select => {}
    });
}
