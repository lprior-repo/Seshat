#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

use crate::ui::theme::{ACCENT, BG_BASE, SELECTION_BOUNDS_STROKE};
use canvas_domain::{
    interaction_reducer::{start_resize_interaction, InteractionMode, ResizeHandle},
    selection_geometry::{selected_node_ids, selection_bounds},
};
use diagram_models::document::DiagramDocument;

#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
pub fn selection_handles_overlay(
    doc: &DiagramDocument,
    interaction_mode: Signal<InteractionMode>,
    doc_signal: Signal<DiagramDocument>,
    canvas_origin: Signal<(f64, f64)>,
    to_screen_coords: impl Fn(
        canvas_domain::CanvasCoord,
        canvas_domain::CanvasCoord,
        f64,
    ) -> canvas_domain::ScreenCoord,
) -> Element {
    let selected_nodes = selected_node_ids(doc);
    let _selected_count = selected_nodes.len();
    let selection = selection_bounds(doc);
    if let Some((bx, by, bw, bh)) = selection {
        let s = &doc.editor_state;
        let canvas_domain::ScreenCoord(sx, sy) = to_screen_coords(
            canvas_domain::CanvasCoord(bx, by),
            canvas_domain::CanvasCoord(s.camera_x.0, s.camera_y.0),
            s.zoom.0,
        );
        let sw = bw * s.zoom.0;
        let sh = bh * s.zoom.0;
        let pad = 4.0;
        let box_w = f64::mul_add(2.0, pad, sw);
        let box_h = f64::mul_add(2.0, pad, sh);
        let hs = 7.0;

        let handles = [
            (
                ResizeHandle::Nw,
                sx - pad,
                sy - pad,
                "nwse-resize",
                "resize-handle-nw",
            ),
            (
                ResizeHandle::Ne,
                sx + sw + pad,
                sy - pad,
                "nesw-resize",
                "resize-handle-ne",
            ),
            (
                ResizeHandle::Se,
                sx + sw + pad,
                sy + sh + pad,
                "nwse-resize",
                "resize-handle-se",
            ),
            (
                ResizeHandle::Sw,
                sx - pad,
                sy + sh + pad,
                "nesw-resize",
                "resize-handle-sw",
            ),
        ];

        rsx! {
            div {
                "data-testid": "selection-bounds",
                class: "absolute pointer-events-none",
                style: "left: {sx - pad}px; top: {sy - pad}px; width: {box_w}px; height: {box_h}px; z-index: 15; border: {SELECTION_BOUNDS_STROKE};"
            }
            if !selected_nodes.is_empty() {
                for (handle, hx, hy, cursor, stable_test_id) in handles {
                    button {
                        key: "{hx}-{hy}",
                        "data-testid": "{stable_test_id}",
                        "data-handle": match handle {
                            ResizeHandle::Nw => "nw",
                            ResizeHandle::N => "n",
                            ResizeHandle::Ne => "ne",
                            ResizeHandle::E => "e",
                            ResizeHandle::Se => "se",
                            ResizeHandle::S => "s",
                            ResizeHandle::Sw => "sw",
                            ResizeHandle::W => "w",
                        },
                        class: "absolute rounded-[2px]",
                        style: "left: {hx - hs/2.0}px; top: {hy - hs/2.0}px; width: {hs}px; height: {hs}px; z-index: 16; border: 1px solid {BG_BASE}; background: {ACCENT}; cursor: {cursor};",
                        onmousedown: move |evt| {
                            if evt.data.trigger_button() != Some(MouseButton::Primary) {
                                return;
                            }
                            evt.stop_propagation();
                            let c = evt.data.coordinates().client();
                            let origin = *canvas_origin.read();
                            start_resize_interaction(
                                interaction_mode,
                                doc_signal,
                                handle,
                                c.x - origin.0,
                                c.y - origin.1,
                                false, // aspect_lock_enabled - TODO: connect to Shift key
                            );
                        },
                        div { class: "absolute inset-0 pointer-events-none opacity-0" }
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}
