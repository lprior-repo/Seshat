#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::cast_precision_loss)]
#![forbid(unsafe_code)]

mod canvas_view;
mod interaction_reducer;
mod selection_geometry;

use crate::history::History;
use crate::app::DraggedIconPayload;
use crate::models::dag::validate_dag;
use crate::models::document::{
    ArrowType, DiagramDocument, Edge, EdgeId, EdgeStyle, Node, NodeId, NodeKind, NodeStyle,
    OrderedFloat,
};
use crate::ui::commands::{
    apply_clear_selection, apply_delete_selected, apply_nudge_selection, apply_zoom_in,
    apply_zoom_out, apply_zoom_reset,
};
use crate::ui::editor::ToolMode;
use crate::ui::interaction::{
    drag_original_positions, dragged_positions_with_snap, has_drag_threshold, node_ids_in_rect,
    select_single, snap_point, snap_value, toggle_selection, with_auto_selected_edges,
};
use crate::ui::theme::{
    ACCENT, ACCENT_DASH_BORDER, BG_BASE, BG_ELEVATED, BORDER, EDGE_DEFAULT, EDGE_SELECTED,
    GRID_DOT, NODE_BG, NODE_BG_SUBGRAPH, NODE_BORDER, TEXT_MAIN, TEXT_MUTED, TOOLBAR_BG,
};
use canvas_view::{
    edge_label_position, edge_marker_id, edge_path, edge_preview_overlay, find_edge_at,
    rubber_band_overlay, selection_handles_overlay, subgraph_preview_overlay,
};
use dioxus::html::geometry::WheelDelta;
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use im::HashMap;
use interaction_reducer::{commit_inline_edit, InteractionMode, ResizeHandle};
use selection_geometry::{selected_node_ids, selection_bounds};
use uuid::Uuid;

fn provider_color(provider: &str) -> &'static str {
    match provider {
        "aws" => "#FF9900",
        "gcp" => "#4285F4",
        "azure" => "#0078D4",
        "k8s" => "#326CE5",
        _ => "#6B7280",
    }
}

fn initials(label: &str) -> String {
    let parts = label
        .split(|ch: char| ch.is_whitespace() || ch == '/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    if parts.len() <= 1 {
        return label.chars().take(3).collect::<String>().to_ascii_uppercase();
    }

    parts
        .iter()
        .filter_map(|part| part.chars().next())
        .take(3)
        .collect::<String>()
        .to_ascii_uppercase()
}

fn icon_tags(icon_key: &str) -> Vec<String> {
    let segments = icon_key.split('/').collect::<Vec<_>>();
    if segments.is_empty() {
        Vec::new()
    } else if segments.len() == 1 {
        vec![segments[0].to_string()]
    } else {
        vec![segments[0].to_string(), segments[1].to_string()]
    }
}

fn fallback_icon_label(icon_key: &str) -> String {
    icon_key
        .split('/')
        .next_back()
        .map_or_else(|| String::from("Node"), |part| {
            let mut chars = part.chars();
            chars.next().map_or_else(String::new, |first| {
                let first_up = first.to_ascii_uppercase();
                format!("{first_up}{}", chars.as_str())
            })
        })
}

fn edge_preserves_dag(doc: &DiagramDocument, edge: &Edge) -> bool {
    let candidate_edges = doc
        .document
        .edges
        .update(EdgeId::new(Uuid::new_v4().to_string()), edge.clone());
    validate_dag(&doc.document.nodes, &candidate_edges).is_ok()
}

fn ordered_nodes(doc: &DiagramDocument) -> Vec<(NodeId, Node)> {
    let mut nodes = doc
        .document
        .nodes
        .iter()
        .map(|(id, node)| (id.clone(), node.clone()))
        .collect::<Vec<_>>();

    nodes.sort_by(|(a_id, a_node), (b_id, b_node)| {
        let a_layer = i32::from(a_node.kind != NodeKind::Subgraph);
        let b_layer = i32::from(b_node.kind != NodeKind::Subgraph);
        (a_layer, a_node.z_index, a_id.to_string()).cmp(&(b_layer, b_node.z_index, b_id.to_string()))
    });

    nodes
}

fn find_node_at(doc: &DiagramDocument, x: f64, y: f64) -> Option<NodeId> {
    ordered_nodes(doc)
        .iter()
        .rev()
        .find(|(_, node)| {
            x >= node.x.0
                && x <= node.x.0 + node.width.0
                && y >= node.y.0
                && y <= node.y.0 + node.height.0
        })
        .map(|(id, _)| id.clone())
}

fn scale_selected_nodes(doc: &mut DiagramDocument, factor: f64) -> bool {
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
    let grid = doc.editor_state.grid_size.0;

    for node_id in selected {
        if let Some(node) = doc.document.nodes.get_mut(&node_id) {
            let rel_x = node.x.0 - center_x;
            let rel_y = node.y.0 - center_y;
            let mut next_x = center_x + (rel_x * factor);
            let mut next_y = center_y + (rel_y * factor);
            let mut next_w = (node.width.0 * factor).round().max(24.0);
            let mut next_h = (node.height.0 * factor).round().max(24.0);

            if snap {
                next_x = snap_value(next_x, true, grid);
                next_y = snap_value(next_y, true, grid);
                next_w = snap_value(next_w, true, grid).max(24.0);
                next_h = snap_value(next_h, true, grid).max(24.0);
            }

            node.x = OrderedFloat(next_x);
            node.y = OrderedFloat(next_y);
            node.width = OrderedFloat(next_w);
            node.height = OrderedFloat(next_h);
        }
    }

    true
}

#[component]
pub fn Canvas() -> Element {
    let mut doc_signal = use_context::<Signal<DiagramDocument>>();
    let mut dragging_icon = use_context::<Signal<Option<DraggedIconPayload>>>();
    let mut history_signal = use_context::<Signal<History>>();
    let mut tool_signal = use_context::<Signal<ToolMode>>();
    let edge_style_default = use_context::<Signal<EdgeStyle>>();
    let arrow_type_default = use_context::<Signal<ArrowType>>();

    let mut interaction_mode = use_signal(|| InteractionMode::Select);
    let mut space_pressed = use_signal(|| false);
    let mut shift_pressed = use_signal(|| false);
    let mut ctrl_pressed = use_signal(|| false);
    let mut meta_pressed = use_signal(|| false);
    let mut drag_over = use_signal(|| false);
    let mut hovered_node = use_signal(|| Option::<NodeId>::None);
    let mut editing_node = use_signal(|| Option::<NodeId>::None);
    let mut editing_edge = use_signal(|| Option::<EdgeId>::None);
    let mut edit_value = use_signal(String::new);
    let mut nudge_batch_active = use_signal(|| false);
    let mut viewport_size = use_context::<Signal<(f64, f64)>>();

    let nodes_list = use_memo(move || ordered_nodes(&doc_signal.read()));
    let edges_list = use_memo(move || doc_signal.read().document.edges.clone());

    let to_canvas_coords = |client_x: f64, client_y: f64, cam_x: f64, cam_y: f64, zoom: f64| {
        ((client_x - cam_x) / zoom, (client_y - cam_y) / zoom)
    };

    let to_screen_coords = |world_x: f64, world_y: f64, cam_x: f64, cam_y: f64, zoom: f64| {
        (world_x.mul_add(zoom, cam_x), world_y.mul_add(zoom, cam_y))
    };

    use_effect(move || {
        let mut eval = document::eval(
            r"
                window.addEventListener('keydown', (e) => {
                    const active = document.activeElement;
                    const editing = active && (
                        active.tagName === 'INPUT' ||
                        active.tagName === 'TEXTAREA' ||
                        active.isContentEditable
                    );
                    if (editing) return;
                    const key = e.key;
                    const isArrow = key === 'ArrowUp' || key === 'ArrowDown' || key === 'ArrowLeft' || key === 'ArrowRight';
                    const isZoom = key === '+' || key === '=' || key === '-' || key === '_' || key === '0';
                    const isDelete = key === 'Delete' || key === 'Backspace';
                    const handled = key === ' ' || key === 'Escape' || isArrow || isZoom || isDelete;
                    if (handled) {
                        e.preventDefault();
                    }
                    dioxus.send({ type: 'keydown', key: key, ctrl: e.ctrlKey, shift: e.shiftKey, meta: e.metaKey, repeat: e.repeat });
                });
                window.addEventListener('keyup', (e) => {
                    const active = document.activeElement;
                    const editing = active && (
                        active.tagName === 'INPUT' ||
                        active.tagName === 'TEXTAREA' ||
                        active.isContentEditable
                    );
                    if (editing) return;
                    dioxus.send({ type: 'keyup', key: e.key, ctrl: e.ctrlKey, shift: e.shiftKey, meta: e.metaKey, repeat: false });
                });
            ",
        );

        spawn(async move {
            while let Ok(json) = eval.recv::<serde_json::Value>().await {
                let event_type = json["type"].as_str().map_or("", |s| s);
                let key = json["key"].as_str().map_or("", |s| s);
                let ctrl = json["ctrl"].as_bool().is_some_and(|v| v);
                let meta = json["meta"].as_bool().is_some_and(|v| v);
                let shift = json["shift"].as_bool().is_some_and(|v| v);
                let modifier = ctrl || meta;

                if key == " " {
                    space_pressed.set(event_type == "keydown");
                }
                if key == "Shift" {
                    shift_pressed.set(event_type == "keydown");
                }
                if key == "Control" {
                    ctrl_pressed.set(event_type == "keydown");
                }
                if key == "Meta" {
                    meta_pressed.set(event_type == "keydown");
                }

                if event_type == "keydown" {
                    match key {
                        "Delete" | "Backspace" => {
                            let _ = apply_delete_selected(doc_signal, history_signal);
                        }
                        "Escape" => {
                            if editing_node.read().is_some() || editing_edge.read().is_some() {
                                editing_node.set(None);
                                editing_edge.set(None);
                            } else {
                                let mode = interaction_mode.read().clone();
                                match mode {
                                    InteractionMode::DraggingSelection { did_move, .. } => {
                                        if did_move {
                                            doc_signal.with_mut(|doc| {
                                                doc.revision = doc.revision.increment();
                                            });
                                        }
                                        interaction_mode.set(InteractionMode::Select);
                                    }
                                    InteractionMode::ResizingSelection { did_resize, .. } => {
                                        if did_resize {
                                            doc_signal.with_mut(|doc| {
                                                doc.revision = doc.revision.increment();
                                            });
                                        }
                                        interaction_mode.set(InteractionMode::Select);
                                    }
                                    InteractionMode::Select => {
                                        apply_clear_selection(doc_signal);
                                    }
                                    _ => {
                                        interaction_mode.set(InteractionMode::Select);
                                    }
                                }
                            }
                        }
                        "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" if !modifier => {
                            let step = if shift { 10.0 } else { 1.0 };
                            let (dx, dy) = match key {
                                "ArrowUp" => (0.0, -step),
                                "ArrowDown" => (0.0, step),
                                "ArrowLeft" => (-step, 0.0),
                                _ => (step, 0.0),
                            };
                            let push_undo = !*nudge_batch_active.read();
                            let nudged = apply_nudge_selection(
                                doc_signal,
                                history_signal,
                                dx,
                                dy,
                                push_undo,
                            );
                            if nudged {
                                nudge_batch_active.set(true);
                            }
                        }
                        "+" | "=" if !modifier => {
                            let viewport_size_now = *viewport_size.read();
                            let _ = apply_zoom_in(doc_signal, history_signal, viewport_size_now);
                        }
                        "-" | "_" if !modifier => {
                            let viewport_size_now = *viewport_size.read();
                            let _ = apply_zoom_out(doc_signal, history_signal, viewport_size_now);
                        }
                        "0" if !modifier => {
                            let _ = apply_zoom_reset(doc_signal, history_signal);
                        }
                        "v" | "V" if !modifier => tool_signal.set(ToolMode::Select),
                        "h" | "H" if !modifier => tool_signal.set(ToolMode::Pan),
                        "l" | "L" if !modifier => tool_signal.set(ToolMode::Edge),
                        "r" | "R" if !modifier => tool_signal.set(ToolMode::Subgraph),
                        "t" | "T" if !modifier => tool_signal.set(ToolMode::Text),
                        _ => {}
                    }
                } else if event_type == "keyup"
                    && matches!(key, "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight")
                {
                    nudge_batch_active.set(false);
                }
            }
        });
    });

    use_effect(move || {
        let mut eval = document::eval(
            r"
                const target = document.querySelector('.canvas-container');
                if (target) {
                    if (window.__seshat_canvas_ro) {
                        window.__seshat_canvas_ro.disconnect();
                    }
                    const notify = () => {
                        const r = target.getBoundingClientRect();
                        dioxus.send({ type: 'resize', width: r.width, height: r.height });
                    };
                    const ro = new ResizeObserver(() => notify());
                    ro.observe(target);
                    window.__seshat_canvas_ro = ro;
                    notify();
                }
            ",
        );

        spawn(async move {
            while let Ok(json) = eval.recv::<serde_json::Value>().await {
                if json["type"].as_str() == Some("resize") {
                    let width = json["width"].as_f64().map_or(1200.0, |v| v.max(1.0));
                    let height = json["height"].as_f64().map_or(800.0, |v| v.max(1.0));
                    viewport_size.set((width, height));
                }
            }
        });
    });

    let handle_drop = move |evt: DragEvent| {
        evt.prevent_default();
        drag_over.set(false);

        dragging_icon.with_mut(|dragging| {
            if let Some(payload) = dragging.take() {
                let icon_key = payload.icon_key;
                let current = doc_signal.read().clone();
                let history = history_signal.read().clone();
                *history_signal.write() = history.push(current);
                let derived_label = payload
                    .label
                    .filter(|label| !label.trim().is_empty())
                    .unwrap_or_else(|| fallback_icon_label(&icon_key));
                let tags = icon_tags(&icon_key);

                doc_signal.with_mut(|doc| {
                    let coords = evt.data.coordinates().client();
                    let (x, y) = to_canvas_coords(
                        coords.x,
                        coords.y,
                        doc.editor_state.camera_x.0,
                        doc.editor_state.camera_y.0,
                        doc.editor_state.zoom.0,
                    );
                    let (x, y) = snap_point(
                        (x - 32.0, y - 32.0),
                        doc.editor_state.snap_to_grid,
                        doc.editor_state.grid_size.0,
                    );
                    let id = NodeId::new(Uuid::new_v4().to_string());
                    let _ = doc.document.nodes.insert(
                        id.clone(),
                        Node {
                            kind: NodeKind::Node,
                            icon: icon_key,
                            label: derived_label,
                            x: OrderedFloat(x),
                            y: OrderedFloat(y),
                            width: OrderedFloat(64.0),
                            height: OrderedFloat(64.0),
                            font_size: None,
                            font_weight: None,
                            locked: true,
                            parent: None,
                            dag_rank: None,
                            tags,
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
        });
    };

    let is_dragging = dragging_icon.read().is_some();
    let bg_color = if *drag_over.read() && is_dragging {
        BG_ELEVATED
    } else {
        BG_BASE
    };
    let border_style = if *drag_over.read() && is_dragging {
        ACCENT_DASH_BORDER
    } else {
        "none"
    };
    let cursor_style = {
        let mode = interaction_mode.read().clone();
        let tool = *tool_signal.read();
        match mode {
            InteractionMode::Panning { .. } => "grabbing",
            InteractionMode::DrawingEdge { .. } => "crosshair",
            InteractionMode::ResizingSelection { handle, .. } => match handle {
                ResizeHandle::Nw | ResizeHandle::Se => "nwse-resize",
                ResizeHandle::Ne | ResizeHandle::Sw => "nesw-resize",
                ResizeHandle::N | ResizeHandle::S => "ns-resize",
                ResizeHandle::E | ResizeHandle::W => "ew-resize",
            },
            InteractionMode::DraggingSelection { .. } => "move",
            _ => {
                if *space_pressed.read() || tool == ToolMode::Pan {
                    "grab"
                } else if tool == ToolMode::Edge || tool == ToolMode::Subgraph {
                    "crosshair"
                } else if tool == ToolMode::Text {
                    "text"
                } else {
                    "default"
                }
            }
        }
    };

    rsx! {
        div {
            class: "canvas-container",
            style: "flex: 1; position: relative; overflow: hidden; background: radial-gradient(circle at 24% 12%, {BG_ELEVATED} 0%, {bg_color} 66%); cursor: {cursor_style}; user-select: none; border: {border_style}; box-sizing: border-box;",

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
                let doc = doc_signal.read().clone();
                let pos = to_canvas_coords(
                    coords.x,
                    coords.y,
                    doc.editor_state.camera_x.0,
                    doc.editor_state.camera_y.0,
                    doc.editor_state.zoom.0,
                );

                let hit_node = ordered_nodes(&doc)
                    .iter()
                    .rev()
                    .find(|(_, n)| {
                        pos.0 >= n.x.0
                            && pos.0 <= n.x.0 + n.width.0
                            && pos.1 >= n.y.0
                            && pos.1 <= n.y.0 + n.height.0
                    })
                    .map(|(id, n)| (id.clone(), n.label.clone()));

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
                    editing_node.set(None);
                    editing_edge.set(Some(eid));
                    edit_value.set(label);
                }
            },

            onwheel: move |evt| {
                evt.prevent_default();
                let (dx, dy) = match evt.data.delta() {
                    WheelDelta::Pixels(v) => (v.x, v.y),
                    WheelDelta::Lines(v) => (v.x * 20.0, v.y * 20.0),
                    WheelDelta::Pages(v) => (v.x * 100.0, v.y * 100.0),
                };
                let zoom_gesture = *ctrl_pressed.read() || *meta_pressed.read();
                let shift = *shift_pressed.read();
                let discrete_wheel = dy.abs() >= 50.0;

                doc_signal.with_mut(|doc| {
                    let s = &mut doc.editor_state;
                    if zoom_gesture {
                        let old_zoom = s.zoom.0;
                        let zoom_factor = (-dy * 0.0015).exp();
                        let new_zoom = (s.zoom.0 * zoom_factor).clamp(0.1, 4.0);
                        let coords = evt.data.coordinates().client();
                        let (wx, wy) = to_canvas_coords(
                            coords.x,
                            coords.y,
                            s.camera_x.0,
                            s.camera_y.0,
                            old_zoom,
                        );
                        s.camera_x = OrderedFloat(wx.mul_add(-new_zoom, coords.x));
                        s.camera_y = OrderedFloat(wy.mul_add(-new_zoom, coords.y));
                        s.zoom = OrderedFloat(new_zoom);
                    } else if shift {
                        s.camera_x = OrderedFloat(s.camera_x.0 - dy);
                    } else if discrete_wheel {
                        let old_zoom = s.zoom.0;
                        let zoom_factor = if dy > 0.0 { 0.9 } else { 1.1 };
                        let new_zoom = (old_zoom * zoom_factor).clamp(0.1, 4.0);
                        let coords = evt.data.coordinates().client();
                        let (wx, wy) = to_canvas_coords(
                            coords.x,
                            coords.y,
                            s.camera_x.0,
                            s.camera_y.0,
                            old_zoom,
                        );
                        s.camera_x = OrderedFloat(wx.mul_add(-new_zoom, coords.x));
                        s.camera_y = OrderedFloat(wy.mul_add(-new_zoom, coords.y));
                        s.zoom = OrderedFloat(new_zoom);
                    } else {
                        s.camera_x = OrderedFloat(s.camera_x.0 - dx);
                        s.camera_y = OrderedFloat(s.camera_y.0 - dy);
                    }
                });
            },

            onmousedown: move |evt| {
                if editing_node.read().is_some() || editing_edge.read().is_some() {
                    commit_inline_edit(
                        doc_signal,
                        history_signal,
                        editing_node,
                        editing_edge,
                        edit_value,
                    );
                }
                let coords = evt.data.coordinates().client();
                let is_middle = evt.data.trigger_button() == Some(MouseButton::Auxiliary);
                let is_right = evt.data.trigger_button() == Some(MouseButton::Secondary);
                let tool = *tool_signal.read();

                if *space_pressed.read() || is_middle || is_right || tool == ToolMode::Pan {
                    interaction_mode.set(InteractionMode::Panning { last_pos: (coords.x, coords.y) });
                    return;
                }

                if evt.data.trigger_button() != Some(MouseButton::Primary) {
                    return;
                }

                let pos = {
                    let doc = doc_signal.read();
                    to_canvas_coords(
                        coords.x,
                        coords.y,
                        doc.editor_state.camera_x.0,
                        doc.editor_state.camera_y.0,
                        doc.editor_state.zoom.0,
                    )
                };

                if tool == ToolMode::Select {
                    let doc = doc_signal.read().clone();
                    if let Some(edge_id) = find_edge_at(&doc, pos.0, pos.1) {
                        let shift = *shift_pressed.read();
                        doc_signal.with_mut(|d| {
                            d.editor_state.selected_items = if shift {
                                toggle_selection(&d.editor_state.selected_items, &edge_id.to_string())
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
                                pos,
                                doc.editor_state.snap_to_grid,
                                doc.editor_state.grid_size.0,
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
                                    locked: true,
                                    parent: None,
                                    dag_rank: None,
                                    tags: Vec::new(),
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
                            pos,
                            doc.editor_state.snap_to_grid,
                            doc.editor_state.grid_size.0,
                        );
                        interaction_mode.set(InteractionMode::DrawingSubgraph {
                            start: snapped_start,
                            current: snapped_start,
                        });
                    }
                    ToolMode::Select => {
                        if !*shift_pressed.read() {
                            doc_signal.with_mut(|d| d.editor_state.selected_items.clear());
                        }
                        interaction_mode.set(InteractionMode::RubberBand { start: pos, current: pos });
                    }
                    ToolMode::Edge | ToolMode::Pan => {}
                }
            },

            onmousemove: move |evt| {
                let coords = evt.data.coordinates().client();
                interaction_mode.with_mut(|mode| {
                    match mode {
                        InteractionMode::DraggingSelection {
                            anchor,
                            original_positions,
                            did_move,
                        } => {
                            doc_signal.with_mut(|doc| {
                                let (curr_x, curr_y) = to_canvas_coords(
                                    coords.x,
                                    coords.y,
                                    doc.editor_state.camera_x.0,
                                    doc.editor_state.camera_y.0,
                                    doc.editor_state.zoom.0,
                                );
                                if !*did_move && has_drag_threshold(*anchor, (curr_x, curr_y)) {
                                    let history = history_signal.read().clone();
                                    *history_signal.write() = history.push(doc.clone());
                                    *did_move = true;
                                }
                                if *did_move {
                                    let positions = dragged_positions_with_snap(
                                        original_positions,
                                        *anchor,
                                        (curr_x, curr_y),
                                        doc.editor_state.snap_to_grid,
                                        doc.editor_state.grid_size.0,
                                    );
                                    for (id, (nx, ny)) in positions.iter() {
                                        doc.document.nodes = doc.document.nodes.alter(
                                            |n| {
                                                n.map(|mut node| {
                                                    node.x = OrderedFloat(*nx);
                                                    node.y = OrderedFloat(*ny);
                                                    node.locked = true;
                                                    node
                                                })
                                            },
                                            id.clone(),
                                        );
                                    }
                                }
                            });
                        }
                        InteractionMode::DrawingEdge { current_pos, .. } => {
                            let doc = doc_signal.read();
                            *current_pos = to_canvas_coords(
                                coords.x,
                                coords.y,
                                doc.editor_state.camera_x.0,
                                doc.editor_state.camera_y.0,
                                doc.editor_state.zoom.0,
                            );
                        }
                        InteractionMode::RubberBand { current, .. }
                        | InteractionMode::DrawingSubgraph { current, .. } => {
                            let doc = doc_signal.read();
                            let raw = to_canvas_coords(
                                coords.x,
                                coords.y,
                                doc.editor_state.camera_x.0,
                                doc.editor_state.camera_y.0,
                                doc.editor_state.zoom.0,
                            );
                            *current = snap_point(
                                raw,
                                doc.editor_state.snap_to_grid,
                                doc.editor_state.grid_size.0,
                            );
                        }
                        InteractionMode::ResizingSelection {
                            handle,
                            original_bounds,
                            originals,
                            anchor,
                            did_resize,
                        } => {
                            let (mx, my) = {
                                let doc = doc_signal.read();
                                to_canvas_coords(
                                    coords.x,
                                    coords.y,
                                    doc.editor_state.camera_x.0,
                                    doc.editor_state.camera_y.0,
                                    doc.editor_state.zoom.0,
                                )
                            };
                            let dx = mx - anchor.0;
                            let dy = my - anchor.1;
                            let (dx, dy) = {
                                let doc = doc_signal.read();
                                snap_point(
                                    (dx, dy),
                                    doc.editor_state.snap_to_grid,
                                    doc.editor_state.grid_size.0,
                                )
                            };

                            if !*did_resize && (dx != 0.0 || dy != 0.0) {
                                let history = history_signal.read().clone();
                                *history_signal.write() = history.push(doc_signal.read().clone());
                                *did_resize = true;
                            }

                            let (obx, oby, obw, obh) = *original_bounds;
                            let north =
                                *handle == ResizeHandle::Nw || *handle == ResizeHandle::N || *handle == ResizeHandle::Ne;
                            let south =
                                *handle == ResizeHandle::Sw || *handle == ResizeHandle::S || *handle == ResizeHandle::Se;
                            let west =
                                *handle == ResizeHandle::Nw || *handle == ResizeHandle::W || *handle == ResizeHandle::Sw;
                            let east =
                                *handle == ResizeHandle::Ne || *handle == ResizeHandle::E || *handle == ResizeHandle::Se;

                            let nx = if west { obx + dx } else { obx };
                            let ny = if north { oby + dy } else { oby };
                            let nw = if west {
                                obw - dx
                            } else if east {
                                obw + dx
                            } else {
                                obw
                            }
                            .max(24.0);
                            let nh = if north {
                                obh - dy
                            } else if south {
                                obh + dy
                            } else {
                                obh
                            }
                            .max(24.0);

                            let scale_x = if obw > 0.0 { nw / obw } else { 1.0 };
                            let scale_y = if obh > 0.0 { nh / obh } else { 1.0 };

                            if *did_resize {
                                doc_signal.with_mut(|d| {
                                    for (id, (ox, oy, ow, oh)) in originals.iter() {
                                        if let Some(node) = d.document.nodes.get_mut(id) {
                                            let nxx = (ox - obx).mul_add(scale_x, nx);
                                            let nyy = (oy - oby).mul_add(scale_y, ny);
                                            let nww = (ow * scale_x).max(24.0);
                                            let nhh = (oh * scale_y).max(24.0);
                                            let snap = d.editor_state.snap_to_grid;
                                            let grid = d.editor_state.grid_size.0;
                                            node.x = OrderedFloat(snap_value(nxx, snap, grid));
                                            node.y = OrderedFloat(snap_value(nyy, snap, grid));
                                            node.width = OrderedFloat(snap_value(nww, snap, grid).max(24.0));
                                            node.height = OrderedFloat(snap_value(nhh, snap, grid).max(24.0));
                                        }
                                    }
                                });
                            }
                        }
                        InteractionMode::Panning { last_pos } => {
                            let dx = coords.x - last_pos.0;
                            let dy = coords.y - last_pos.1;
                            *last_pos = (coords.x, coords.y);
                            doc_signal.with_mut(|doc| {
                                doc.editor_state.camera_x = OrderedFloat(doc.editor_state.camera_x.0 + dx);
                                doc.editor_state.camera_y = OrderedFloat(doc.editor_state.camera_y.0 + dy);
                            });
                        }
                        InteractionMode::Select => {}
                    }
                });
            },

            onmouseup: move |evt| {
                interaction_mode.with_mut(|mode| {
                    match mode {
                        InteractionMode::DrawingEdge { from_node, .. } => {
                            let coords = evt.data.coordinates().client();
                            let doc = doc_signal.read().clone();
                            let pos = to_canvas_coords(
                                coords.x,
                                coords.y,
                                doc.editor_state.camera_x.0,
                                doc.editor_state.camera_y.0,
                                doc.editor_state.zoom.0,
                            );
                            let target = find_node_at(&doc, pos.0, pos.1);
                            if let Some(target_id) = target.clone() {
                                if &target_id != from_node {
                                    let candidate_edge = Edge {
                                        source: from_node.clone(),
                                        target: target_id,
                                        label: String::new(),
                                        style: *edge_style_default.read(),
                                        arrow_type: *arrow_type_default.read(),
                                        label_offset_t: OrderedFloat(0.5),
                                        color: None,
                                        thickness: OrderedFloat(1.5),
                                        directed: true,
                                        bend_points: Vec::new(),
                                        tags: Vec::new(),
                                        metadata: HashMap::new(),
                                        font_size: None,
                                    };

                                    if !edge_preserves_dag(&doc, &candidate_edge) {
                                        if *tool_signal.read() == ToolMode::Edge {
                                            if let Some(target_id) = target {
                                                *mode = InteractionMode::DrawingEdge {
                                                    from_node: target_id,
                                                    current_pos: pos,
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
                                        current_pos: pos,
                                    };
                                } else {
                                    *mode = InteractionMode::Select;
                                }
                            } else {
                                *mode = InteractionMode::Select;
                            }
                        }
                        InteractionMode::RubberBand { start, current } => {
                            let shift = *shift_pressed.read();
                            doc_signal.with_mut(|doc| {
                                let boxed = node_ids_in_rect(doc, *start, *current);
                                let selected = if shift {
                                    boxed
                                        .iter()
                                        .fold(doc.editor_state.selected_items.clone(), |acc, id| {
                                            toggle_selection(&acc, id)
                                        })
                                } else {
                                    boxed
                                };
                                doc.editor_state.selected_items = with_auto_selected_edges(doc, &selected);
                            });
                            *mode = InteractionMode::Select;
                        }
                        InteractionMode::DrawingSubgraph { start, current } => {
                            let doc_now = doc_signal.read().clone();
                            let snap = doc_now.editor_state.snap_to_grid;
                            let grid = doc_now.editor_state.grid_size.0;
                            let mut x = start.0.min(current.0);
                            let mut y = start.1.min(current.1);
                            let mut w = (start.0 - current.0).abs();
                            let mut h = (start.1 - current.1).abs();
                            if snap {
                                x = snap_value(x, true, grid);
                                y = snap_value(y, true, grid);
                                w = snap_value(w, true, grid).max(grid.max(20.0));
                                h = snap_value(h, true, grid).max(grid.max(20.0));
                            }
                            if w > 20.0 && h > 20.0 {
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
                                            locked: true,
                                            parent: None,
                                            dag_rank: None,
                                            tags: Vec::new(),
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
                        InteractionMode::ResizingSelection { did_resize, .. } => {
                            if *did_resize {
                                doc_signal.with_mut(|doc| {
                                    doc.revision = doc.revision.increment();
                                });
                            }
                            *mode = InteractionMode::Select;
                        }
                        InteractionMode::DraggingSelection { did_move, .. } => {
                            if *did_move {
                                doc_signal.with_mut(|doc| {
                                    doc.revision = doc.revision.increment();
                                });
                            }
                            *mode = InteractionMode::Select;
                        }
                        _ => *mode = InteractionMode::Select,
                    }
                });
            },

            onmouseleave: move |_| {
                interaction_mode.with_mut(|mode| {
                    match mode {
                        InteractionMode::DraggingSelection { did_move, .. } => {
                            if *did_move {
                                doc_signal.with_mut(|doc| {
                                    doc.revision = doc.revision.increment();
                                });
                            }
                            *mode = InteractionMode::Select;
                        }
                        InteractionMode::ResizingSelection { did_resize, .. } => {
                            if *did_resize {
                                doc_signal.with_mut(|doc| {
                                    doc.revision = doc.revision.increment();
                                });
                            }
                            *mode = InteractionMode::Select;
                        }
                        InteractionMode::Panning { .. }
                        | InteractionMode::DrawingEdge { .. }
                        | InteractionMode::DrawingSubgraph { .. }
                        | InteractionMode::RubberBand { .. } => {
                            *mode = InteractionMode::Select;
                        }
                        InteractionMode::Select => {}
                    }
                });
            },

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

                {
                    let doc = doc_signal.read();
                    let s = &doc.editor_state;
                    let (vw, vh) = *viewport_size.read();
                    let pattern_step = (s.grid_size.0.max(8.0) * s.zoom.0).max(4.0);
                    let pattern_x = s.camera_x.0.rem_euclid(pattern_step);
                    let pattern_y = s.camera_y.0.rem_euclid(pattern_step);
                    let dot_r = if s.zoom.0 >= 0.75 { 1.0 } else { 0.8 };

                    if s.show_grid && s.zoom.0 >= 0.3 {
                        rsx! {
                            defs {
                                pattern {
                                    id: "canvas-grid-dot-pattern",
                                    pattern_units: "userSpaceOnUse",
                                    x: "{pattern_x}",
                                    y: "{pattern_y}",
                                    width: "{pattern_step}",
                                    height: "{pattern_step}",
                                    circle {
                                        cx: "0",
                                        cy: "0",
                                        r: "{dot_r}",
                                        fill: "{GRID_DOT}",
                                    }
                                }
                            }
                            rect {
                                x: "0",
                                y: "0",
                                width: "{vw.max(1.0)}",
                                height: "{vh.max(1.0)}",
                                fill: "url(#canvas-grid-dot-pattern)",
                            }
                        }
                    } else {
                        rsx! {}
                    }
                }

                {
                    let doc = doc_signal.read();
                    let s = &doc.editor_state;
                    edges_list.read().iter().filter_map(|(id, edge)| {
                        doc.document.nodes
                            .get(&edge.source)
                            .zip(doc.document.nodes.get(&edge.target))
                            .map(|(src, tgt)| {
                                let (sx, sy) = to_screen_coords(src.x.0 + src.width.0 / 2.0, src.y.0 + src.height.0 / 2.0, s.camera_x.0, s.camera_y.0, s.zoom.0);
                                let (tx, ty) = to_screen_coords(tgt.x.0 + tgt.width.0 / 2.0, tgt.y.0 + tgt.height.0 / 2.0, s.camera_x.0, s.camera_y.0, s.zoom.0);
                                let d = edge_path(sx, sy, tx, ty, edge);
                                let (mid_x, mid_y) = edge_label_position(sx, sy, tx, ty, edge);
                                let is_selected = s.selected_items.contains(&id.to_string());
                                let stroke_color = if is_selected {
                                    EDGE_SELECTED
                                } else {
                                    EDGE_DEFAULT
                                };
                                let stroke_width = if is_selected { 2.5 } else { 1.5 };
                                let marker_name = edge_marker_id(edge.arrow_type, is_selected);
                                let marker = format!("url(#{marker_name})");
                                let dash = if edge.style == EdgeStyle::Dashed {
                                    "8,4"
                                } else if edge.style == EdgeStyle::Dotted {
                                    "2,4"
                                } else {
                                    ""
                                };
                                let font_size = edge.font_size.map_or(10.0, |f| f.0) * s.zoom.0;
                                let is_editing_edge = editing_edge.read().as_ref() == Some(id);
                                rsx! {
                                    path {
                                        key: "{id:?}",
                                        d: "{d}",
                                        fill: "none",
                                        stroke: "{stroke_color}",
                                        stroke_width: "{stroke_width}",
                                        stroke_dasharray: "{dash}",
                                        marker_end: "{marker}",
                                    }
                                    if is_editing_edge {
                                        foreignObject {
                                            x: "{mid_x - 50.0}",
                                            y: "{mid_y - 12.0}",
                                            width: "100",
                                            height: "24",
                                            input {
                                                value: "{edit_value}",
                                                style: "width:100px; height:22px; padding:2px 6px; border-radius:4px; border:1px solid {ACCENT}; background:{BG_BASE}; color:{TEXT_MAIN}; font-size:11px;",
                                                onmousedown: move |evt| evt.stop_propagation(),
                                                oninput: move |evt| edit_value.set(evt.value()),
                                                onblur: move |_| {
                                                    commit_inline_edit(
                                                        doc_signal,
                                                        history_signal,
                                                        editing_node,
                                                        editing_edge,
                                                        edit_value,
                                                    );
                                                },
                                                onkeydown: move |evt| {
                                                    if evt.key() == Key::Enter {
                                                        commit_inline_edit(
                                                            doc_signal,
                                                            history_signal,
                                                            editing_node,
                                                            editing_edge,
                                                            edit_value,
                                                        );
                                                    } else if evt.key() == Key::Escape {
                                                        editing_edge.set(None);
                                                    }
                                                }
                                            }
                                        }
                                    } else if !edge.label.is_empty() {
                                        text {
                                            x: "{mid_x}",
                                            y: "{mid_y - 6.0}",
                                            text_anchor: "middle",
                                            style: "fill:{TEXT_MUTED}; font-size:{font_size}px;",
                                            "{edge.label}"
                                        }
                                    } else if is_selected {
                                        text {
                                            x: "{mid_x}",
                                            y: "{mid_y - 6.0}",
                                            text_anchor: "middle",
                                            style: "fill:{TEXT_MUTED}; font-size:9px; opacity:0.6;",
                                            "label"
                                        }
                                    }
                                }
                            })
                    }).collect::<Vec<_>>().into_iter()
                }

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

            {
                let editor_for_nodes = doc_signal.read().editor_state.clone();
                let hovered_now = hovered_node.read().clone();
                nodes_list.read().iter().map(|(id, node)| {
                    let id_mousedown = id.clone();
                    let id_mouseup = id.clone();
                    let id_mouseenter = id.clone();
                    let id_mouseleave = id.clone();
                    let is_selected = editor_for_nodes.selected_items.contains(&id.to_string());
                    let (left, top) = to_screen_coords(
                        node.x.0,
                        node.y.0,
                        editor_for_nodes.camera_x.0,
                        editor_for_nodes.camera_y.0,
                        editor_for_nodes.zoom.0,
                    );
                    let (width, height) = (
                        node.width.0 * editor_for_nodes.zoom.0,
                        node.height.0 * editor_for_nodes.zoom.0,
                    );
                    let is_hovered = hovered_now.as_ref() == Some(id);
                    let border = if is_selected {
                        format!("2px solid {ACCENT}")
                    } else if is_hovered {
                        format!("1px solid color-mix(in oklch, {ACCENT} 50%, transparent)")
                    } else {
                        format!("1px solid {NODE_BORDER}")
                    };
                    let bg = if node.kind == NodeKind::Subgraph { NODE_BG_SUBGRAPH } else { NODE_BG };
                    let z_index = node.z_index + if node.kind == NodeKind::Subgraph { 0 } else { 1000 };
                    let is_editing_node = editing_node.read().as_ref() == Some(id);
                    let font_px = node.font_size.map_or(11.0, |f| f.0) * editor_for_nodes.zoom.0;
                    let fallback_provider = node.icon.split('/').next().map_or("generic", |p| p);
                    let provider = node.tags.first().map_or(fallback_provider, |p| p.as_str());
                    let provider_top = provider_color(provider);
                    let node_initials = initials(&node.label);

                    rsx! {
                        div {
                            key: "{id:?}",
                            style: "position: absolute; left: {left}px; top: {top}px; width: {width}px; height: {height}px; border: {border}; border-radius: 10px; background: linear-gradient(180deg, color-mix(in oklch, {bg} 92%, {BG_BASE}) 0%, {bg} 100%); display: flex; flex-direction: column; align-items: center; justify-content: center; cursor: inherit; z-index: {z_index}; box-shadow: 0 6px 18px color-mix(in oklch, black 24%, transparent);",

                            onmouseenter: {
                                move |_| hovered_node.set(Some(id_mouseenter.clone()))
                            },
                            onmouseleave: move |_| {
                                if hovered_node.read().as_ref() == Some(&id_mouseleave) {
                                    hovered_node.set(None);
                                }
                            },

                            onmousedown: move |evt| {
                                evt.stop_propagation();
                                let tool = *tool_signal.read();
                                let doc = doc_signal.read().clone();
                                let shift = *shift_pressed.read();
                                let is_middle = evt.data.trigger_button() == Some(MouseButton::Auxiliary);
                                let is_right = evt.data.trigger_button() == Some(MouseButton::Secondary);
                                let is_primary = evt.data.trigger_button() == Some(MouseButton::Primary);
                                let coords = evt.data.coordinates().client();
                                let pos = to_canvas_coords(coords.x, coords.y, doc.editor_state.camera_x.0, doc.editor_state.camera_y.0, doc.editor_state.zoom.0);

                                if *space_pressed.read() || is_middle || is_right || tool == ToolMode::Pan {
                                    interaction_mode.set(InteractionMode::Panning { last_pos: (coords.x, coords.y) });
                                    return;
                                }

                                if !is_primary {
                                    return;
                                }

                                if tool == ToolMode::Edge {
                                    interaction_mode.set(InteractionMode::DrawingEdge { from_node: id_mousedown.clone(), current_pos: pos });
                                } else {
                                    let was_selected = doc.editor_state.selected_items.contains(&id_mousedown.to_string());

                                    doc_signal.with_mut(|d| {
                                        let selected = if shift {
                                            toggle_selection(&d.editor_state.selected_items, &id_mousedown.to_string())
                                        } else if !was_selected {
                                            select_single(id_mousedown.to_string())
                                        } else {
                                            d.editor_state.selected_items.clone()
                                        };
                                        d.editor_state.selected_items = with_auto_selected_edges(d, &selected);
                                    });

                                    let current_doc = doc_signal.read().clone();
                                    let original_positions =
                                        drag_original_positions(&current_doc, &current_doc.editor_state.selected_items);
                                    interaction_mode.set(InteractionMode::DraggingSelection {
                                        anchor: pos,
                                        original_positions,
                                        did_move: false,
                                    });
                                }
                            },

                            onmouseup: move |evt| {
                                evt.stop_propagation();
                                let mode = interaction_mode.read().clone();
                                match mode {
                                    InteractionMode::DrawingEdge { from_node, .. } => {
                                        if from_node != id_mouseup {
                                            let doc_now = doc_signal.read().clone();
                                            let candidate_edge = Edge {
                                                source: from_node,
                                                target: id_mouseup.clone(),
                                                label: String::new(),
                                                style: *edge_style_default.read(),
                                                arrow_type: *arrow_type_default.read(),
                                                label_offset_t: OrderedFloat(0.5),
                                                color: None,
                                                thickness: OrderedFloat(1.5),
                                                directed: true,
                                                bend_points: Vec::new(),
                                                tags: Vec::new(),
                                                metadata: HashMap::new(),
                                                font_size: None,
                                            };

                                            if edge_preserves_dag(&doc_now, &candidate_edge) {
                                                let history = history_signal.read().clone();
                                                *history_signal.write() = history.push(doc_now);
                                                doc_signal.with_mut(|doc| {
                                                    doc.document.edges = doc.document.edges.update(
                                                        EdgeId::new(Uuid::new_v4().to_string()),
                                                        candidate_edge,
                                                    );
                                                    doc.revision = doc.revision.increment();
                                                });
                                            }
                                        }
                                        if *tool_signal.read() == ToolMode::Edge {
                                            let doc_now = doc_signal.read().clone();
                                            let coords = evt.data.coordinates().client();
                                            let pos = to_canvas_coords(
                                                coords.x,
                                                coords.y,
                                                doc_now.editor_state.camera_x.0,
                                                doc_now.editor_state.camera_y.0,
                                                doc_now.editor_state.zoom.0,
                                            );
                                            interaction_mode.set(InteractionMode::DrawingEdge {
                                                from_node: id_mouseup.clone(),
                                                current_pos: pos,
                                            });
                                        } else {
                                            interaction_mode.set(InteractionMode::Select);
                                        }
                                    }
                                    InteractionMode::DraggingSelection { did_move, .. } => {
                                        if did_move {
                                            doc_signal.with_mut(|doc| {
                                                doc.revision = doc.revision.increment();
                                            });
                                        }
                                        interaction_mode.set(InteractionMode::Select);
                                    }
                                    InteractionMode::ResizingSelection { did_resize, .. } => {
                                        if did_resize {
                                            doc_signal.with_mut(|doc| {
                                                doc.revision = doc.revision.increment();
                                            });
                                        }
                                        interaction_mode.set(InteractionMode::Select);
                                    }
                                    _ => {}
                                }

                                if *tool_signal.read() != ToolMode::Edge {
                                    tool_signal.set(ToolMode::Select);
                                }
                            },

                            if node.kind == NodeKind::Text {
                                if is_editing_node {
                                    input {
                                        value: "{edit_value}",
                                        style: "width: 100%; padding: 2px 4px; border-radius: 2px; border: 1px solid {ACCENT}; background: transparent; color: {TEXT_MAIN}; font-size: {font_px}px; text-align: center;",
                                        onmousedown: move |evt| evt.stop_propagation(),
                                        oninput: move |evt| edit_value.set(evt.value()),
                                        onblur: move |_| {
                                            commit_inline_edit(
                                                doc_signal,
                                                history_signal,
                                                editing_node,
                                                editing_edge,
                                                edit_value,
                                            );
                                        },
                                        onkeydown: move |evt| {
                                            if evt.key() == Key::Enter {
                                                commit_inline_edit(
                                                    doc_signal,
                                                    history_signal,
                                                    editing_node,
                                                    editing_edge,
                                                    edit_value,
                                                );
                                            } else if evt.key() == Key::Escape {
                                                editing_node.set(None);
                                            }
                                        }
                                    }
                                } else {
                                    span {
                                        style: "font-size: {font_px}px; color: {TEXT_MAIN};",
                                        ondoubleclick: {
                                            let edit_id = id.clone();
                                            let edit_label = node.label.clone();
                                            move |evt| {
                                                evt.stop_propagation();
                                                editing_edge.set(None);
                                                editing_node.set(Some(edit_id.clone()));
                                                edit_value.set(edit_label.clone());
                                            }
                                        },
                                        "{node.label}"
                                    }
                                }
                            } else if node.kind == NodeKind::Subgraph {
                                div {
                                    style: "position:absolute; inset:0; border-radius:10px; border:2px dashed {BORDER}; background: color-mix(in oklch, {TEXT_MUTED} 14%, transparent);"
                                }
                                if is_editing_node {
                                    input {
                                        value: "{edit_value}",
                                        style: "position:absolute; top:8px; left:8px; right:8px; width: calc(100% - 16px); padding: 2px 4px; border-radius: 4px; border: 1px solid {ACCENT}; background: {BG_BASE}; color: {TEXT_MAIN}; font-size: {font_px}px;",
                                        onmousedown: move |evt| evt.stop_propagation(),
                                        oninput: move |evt| edit_value.set(evt.value()),
                                        onblur: move |_| {
                                            commit_inline_edit(
                                                doc_signal,
                                                history_signal,
                                                editing_node,
                                                editing_edge,
                                                edit_value,
                                            );
                                        },
                                        onkeydown: move |evt| {
                                            if evt.key() == Key::Enter {
                                                commit_inline_edit(
                                                    doc_signal,
                                                    history_signal,
                                                    editing_node,
                                                    editing_edge,
                                                    edit_value,
                                                );
                                            } else if evt.key() == Key::Escape {
                                                editing_node.set(None);
                                            }
                                        }
                                    }
                                } else {
                                    span {
                                        style: "position:absolute; top:8px; left:10px; font-size:{font_px}px; color:{TEXT_MUTED};",
                                        ondoubleclick: {
                                            let edit_id = id.clone();
                                            let edit_label = node.label.clone();
                                            move |evt| {
                                                evt.stop_propagation();
                                                editing_edge.set(None);
                                                editing_node.set(Some(edit_id.clone()));
                                                edit_value.set(edit_label.clone());
                                            }
                                        },
                                        "{node.label}"
                                    }
                                }
                            } else {
                                div {
                                    style: "position:absolute; left:0; right:0; top:0; height:4px; border-radius:8px 8px 0 0; background:{provider_top}; opacity:0.75;"
                                }

                                {
                                    let hovered_match = hovered_node.read().as_ref() == Some(&id_mouseup);
                                    let show_connection_dots = is_selected
                                        || hovered_match
                                        || *tool_signal.read() == ToolMode::Edge;
                                    if show_connection_dots {
                                        let cx = width / 2.0;
                                        let cy = height / 2.0;
                                        let dot_specs = [
                                            (cx, 0.0),
                                            (cx, height),
                                            (0.0, cy),
                                            (width, cy),
                                        ];
                                        rsx! {
                                            for (dot_x, dot_y) in dot_specs {
                                                div {
                                                    style: "position:absolute; left:{dot_x - 10.0}px; top:{dot_y - 10.0}px; width:20px; height:20px; border-radius:999px; background: transparent; cursor: crosshair;",
                                                    onmousedown: {
                                                        let current_id = id.clone();
                                                        move |evt| {
                                                            if evt.data.trigger_button() != Some(MouseButton::Primary) {
                                                                return;
                                                            }
                                                            evt.stop_propagation();
                                                            let coords = evt.data.coordinates().client();
                                                            let doc = doc_signal.read().clone();
                                                            let mouse_pos = to_canvas_coords(
                                                                coords.x,
                                                                coords.y,
                                                                doc.editor_state.camera_x.0,
                                                                doc.editor_state.camera_y.0,
                                                                doc.editor_state.zoom.0,
                                                            );
                                                            interaction_mode.set(InteractionMode::DrawingEdge {
                                                                from_node: current_id.clone(),
                                                                current_pos: mouse_pos,
                                                            });
                                                        }
                                                    },
                                                    div {
                                                        style: "position:absolute; left:5px; top:5px; width:10px; height:10px; border-radius:999px; background:{ACCENT}; border:1px solid {BG_BASE}; opacity:0.9; pointer-events:none;"
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        rsx! {}
                                    }
                                }

                                span {
                                    style: "font-size: {font_px * 1.1}px; color: {provider_top}; font-weight: 700; font-family: monospace;",
                                    "{node_initials}"
                                }
                                if is_editing_node {
                                    input {
                                        value: "{edit_value}",
                                        style: "position:absolute; left:6px; right:6px; bottom:-22px; width: calc(100% - 12px); padding: 2px 4px; border-radius: 4px; border: 1px solid {ACCENT}; background: {BG_BASE}; color: {TEXT_MAIN}; font-size: {font_px}px; text-align:center;",
                                        onmousedown: move |evt| evt.stop_propagation(),
                                        oninput: move |evt| edit_value.set(evt.value()),
                                        onblur: move |_| {
                                            commit_inline_edit(
                                                doc_signal,
                                                history_signal,
                                                editing_node,
                                                editing_edge,
                                                edit_value,
                                            );
                                        },
                                        onkeydown: move |evt| {
                                            if evt.key() == Key::Enter {
                                                commit_inline_edit(
                                                    doc_signal,
                                                    history_signal,
                                                    editing_node,
                                                    editing_edge,
                                                    edit_value,
                                                );
                                            } else if evt.key() == Key::Escape {
                                                editing_node.set(None);
                                            }
                                        }
                                    }
                                } else {
                                    span {
                                        style: "position:absolute; left:0; right:0; bottom:-18px; text-align:center; font-size:{font_px}px; color:{TEXT_MAIN};",
                                        ondoubleclick: {
                                            let edit_id = id.clone();
                                            let edit_label = node.label.clone();
                                            move |evt| {
                                                evt.stop_propagation();
                                                editing_edge.set(None);
                                                editing_node.set(Some(edit_id.clone()));
                                                edit_value.set(edit_label.clone());
                                            }
                                        },
                                        "{node.label}"
                                    }
                                }
                            }
                        }
                    }
                }).collect::<Vec<_>>().into_iter()
            }

            {
                let doc = doc_signal.read().clone();
                selection_handles_overlay(
                    &doc,
                    interaction_mode,
                    doc_signal,
                    to_screen_coords,
                )
            }

            {
                let doc = doc_signal.read().clone();
                let mode = interaction_mode.read().clone();
                let hide_toolbar = matches!(
                    mode,
                    InteractionMode::DraggingSelection { .. }
                        | InteractionMode::ResizingSelection { .. }
                        | InteractionMode::RubberBand { .. }
                );
                let selected_nodes = selected_node_ids(&doc);
                let selected_edges = doc
                    .editor_state
                    .selected_items
                    .iter()
                    .filter(|id| doc.document.edges.contains_key(&EdgeId::new((*id).clone())))
                    .count();

                if hide_toolbar {
                    rsx! {}
                } else if let Some((bx, by, bw, _bh)) = selection_bounds(&doc) {
                    let s = &doc.editor_state;
                    let (screen_x, screen_y) = to_screen_coords(
                        bx + (bw / 2.0),
                        by,
                        s.camera_x.0,
                        s.camera_y.0,
                        s.zoom.0,
                    );
                    let top = (screen_y - 48.0).max(4.0);
                    let can_scale = !selected_nodes.is_empty();
                    let scale_cursor = if can_scale { "pointer" } else { "not-allowed" };
                    let scale_opacity = if can_scale { "1" } else { "0.5" };
                    let first_node = selected_nodes.first().and_then(|id| doc.document.nodes.get(id));
                    let first_edge = if selected_nodes.is_empty() && selected_edges == 1 {
                        doc.editor_state
                            .selected_items
                            .iter()
                            .find_map(|id| doc.document.edges.get(&EdgeId::new(id.clone())))
                    } else {
                        None
                    };
                    let font_size = first_node
                        .and_then(|n| n.font_size)
                        .or_else(|| first_edge.and_then(|e| e.font_size))
                        .map_or(11.0, |v| v.0);
                    rsx! {
                        div {
                            style: "position:absolute; left:{screen_x}px; top:{top}px; transform: translateX(-50%); z-index:25; display:flex; align-items:center; gap:6px; padding:6px 8px; border:1px solid {BORDER}; border-radius:8px; background:{TOOLBAR_BG}f2; backdrop-filter: blur(6px);",
                            onmousedown: move |evt| evt.stop_propagation(),

                            button {
                                style: "border:1px solid {BORDER}; border-radius:6px; background:{BG_BASE}; color:{TEXT_MAIN}; width:24px; height:24px; cursor:pointer;",
                                onclick: move |_| {
                                    let selected = doc_signal.read().editor_state.selected_items.clone();
                                    let current = doc_signal.read().clone();
                                    let history = history_signal.read().clone();
                                    *history_signal.write() = history.push(current);
                                    doc_signal.with_mut(|d| {
                                        for selected_id in selected.iter() {
                                            let nid = NodeId::new(selected_id.clone());
                                            if let Some(node) = d.document.nodes.get_mut(&nid) {
                                                let next = node.font_size.map_or(10.0, |v| v.0) - 1.0;
                                                node.font_size = Some(OrderedFloat(next.clamp(8.0, 72.0)));
                                            } else {
                                                let eid = EdgeId::new(selected_id.clone());
                                                if let Some(edge) = d.document.edges.get_mut(&eid) {
                                                    let next = edge.font_size.map_or(10.0, |v| v.0) - 1.0;
                                                    edge.font_size = Some(OrderedFloat(next.clamp(8.0, 72.0)));
                                                }
                                            }
                                        }
                                        d.revision = d.revision.increment();
                                    });
                                },
                                "-"
                            }
                            span { style: "font-size:11px; color:{TEXT_MUTED}; min-width:26px; text-align:center;", "{font_size.round()}" }
                            button {
                                style: "border:1px solid {BORDER}; border-radius:6px; background:{BG_BASE}; color:{TEXT_MAIN}; width:24px; height:24px; cursor:pointer;",
                                onclick: move |_| {
                                    let selected = doc_signal.read().editor_state.selected_items.clone();
                                    let current = doc_signal.read().clone();
                                    let history = history_signal.read().clone();
                                    *history_signal.write() = history.push(current);
                                    doc_signal.with_mut(|d| {
                                        for selected_id in selected.iter() {
                                            let nid = NodeId::new(selected_id.clone());
                                            if let Some(node) = d.document.nodes.get_mut(&nid) {
                                                let next = node.font_size.map_or(10.0, |v| v.0) + 1.0;
                                                node.font_size = Some(OrderedFloat(next.clamp(8.0, 72.0)));
                                            } else {
                                                let eid = EdgeId::new(selected_id.clone());
                                                if let Some(edge) = d.document.edges.get_mut(&eid) {
                                                    let next = edge.font_size.map_or(10.0, |v| v.0) + 1.0;
                                                    edge.font_size = Some(OrderedFloat(next.clamp(8.0, 72.0)));
                                                }
                                            }
                                        }
                                        d.revision = d.revision.increment();
                                    });
                                },
                                "+"
                            }

                            button {
                                style: "border:1px solid {BORDER}; border-radius:6px; background:{BG_BASE}; color:{TEXT_MAIN}; padding:0 8px; height:24px; cursor:{scale_cursor}; opacity:{scale_opacity};",
                                disabled: !can_scale,
                                onclick: move |_| {
                                    let current = doc_signal.read().clone();
                                    let history = history_signal.read().clone();
                                    *history_signal.write() = history.push(current);
                                    doc_signal.with_mut(|d| {
                                        if scale_selected_nodes(d, 0.8) {
                                            d.revision = d.revision.increment();
                                        }
                                    });
                                },
                                "Shrink"
                            }
                            button {
                                style: "border:1px solid {BORDER}; border-radius:6px; background:{BG_BASE}; color:{TEXT_MAIN}; padding:0 8px; height:24px; cursor:{scale_cursor}; opacity:{scale_opacity};",
                                disabled: !can_scale,
                                onclick: move |_| {
                                    let current = doc_signal.read().clone();
                                    let history = history_signal.read().clone();
                                    *history_signal.write() = history.push(current);
                                    doc_signal.with_mut(|d| {
                                        if scale_selected_nodes(d, 1.25) {
                                            d.revision = d.revision.increment();
                                        }
                                    });
                                },
                                "Grow"
                            }
                        }
                    }
                } else {
                    rsx! {}
                }
            }

            {
                let selected_count = doc_signal.read().editor_state.selected_items.len();
                let plural = if selected_count == 1 { "" } else { "s" };
                if selected_count > 0 {
                    rsx! {
                        div {
                            style: "position:absolute; left:12px; bottom:12px; z-index:20; border:1px solid {BORDER}; border-radius:8px; background:{TOOLBAR_BG}e8; color:{TEXT_MUTED}; font-size:11px; padding:5px 9px; backdrop-filter: blur(6px); box-shadow: 0 4px 12px color-mix(in oklch, black 24%, transparent);",
                            "{selected_count} item{plural} selected"
                        }
                    }
                } else {
                    rsx! {}
                }
            }
        }
    }
}
