use crate::ui::canvas::canvas_view::find_edge_at;
use crate::ui::canvas::document_ops::{find_node_at, sync_canvas_origin};
use crate::ui::canvas::state::CanvasState;
use crate::ui::editor::ToolMode;
use crate::ui::grid::snap_point;
use crate::ui::interaction::{select_single, toggle_selection};
use canvas_domain::interaction_reducer::{commit_inline_edit, InteractionMode};
use canvas_domain::perf::to_canvas_coords;
use diagram_models::document::{LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat};
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use im::HashMap;
use uuid::Uuid;

pub fn handle_mouse_down(state: CanvasState, evt: Event<dioxus::prelude::MouseData>) {
    let mut doc_signal = state.doc_signal;
    let mut history_signal = state.history_signal;
    let mut tool_signal = state.tool_signal;
    let mut interaction_mode = state.interaction_mode;
    let space_pressed = state.space_pressed;
    let shift_pressed = state.shift_pressed;
    let ctrl_pressed = state.ctrl_pressed;
    let meta_pressed = state.meta_pressed;
    let mut editing_node = state.editing_node;
    let mut editing_edge = state.editing_edge;
    let mut edit_value = state.edit_value;
    let mut space_pan_active = state.space_pan_active;
    let multi_touch_active = state.multi_touch_active;
    let canvas_origin = state.canvas_origin;
    let db_tx = state.db_tx;

    if *multi_touch_active.read() {
        return;
    }
    if editing_node.read().is_some() || editing_edge.read().is_some() {
        commit_inline_edit(
            doc_signal,
            history_signal,
            editing_node,
            editing_edge,
            edit_value,
            db_tx,
        )
        .ok();
    }
    let coords = evt.data.coordinates().client();
    // Use origin from the signal - it should be fresh now because the JS pointerdown
    // handler sends a 'resize' message first to update it
    let origin = sync_canvas_origin().unwrap_or_else(|| *canvas_origin.read());
    let local_x = coords.x - origin.0;
    let local_y = coords.y - origin.1;
    let is_middle = evt.data.trigger_button() == Some(MouseButton::Auxiliary);
    let is_right = evt.data.trigger_button() == Some(MouseButton::Secondary);
    let tool = *tool_signal.read();

    if is_middle {
        evt.prevent_default();
    }

    if *space_pressed.read() || is_middle || is_right || tool == ToolMode::Pan {
        space_pan_active
            .set(*space_pressed.read() && !is_middle && !is_right && tool != ToolMode::Pan);
        interaction_mode.set(InteractionMode::Panning {
            last_pos: (local_x, local_y),
        });
        return;
    }

    if evt.data.trigger_button() != Some(MouseButton::Primary) {
        return;
    }

    let pos = {
        let doc = doc_signal.read();
        to_canvas_coords(
            canvas_domain::ScreenCoord(local_x, local_y),
            canvas_domain::CanvasCoord(doc.editor_state.camera_x.0, doc.editor_state.camera_y.0),
            doc.editor_state.zoom.0,
        )
    };

    if tool == ToolMode::Select {
        let doc = doc_signal.read().clone();
        if let Some(node_id) = find_node_at(&doc, pos.0, pos.1) {
            let additive = *shift_pressed.read() || *ctrl_pressed.read() || *meta_pressed.read();
            doc_signal.with_mut(|d| {
                d.editor_state.selected_items = if additive {
                    toggle_selection(&d.editor_state.selected_items, &node_id.to_string())
                } else if d.editor_state.selected_items.contains(&node_id.to_string()) {
                    d.editor_state.selected_items.clone()
                } else {
                    select_single(node_id.to_string())
                };
            });

            let current_doc = doc_signal.read().clone();
            let mut original_positions = HashMap::new();
            for id_str in &current_doc.editor_state.selected_items {
                let id = diagram_models::document::NodeId::new(id_str.clone());
                if let Some(node) = current_doc.document.nodes.get(&id) {
                    original_positions.insert(id, (node.x.0, node.y.0));
                }
            }

            interaction_mode.set(InteractionMode::DraggingSelection {
                anchor_canvas: (pos.0, pos.1),
                anchor_client: (local_x, local_y),
                original_positions,
                did_move: false,
            });
            return;
        } else if let Some(edge_id) = find_edge_at(&doc, pos.0, pos.1) {
            let additive = *shift_pressed.read() || *ctrl_pressed.read() || *meta_pressed.read();
            doc_signal.with_mut(|d| {
                d.editor_state.selected_items = if additive {
                    toggle_selection(&d.editor_state.selected_items, edge_id.to_string().as_str())
                } else {
                    select_single(edge_id.to_string())
                };
            });
            interaction_mode.set(InteractionMode::Select);
            return;
        }
    }

    match tool {
        ToolMode::Text => {
            let id = NodeId::new(Uuid::new_v4().to_string());
            let current = doc_signal.read().clone();
            let history = history_signal.read().clone();
            *history_signal.write() = history.push(current);
            doc_signal.with_mut(|doc| {
                let (x, y) = snap_point(
                    (pos.0, pos.1),
                    doc.editor_state.snap_to_grid,
                    doc.editor_state.grid_size,
                );
                let _ = doc.document.nodes.insert(
                    id.clone(),
                    Node {
                        kind: NodeKind::Text,
                        icon: String::new(),
                        label: String::from("Text"),
                        x: OrderedFloat(x),
                        y: OrderedFloat(y),
                        width: OrderedFloat(100.0),
                        height: OrderedFloat(24.0),
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
                    },
                );
                doc.editor_state.selected_items.clear();
                let _ = doc.editor_state.selected_items.insert(id.to_string());
                doc.revision = doc.revision.increment();
            });
            editing_edge.set(None);
            editing_node.set(None);
            edit_value.set(String::new());
            tool_signal.set(ToolMode::Select);
        }
        ToolMode::Subgraph => {
            let doc = doc_signal.read().clone();
            let snapped_start = snap_point(
                (pos.0, pos.1),
                doc.editor_state.snap_to_grid,
                doc.editor_state.grid_size,
            );
            interaction_mode.set(InteractionMode::DrawingSubgraph {
                start: snapped_start,
                current: snapped_start,
            });
        }
        ToolMode::Select => {
            let additive = *shift_pressed.read() || *ctrl_pressed.read() || *meta_pressed.read();
            if !additive {
                doc_signal.with_mut(|d| {
                    d.editor_state.selected_items.clear();
                });
            }
            interaction_mode.set(InteractionMode::RubberBand {
                start: (pos.0, pos.1),
                current: (pos.0, pos.1),
            });
        }
        ToolMode::Edge => {
            let doc = doc_signal.read().clone();
            if let Some(from_node) = find_node_at(&doc, pos.0, pos.1) {
                interaction_mode.set(InteractionMode::DrawingEdge {
                    from_node,
                    current_pos: (pos.0, pos.1),
                });
            }
        }
        ToolMode::Pan | ToolMode::Draw => {}
    }
}
