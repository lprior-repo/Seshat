use crate::history::History;
use crate::ui::interaction::{dragged_positions_with_snap, has_drag_threshold};
use canvas_domain::perf::to_canvas_coords;
use diagram_models::document::{DiagramDocument, Node, OrderedFloat};
use dioxus::prelude::*;
use im::HashMap;

fn node_position_changed(node: &Node, next_x: f64, next_y: f64) -> bool {
    (node.x.0 - next_x).abs() > f64::EPSILON || (node.y.0 - next_y).abs() > f64::EPSILON
}

#[allow(clippy::too_many_arguments)]
pub fn handle_dragging(
    doc_signal: &mut Signal<DiagramDocument>,
    history_signal: &mut Signal<History>,
    client_x: f64,
    client_y: f64,
    anchor_canvas: &(f64, f64),
    anchor_client: &(f64, f64),
    original_positions: &HashMap<diagram_models::document::NodeId, (f64, f64)>,
    did_move: &mut bool,
) -> bool {
    let (camera_x, camera_y, zoom, snap_to_grid, grid_size) = doc_signal.with(|doc| {
        (
            doc.editor_state.camera_x.0,
            doc.editor_state.camera_y.0,
            doc.editor_state.zoom.0,
            doc.editor_state.snap_to_grid,
            doc.editor_state.grid_size,
        )
    });
    let current_pos = to_canvas_coords(
        canvas_domain::ScreenCoord(client_x, client_y),
        canvas_domain::CanvasCoord(camera_x, camera_y),
        zoom,
    );

    let has_movable_nodes = original_positions.keys().any(|id| {
        doc_signal
            .peek()
            .document
            .nodes
            .get(id)
            .is_some_and(|node| node.lock_state.is_movable(&node.kind))
    });

    let exceeds = has_drag_threshold(*anchor_client, (client_x, client_y));
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!("DraggingSelection threshold check: did_move={} has_movable={} exceeds={} anchor={:?} client=({:?},{:?})", *did_move, has_movable_nodes, exceeds, anchor_client, client_x, client_y)));

    if !*did_move && has_movable_nodes && exceeds {
        let history = history_signal.read().clone();
        let snapshot = doc_signal.read().clone();
        *history_signal.write() = history.push(snapshot);
        *did_move = true;
    }

    if *did_move {
        let positions = dragged_positions_with_snap(
            original_positions,
            *anchor_canvas,
            (current_pos.0, current_pos.1),
            snap_to_grid,
            grid_size,
        );
        let has_changes = positions.iter().any(|(id, (nx, ny))| {
            doc_signal
                .peek()
                .document
                .nodes
                .get(id)
                .is_some_and(|node| {
                    node.lock_state.is_movable(&node.kind) && node_position_changed(node, *nx, *ny)
                })
        });

        if has_changes {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str("DraggingSelection has_changes = true"));
            
            doc_signal.with_mut(|doc_mut| {
                for (id, (nx, ny)) in positions.iter() {
                    let should_update = doc_mut.document.nodes.get(id).is_some_and(|node| {
                        node.lock_state.is_movable(&node.kind)
                            && node_position_changed(node, *nx, *ny)
                    });
                    if should_update {
                        doc_mut.document.nodes = doc_mut.document.nodes.alter(
                            |n| {
                                n.map(|node| Node {
                                    x: OrderedFloat(*nx),
                                    y: OrderedFloat(*ny),
                                    ..node
                                })
                            },
                            id.clone(),
                        );
                    }
                }
            });
            return true;
        } else {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str("DraggingSelection has_changes = false"));
        }
    }

    false
}
