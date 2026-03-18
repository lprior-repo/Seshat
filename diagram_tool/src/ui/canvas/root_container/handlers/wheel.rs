use crate::ui::canvas::document_ops::{sync_canvas_origin, WheelSample};
use crate::ui::canvas::state::CanvasState;
use dioxus::html::geometry::WheelDelta;
use dioxus::prelude::*;

pub fn handle_wheel(state: CanvasState, evt: Event<dioxus::prelude::WheelData>) {
    let mut pending_wheel_sample = state.pending_wheel_sample;
    let multi_touch_active = state.multi_touch_active;
    let ctrl_pressed = state.ctrl_pressed;
    let meta_pressed = state.meta_pressed;
    let shift_pressed = state.shift_pressed;
    let canvas_origin = state.canvas_origin;

    if *multi_touch_active.read() {
        return;
    }
    evt.prevent_default();
    let (dx, dy, discrete_wheel) = match evt.data.delta() {
        WheelDelta::Pixels(v) => (v.x, v.y, v.y.abs() >= 50.0),
        WheelDelta::Lines(v) => (v.x, v.y, false),
        WheelDelta::Pages(v) => (v.x, v.y, false),
    };
    let coords = evt.data.coordinates().client();
    let origin = sync_canvas_origin().unwrap_or_else(|| *canvas_origin.read());
    let local_x = coords.x - origin.0;
    let local_y = coords.y - origin.1;
    pending_wheel_sample.set(Some(WheelSample {
        client_x: local_x,
        client_y: local_y,
        dx,
        dy,
        zoom_gesture: *ctrl_pressed.read() || *meta_pressed.read(),
        shift_pan: *shift_pressed.read(),
        discrete_wheel,
    }));
}
