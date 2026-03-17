use crate::history::History;
use crate::ui::editor::ToolMode;
use crate::ui::toast::ToastApi;
use canvas_domain::interaction_reducer::InteractionMode;
use diagram_models::document::{ArrowType, DiagramDocument, EdgeId, EdgeStyle, NodeId};
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone)]
pub struct PointerDeps {
    pub doc_signal: Signal<DiagramDocument>,
    pub history_signal: Signal<History>,
    pub tool_signal: Signal<ToolMode>,
    pub interaction_mode: Signal<InteractionMode>,
    pub edge_style_default: Signal<EdgeStyle>,
    pub arrow_type_default: Signal<ArrowType>,
    pub editing_node: Signal<Option<NodeId>>,
    pub editing_edge: Signal<Option<EdgeId>>,
    pub edit_value: Signal<String>,
    pub space_pressed: Signal<bool>,
    pub shift_pressed: Signal<bool>,
    pub ctrl_pressed: Signal<bool>,
    pub meta_pressed: Signal<bool>,
    pub space_pan_active: Signal<bool>,
    pub multi_touch_active: Signal<bool>,
    pub pending_pointer_sample: Signal<Option<(f64, f64)>>,
    pub captured_pointer: Signal<Option<u32>>,
    pub active_pointers: Signal<HashSet<u32>>,
    pub canvas_origin: Signal<(f64, f64)>,
    pub db_tx: Option<Coroutine<diagram_models::envelope::EventEnvelope>>,
    pub toast: ToastApi,
}
