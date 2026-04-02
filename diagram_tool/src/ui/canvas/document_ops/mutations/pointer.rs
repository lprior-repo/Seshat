use crate::history::History;
use canvas_domain::interaction_reducer::InteractionMode;
use diagram_models::document::DiagramDocument;
use dioxus::prelude::*;

use super::pointer_drag::handle_dragging;
use super::pointer_pan::handle_panning;
use super::pointer_resize::handle_resizing;

fn bump_geometry_render_tick(geometry_render_tick: &mut Signal<u64>) {
    geometry_render_tick.with_mut(|tick| *tick += 1);
}

pub fn flush_pending_pointer_update(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    mut interaction_mode: Signal<InteractionMode>,
    mut pending_pointer_sample: Signal<Option<(f64, f64)>>,
    mut geometry_render_tick: Signal<u64>,
    _db_tx: Option<dioxus::prelude::Coroutine<diagram_models::envelope::EventEnvelope>>,
) {
    let pending = pending_pointer_sample.read().as_ref().copied();
    let Some((client_x, client_y)) = pending else {
        return;
    };
    pending_pointer_sample.set(None);

    interaction_mode.with_mut(|mode| match mode {
        InteractionMode::DraggingSelection {
            anchor_canvas,
            anchor_client,
            original_positions,
            did_move,
        } => {
            if handle_dragging(
                &mut doc_signal,
                &mut history_signal,
                client_x,
                client_y,
                anchor_canvas,
                anchor_client,
                original_positions,
                did_move,
            ) {
                bump_geometry_render_tick(&mut geometry_render_tick);
            }
        }
        InteractionMode::ResizingSelection {
            handle,
            original_bounds,
            originals,
            anchor,
            did_resize,
            aspect_ratio,
        } => {
            if handle_resizing(
                &mut doc_signal,
                &mut history_signal,
                client_x,
                client_y,
                handle,
                original_bounds,
                originals,
                anchor,
                did_resize,
                aspect_ratio,
            ) {
                bump_geometry_render_tick(&mut geometry_render_tick);
            }
        }
        InteractionMode::Panning { last_pos } => {
            if handle_panning(&mut doc_signal, client_x, client_y, last_pos) {
                bump_geometry_render_tick(&mut geometry_render_tick);
            }
        }
        InteractionMode::DraggingBendPoint {
            edge_id,
            bend_index,
        } => {
            let doc = doc_signal.read().clone();
            let raw = canvas_domain::perf::to_canvas_coords(
                canvas_domain::ScreenCoord(client_x, client_y),
                canvas_domain::CanvasCoord(
                    doc.editor_state.camera_x.0,
                    doc.editor_state.camera_y.0,
                ),
                doc.editor_state.zoom.0,
            );
            let snapped = crate::ui::grid::snap_point(
                (raw.0, raw.1),
                doc.editor_state.snap_to_grid,
                doc.editor_state.grid_size,
            );

            if let Some(snapped_point) =
                diagram_models::geometry::FinitePoint::new(snapped.0, snapped.1)
            {
                if let Ok(new_doc) =
                    diagram_models::document::routing_interactions::handle_bend_point_drag(
                        &doc,
                        edge_id,
                        diagram_models::document::routing_interactions::BendPointIndex(*bend_index),
                        snapped_point,
                    )
                {
                    doc_signal.set(new_doc);
                    bump_geometry_render_tick(&mut geometry_render_tick);
                }
            }
        }
        InteractionMode::Select
        | InteractionMode::RubberBand { .. }
        | InteractionMode::DrawingEdge { .. }
        | InteractionMode::DrawingSubgraph { .. } => {}
    });
}
