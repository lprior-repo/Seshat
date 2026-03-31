//! Canvas state management and React-like hooks

use crate::app::DraggedIconPayload;
use crate::history::History;
use crate::ui::canvas::document_ops::ordered_node_ids;
use crate::ui::canvas::root_handlers::{
    use_keyboard_handler, use_middle_pan_handler, use_raf_handler, use_resize_handler,
    use_touch_handler,
};
use crate::ui::editor::ToolMode;
use crate::ui::toast::use_toast;
use canvas_domain::interaction_reducer::InteractionMode;
use diagram_models::document::{ArrowType, DiagramDocument, EdgeStyle, NodeId};
use dioxus::prelude::*;
use im::HashSet as ImHashSet;
use std::collections::HashSet as StdHashSet;

use super::editor_fsm::{EditorError, EditorState};
use crate::ui::canvas::document_ops::WheelSample;

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
    pub editor_state: Signal<EditorState>,
    pub edit_value: Signal<String>,
    pub nudge_batch_active: Signal<bool>,
    pub space_pan_active: Signal<bool>,
    pub viewport_size: Signal<(f64, f64)>,
    pub pending_pointer_sample: Signal<Option<(f64, f64)>>,
    pub pending_wheel_sample: Signal<Option<WheelSample>>,
    pub multi_touch_active: Signal<bool>,
    pub captured_pointer: Signal<Option<u32>>,
    pub active_pointers: Signal<StdHashSet<u32>>,
    pub canvas_origin: Signal<(f64, f64)>,
    pub ordered_node_cache: Memo<Vec<NodeId>>,
    /// Lightweight trigger Memo that extracts only the camera/selection data.
    /// `NodeLayer` and `EdgeLayer` subscribe to this instead of the full document,
    /// avoiding re-renders when only unrelated document fields change.
    pub node_viewport_trigger: Memo<(f64, f64, f64, ImHashSet<String>)>,
    pub db_tx: Option<Coroutine<diagram_models::envelope::EventEnvelope>>,
}

pub fn use_canvas_state() -> CanvasState {
    let app_state = use_context::<crate::app::AppState>();
    let doc_signal = app_state.document;
    let dragging_icon = app_state.dragging_icon;
    let history_signal = app_state.history;
    let tool_signal = app_state.tool_mode;
    let edge_style_default = app_state.edge_style;
    let arrow_type_default = app_state.arrow_type;
    let _toast = use_toast();

    let interaction_mode = use_signal(|| InteractionMode::Select);
    let space_pressed = use_signal(|| false);
    let shift_pressed = use_signal(|| false);
    let ctrl_pressed = use_signal(|| false);
    let meta_pressed = use_signal(|| false);
    let drag_over = use_signal(|| false);
    let editor_state = use_signal(|| EditorState::Idle);
    let edit_value = use_signal(String::new);
    let nudge_batch_active = use_signal(|| false);
    let space_pan_active = use_signal(|| false);
    let viewport_size = app_state.viewport_size;
    let pending_pointer_sample = use_signal(|| Option::<(f64, f64)>::None);
    let pending_wheel_sample = use_signal(|| Option::<WheelSample>::None);
    let multi_touch_active = use_signal(|| false);
    let captured_pointer = use_signal(|| Option::<u32>::None);
    let active_pointers = use_signal(StdHashSet::<u32>::new);
    let canvas_origin = use_signal(|| (0.0_f64, 0.0_f64));
    let ordered_node_cache = use_memo(move || {
        let doc = doc_signal.read();
        ordered_node_ids(&doc)
    });
    // Lightweight Memo: only camera_x, camera_y, zoom, selected_items.
    // Components subscribe to THIS instead of doc_signal to avoid full-doc re-renders.
    let node_viewport_trigger = use_memo(move || {
        let doc = doc_signal.read();
        let es = &doc.editor_state;
        (
            es.camera_x.0,
            es.camera_y.0,
            es.zoom.0,
            es.selected_items.clone(),
        )
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
        editor_state,
        edit_value,
        viewport_size,
        db_tx,
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
        db_tx,
    );

    use_resize_handler(canvas_origin, viewport_size);

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
        editor_state,
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
        node_viewport_trigger,
        db_tx,
    }
}

/// Imperative shell: applies the calculated state transition to the canvas state.
/// This function has side effects (Signal mutations) and is the boundary between
/// the pure state machine and the reactive UI.
pub fn apply_transition(
    canvas_state: &mut CanvasState,
    next_state: EditorState,
) -> Result<(), EditorError> {
    use dioxus::prelude::{ReadableExt, WritableExt};

    if next_state == EditorState::Idle {
        canvas_state.edit_value.set(String::new());
    }

    canvas_state.editor_state.set(next_state.clone());

    if next_state == EditorState::Idle && !canvas_state.edit_value.read().is_empty() {
        return Err(EditorError::InconsistentState);
    }

    Ok(())
}
