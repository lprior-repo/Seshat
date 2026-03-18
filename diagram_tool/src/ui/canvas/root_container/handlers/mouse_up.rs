use crate::ui::canvas::document_ops::{
    apply_rubber_band_release, edge_preserves_dag, find_node_at, flush_pending_pointer_update,
    subgraph_release_bounds, sync_canvas_origin,
};
use crate::ui::canvas::state::CanvasState;
use crate::ui::editor::ToolMode;
use canvas_domain::interaction_reducer::{finalize_motion_release, InteractionMode};
use canvas_domain::perf::to_canvas_coords;
use diagram_models::document::{
    EdgeId, LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
};
use dioxus::prelude::*;
use im::HashMap;
use uuid::Uuid;

pub fn handle_mouse_up(state: CanvasState, evt: Event<dioxus::prelude::MouseData>) {
    let mut interaction_mode = state.interaction_mode;
    let mut doc_signal = state.doc_signal;
    let mut history_signal = state.history_signal;
    let mut tool_signal = state.tool_signal;
    let edge_style_default = state.edge_style_default;
    let arrow_type_default = state.arrow_type_default;
    let mut space_pan_active = state.space_pan_active;
    let shift_pressed = state.shift_pressed;
    let ctrl_pressed = state.ctrl_pressed;
    let meta_pressed = state.meta_pressed;
    let pending_pointer_sample = state.pending_pointer_sample;
    let canvas_origin = state.canvas_origin;
    let db_tx = state.db_tx;

    flush_pending_pointer_update(
        doc_signal,
        history_signal,
        interaction_mode,
        pending_pointer_sample,
        db_tx,
    );

    let toast = crate::ui::toast::use_toast();

    interaction_mode.with_mut(|mode| match mode {
        InteractionMode::DrawingEdge { from_node, .. } => {
            let coords = evt.data.coordinates().client();
            let origin = sync_canvas_origin().unwrap_or_else(|| *canvas_origin.read());
            let local_x = coords.x - origin.0;
            let local_y = coords.y - origin.1;
            let doc = doc_signal.read().clone();
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
                        target: target_id,
                        label: String::new(),
                        style: *edge_style_default.read(),
                        arrow_type: *arrow_type_default.read(),
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

                    if !edge_preserves_dag(&doc, &candidate_edge) {
                        let _ = toast.show(
                            crate::ui::toast::ToastIntent::Warning,
                            "Cannot create circular connection",
                            None,
                        );
                        if *tool_signal.read() == ToolMode::Edge {
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

                    let current = doc_signal.read().clone();
                    let history = history_signal.read().clone();
                    *history_signal.write() = history.push(current);
                    doc_signal.with_mut(|doc_mut| {
                        doc_mut.document.edges = doc_mut
                            .document
                            .edges
                            .update(EdgeId::new(Uuid::new_v4().to_string()), candidate_edge);
                        doc_mut.revision = doc_mut.revision.increment();
                    });
                }
            }
            if *tool_signal.read() == ToolMode::Edge {
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
            let additive = *shift_pressed.read() || *ctrl_pressed.read() || *meta_pressed.read();
            doc_signal.with_mut(|doc| {
                apply_rubber_band_release(doc, *start, *current, additive);
            });
            *mode = InteractionMode::Select;
        }
        InteractionMode::DrawingSubgraph { start, current } => {
            let doc_now = doc_signal.read().clone();
            let snap = doc_now.editor_state.snap_to_grid;
            let grid = doc_now.editor_state.grid_size;
            if let Some((x, y, w, h)) = subgraph_release_bounds(*start, *current, snap, grid) {
                let id = NodeId::new(Uuid::new_v4().to_string());
                let current_doc = doc_signal.read().clone();
                let history = history_signal.read().clone();
                *history_signal.write() = history.push(current_doc);
                doc_signal.with_mut(|doc| {
                    let _ = doc.document.nodes.insert(
                        id.clone(),
                        Node {
                            kind: NodeKind::Subgraph,
                            icon: String::new(),
                            label: String::from("Subgraph"),
                            x: OrderedFloat(x),
                            y: OrderedFloat(y),
                            width: OrderedFloat(w),
                            height: OrderedFloat(h),
                            font_size: None,
                            font_weight: None,
                            lock_state: LockState::Locked,
                            parent: None,
                            dag_rank: None,
                            tags: im::Vector::new(),
                            metadata: HashMap::new(),
                            z_index: -1,
                            style: Some(NodeStyle::Box),
                            collapsed: Some(false),
                        },
                    );
                    doc.editor_state.selected_items.clear();
                    let _ = doc.editor_state.selected_items.insert(id.to_string());
                    doc.revision = doc.revision.increment();
                });
            }
            tool_signal.set(ToolMode::Select);
            *mode = InteractionMode::Select;
        }
        InteractionMode::ResizingSelection { .. } | InteractionMode::DraggingSelection { .. } => {
            let mut doc_clone = doc_signal.read().clone();
            let did_change = finalize_motion_release(mode, &mut doc_clone, &db_tx);
            if did_change {
                doc_signal.set(doc_clone);
            }
        }
        InteractionMode::Panning { .. } => {
            *mode = InteractionMode::Select;
        }
        InteractionMode::Select => *mode = InteractionMode::Select,
    });
    space_pan_active.set(false);
}
