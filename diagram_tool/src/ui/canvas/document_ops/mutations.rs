use crate::{
    history::History,
    ui::{
        grid::snap_value,
        interaction::{
            dragged_positions_with_snap, has_drag_threshold, node_ids_in_rect, toggle_selection,
            with_auto_selected_edges,
        },
    },
};
use canvas_domain::{
    interaction_reducer::{InteractionMode, ResizeHandle},
    perf::{to_canvas_coords, wheel_update, WheelInput},
    selection_geometry::{selected_node_ids, selection_bounds},
};
use diagram_models::document::{DiagramDocument, Node, OrderedFloat};
use dioxus::prelude::*;

use super::queries::safe_zoom;

pub fn scale_selected_nodes(doc: &mut DiagramDocument, factor: f64) -> bool {
    let Some((bx, by, bw, bh)) = selection_bounds(doc) else {
        return false;
    };
    let selected = selected_node_ids(doc);
    if selected.is_empty() {
        return false;
    }

    let center_x = bx + (bw / 2.0);
    let center_y = by + (bh / 2.0);
    let snap = doc.editor_state.snap_to_grid;
    let grid = doc.editor_state.grid_size;
    let mut changed = false;

    for node_id in selected {
        if let Some(node) = doc.document.nodes.get_mut(&node_id) {
            if !node.lock_state.is_movable(&node.kind) {
                continue;
            }
            let rel_x = node.x.0 - center_x;
            let rel_y = node.y.0 - center_y;
            let mut next_x = center_x + (rel_x * factor);
            let mut next_y = center_y + (rel_y * factor);
            let mut next_w = (node.width.0 * factor).round().max(24.0);
            let mut next_h = (node.height.0 * factor).round().max(24.0);

            if snap {
                next_x = snap_value(next_x, true, grid.into());
                next_y = snap_value(next_y, true, grid.into());
                next_w = snap_value(next_w, true, grid.into()).max(24.0);
                next_h = snap_value(next_h, true, grid.into()).max(24.0);
            }

            node.x = OrderedFloat(next_x);
            node.y = OrderedFloat(next_y);
            node.width = OrderedFloat(next_w);
            node.height = OrderedFloat(next_h);
            changed = true;
        }
    }

    changed
}

pub fn apply_rubber_band_release(
    doc: &mut DiagramDocument,
    start: (f64, f64),
    current: (f64, f64),
    additive: bool,
) {
    if !has_drag_threshold(start, current) {
        return;
    }

    let boxed = node_ids_in_rect(doc, start, current);
    let selected = if additive {
        boxed
            .iter()
            .fold(doc.editor_state.selected_items.clone(), |acc, id| {
                toggle_selection(&acc, id)
            })
    } else {
        // Clear existing selection before applying new marquee selection
        doc.editor_state.selected_items.clear();
        boxed
    };
    doc.editor_state.selected_items = with_auto_selected_edges(doc, &selected);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WheelSample {
    pub client_x: f64,
    pub client_y: f64,
    pub dx: f64,
    pub dy: f64,
    pub zoom_gesture: bool,
    pub shift_pan: bool,
    pub discrete_wheel: bool,
}

pub fn flush_pending_wheel_update(
    mut doc_signal: Signal<DiagramDocument>,
    mut pending_wheel_sample: Signal<Option<WheelSample>>,
) {
    let pending = pending_wheel_sample.read().as_ref().copied();
    let Some(sample) = pending else {
        return;
    };
    pending_wheel_sample.set(None);

    let current = doc_signal.read().editor_state.clone();
    if let Some((next_x, next_y, next_zoom)) = wheel_update(WheelInput {
        camera_x: current.camera_x,
        camera_y: current.camera_y,
        zoom: current.zoom,
        client_x: sample.client_x,
        client_y: sample.client_y,
        dx: sample.dx,
        dy: sample.dy,
        zoom_gesture: sample.zoom_gesture,
        shift_pan: sample.shift_pan,
        discrete_wheel: sample.discrete_wheel,
    }) {
        doc_signal.with_mut(|doc| {
            doc.editor_state.camera_x = next_x;
            doc.editor_state.camera_y = next_y;
            doc.editor_state.zoom = next_zoom;
        });
    }
}

#[allow(clippy::too_many_lines, clippy::similar_names)]
pub fn flush_pending_pointer_update(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    mut interaction_mode: Signal<InteractionMode>,
    mut pending_pointer_sample: Signal<Option<(f64, f64)>>,
    db_tx: Option<dioxus::prelude::Coroutine<diagram_models::envelope::EventEnvelope>>,
) {
    let pending = pending_pointer_sample.read().as_ref().copied();
    let Some((client_x, client_y)) = pending else {
        return;
    };
    pending_pointer_sample.set(None);

    interaction_mode.with_mut(|mode| match mode {
        InteractionMode::DraggingSelection {
            anchor_canvas,
            anchor_client,
            original_positions,
            did_move,
        } => {
            let doc = doc_signal.read().clone();
            let current_pos = to_canvas_coords(
                canvas_domain::ScreenCoord(client_x, client_y),
                canvas_domain::CanvasCoord(
                    doc.editor_state.camera_x.0,
                    doc.editor_state.camera_y.0,
                ),
                doc.editor_state.zoom.0,
            );

            let has_movable_nodes = original_positions.keys().any(|id| {
                doc.document
                    .nodes
                    .get(&id)
                    .is_some_and(|node| node.lock_state.is_movable(&node.kind))
            });

            if !*did_move
                && has_movable_nodes
                && has_drag_threshold(*anchor_client, (client_x, client_y))
            {
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
                    doc.editor_state.grid_size.into(),
                );
                let has_changes = positions.iter().any(|(id, (nx, ny))| {
                    doc.document.nodes.get(&id).is_some_and(|node| {
                        node.lock_state.is_movable(&node.kind)
                            && ((node.x.0 - *nx).abs() > f64::EPSILON
                                || (node.y.0 - *ny).abs() > f64::EPSILON)
                    })
                });

                if has_changes {
                    doc_signal.with_mut(|doc_mut| {
                        for (id, (nx, ny)) in positions.iter() {
                            let should_update =
                                doc_mut.document.nodes.get(&id).is_some_and(|node| {
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
                                if let Some(tx) = &db_tx {
                                    tx.send(diagram_models::envelope::EventEnvelope {
                                        op_id: uuid::Uuid::new_v4().to_string(),
                                        operation: diagram_models::envelope::DomainOp::NodeMove {
                                            id: id.clone(),
                                            x: *nx,
                                            y: *ny,
                                        },
                                        author: diagram_models::envelope::Author {
                                            id: "local-user".to_string(),
                                            name: "Local User".to_string(),
                                            email: None,
                                        },
                                        timestamp: {
                                            #[cfg(target_arch = "wasm32")]
                                            {
                                                js_sys::Date::now() as i64
                                            }
                                            #[cfg(not(target_arch = "wasm32"))]
                                            {
                                                std::time::SystemTime::now()
                                                    .duration_since(std::time::UNIX_EPOCH)
                                                    .unwrap_or_default()
                                                    .as_millis()
                                                    as i64
                                            }
                                        },
                                    });
                                }
                            }
                        }
                    });
                }
            }
        }
        InteractionMode::ResizingSelection {
            handle,
            original_bounds,
            originals,
            anchor,
            did_resize,
            aspect_ratio,
        } => {
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
            let dx = snap_value(delta_x_raw, snap, grid.into());
            let dy = snap_value(delta_y_raw, snap, grid.into());

            let has_resizable_nodes = originals.keys().any(|id| {
                doc_for_mouse
                    .document
                    .nodes
                    .get(&id)
                    .is_some_and(|node| node.lock_state.is_movable(&node.kind))
            });

            if !*did_resize && has_resizable_nodes && (dx != 0.0 || dy != 0.0) {
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

                // Apply aspect ratio constraint if locked
                #[allow(clippy::option_if_let_else)]
                let (nw, nh) = if let Some(ratio) = aspect_ratio {
                    let ratio = *ratio;
                    // Determine handle type
                    let is_corner_handle = matches!(
                        handle,
                        ResizeHandle::Nw | ResizeHandle::Ne | ResizeHandle::Sw | ResizeHandle::Se
                    );
                    let is_north_south = matches!(handle, ResizeHandle::N | ResizeHandle::S);

                    if is_corner_handle {
                        // Corner handles: constrain both dimensions to maintain ratio
                        // Use the larger change to determine which dimension to adjust
                        let constrained_nw = nh * ratio;
                        let constrained_nh = nw / ratio;

                        // Use whichever keeps size closer to dragged amount
                        if (constrained_nw - nw).abs() < (constrained_nh - nh).abs() {
                            (constrained_nw.max(24.0), nh)
                        } else {
                            (nw, constrained_nh.max(24.0))
                        }
                    } else if is_north_south {
                        // N/S handles: constrain width based on height
                        (nh * ratio, nh)
                    } else {
                        // E/W handles: constrain height based on width
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
        InteractionMode::Panning { last_pos } => {
            let dx = client_x - last_pos.0;
            let dy = client_y - last_pos.1;
            *last_pos = (client_x, client_y);
            if dx.abs() > f64::EPSILON || dy.abs() > f64::EPSILON {
                doc_signal.with_mut(|doc| {
                    let zoom = safe_zoom(doc.editor_state.zoom.0);
                    doc.editor_state.camera_x =
                        OrderedFloat(doc.editor_state.camera_x.0 - (dx / zoom));
                    doc.editor_state.camera_y =
                        OrderedFloat(doc.editor_state.camera_y.0 - (dy / zoom));
                });
            }
        }
        InteractionMode::Select
        | InteractionMode::RubberBand { .. }
        | InteractionMode::DrawingEdge { .. }
        | InteractionMode::DrawingSubgraph { .. } => {}
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use diagram_models::document::{LockState, NodeId, NodeKind, NodeStyle};
    use im::HashMap;

    fn node_at(x: f64, y: f64) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: String::from("N"),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(50.0),
            height: OrderedFloat(50.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_rubber_band_release_when_applied_then_selection_is_committed() {
        let mut doc = DiagramDocument::default();
        let node_id = NodeId::new(String::from("n1"));
        doc.document.nodes = doc
            .document
            .nodes
            .update(node_id.clone(), node_at(10.0, 10.0));

        apply_rubber_band_release(&mut doc, (0.0, 0.0), (80.0, 80.0), false);

        assert!(doc
            .editor_state
            .selected_items
            .contains(&node_id.to_string()));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_noop_rubber_band_when_released_then_selection_is_preserved() {
        let mut doc = DiagramDocument::default();
        let node_id = NodeId::new(String::from("n1"));
        doc.document.nodes = doc
            .document
            .nodes
            .update(node_id.clone(), node_at(10.0, 10.0));
        doc.editor_state.selected_items =
            doc.editor_state.selected_items.update(node_id.to_string());

        apply_rubber_band_release(&mut doc, (10.0, 10.0), (10.0, 10.0), false);

        assert!(doc
            .editor_state
            .selected_items
            .contains(&node_id.to_string()));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_existing_selection_when_rubber_band_released_then_selection_is_cleared() {
        let mut doc = DiagramDocument::default();
        // Create two nodes
        let node1_id = NodeId::new(String::from("n1"));
        let node2_id = NodeId::new(String::from("n2"));
        doc.document.nodes = doc
            .document
            .nodes
            .update(node1_id.clone(), node_at(10.0, 10.0))
            .update(node2_id.clone(), node_at(100.0, 100.0));
        // Select node1 first
        doc.editor_state.selected_items =
            doc.editor_state.selected_items.update(node1_id.to_string());

        // Apply rubber band that only contains node2
        apply_rubber_band_release(&mut doc, (50.0, 50.0), (150.0, 150.0), false);

        // Selection should be cleared and only node2 should be selected
        assert!(!doc
            .editor_state
            .selected_items
            .contains(&node1_id.to_string()));
        assert!(doc
            .editor_state
            .selected_items
            .contains(&node2_id.to_string()));
    }
}
