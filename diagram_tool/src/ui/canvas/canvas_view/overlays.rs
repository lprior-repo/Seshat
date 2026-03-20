use canvas_domain::interaction_reducer::{start_resize_interaction, InteractionMode, ResizeHandle};
use canvas_domain::perf::to_screen_coords;
use canvas_domain::selection_geometry::{selected_node_ids, selection_bounds};
use diagram_models::document::DiagramDocument;

use dioxus::{html::input_data::MouseButton, prelude::*};

use super::rect_ray_intersection;
use crate::ui::theme::{
    ACCENT, BG_BASE, SELECTION_BOUNDS_STROKE, SELECTION_RECT_FILL, SELECTION_RECT_STROKE,
    SUBGRAPH_PREVIEW_FILL, SUBGRAPH_PREVIEW_STROKE,
};

use diagram_models::document::DiagramDocument;

#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
pub(crate) fn selection_handles_overlay(
    doc: &DiagramDocument,
    interaction_mode: Signal<InteractionMode>,
    doc_signal: Signal<DiagramDocument>,
    canvas_origin: Signal<(f64, f64)>,
    to_screen_coords: impl Fn(CanvasCoord, CanvasCoord, f64) -> ScreenCoord,
) -> Element {
    let selected_nodes = selected_node_ids(doc);
    let _selected_count = selected_nodes.len();
    let selection = selection_bounds(doc);
    if let Some((bx, by, bw, bh)) = selection {
        let s = &doc.editor_state;
        let ScreenCoord(sx, sy) = to_screen_coords(
            CanvasCoord(bx, by),
            CanvasCoord(s.camera_x.0, s.camera_y.0),
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
                                false,
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

pub(crate) fn edge_preview_overlay(
    mode: &InteractionMode,
    doc: &DiagramDocument,
    to_screen_coords: impl Fn(CanvasCoord, CanvasCoord, f64) -> ScreenCoord,
) -> Element {
    let s = &doc.editor_state;
    if let InteractionMode::DrawingEdge {
        from_node,
        current_pos,
    } = mode
    {
        doc.document.nodes.get(from_node).map_or_else(
            || rsx! {},
            |src| {
                let scx = src.x.0 + src.width.0 / 2.0;
                let scy = src.y.0 + src.height.0 / 2.0;
                let (edge_x, edge_y) = rect_ray_intersection(
                    scx,
                    scy,
                    src.width.0,
                    src.height.0,
                    current_pos.0,
                    current_pos.1,
                );

                let ScreenCoord(sx, sy) = to_screen_coords(canvas_domain::CanvasCoord(edge_x, edge_y), canvas_domain::CanvasCoord(s.camera_x.0, s.camera_y.0), s.zoom.0);
                let ScreenCoord(tx, ty) = to_screen_coords(canvas_domain::CanvasCoord(current_pos.0, current_pos.1), canvas_domain::CanvasCoord(s.camera_x.0, s.camera_y.0), s.zoom.0);
                rsx! {
                    line {
                        x1: "{sx}", y1: "{sy}", x2: "{tx}", y2: "{ty}",
                        stroke: "{ACCENT}", stroke_width: "1.8", stroke_dasharray: "5,5", marker_end: "url(#arrow-pending)"
                    }
                }
            },
        )
    } else {
        rsx! {}
    }
}

pub(crate) fn rubber_band_overlay(
    mode: &InteractionMode,
    doc: &DiagramDocument,
    to_screen_coords: impl Fn(CanvasCoord, CanvasCoord, f64) -> ScreenCoord,
) -> Element {
    if let InteractionMode::RubberBand { start, current } = mode {
        let s = &doc.editor_state;
        let ScreenCoord(rx, ry) = to_screen_coords(
            canvas_domain::CanvasCoord(start.0.min(current.0), start.1.min(current.1)),
            canvas_domain::CanvasCoord(s.camera_x.0, s.camera_y.0),
            s.zoom.0,
        );
        let rw = (start.0 - current.0).abs() * s.zoom.0;
        let rh = (start.1 - current.1).abs() * s.zoom.0;
        rsx! {
            rect {
                x: "{rx}", y: "{ry}", width: "{rw}", height: "{rh}",
                fill: "{SELECTION_RECT_FILL}", stroke: "{SELECTION_RECT_STROKE}", stroke_width: "1", stroke_dasharray: "4,2"
            }
        }
    } else {
        rsx! {}
    }
}

pub(crate) fn subgraph_preview_overlay(
    mode: &InteractionMode,
    doc: &DiagramDocument,
    to_screen_coords: impl Fn(CanvasCoord, CanvasCoord, f64) -> ScreenCoord,
) -> Element {
    if let InteractionMode::DrawingSubgraph { start, current } = mode {
        let editor = &doc.editor_state;
        let min_x = start.0.min(current.0);
        let min_y = start.1.min(current.1);
        let width = (start.0 - current.0).abs();
        let height = (start.1 - current.1).abs();
        let ScreenCoord(screen_x, screen_y) = to_screen_coords(
            canvas_domain::CanvasCoord(min_x, min_y),
            canvas_domain::CanvasCoord(editor.camera_x.0, editor.camera_y.0),
            editor.zoom.0,
        );
        rsx! {
            rect {
                x: "{screen_x}", y: "{screen_y}", width: "{width * editor.zoom.0}", height: "{height * editor.zoom.0}",
                fill: "{SUBGRAPH_PREVIEW_FILL}", stroke: "{SUBGRAPH_PREVIEW_STROKE}", stroke_width: "1.2", stroke_dasharray: "6,3"
            }
        }
    } else {
        rsx! {}
    }
}
