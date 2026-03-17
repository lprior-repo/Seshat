use dioxus::prelude::*;
use diagram_models::document::DiagramDocument;
use crate::history::History;
use canvas_domain::interaction_reducer::InteractionMode;
use std::collections::HashSet;
use crate::ui::canvas::document_ops::{flush_pending_pointer_update, flush_pending_wheel_update, WheelSample};

pub fn use_root_effects(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    mut interaction_mode: Signal<InteractionMode>,
    mut space_pressed: Signal<bool>,
    mut shift_pressed: Signal<bool>,
    mut ctrl_pressed: Signal<bool>,
    mut meta_pressed: Signal<bool>,
    mut space_pan_active: Signal<bool>,
    mut multi_touch_active: Signal<bool>,
    mut captured_pointer: Signal<Option<u32>>,
    mut active_pointers: Signal<HashSet<u32>>,
    mut pending_pointer_sample: Signal<Option<(f64, f64)>>,
    mut pending_wheel_sample: Signal<Option<WheelSample>>,
    db_tx: Option<Coroutine<diagram_models::envelope::EventEnvelope>>,
) {

}
