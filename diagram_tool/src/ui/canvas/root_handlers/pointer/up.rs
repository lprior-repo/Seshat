use crate::ui::canvas::document_ops::{
    apply_rubber_band_release, find_node_at, flush_pending_pointer_update, subgraph_release_bounds,
};
use crate::ui::dispatch::send::edge::handle_edge_drawing_complete;
use crate::ui::editor::ToolMode;
use canvas_domain::interaction_reducer::{finalize_motion_release, InteractionMode};
use canvas_domain::perf::to_canvas_coords;
use diagram_models::document::{EdgeId, Node, NodeId, NodeKind, NodeStyle, OrderedFloat};
use diagram_models::LockState;
use dioxus::prelude::*;
use im::HashMap;
use uuid::Uuid;

use super::types::PointerDeps;

pub fn handle_pointer_up(
    deps: &mut PointerDeps,
    json: &serde_json::Value,
    local_x: f64,
    local_y: f64,
) {
    if *deps.multi_touch_active.read() {
        return;
    }

    let up_pointer_id = json["pointerId"].as_u64().map_or(0_u32, |v| v as u32);

    let was_captured = deps
        .captured_pointer
        .read()
        .is_some_and(|id| id == up_pointer_id);

    if was_captured {
        deps.captured_pointer.set(None);
    }

    deps.active_pointers.with_mut(|set| {
        set.remove(&up_pointer_id);
    });

    if was_captured {
        deps.interaction_mode.set(InteractionMode::Select);
    }

    flush_pending_pointer_update(
        deps.doc_signal,
        deps.history_signal,
        deps.interaction_mode,
        deps.pending_pointer_sample,
        deps.db_tx,
    );

    deps.interaction_mode.with_mut(|mode| match mode {
        InteractionMode::DrawingEdge { from_node, .. } => {
            let doc = deps.doc_signal.read().clone();
            let pos = to_canvas_coords(
                canvas_domain::ScreenCoord(local_x, local_y),
                canvas_domain::CanvasCoord(
                    doc.editor_state.camera_x.0,
                    doc.editor_state.camera_y.0,
                ),
                doc.editor_state.zoom.0,
            );
            let target = find_node_at(&doc, pos.0, pos.1);
            if let Some(target_id) = target.clone() {
                if &target_id != from_node {
                    let candidate_edge = diagram_models::document::Edge {
                        source: from_node.clone(),
                        target: target_id.clone(),
                        label: String::new(),
                        style: *deps.edge_style_default.read(),
                        arrow_type: *deps.arrow_type_default.read(),
                        label_offset_t: OrderedFloat(0.5),
                        color: None,
                        thickness: OrderedFloat(1.5),
                        directed: true,
                        bend_points: im::Vector::new(),
                        tags: im::Vector::new(),
                        metadata: HashMap::new(),
                        font_size: None,
                        source_port: None,
                        target_port: None,
                    };

                    let edge_preserves_dag = crate::ui::canvas::document_ops::edge_preserves_dag;

                    if !edge_preserves_dag(&doc, &candidate_edge) {
                        let _ = deps.toast.show(
                            crate::ui::toast::ToastIntent::Warning,
                            "Cannot create circular connection",
                            None,
                        );
                        if *deps.tool_signal.read() == ToolMode::Edge {
                            if let Some(target_id) = target {
                                *mode = InteractionMode::DrawingEdge {
                                    from_node: target_id,
                                    current_pos: (pos.0, pos.1),
                                };
                            } else {
                                *mode = InteractionMode::Select;
                            }
                        } else {
                            *mode = InteractionMode::Select;
                        }
                        return;
                    }

                    let doc_for_dispatch = deps.doc_signal.read();
                    let dispatch_result = handle_edge_drawing_complete(
                        deps.db_tx,
                        &doc_for_dispatch,
                        from_node.to_string(),
                        target_id.to_string(),
                    );
                    if let Err(e) = dispatch_result {
                        eprintln!(
                            "EdgeConnect dispatch failed: {e:?}, continuing with local mutation"
                        );
                    }
                    drop(doc_for_dispatch);

                    let current = deps.doc_signal.read().clone();
                    let history = deps.history_signal.read().clone();
                    *deps.history_signal.write() = history.push(current);
                    deps.doc_signal.with_mut(|doc_mut| {
                        doc_mut.document.edges = doc_mut
                            .document
                            .edges
                            .update(EdgeId::new(Uuid::new_v4().to_string()), candidate_edge);
                        doc_mut.revision = doc_mut.revision.increment();
                    });
                }
            }
            if *deps.tool_signal.read() == ToolMode::Edge {
                if let Some(target_id) = target {
                    *mode = InteractionMode::DrawingEdge {
                        from_node: target_id,
                        current_pos: (pos.0, pos.1),
                    };
                } else {
                    *mode = InteractionMode::Select;
                }
            } else {
                *mode = InteractionMode::Select;
            }
        }
        InteractionMode::RubberBand { start, current } => {
            let additive = *deps.shift_pressed.read()
                || *deps.ctrl_pressed.read()
                || *deps.meta_pressed.read();
            deps.doc_signal.with_mut(|doc| {
                apply_rubber_band_release(doc, *start, *current, additive);
            });
            *mode = InteractionMode::Select;
        }
        InteractionMode::DrawingSubgraph { start, current } => {
            let doc_now = deps.doc_signal.read().clone();
            let snap = doc_now.editor_state.snap_to_grid;
            let grid = doc_now.editor_state.grid_size;
            if let Some((x, y, _w, _h)) = subgraph_release_bounds(*start, *current, snap, grid) {
                let id = NodeId::new(Uuid::new_v4().to_string());
                let current_doc = deps.doc_signal.read().clone();
                let history = deps.history_signal.read().clone();
                *deps.history_signal.write() = history.push(current_doc);
                deps.doc_signal.with_mut(|doc| {
                    doc.document.nodes = doc.document.nodes.update(
                        id.clone(),
                        Node {
                            kind: NodeKind::Subgraph,
                            icon: String::new(),
                            label: String::new(),
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
            }
            deps.tool_signal.set(ToolMode::Select);
            *mode = InteractionMode::Select;
        }
        InteractionMode::ResizingSelection { .. } | InteractionMode::DraggingSelection { .. } => {
            let mut doc_clone = deps.doc_signal.read().clone();
            let did_change = finalize_motion_release(mode, &mut doc_clone, &deps.db_tx);
            if did_change {
                deps.doc_signal.set(doc_clone);
                *mode = InteractionMode::Select;
            }
        }
        InteractionMode::Panning { .. } => {
            *mode = InteractionMode::Select;
        }
        InteractionMode::Select => {}
    });
    deps.space_pan_active.set(false);
}
