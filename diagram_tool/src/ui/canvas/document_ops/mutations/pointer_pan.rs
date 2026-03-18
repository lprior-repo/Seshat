use diagram_models::document::{DiagramDocument, OrderedFloat};
use dioxus::prelude::*;

use super::super::queries::safe_zoom;

pub fn handle_panning(
    doc_signal: &mut Signal<DiagramDocument>,
    client_x: f64,
    client_y: f64,
    last_pos: &mut (f64, f64),
) {
    let dx = client_x - last_pos.0;
    let dy = client_y - last_pos.1;
    *last_pos = (client_x, client_y);
    if dx.abs() > f64::EPSILON || dy.abs() > f64::EPSILON {
        doc_signal.with_mut(|doc| {
            let zoom = safe_zoom(doc.editor_state.zoom.0);
            doc.editor_state.camera_x = OrderedFloat(doc.editor_state.camera_x.0 - (dx / zoom));
            doc.editor_state.camera_y = OrderedFloat(doc.editor_state.camera_y.0 - (dy / zoom));
        });
    }
}
