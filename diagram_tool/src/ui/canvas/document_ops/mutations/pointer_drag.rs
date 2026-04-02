use crate::history::History;
use crate::ui::interaction::{dragged_positions_with_snap, has_drag_threshold};
use canvas_domain::perf::to_canvas_coords;
use diagram_models::document::{DiagramDocument, Node, OrderedFloat};
use dioxus::prelude::*;
use im::HashMap;

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
    let doc = doc_signal.read().clone();
    let current_pos = to_canvas_coords(
        canvas_domain::ScreenCoord(client_x, client_y),
        canvas_domain::CanvasCoord(doc.editor_state.camera_x.0, doc.editor_state.camera_y.0),
        doc.editor_state.zoom.0,
    );

    let has_movable_nodes = original_positions.keys().any(|id| {
        doc.document
            .nodes
            .get(id)
            .is_some_and(|node| node.lock_state.is_movable(&node.kind))
    });

    if !*did_move && has_movable_nodes && has_drag_threshold(*anchor_client, (client_x, client_y)) {
        let history = history_signal.read().clone();
        *history_signal.write() = history.push(doc.clone());
        *did_move = true;
    }

    if *did_move {
        let positions = dragged_positions_with_snap(
            original_positions,
            *anchor_canvas,
            (current_pos.0, current_pos.1),
            doc.editor_state.snap_to_grid,
            doc.editor_state.grid_size,
        );
        let has_changes = positions.iter().any(|(id, (nx, ny))| {
            doc.document.nodes.get(id).is_some_and(|node| {
                node.lock_state.is_movable(&node.kind)
                    && ((node.x.0 - *nx).abs() > f64::EPSILON
                        || (node.y.0 - *ny).abs() > f64::EPSILON)
            })
        });

        if has_changes {
            doc_signal.with_mut(|doc_mut| {
                for (id, (nx, ny)) in positions.iter() {
                    let should_update = doc_mut.document.nodes.get(id).is_some_and(|node| {
                        node.lock_state.is_movable(&node.kind)
                            && ((node.x.0 - *nx).abs() > f64::EPSILON
                                || (node.y.0 - *ny).abs() > f64::EPSILON)
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
        }
    }

    false
}
