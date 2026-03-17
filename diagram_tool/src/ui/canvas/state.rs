use super::root_handlers::pointer::types::PointerDeps;
use super::root_handlers::{
    use_keyboard_handler, use_middle_pan_handler, use_pointer_handler, use_raf_handler,
    use_resize_handler, use_touch_handler,
};
use crate::app::DraggedIconPayload;
use crate::history::History;
use crate::ui::canvas::document_ops::{ordered_node_ids, WheelSample};
use crate::ui::editor::ToolMode;
use crate::ui::toast::use_toast;
use canvas_domain::interaction_reducer::InteractionMode;
use diagram_models::document::{ArrowType, DiagramDocument, EdgeId, EdgeStyle, NodeId};
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, PartialEq)]
pub struct CanvasState {
    pub doc_signal: Signal<DiagramDocument>,
    pub dragging_icon: Signal<Option<DraggedIconPayload>>,
    pub history_signal: Signal<History>,
    pub tool_signal: Signal<ToolMode>,
    pub edge_style_default: Signal<EdgeStyle>,
    pub arrow_type_default: Signal<ArrowType>,
    pub interaction_mode: Signal<InteractionMode>,
    pub space_pressed: Signal<bool>,
    pub shift_pressed: Signal<bool>,
    pub ctrl_pressed: Signal<bool>,
    pub meta_pressed: Signal<bool>,
    pub drag_over: Signal<bool>,
    pub hovered_node: Signal<Option<NodeId>>,
    pub editing_node: Signal<Option<NodeId>>,
    pub editing_edge: Signal<Option<EdgeId>>,
    pub edit_value: Signal<String>,
    pub nudge_batch_active: Signal<bool>,
    pub space_pan_active: Signal<bool>,
    pub viewport_size: Signal<(f64, f64)>,
    pub pending_pointer_sample: Signal<Option<(f64, f64)>>,
    pub pending_wheel_sample: Signal<Option<WheelSample>>,
    pub multi_touch_active: Signal<bool>,
    pub captured_pointer: Signal<Option<u32>>,
    pub active_pointers: Signal<HashSet<u32>>,
    pub canvas_origin: Signal<(f64, f64)>,
    pub ordered_node_cache: Memo<Vec<NodeId>>,
    pub db_tx: Option<Coroutine<diagram_models::envelope::EventEnvelope>>,
}

pub fn use_canvas_state() -> CanvasState {
    let doc_signal = use_context::<Signal<DiagramDocument>>();
    let dragging_icon = use_context::<Signal<Option<DraggedIconPayload>>>();
    let history_signal = use_context::<Signal<History>>();
    let tool_signal = use_context::<Signal<ToolMode>>();
    let edge_style_default = use_context::<Signal<EdgeStyle>>();
    let arrow_type_default = use_context::<Signal<ArrowType>>();
    let toast = use_toast();

    let interaction_mode = use_signal(|| InteractionMode::Select);
    let space_pressed = use_signal(|| false);
    let shift_pressed = use_signal(|| false);
    let ctrl_pressed = use_signal(|| false);
    let meta_pressed = use_signal(|| false);
    let drag_over = use_signal(|| false);
    let hovered_node = use_signal(|| Option::<NodeId>::None);
    let editing_node = use_signal(|| Option::<NodeId>::None);
    let editing_edge = use_signal(|| Option::<EdgeId>::None);
    let edit_value = use_signal(String::new);
    let nudge_batch_active = use_signal(|| false);
    let space_pan_active = use_signal(|| false);
    let viewport_size = use_context::<Signal<(f64, f64)>>();
    let pending_pointer_sample = use_signal(|| Option::<(f64, f64)>::None);
    let pending_wheel_sample = use_signal(|| Option::<WheelSample>::None);
    let multi_touch_active = use_signal(|| false);
    let captured_pointer = use_signal(|| Option::<u32>::None);
    let active_pointers = use_signal(HashSet::<u32>::new);
    let canvas_origin = use_signal(|| (0.0_f64, 0.0_f64));
    let ordered_node_cache = use_memo(move || {
        let doc = doc_signal.read();
        ordered_node_ids(&doc)
    });
    let db_tx = use_context::<Option<Coroutine<diagram_models::envelope::EventEnvelope>>>();

    use_keyboard_handler(
        doc_signal,
        history_signal,
        interaction_mode,
        tool_signal,
        space_pressed,
        shift_pressed,
        ctrl_pressed,
        meta_pressed,
        nudge_batch_active,
        space_pan_active,
        editing_node,
        editing_edge,
        edit_value,
        viewport_size,
        db_tx.clone(),
    );

    use_touch_handler(
        multi_touch_active,
        pending_pointer_sample,
        pending_wheel_sample,
        space_pan_active,
        interaction_mode,
    );

    use_middle_pan_handler();

    use_raf_handler(
        doc_signal,
        history_signal,
        interaction_mode,
        pending_pointer_sample,
        pending_wheel_sample,
        db_tx.clone(),
    );

    use_resize_handler(canvas_origin, viewport_size);

    let mut current_deps = PointerDeps {
        doc_signal: doc_signal.clone(),
        history_signal: history_signal.clone(),
        tool_signal: tool_signal.clone(),
        interaction_mode: interaction_mode.clone(),
        edge_style_default: edge_style_default.clone(),
        arrow_type_default: arrow_type_default.clone(),
        editing_node: editing_node.clone(),
        editing_edge: editing_edge.clone(),
        edit_value: edit_value.clone(),
        space_pressed: space_pressed.clone(),
        shift_pressed: shift_pressed.clone(),
        ctrl_pressed: ctrl_pressed.clone(),
        meta_pressed: meta_pressed.clone(),
        space_pan_active: space_pan_active.clone(),
        multi_touch_active: multi_touch_active.clone(),
        pending_pointer_sample: pending_pointer_sample.clone(),
        captured_pointer: captured_pointer.clone(),
        active_pointers: active_pointers.clone(),
        canvas_origin: canvas_origin.clone(),
        db_tx: db_tx.clone(),
        toast: toast.clone(),
    };
    use_pointer_handler(current_deps);

    CanvasState {
        doc_signal,
        dragging_icon,
        history_signal,
        tool_signal,
        edge_style_default,
        arrow_type_default,
        interaction_mode,
        space_pressed,
        shift_pressed,
        ctrl_pressed,
        meta_pressed,
        drag_over,
        hovered_node,
        editing_node,
        editing_edge,
        edit_value,
        nudge_batch_active,
        space_pan_active,
        viewport_size,
        pending_pointer_sample,
        pending_wheel_sample,
        multi_touch_active,
        captured_pointer,
        active_pointers,
        canvas_origin,
        ordered_node_cache,
        db_tx,
    }
}
