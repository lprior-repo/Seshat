pub mod down;
pub mod move_event;
pub mod types;
pub mod up;

use dioxus::prelude::*;
use types::PointerDeps;

pub fn use_pointer_handler(deps: PointerDeps) {
    let mut canvas_origin = deps.canvas_origin;
    let mut doc_signal = deps.doc_signal;
    let mut history_signal = deps.history_signal;
    let mut tool_signal = deps.tool_signal;
    let mut interaction_mode = deps.interaction_mode;
    let edge_style_default = deps.edge_style_default;
    let arrow_type_default = deps.arrow_type_default;
    let mut editing_node = deps.editing_node;
    let mut editing_edge = deps.editing_edge;
    let mut edit_value = deps.edit_value;
    let space_pressed = deps.space_pressed;
    let shift_pressed = deps.shift_pressed;
    let ctrl_pressed = deps.ctrl_pressed;
    let meta_pressed = deps.meta_pressed;
    let mut space_pan_active = deps.space_pan_active;
    let multi_touch_active = deps.multi_touch_active;
    let mut pending_pointer_sample = deps.pending_pointer_sample;
    let mut captured_pointer = deps.captured_pointer;
    let mut active_pointers = deps.active_pointers;
    let db_tx = deps.db_tx.clone();
    let toast = deps.toast.clone();

    use_effect(move || {
        let mut eval = document::eval(
            r"
                if (window.__seshat_canvas_pointer_global_cleanup) {
                    window.__seshat_canvas_pointer_global_cleanup();
                }

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
                    dioxus.send({ type: 'pointermove', x: event.clientX, y: event.clientY, originX: origin.x, originY: origin.y, pointerId: event.pointerId });
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

                const onMouseDownCapture = (event) => {
                    if (window.__seshat_pointerdown_handled) {
                        window.__seshat_pointerdown_handled = false;
                    }
                };

                const onPointerUpReset = (event) => {
                    window.__seshat_pointerdown_handled = false;
                };

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
            ",
        );

        spawn(async move {
            while let Ok(json) = eval.recv::<serde_json::Value>().await {
                let event_type = json["type"].as_str().map_or("", |s| s);

                if event_type == "resize" {
                    canvas_origin.set((
                        json["left"].as_f64().map_or(0.0, |v| v),
                        json["top"].as_f64().map_or(0.0, |v| v),
                    ));
                    continue;
                }

                let client_x = json["x"].as_f64().map_or(0.0, |v| v);
                let client_y = json["y"].as_f64().map_or(0.0, |v| v);
                let origin_x = json["originX"].as_f64().map_or(0.0, |v| v);
                let origin_y = json["originY"].as_f64().map_or(0.0, |v| v);
                let local_x = client_x - origin_x;
                let local_y = client_y - origin_y;

                let mut current_deps = PointerDeps {
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
                    db_tx: db_tx.clone(),
                    toast: toast.clone(),
                };

                if event_type == "pointerdown" {
                    down::handle_pointer_down(
                        &mut current_deps,
                        &json,
                        local_x,
                        local_y,
                        origin_x,
                        origin_y,
                    );
                    continue;
                }

                if event_type == "pointermove" {
                    move_event::handle_pointer_move(&mut current_deps, &json, local_x, local_y);
                    continue;
                }

                if event_type == "pointerup" {
                    up::handle_pointer_up(&mut current_deps, &json, local_x, local_y);
                }
            }
        });
    });
}
