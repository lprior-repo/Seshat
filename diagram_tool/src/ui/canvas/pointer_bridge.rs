use canvas_domain::interaction_reducer::InteractionMode;


#[path = "pointer_bridge/down.rs"]
mod down;
#[path = "pointer_bridge/state.rs"]
mod state;
#[path = "pointer_bridge/up.rs"]
mod up;

use dioxus::prelude::*;

use state::{build_pointer_bridge_deps, handle_pointer_move};

const POINTER_BRIDGE_JS: &str = r"
    if (window.__seshat_canvas_pointer_global_cleanup) { window.__seshat_canvas_pointer_global_cleanup(); }
    window.__seshat_get_canvas_origin = () => {
        const target = document.querySelector('.canvas-container');
        if (!target) return { x: 0, y: 0 };
        const rect = target.getBoundingClientRect();
        return { x: rect.left, y: rect.top };
    };
    const getCanvasOrigin = () => {
        const target = document.querySelector('.canvas-container');
        if (!target) return { x: 0, y: 0 };
        const rect = target.getBoundingClientRect();
        return { x: rect.left, y: rect.top };
    };
                const onPointerMove = (event) => {
                    const origin = getCanvasOrigin();
                    console.log("[POINTER_BRIDGE] pointermove id=" + event.pointerId + " captured=" + window.__seshat_captured_id);
                    dioxus.send({ type: 'pointermove', x: event.clientX, y: event.clientY, originX: origin.x, originY: origin.y, pointerId: event.pointerId });
                };

                const onPointerUp = (event) => {
                    const origin = getCanvasOrigin();
                    console.log("[POINTER_BRIDGE] pointerup id=" + event.pointerId);
                    window.__seshat_captured_id = undefined;
                    dioxus.send({ type: 'pointerup', x: event.clientX, y: event.clientY, originX: origin.x, originY: origin.y, pointerId: event.pointerId });
                };

                const onPointerDown = (event) => {
                    const origin = getCanvasOrigin();
                    window.__seshat_current_origin = { x: origin.x, y: origin.y };
                    window.__seshat_pointerdown_handled = true;
                    window.__seshat_captured_id = event.pointerId;
                    console.log("[POINTER_BRIDGE] pointerdown id=" + event.pointerId);
                    dioxus.send({
                        type: 'pointerdown',
                        x: event.clientX,
                        y: event.clientY,
                        originX: origin.x,
                        originY: origin.y,
                        button: event.button.toString(),
                        pointerId: event.pointerId,
                        tool: window.__seshat_current_tool || 'select',
                        shiftKey: event.shiftKey,
                        ctrlKey: event.ctrlKey,
                        metaKey: event.metaKey,
                    });
                };
    const onPointerUp = (event) => {
        const origin = getCanvasOrigin();
        dioxus.send({ type: 'pointerup', x: event.clientX, y: event.clientY, originX: origin.x, originY: origin.y, pointerId: event.pointerId });
    };
    const onPointerDown = (event) => {
        const origin = getCanvasOrigin();
        window.__seshat_current_origin = { x: origin.x, y: origin.y };
        window.__seshat_pointerdown_handled = true;
        dioxus.send({
            type: 'pointerdown', x: event.clientX, y: event.clientY, originX: origin.x, originY: origin.y,
            button: event.button.toString(), pointerId: event.pointerId, tool: window.__seshat_current_tool || 'select',
            shiftKey: event.shiftKey, ctrlKey: event.ctrlKey, metaKey: event.metaKey,
        });
    };
    const onMouseDownCapture = () => {
        if (window.__seshat_pointerdown_handled) {
            window.__seshat_pointerdown_handled = false;
        }
    };
    const onPointerUpReset = () => { window.__seshat_pointerdown_handled = false; };
    window.addEventListener('pointermove', onPointerMove, { passive: true });
    window.addEventListener('pointerup', onPointerUp, { passive: true });
    window.addEventListener('pointerup', onPointerUpReset, { passive: true });
    window.addEventListener('pointerdown', onPointerDown, { passive: true });
    window.addEventListener('mousedown', onMouseDownCapture, { capture: true, passive: false });
    window.__seshat_canvas_pointer_global_cleanup = () => {
        window.removeEventListener('pointermove', onPointerMove);
        window.removeEventListener('pointerup', onPointerUp);
        window.removeEventListener('pointerup', onPointerUpReset);
        window.removeEventListener('pointerdown', onPointerDown);
        window.removeEventListener('mousedown', onMouseDownCapture, true);
    };
";

#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
pub(super) fn use_canvas_pointer_bridge(
    doc_signal: Signal<diagram_models::document::DiagramDocument>,
    history_signal: Signal<crate::history::History>,
    tool_signal: Signal<crate::ui::editor::ToolMode>,
    interaction_mode: Signal<canvas_domain::interaction_reducer::InteractionMode>,
    edge_style_default: Signal<diagram_models::document::EdgeStyle>,
    arrow_type_default: Signal<diagram_models::document::ArrowType>,
    editing_node: Signal<Option<diagram_models::document::NodeId>>,
    editing_edge: Signal<Option<diagram_models::document::EdgeId>>,
    edit_value: Signal<String>,
    space_pressed: Signal<bool>,
    shift_pressed: Signal<bool>,
    ctrl_pressed: Signal<bool>,
    meta_pressed: Signal<bool>,
    space_pan_active: Signal<bool>,
    multi_touch_active: Signal<bool>,
    pending_pointer_sample: Signal<Option<(f64, f64)>>,
    captured_pointer: Signal<Option<u32>>,
    active_pointers: Signal<im::HashSet<u32>>,
    canvas_origin: Signal<(f64, f64)>,
    db_tx: Option<Coroutine<diagram_models::envelope::EventEnvelope>>,
    toast: crate::ui::toast::ToastApi,
) {
    let deps = build_pointer_bridge_deps(
        doc_signal,
        history_signal,
        tool_signal,
        interaction_mode,
        edge_style_default,
        arrow_type_default,
        editing_node,
        editing_edge,
        edit_value,
        space_pressed,
        shift_pressed,
        ctrl_pressed,
        meta_pressed,
        space_pan_active,
        multi_touch_active,
        pending_pointer_sample,
        captured_pointer,
        active_pointers,
        canvas_origin,
        db_tx,
        toast,
    );

    use_effect(move || {
        let mut eval = document::eval(POINTER_BRIDGE_JS);
        let mut deps = deps.clone();

        spawn(async move {
            while let Ok(json) = eval.recv::<serde_json::Value>().await {
                let event_type = json["type"].as_str().map_or("", |s| s);

                if event_type == "resize" {
                    deps.canvas_origin.set((
                        json["left"].as_f64().map_or(0.0, |v| v),
                        json["top"].as_f64().map_or(0.0, |v| v),
                    ));
                    continue;
                }

                let local_x = json["x"].as_f64().map_or(0.0, |v| v)
                    - json["originX"].as_f64().map_or(0.0, |v| v);
                let local_y = json["y"].as_f64().map_or(0.0, |v| v)
                    - json["originY"].as_f64().map_or(0.0, |v| v);

                match event_type {
                    "pointerdown" => down::handle_pointer_down(&mut deps, &json, local_x, local_y),
                    "pointermove" => handle_pointer_move(&mut deps, &json, local_x, local_y),
                    "pointerup" => up::handle_pointer_up(&mut deps, &json, local_x, local_y),
                    _ => {}
                }
            }
        });
    });
}
