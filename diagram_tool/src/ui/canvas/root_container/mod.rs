use crate::app::DraggedIconPayload;
use crate::history::History;
use crate::ui::canvas::canvas_view::find_edge_at;
use crate::ui::canvas::canvas_view::{
    edge_preview_overlay, rubber_band_overlay, subgraph_preview_overlay,
};
use crate::ui::canvas::document_ops::*;
use crate::ui::canvas::edge_layer::EdgeLayer;
use crate::ui::canvas::grid_layer::GridLayer;
use crate::ui::canvas::node_layer::NodeLayer;
use crate::ui::canvas::state::CanvasState;
use crate::ui::canvas::toolbar::{SelectionPill, Toolbar};
use crate::ui::editor::ToolMode;
use crate::ui::grid::{snap_point, snap_value};
use crate::ui::interaction::{
    drag_original_positions, dragged_positions_with_snap, has_drag_threshold, node_ids_in_rect,
    select_single, toggle_selection, with_auto_selected_edges,
};
use crate::ui::theme::{BG_BASE, BG_ELEVATED, EDGE_DEFAULT, EDGE_SELECTED};
use crate::ui::toast::use_toast;
use canvas_domain::interaction_reducer::{
    commit_inline_edit, finalize_motion_release, InteractionMode, ResizeHandle,
};
use canvas_domain::perf::{to_canvas_coords, to_screen_coords};
use canvas_domain::selection_geometry::{selected_node_ids, selection_bounds};
use diagram_models::document::{
    ArrowType, DiagramDocument, EdgeId, EdgeStyle, LockState, Node, NodeId, NodeKind, NodeStyle,
    OrderedFloat,
};
use dioxus::html::geometry::WheelDelta;
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::DragData;
use dioxus::prelude::*;
use im::HashMap;
use std::collections::HashSet;
use uuid::Uuid;

#[component]
pub fn RootContainer(state: CanvasState) -> Element {
    let mut doc_signal = state.doc_signal;
    let mut dragging_icon = state.dragging_icon;
    let mut history_signal = state.history_signal;
    let mut tool_signal = state.tool_signal;
    let edge_style_default = state.edge_style_default;
    let arrow_type_default = state.arrow_type_default;
    let mut interaction_mode = state.interaction_mode;
    let mut space_pressed = state.space_pressed;
    let mut shift_pressed = state.shift_pressed;
    let mut ctrl_pressed = state.ctrl_pressed;
    let mut meta_pressed = state.meta_pressed;
    let mut drag_over = state.drag_over;
    let mut hovered_node = state.hovered_node;
    let mut editing_node = state.editing_node;
    let mut editing_edge = state.editing_edge;
    let mut edit_value = state.edit_value;
    let mut nudge_batch_active = state.nudge_batch_active;
    let mut space_pan_active = state.space_pan_active;
    let viewport_size = state.viewport_size;
    let mut pending_pointer_sample = state.pending_pointer_sample;
    let mut pending_wheel_sample = state.pending_wheel_sample;
    let mut multi_touch_active = state.multi_touch_active;
    let mut captured_pointer = state.captured_pointer;
    let mut active_pointers = state.active_pointers;
    let mut canvas_origin = state.canvas_origin;
    let ordered_node_cache = state.ordered_node_cache;
    let db_tx = state.db_tx.clone();

    let doc_for_bg = doc_signal.read();
    let bg_color = if doc_for_bg.editor_state.show_grid {
        BG_BASE
    } else {
        BG_BASE
    };
    let border_style = if *drag_over.read() {
        "2px dashed #8b5cf6"
    } else {
        "none"
    };
    let cursor_style = if *space_pressed.read() {
        if *space_pan_active.read() {
            "grabbing"
        } else {
            "grab"
        }
    } else {
        match *interaction_mode.read() {
            InteractionMode::Panning { .. } => "grabbing",
            InteractionMode::DrawingEdge { .. } => "crosshair",
            InteractionMode::RubberBand { .. } => "crosshair",
            InteractionMode::ResizingSelection { handle, .. } => match handle {
                ResizeHandle::Nw | ResizeHandle::Se => "nwse-resize",
                ResizeHandle::Ne | ResizeHandle::Sw => "nesw-resize",
                ResizeHandle::N | ResizeHandle::S => "ns-resize",
                ResizeHandle::E | ResizeHandle::W => "ew-resize",
            },
            InteractionMode::DraggingSelection { .. } => "move",
            _ => "default",
        }
    };

    let toast = use_toast();
    let handle_drop = move |evt: Event<dioxus::prelude::DragData>| {
        evt.prevent_default();
        drag_over.set(false);
    };

    rsx! {
            div {
                class: "canvas-container",
                "data-testid": "canvas-root",
                style: "flex: 1; position: relative; overflow: hidden; overscroll-behavior: none; touch-action: none; background: radial-gradient(circle at 24% 12%, {BG_ELEVATED} 0%, {bg_color} 66%); cursor: {cursor_style}; user-select: none; border: {border_style}; box-sizing: border-box;",

                ondragover: move |evt| { evt.prevent_default(); },
                ondragenter: move |_| { drag_over.set(true); },
                ondragleave: move |_| { drag_over.set(false); },
                ondrop: handle_drop,
                oncontextmenu: move |evt| evt.prevent_default(),
                onauxclick: move |evt| {
                    if evt.data.trigger_button() == Some(MouseButton::Auxiliary) {
                        evt.prevent_default();
                    }
                },
                ondoubleclick: move |evt| {
                    let coords = evt.data.coordinates().client();
                    let origin = sync_canvas_origin().unwrap_or_else(|| *canvas_origin.read());
                    let local_x = coords.x - origin.0;
                    let local_y = coords.y - origin.1;
                    let doc = doc_signal.read().clone();
                                let pos = to_canvas_coords(
                                    canvas_domain::ScreenCoord(local_x, local_y),
                                    canvas_domain::CanvasCoord(doc.editor_state.camera_x.0, doc.editor_state.camera_y.0),
                                    doc.editor_state.zoom.0,
                                );

                    let hit_node = ordered_node_cache
                        .read()
                        .iter()
                        .rev()
                        .find_map(|id| {
                            doc.document.nodes.get(&id).and_then(|node| {
                                (pos.0 >= node.x.0
                                    && pos.0 <= node.x.0 + node.width.0
                                    && pos.1 >= node.y.0
                                    && pos.1 <= node.y.0 + node.height.0)
                                    .then(|| (id.clone(), node.label.clone()))
                            })
                        })
                        ;

                    if let Some((nid, label)) = hit_node {
                        editing_edge.set(None);
                        editing_node.set(Some(nid));
                        edit_value.set(label);
                        return;
                    }

                    if let Some(eid) = find_edge_at(&doc, pos.0, pos.1) {
                        let label = doc
                            .document
                            .edges
                            .get(&eid)
                            .map_or_else(String::new, |e| e.label.clone());
                        doc_signal.with_mut(|d| {
                            d.editor_state.selected_items = select_single(eid.to_string());
                        });
                        editing_node.set(None);
                        editing_edge.set(Some(eid));
                        edit_value.set(label);
                        return;
                    }

                    // Double-click on empty canvas creates a new node in Select mode
                    let tool = *tool_signal.read();
                    if tool == ToolMode::Select {
                        let id = NodeId::new(Uuid::new_v4().to_string());
                        let current = doc_signal.read().clone();
                        let history = history_signal.read().clone();
                        *history_signal.write() = history.push(current);
                        doc_signal.with_mut(|d| {
                            let (x, y) = snap_point(
                                (pos.0, pos.1),
                                d.editor_state.snap_to_grid,
                                d.editor_state.grid_size.into(),
                            );
                            let _ = d.document.nodes.insert(
                                id.clone(),
                                Node {
                                    kind: NodeKind::Node,
                                    icon: String::new(),
                                    label: String::from("Node"),
                                    x: OrderedFloat(x - 32.0),
                                    y: OrderedFloat(y - 32.0),
                                    width: OrderedFloat(64.0),
                                    height: OrderedFloat(64.0),
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
                            d.editor_state.selected_items.clear();
                            let _ = d.editor_state.selected_items.insert(id.to_string());
                            d.revision = d.revision.increment();
                        });
                        editing_edge.set(None);
                        editing_node.set(None);
                        edit_value.set(String::new());
                    }
                },

                onwheel: move |evt| {
                    if *multi_touch_active.read() {
                        return;
                    }
                    evt.prevent_default();
                    let (dx, dy, discrete_wheel) = match evt.data.delta() {
                        WheelDelta::Pixels(v) => (v.x, v.y, v.y.abs() >= 50.0),
                        WheelDelta::Lines(v) => (v.x, v.y, false),
                        WheelDelta::Pages(v) => (v.x, v.y, false),
                    };
                    let coords = evt.data.coordinates().client();
                    let origin = sync_canvas_origin().unwrap_or_else(|| *canvas_origin.read());
                    let local_x = coords.x - origin.0;
                    let local_y = coords.y - origin.1;
                    pending_wheel_sample.set(Some(WheelSample {
                        client_x: local_x,
                        client_y: local_y,
                        dx,
                        dy,
                        zoom_gesture: *ctrl_pressed.read() || *meta_pressed.read(),
                        shift_pan: *shift_pressed.read(),
                        discrete_wheel,
                    }));
                },

                onmousedown: move |evt| {
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
                        space_pan_active.set(*space_pressed.read() && !is_middle && !is_right && tool != ToolMode::Pan);
                        interaction_mode.set(InteractionMode::Panning { last_pos: (local_x, local_y) });
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
        doc.editor_state.zoom.0
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
                                doc.editor_state.grid_size.into(),
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
                            interaction_mode.set(InteractionMode::RubberBand { start: (pos.0, pos.1), current: (pos.0, pos.1) });
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
                },

                onmousemove: move |evt| {
                    let coords = evt.data.coordinates().client();
                    let origin = sync_canvas_origin().unwrap_or_else(|| *canvas_origin.read());
                    let local_x = coords.x - origin.0;
                    let local_y = coords.y - origin.1;
                    interaction_mode.with_mut(|mode| {
                        match mode {
                            InteractionMode::DrawingEdge { current_pos, .. } => {
                                let doc = doc_signal.read();
                                let canvas_domain::CanvasCoord(px, py) = to_canvas_coords(
        canvas_domain::ScreenCoord(local_x, local_y),
        canvas_domain::CanvasCoord(doc.editor_state.camera_x.0, doc.editor_state.camera_y.0),
        doc.editor_state.zoom.0
                                );
                                *current_pos = (px, py);
                            }
                            InteractionMode::RubberBand { current, .. }
                            | InteractionMode::DrawingSubgraph { current, .. } => {
                                let doc = doc_signal.read();
                                let raw = to_canvas_coords(
                                    canvas_domain::ScreenCoord(local_x, local_y),
                                    canvas_domain::CanvasCoord(doc.editor_state.camera_x.0, doc.editor_state.camera_y.0),
                                    doc.editor_state.zoom.0,
                                );
                                *current = snap_point(
                                    (raw.0, raw.1),
                                    doc.editor_state.snap_to_grid,
                                    doc.editor_state.grid_size.into(),
                                );
                            }
                            InteractionMode::DraggingSelection { .. }
                            | InteractionMode::ResizingSelection { .. }
                            | InteractionMode::Panning { .. } => {
                                pending_pointer_sample.set(Some((local_x, local_y)));
                            }
                            InteractionMode::Select => {}
                        }
                    });
                },

                onmouseup: move |evt| {
                    flush_pending_pointer_update(
                        doc_signal,
                        history_signal,
                        interaction_mode,
                        pending_pointer_sample,
                        db_tx,
                    );
                    interaction_mode.with_mut(|mode| {
                        match mode {
                            InteractionMode::DrawingEdge { from_node, .. } => {
                                let coords = evt.data.coordinates().client();
                                let origin = sync_canvas_origin().unwrap_or_else(|| *canvas_origin.read());
                                let local_x = coords.x - origin.0;
                                let local_y = coords.y - origin.1;
                                let doc = doc_signal.read().clone();
                                let pos = to_canvas_coords(canvas_domain::ScreenCoord(local_x, local_y), canvas_domain::CanvasCoord(doc.editor_state.camera_x.0, doc.editor_state.camera_y.0), doc.editor_state.zoom.0);
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
                                            doc_mut.document.edges = doc_mut.document.edges.update(
                                                EdgeId::new(Uuid::new_v4().to_string()),
                                                candidate_edge,
                                            );
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
                                if let Some((x, y, w, h)) =
                                    subgraph_release_bounds(*start, *current, snap, grid.into())
                                {
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
                            InteractionMode::ResizingSelection { .. }
                            | InteractionMode::DraggingSelection { .. } => {
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
                        }
                    });
                    space_pan_active.set(false);
                },

                onmouseleave: move |_| {},

                div {
                    "data-testid": "canvas-hit-layer",
                    style: "position:absolute; inset:0; pointer-events:none; opacity:0;"
                }

                svg {
                    style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; pointer-events: none; z-index: 0;",
                    defs {
                        marker {
                            id: "arrowhead",
                            marker_width: "10",
                            marker_height: "7",
                            ref_x: "9",
                            ref_y: "3.5",
                            orient: "auto",
                            polygon { points: "0 0, 10 3.5, 0 7", fill: "{EDGE_DEFAULT}" }
                        }
                        marker {
                            id: "arrowhead-selected",
                            marker_width: "10",
                            marker_height: "7",
                            ref_x: "9",
                            ref_y: "3.5",
                            orient: "auto",
                            polygon { points: "0 0, 10 3.5, 0 7", fill: "{EDGE_SELECTED}" }
                        }
                        marker {
                            id: "arrow-pending",
                            marker_width: "10",
                            marker_height: "7",
                            ref_x: "9",
                            ref_y: "3.5",
                            orient: "auto",
                            polygon { points: "0 0, 10 3.5, 0 7", fill: "{EDGE_SELECTED}", opacity: "0.5" }
                        }
                    }

                    GridLayer { doc_signal: doc_signal.into(), viewport_size: viewport_size.into() }

                    EdgeLayer { doc_signal: doc_signal.into(), history_signal: history_signal.into(), editing_node: editing_node.into(), editing_edge: editing_edge.into(), edit_value: edit_value.into(), viewport_size: viewport_size.into(), db_tx: db_tx.clone() }

                    {
                        let mode_now = interaction_mode.read().clone();
                        let doc_now = doc_signal.read();
                        let edge_overlay = edge_preview_overlay(&mode_now, &doc_now, to_screen_coords);
                        let band_overlay = rubber_band_overlay(&mode_now, &doc_now, to_screen_coords);
                        let subgraph_overlay = subgraph_preview_overlay(&mode_now, &doc_now, to_screen_coords);
                        rsx! {
                            {edge_overlay}
                            {band_overlay}
                            {subgraph_overlay}
                        }
                    }
                }

                NodeLayer { doc_signal: doc_signal.into(), history_signal: history_signal.into(), tool_signal: tool_signal.into(), interaction_mode: interaction_mode.into(), editing_node: editing_node.into(), editing_edge: editing_edge.into(), edit_value: edit_value.into(), hovered_node: hovered_node.into(), viewport_size: viewport_size.into(), ordered_node_cache: ordered_node_cache.into(), canvas_origin: canvas_origin.into(), shift_pressed: shift_pressed.into(), ctrl_pressed: ctrl_pressed.into(), meta_pressed: meta_pressed.into(), space_pressed: space_pressed.into(), multi_touch_active: multi_touch_active.into(), space_pan_active: space_pan_active.into(), db_tx: db_tx.clone() }

                Toolbar { doc_signal: doc_signal.into(), history_signal: history_signal.into(), interaction_mode: interaction_mode.into() }
                SelectionPill { doc_signal: doc_signal.into() }
            }
        }
}
