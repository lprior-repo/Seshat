use crate::ui::canvas::document_ops::{
    apply_rubber_band_release, dispatch_drag_move_batch, edge_preserves_dag, find_node_at,
    flush_pending_pointer_update, snapped_edge_ports, subgraph_release_bounds, sync_canvas_origin,
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
    let history_signal = state.history_signal;
    let pending_pointer_sample = state.pending_pointer_sample;
    let db_tx = state.db_tx;
    let mut space_pan_active = state.space_pan_active;

    flush_pending_pointer_update(
        doc_signal,
        history_signal,
        interaction_mode,
        pending_pointer_sample,
        state.geometry_render_tick,
        db_tx,
    );

    let toast = crate::ui::toast::use_toast();

    interaction_mode.with_mut(|mode| match mode {
        InteractionMode::DrawingEdge {
            from_node,
            start_port,
            ..
        } => {
            let next_mode =
                handle_drawing_edge_release(&state, &evt, from_node.clone(), *start_port, toast);
            *mode = next_mode;
        }
        InteractionMode::RubberBand { start, current } => {
            let additive = *state.shift_pressed.read()
                || *state.ctrl_pressed.read()
                || *state.meta_pressed.read();
            doc_signal.with_mut(|doc| {
                apply_rubber_band_release(doc, *start, *current, additive);
            });
            *mode = InteractionMode::Select;
        }
        InteractionMode::DrawingSubgraph { start, current } => {
            let next_mode = handle_drawing_subgraph_release(&state, *start, *current);
            *mode = next_mode;
        }
        InteractionMode::ResizingSelection { .. } | InteractionMode::DraggingSelection { .. } => {
            let mut doc_clone = doc_signal.read().clone();
            let original_positions = match mode {
                InteractionMode::DraggingSelection {
                    original_positions, ..
                } => Some(original_positions.clone()),
                _ => None,
            };
            if finalize_motion_release(mode, &mut doc_clone, &db_tx) {
                if let Some(original_positions) = original_positions.as_ref() {
                    dispatch_drag_move_batch(original_positions, &doc_clone, &db_tx);
                }
                doc_signal.set(doc_clone);
            }
        }
        InteractionMode::Panning { .. }
        | InteractionMode::DraggingBendPoint { .. }
        | InteractionMode::Select => {
            *mode = InteractionMode::Select;
        }
    });
    space_pan_active.set(false);
}

fn handle_drawing_edge_release(
    state: &CanvasState,
    evt: &Event<dioxus::prelude::MouseData>,
    from_node: NodeId,
    start_port: Option<diagram_models::port::PortAnchor>,
    toast: crate::ui::toast::ToastApi,
) -> InteractionMode {
    let mut doc_signal = state.doc_signal;
    let mut history_signal = state.history_signal;
    let coords = evt.data.coordinates().client();
    let origin = sync_canvas_origin().unwrap_or_else(|| *state.canvas_origin.read());
    let doc = doc_signal.read().clone();
    let pos = to_canvas_coords(
        canvas_domain::ScreenCoord(coords.x - origin.0, coords.y - origin.1),
        canvas_domain::CanvasCoord(doc.editor_state.camera_x.0, doc.editor_state.camera_y.0),
        doc.editor_state.zoom.0,
    );
    let target = find_node_at(&doc, pos.0, pos.1);

    if let Some(target_id) = target.clone() {
        if target_id != from_node {
            let (calculated_start_port, end_port) = calculate_ports(&doc, &from_node, &target_id);
            let candidate_edge = diagram_models::document::Edge {
                source: from_node.clone(),
                target: target_id.clone(),
                label: String::new(),
                style: *state.edge_style_default.read(),
                arrow_type: *state.arrow_type_default.read(),
                label_offset_t: OrderedFloat(0.5),
                color: None,
                thickness: OrderedFloat(1.5),
                directed: true,
                bend_points: im::Vector::new(),
                tags: im::Vector::new(),
                metadata: HashMap::new(),
                font_size: None,
                source_port: start_port.or(calculated_start_port),
                target_port: end_port,
            };

            if !edge_preserves_dag(&doc, &candidate_edge) {
                let _ = toast.show(
                    crate::ui::toast::ToastIntent::Warning,
                    "Cannot create circular connection",
                    None,
                );
                if *state.tool_signal.read() == ToolMode::Edge {
                    return InteractionMode::DrawingEdge {
                        from_node: target_id,
                        current_pos: (pos.0, pos.1),
                        start_port: end_port,
                    };
                }
                return InteractionMode::Select;
            }

            let history = history_signal.read().clone();
            *history_signal.write() = history.push(doc);
            doc_signal.with_mut(|doc_mut| {
                doc_mut.document.edges = doc_mut
                    .document
                    .edges
                    .update(EdgeId::new(Uuid::new_v4().to_string()), candidate_edge);
                doc_mut.revision = doc_mut.revision.increment();
            });
        }
    }

    if *state.tool_signal.read() == ToolMode::Edge {
        if let Some(target_id) = target {
            let (_, end_port) = calculate_ports(&doc_signal.read(), &from_node, &target_id);
            return InteractionMode::DrawingEdge {
                from_node: target_id,
                current_pos: (pos.0, pos.1),
                start_port: end_port,
            };
        }
    }
    InteractionMode::Select
}

fn calculate_ports(
    doc: &diagram_models::document::DiagramDocument,
    source_id: &NodeId,
    target_id: &NodeId,
) -> (
    Option<diagram_models::port::PortAnchor>,
    Option<diagram_models::port::PortAnchor>,
) {
    doc.document
        .nodes
        .get(source_id)
        .zip(doc.document.nodes.get(target_id))
        .map_or((None, None), |(source, target)| {
            let (start, end) = snapped_edge_ports(source, target);
            (Some(start), Some(end))
        })
}

fn handle_drawing_subgraph_release(
    state: &CanvasState,
    start: (f64, f64),
    current: (f64, f64),
) -> InteractionMode {
    let mut doc_signal = state.doc_signal;
    let mut history_signal = state.history_signal;
    let mut tool_signal = state.tool_signal;

    let doc_now = doc_signal.read().clone();
    let snap = doc_now.editor_state.snap_to_grid;
    let grid = doc_now.editor_state.grid_size;
    if let Some((x, y, w, h)) = subgraph_release_bounds(start, current, snap, grid) {
        let id = NodeId::new(Uuid::new_v4().to_string());
        let history = history_signal.read().clone();
        *history_signal.write() = history.push(doc_now);
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
    InteractionMode::Select
}
