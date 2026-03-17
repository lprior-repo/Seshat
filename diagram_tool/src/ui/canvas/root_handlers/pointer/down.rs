use crate::ui::canvas::canvas_view::find_edge_at;
use crate::ui::canvas::document_ops::find_node_at;
use crate::ui::editor::ToolMode;
use crate::ui::grid::snap_point;
use crate::ui::interaction::{select_single, toggle_selection};
use canvas_domain::interaction_reducer::{commit_inline_edit, InteractionMode};
use canvas_domain::perf::to_canvas_coords;
use diagram_models::document::{Node, NodeId, NodeKind, NodeStyle, OrderedFloat};
use diagram_models::LockState;
use dioxus::prelude::*;
use im::HashMap;
use uuid::Uuid;

use super::types::PointerDeps;

pub fn handle_pointer_down(
    deps: &mut PointerDeps,
    json: &serde_json::Value,
    local_x: f64,
    local_y: f64,
    origin_x: f64,
    origin_y: f64,
) {
    deps.canvas_origin.set((origin_x, origin_y));

    let pointer_id = json["pointerId"].as_u64().map_or(0_u32, |v| v as u32);

    if deps.captured_pointer.read().is_some() {
        deps.active_pointers.with_mut(|set| {
            set.insert(pointer_id);
        });
        return;
    }

    deps.captured_pointer.set(Some(pointer_id));
    deps.active_pointers.with_mut(|set| {
        set.insert(pointer_id);
    });

    if deps.editing_node.read().is_some() || deps.editing_edge.read().is_some() {
        commit_inline_edit(
            deps.doc_signal,
            deps.history_signal,
            deps.editing_node,
            deps.editing_edge,
            deps.edit_value,
            deps.db_tx.clone(),
        )
        .ok();
    }

    let button = json["button"].as_str().map_or("0", |s| s);
    let is_middle = button == "1";
    let is_right = button == "2";
    let tool = *deps.tool_signal.read();
    let shift = json["shiftKey"].as_bool().unwrap_or(false);
    let ctrl = json["ctrlKey"].as_bool().unwrap_or(false);
    let meta = json["metaKey"].as_bool().unwrap_or(false);

    if *deps.space_pressed.read() || is_middle || is_right || tool == ToolMode::Pan {
        deps.space_pan_active
            .set(*deps.space_pressed.read() && !is_middle && !is_right && tool != ToolMode::Pan);
        deps.interaction_mode.set(InteractionMode::Panning {
            last_pos: (local_x, local_y),
        });
        return;
    }

    if button != "0" {
        return;
    }

    let pos = {
        let doc = deps.doc_signal.read();
        to_canvas_coords(
            canvas_domain::ScreenCoord(local_x, local_y),
            canvas_domain::CanvasCoord(doc.editor_state.camera_x.0, doc.editor_state.camera_y.0),
            doc.editor_state.zoom.0,
        )
    };

    if tool == ToolMode::Select {
        let doc = deps.doc_signal.read().clone();
        if let Some(node_id) = find_node_at(&doc, pos.0, pos.1) {
            let additive = shift || ctrl || meta;
            deps.doc_signal.with_mut(|d| {
                d.editor_state.selected_items = if additive {
                    toggle_selection(&d.editor_state.selected_items, &node_id.to_string())
                } else if d.editor_state.selected_items.contains(&node_id.to_string()) {
                    d.editor_state.selected_items.clone()
                } else {
                    select_single(node_id.to_string())
                };
            });

            let current_doc = deps.doc_signal.read().clone();
            let mut original_positions = HashMap::new();
            for id_str in &current_doc.editor_state.selected_items {
                let id = diagram_models::document::NodeId::new(id_str.clone());
                if let Some(node) = current_doc.document.nodes.get(&id) {
                    original_positions.insert(id, (node.x.0, node.y.0));
                }
            }

            deps.interaction_mode
                .set(InteractionMode::DraggingSelection {
                    anchor_canvas: (pos.0, pos.1),
                    anchor_client: (local_x, local_y),
                    original_positions,
                    did_move: false,
                });
            return;
        } else if let Some(edge_id) = find_edge_at(&doc, pos.0, pos.1) {
            let additive = shift || ctrl || meta;
            deps.doc_signal.with_mut(|d| {
                d.editor_state.selected_items = if additive {
                    toggle_selection(&d.editor_state.selected_items, edge_id.to_string().as_str())
                } else {
                    select_single(edge_id.to_string())
                };
            });
            deps.interaction_mode.set(InteractionMode::Select);
            return;
        } else {
            let additive = shift || ctrl || meta;
            if !additive {
                deps.doc_signal.with_mut(|d| {
                    d.editor_state.selected_items.clear();
                });
            }
        }
    }

    if tool == ToolMode::Text {
        let id = NodeId::new(Uuid::new_v4().to_string());
        let current = deps.doc_signal.read().clone();
        let history = deps.history_signal.read().clone();
        *deps.history_signal.write() = history.push(current);
        deps.doc_signal.with_mut(|doc| {
            let (x, y) = snap_point(
                (pos.0, pos.1),
                doc.editor_state.snap_to_grid,
                doc.editor_state.grid_size.into(),
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
        deps.editing_edge.set(None);
        deps.editing_node.set(None);
        deps.edit_value.set(String::new());
        deps.tool_signal.set(ToolMode::Select);
    }
}
