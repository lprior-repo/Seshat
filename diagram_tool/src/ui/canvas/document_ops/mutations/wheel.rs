use canvas_domain::perf::{wheel_update, WheelInput};
use diagram_models::document::DiagramDocument;
use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WheelSample {
    pub client_x: f64,
    pub client_y: f64,
    pub dx: f64,
    pub dy: f64,
    pub zoom_gesture: bool,
    pub shift_pan: bool,
    pub discrete_wheel: bool,
}

pub fn flush_pending_wheel_update(
    mut doc_signal: Signal<DiagramDocument>,
    mut pending_wheel_sample: Signal<Option<WheelSample>>,
) {
    let pending = pending_wheel_sample.read().as_ref().copied();
    let Some(sample) = pending else {
        return;
    };
    pending_wheel_sample.set(None);

    let current = doc_signal.read().editor_state.clone();
    if let Some((next_x, next_y, next_zoom)) = wheel_update(WheelInput {
        camera_x: current.camera_x,
        camera_y: current.camera_y,
        zoom: current.zoom,
        client_x: sample.client_x,
        client_y: sample.client_y,
        dx: sample.dx,
        dy: sample.dy,
        zoom_gesture: sample.zoom_gesture,
        shift_pan: sample.shift_pan,
        discrete_wheel: sample.discrete_wheel,
    }) {
        doc_signal.with_mut(|doc| {
            doc.editor_state.camera_x = next_x;
            doc.editor_state.camera_y = next_y;
            doc.editor_state.zoom = next_zoom;
        });
    }
}
