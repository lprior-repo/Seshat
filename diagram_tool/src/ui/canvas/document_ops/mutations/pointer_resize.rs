use crate::history::History;
use crate::ui::grid::snap_value;
use canvas_domain::interaction_reducer::ResizeHandle;
use canvas_domain::perf::to_canvas_coords;
use diagram_models::document::{DiagramDocument, OrderedFloat};
use dioxus::prelude::*;
use im::HashMap;

use super::super::queries::safe_zoom;

#[allow(clippy::too_many_arguments)]
pub fn handle_resizing(
    doc_signal: &mut Signal<DiagramDocument>,
    history_signal: &mut Signal<History>,
    client_x: f64,
    client_y: f64,
    handle: &ResizeHandle,
    original_bounds: &(f64, f64, f64, f64),
    originals: &HashMap<diagram_models::document::NodeId, (f64, f64, f64, f64)>,
    anchor: &(f64, f64),
    did_resize: &mut bool,
    aspect_ratio: &Option<f64>,
) {
    let doc_for_mouse = doc_signal.read().clone();
    let canvas_domain::CanvasCoord(mx, my) = to_canvas_coords(
        canvas_domain::ScreenCoord(client_x, client_y),
        canvas_domain::CanvasCoord(
            doc_for_mouse.editor_state.camera_x.0,
            doc_for_mouse.editor_state.camera_y.0,
        ),
        safe_zoom(doc_for_mouse.editor_state.zoom.0),
    );
    let delta_x_raw = mx - anchor.0;
    let delta_y_raw = my - anchor.1;
    let snap = doc_for_mouse.editor_state.snap_to_grid;
    let grid = doc_for_mouse.editor_state.grid_size;
    let dx = snap_value(delta_x_raw, snap, grid);
    let dy = snap_value(delta_y_raw, snap, grid);

    let has_resizable_nodes = originals.keys().any(|id| {
        doc_for_mouse
            .document
            .nodes
            .get(id)
            .is_some_and(|node| node.lock_state.is_movable(&node.kind))
    });

    if !*did_resize && has_resizable_nodes && (dx.abs() > f64::EPSILON || dy.abs() > f64::EPSILON) {
        let history = history_signal.read().clone();
        *history_signal.write() = history.push(doc_for_mouse);
        *did_resize = true;
    }

    if *did_resize {
        let (obx, oby, obw, obh) = *original_bounds;
        let north = *handle == ResizeHandle::Nw
            || *handle == ResizeHandle::N
            || *handle == ResizeHandle::Ne;
        let south = *handle == ResizeHandle::Sw
            || *handle == ResizeHandle::S
            || *handle == ResizeHandle::Se;
        let west = *handle == ResizeHandle::Nw
            || *handle == ResizeHandle::W
            || *handle == ResizeHandle::Sw;
        let east = *handle == ResizeHandle::Ne
            || *handle == ResizeHandle::E
            || *handle == ResizeHandle::Se;

        let mut dx_clamped = dx;
        let mut dy_clamped = dy;

        if west {
            dx_clamped = dx_clamped.min(obw - 24.0);
        } else if east {
            dx_clamped = dx_clamped.max(24.0 - obw);
        }

        if north {
            dy_clamped = dy_clamped.min(obh - 24.0);
        } else if south {
            dy_clamped = dy_clamped.max(24.0 - obh);
        }

        let nx = if west { obx + dx_clamped } else { obx };
        let ny = if north { oby + dy_clamped } else { oby };
        let nw: f64 = if west {
            obw - dx_clamped
        } else if east {
            obw + dx_clamped
        } else {
            obw
        }
        .max(24.0);
        let nh: f64 = if north {
            obh - dy_clamped
        } else if south {
            obh + dy_clamped
        } else {
            obh
        }
        .max(24.0);

        #[allow(clippy::option_if_let_else)]
        let (nw, nh) = if let Some(ratio) = aspect_ratio {
            let ratio = *ratio;
            let is_corner_handle = matches!(
                handle,
                ResizeHandle::Nw | ResizeHandle::Ne | ResizeHandle::Sw | ResizeHandle::Se
            );
            let is_north_south = matches!(handle, ResizeHandle::N | ResizeHandle::S);

            if is_corner_handle {
                let constrained_nw = nh * ratio;
                let constrained_nh = nw / ratio;

                if (constrained_nw - nw).abs() < (constrained_nh - nh).abs() {
                    (constrained_nw.max(24.0), nh)
                } else {
                    (nw, constrained_nh.max(24.0))
                }
            } else if is_north_south {
                (nh * ratio, nh)
            } else {
                (nw, nw / ratio)
            }
        } else {
            (nw, nh)
        };

        let scale_x = if obw > 0.0 { nw / obw } else { 1.0 };
        let scale_y = if obh > 0.0 { nh / obh } else { 1.0 };

        doc_signal.with_mut(|doc_mut| {
            for (id, (ox, oy, ow, oh)) in originals.iter() {
                if let Some(node) = doc_mut.document.nodes.get_mut(id) {
                    if !node.lock_state.is_movable(&node.kind) {
                        continue;
                    }
                    let nxx: f64 = (ox - obx).mul_add(scale_x, nx);
                    let nyy: f64 = (oy - oby).mul_add(scale_y, ny);
                    let nww = (ow * scale_x).max(24.0);
                    let nhh = (oh * scale_y).max(24.0);
                    node.x = OrderedFloat(nxx);
                    node.y = OrderedFloat(nyy);
                    node.width = OrderedFloat(nww);
                    node.height = OrderedFloat(nhh);
                }
            }
        });
    }
}
