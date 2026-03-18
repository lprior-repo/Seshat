use crate::history::History;
use canvas_domain::interaction_reducer::InteractionMode;
use diagram_models::document::DiagramDocument;
use dioxus::prelude::*;

use super::pointer_drag::handle_dragging;
use super::pointer_pan::handle_panning;
use super::pointer_resize::handle_resizing;

pub fn flush_pending_pointer_update(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    mut interaction_mode: Signal<InteractionMode>,
    mut pending_pointer_sample: Signal<Option<(f64, f64)>>,
    db_tx: Option<dioxus::prelude::Coroutine<diagram_models::envelope::EventEnvelope>>,
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
            handle_dragging(
                &mut doc_signal,
                &mut history_signal,
                client_x,
                client_y,
                anchor_canvas,
                anchor_client,
                original_positions,
                did_move,
                &db_tx,
            );
        }
        InteractionMode::ResizingSelection {
            handle,
            original_bounds,
            originals,
            anchor,
            did_resize,
            aspect_ratio,
        } => {
            handle_resizing(
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
            );
        }
        InteractionMode::Panning { last_pos } => {
            handle_panning(&mut doc_signal, client_x, client_y, last_pos);
        }
        InteractionMode::Select
        | InteractionMode::RubberBand { .. }
        | InteractionMode::DrawingEdge { .. }
        | InteractionMode::DrawingSubgraph { .. } => {}
    });
}
