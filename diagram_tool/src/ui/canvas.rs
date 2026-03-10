#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::cast_precision_loss)]
#![forbid(unsafe_code)]

mod canvas_view;
pub mod drag_math;
mod interaction_reducer;
pub mod math;
pub mod domain;
mod perf;
mod selection_geometry;

use base64::{engine::general_purpose, Engine as _};
use canvas_view::{
    edge_label_position, edge_marker_ref, edge_path, edge_preview_overlay, find_edge_at,
    rubber_band_overlay, selection_handles_overlay, subgraph_preview_overlay,
};
use dioxus::{
    html::{geometry::WheelDelta, input_data::MouseButton},
    prelude::*,
};
use im::HashMap;
use interaction_reducer::{
    commit_inline_edit, finalize_motion_release, InteractionMode, ResizeHandle,
};
use perf::{
    normalize_viewport, to_canvas_coords, to_screen_coords, viewport_changed, wheel_update,
    WheelInput,
};
use selection_geometry::{selected_node_ids, selection_bounds};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    app::DraggedIconPayload,
    history::History,
    icons::{icon_index, ICONS},
    models::{
        dag::validate_dag,
        document::{
            ArrowType, DiagramDocument, Edge, EdgeId, EdgeStyle, Node, NodeId, NodeKind, NodeStyle,
            OrderedFloat, Revision,
        },
    },
    ui::{
        commands::{
            apply_clear_selection, apply_delete_selected, apply_nudge_selection, apply_zoom_in,
            apply_zoom_out, apply_zoom_reset,
        },
        editor::ToolMode,
        interaction::{
            drag_original_positions, dragged_positions_with_snap, has_drag_threshold,
            node_ids_in_rect, select_single, toggle_selection, with_auto_selected_edges,
        },
        theme::{
            ACCENT, ACCENT_DASH_BORDER, BG_BASE, BG_ELEVATED, BORDER, EDGE_DEFAULT, EDGE_SELECTED,
            GRID_DOT, NODE_BG, NODE_BG_SUBGRAPH, NODE_BORDER, TEXT_MAIN, TEXT_MUTED, TOOLBAR_BG,
        },
        toast::use_toast,
    },
};

use crate::ui::grid::{snap_point, snap_value, GridSize};

#[cfg(target_arch = "wasm32")]
pub fn sync_canvas_origin() -> Option<(f64, f64)> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let el = document
        .query_selector(".canvas-container")
        .ok()
        .flatten()?;
    let rect = el.get_bounding_client_rect();
    Some((rect.left(), rect.top()))
}

#[cfg(not(target_arch = "wasm32"))]
#[must_use]
pub const fn sync_canvas_origin() -> Option<(f64, f64)> {
    None
}

/// Fallback for provider color mapping
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
        return label
            .chars()
            .take(3)
            .collect::<String>()
            .to_ascii_uppercase();
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
    icon_key.split('/').next_back().map_or_else(
        || String::from("Node"),
        |part| {
            let mut chars = part.chars();
            chars.next().map_or_else(String::new, |first| {
                let first_up = first.to_ascii_uppercase();
                format!("{first_up}{}", chars.as_str())
            })
        },
    )
}

fn data_url_for_relpath(file_relpath: &str) -> Option<String> {
    let file = ICONS.get_file(file_relpath)?;
    let mime = std::path::Path::new(file_relpath)
        .extension()
        .and_then(|ext| ext.to_str())
        .map_or("image/png", |ext| {
            if ext.eq_ignore_ascii_case("svg") {
                "image/svg+xml"
            } else {
                "image/png"
            }
        });

    Some(format!(
        "data:{mime};base64,{}",
        general_purpose::STANDARD.encode(file.contents())
    ))
}

fn icon_data_url(icon_key: &str) -> Option<String> {
    icon_index()
        .by_key
        .get(icon_key)
        .and_then(|meta| data_url_for_relpath(&meta.file_relpath))
        .or_else(|| data_url_for_relpath(icon_key))
}

fn node_image_data_url(node: &Node) -> Option<String> {
    node.metadata
        .get("icon_data_url")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| icon_data_url(&node.icon))
}

fn edge_preserves_dag(doc: &DiagramDocument, edge: &Edge) -> bool {
    let candidate_edges = doc
        .document
        .edges
        .update(EdgeId::new(Uuid::new_v4().to_string()), edge.clone());
    validate_dag(&doc.document.nodes, &candidate_edges).is_ok()
}

fn ordered_node_ids(doc: &DiagramDocument) -> Vec<NodeId> {
    let mut node_ids = doc.document.nodes.keys().cloned().collect::<Vec<_>>();
    node_ids.sort_by(|a_id, b_id| {
        doc.document
            .nodes
            .get(a_id)
            .zip(doc.document.nodes.get(b_id))
            .map_or(std::cmp::Ordering::Equal, |(a_node, b_node)| {
                let a_layer = i32::from(a_node.kind != NodeKind::Subgraph);
                let b_layer = i32::from(b_node.kind != NodeKind::Subgraph);
                (a_layer, a_node.z_index, a_id.to_string()).cmp(&(
                    b_layer,
                    b_node.z_index,
                    b_id.to_string(),
                ))
            })
    });

    node_ids
}

fn find_node_at(doc: &DiagramDocument, x: f64, y: f64) -> Option<NodeId> {
    ordered_node_ids(doc)
        .iter()
        .rev()
        .find(|id| {
            doc.document.nodes.get(*id).is_some_and(|node| {
                x >= node.x.0
                    && x <= node.x.0 + node.width.0
                    && y >= node.y.0
                    && y <= node.y.0 + node.height.0
            })
        })
        .cloned()
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
    let grid = doc.editor_state.grid_size;
    let mut changed = false;

    for node_id in selected {
        if let Some(node) = doc.document.nodes.get_mut(&node_id) {
            if node.locked && node.kind != NodeKind::Subgraph {
                continue;
            }
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
            changed = true;
        }
    }

    changed
}

fn apply_rubber_band_release(
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

fn subgraph_release_bounds(
    start: (f64, f64),
    current: (f64, f64),
    snap: bool,
    grid: GridSize,
) -> Option<(f64, f64, f64, f64)> {
    let mut x = start.0.min(current.0);
    let mut y = start.1.min(current.1);
    let mut w = (start.0 - current.0).abs();
    let mut h = (start.1 - current.1).abs();
    let grid_inner = grid.inner();
    if snap {
        x = snap_value(x, true, grid);
        y = snap_value(y, true, grid);
        w = snap_value(w, true, grid).max(grid_inner.max(20.0));
        h = snap_value(h, true, grid).max(grid_inner.max(20.0));
    }

    (w > 20.0 && h > 20.0).then_some((x, y, w, h))
}

fn safe_zoom(zoom: f64) -> f64 {
    math::safe_zoom(zoom).unwrap_or(1.0)
}

fn fit_icon_side(side: f64) -> f64 {
    if !side.is_finite() {
        return 0.0;
    }

    let max = (side - 8.0).max(0.0);
    let min = 20.0_f64.min(max);
    let preferred = side * 0.52;

    if !preferred.is_finite() {
        return min;
    }

    preferred.clamp(min, max)
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct WheelSample {
    client_x: f64,
    client_y: f64,
    dx: f64,
    dy: f64,
    zoom_gesture: bool,
    shift_pan: bool,
    discrete_wheel: bool,
}

fn flush_pending_wheel_update(
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
fn flush_pending_pointer_update(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    mut interaction_mode: Signal<InteractionMode>,
    mut pending_pointer_sample: Signal<Option<(f64, f64)>>,
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
            let (curr_x, curr_y) = to_canvas_coords(
                client_x,
                client_y,
                doc.editor_state.camera_x.0,
                doc.editor_state.camera_y.0,
                doc.editor_state.zoom.0,
            );

            let has_movable_nodes = original_positions.keys().any(|id| {
                doc.document
                    .nodes
                    .get(id)
                    .is_some_and(|node| !node.locked || node.kind == NodeKind::Subgraph)
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
                    (curr_x, curr_y),
                    doc.editor_state.snap_to_grid,
                    doc.editor_state.grid_size,
                );
                let has_changes = positions.iter().any(|(id, (nx, ny))| {
                    doc.document.nodes.get(id).is_some_and(|node| {
                        !node.locked
                            && ((node.x.0 - *nx).abs() > f64::EPSILON
                                || (node.y.0 - *ny).abs() > f64::EPSILON)
                    })
                });

                if has_changes {
                    doc_signal.with_mut(|doc_mut| {
                        for (id, (nx, ny)) in positions.iter() {
                            let should_update =
                                doc_mut.document.nodes.get(id).is_some_and(|node| {
                                    !node.locked
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
                }
            }
        }
        InteractionMode::ResizingSelection {
            handle,
            original_bounds,
            originals,
            anchor,
            did_resize,
        } => {
            let doc_for_mouse = doc_signal.read().clone();
            let (mx, my) = to_canvas_coords(
                client_x,
                client_y,
                doc_for_mouse.editor_state.camera_x.0,
                doc_for_mouse.editor_state.camera_y.0,
                safe_zoom(doc_for_mouse.editor_state.zoom.0),
            );
            let delta_x_raw = mx - anchor.0;
            let delta_y_raw = my - anchor.1;
            let snap = doc_for_mouse.editor_state.snap_to_grid;
            let grid = doc_for_mouse.editor_state.grid_size;
            let dx = snap_value(delta_x_raw, snap, grid);
            let dy = snap_value(delta_y_raw, snap, grid);

            let has_resizable_nodes = originals.keys().any(|id| {
                doc_for_mouse
                    .document
                    .nodes
                    .get(id)
                    .is_some_and(|node| !node.locked || node.kind == NodeKind::Subgraph)
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
                let nw = if west {
                    obw - dx_clamped
                } else if east {
                    obw + dx_clamped
                } else {
                    obw
                }
                .max(24.0);
                let nh = if north {
                    obh - dy_clamped
                } else if south {
                    obh + dy_clamped
                } else {
                    obh
                }
                .max(24.0);

                let scale_x = if obw > 0.0 { nw / obw } else { 1.0 };
                let scale_y = if obh > 0.0 { nh / obh } else { 1.0 };

                doc_signal.with_mut(|doc_mut| {
                    for (id, (ox, oy, ow, oh)) in originals.iter() {
                        if let Some(node) = doc_mut.document.nodes.get_mut(id) {
                            if node.locked && node.kind != NodeKind::Subgraph {
                                continue;
                            }
                            let nxx = (ox - obx).mul_add(scale_x, nx);
                            let nyy = (oy - oby).mul_add(scale_y, ny);
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

#[component]
pub fn Canvas() -> Element {
    let mut doc_signal = use_context::<Signal<DiagramDocument>>();
    let mut dragging_icon = use_context::<Signal<Option<DraggedIconPayload>>>();
    let mut history_signal = use_context::<Signal<History>>();
    let mut tool_signal = use_context::<Signal<ToolMode>>();
    let edge_style_default = use_context::<Signal<EdgeStyle>>();
    let arrow_type_default = use_context::<Signal<ArrowType>>();
    let toast = use_toast();

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
    let mut space_pan_active = use_signal(|| false);
    let mut viewport_size = use_context::<Signal<(f64, f64)>>();
    let mut pending_pointer_sample = use_signal(|| Option::<(f64, f64)>::None);
    let mut pending_wheel_sample = use_signal(|| Option::<WheelSample>::None);
    let mut multi_touch_active = use_signal(|| false);
    let mut canvas_origin = use_signal(|| (0.0_f64, 0.0_f64));
    let mut ordered_node_cache = use_signal(Vec::<NodeId>::new);
    let mut ordered_node_revision = use_signal(|| Option::<Revision>::None);

    use_effect(move || {
        let doc = doc_signal.read();
        let revision = doc.revision;
        if ordered_node_revision.read().as_ref() != Some(&revision) {
            ordered_node_cache.set(ordered_node_ids(&doc));
            ordered_node_revision.set(Some(revision));
        }
    });

    use_effect(move || {
        let mut eval = document::eval(
            r"
                if (window.__seshat_canvas_keyboard_cleanup) {
                    window.__seshat_canvas_keyboard_cleanup();
                }

                const onKeyDown = (e) => {
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
                };

                const onKeyUp = (e) => {
                    const active = document.activeElement;
                    const editing = active && (
                        active.tagName === 'INPUT' ||
                        active.tagName === 'TEXTAREA' ||
                        active.isContentEditable
                    );
                    if (editing) return;
                    dioxus.send({ type: 'keyup', key: e.key, ctrl: e.ctrlKey, shift: e.shiftKey, meta: e.metaKey, repeat: false });
                };

                const onWindowBlur = () => {
                    dioxus.send({ type: 'blur', key: '', ctrl: false, shift: false, meta: false, repeat: false });
                };

                window.addEventListener('keydown', onKeyDown);
                window.addEventListener('keyup', onKeyUp);
                window.addEventListener('blur', onWindowBlur);
                window.__seshat_canvas_keyboard_cleanup = () => {
                    window.removeEventListener('keydown', onKeyDown);
                    window.removeEventListener('keyup', onKeyUp);
                    window.removeEventListener('blur', onWindowBlur);
                };
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
                let is_arrow_key =
                    matches!(key, "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight");

                if event_type == "blur" {
                    space_pressed.set(false);
                    shift_pressed.set(false);
                    ctrl_pressed.set(false);
                    meta_pressed.set(false);
                    nudge_batch_active.set(false);
                    space_pan_active.set(false);
                    continue;
                }

                if key == " " {
                    space_pressed.set(event_type == "keydown");
                    if event_type == "keyup" {
                        let should_cancel_space_pan = *space_pan_active.read()
                            && matches!(*interaction_mode.read(), InteractionMode::Panning { .. })
                            && *tool_signal.read() != ToolMode::Pan;
                        if should_cancel_space_pan {
                            interaction_mode.set(InteractionMode::Select);
                        }
                        space_pan_active.set(false);
                    }
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
                    if !is_arrow_key {
                        nudge_batch_active.set(false);
                    }
                    match key {
                        "Delete" | "Backspace" => {
                            let _ = apply_delete_selected(doc_signal, history_signal);
                        }
                        "Escape" => {
                            if editing_node.read().is_some() || editing_edge.read().is_some() {
                                editing_node.set(None);
                                editing_edge.set(None);
                                edit_value.set(String::new());
                                apply_clear_selection(doc_signal);
                            } else {
                                let mode = interaction_mode.read().clone();
                                match mode {
                                    InteractionMode::DraggingSelection { .. }
                                    | InteractionMode::ResizingSelection { .. } => {
                                        interaction_mode.with_mut(|mode_mut| {
                                            doc_signal.with_mut(|doc| {
                                                let _ = finalize_motion_release(mode_mut, doc);
                                            });
                                        });
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
                            let viewport_size_now = *viewport_size.read();
                            let _ = apply_zoom_reset(doc_signal, history_signal, viewport_size_now);
                        }
                        "v" | "V" if !modifier => tool_signal.set(ToolMode::Select),
                        "h" | "H" if !modifier => tool_signal.set(ToolMode::Pan),
                        "l" | "L" if !modifier => tool_signal.set(ToolMode::Edge),
                        "r" | "R" if !modifier => tool_signal.set(ToolMode::Subgraph),
                        "t" | "T" if !modifier => tool_signal.set(ToolMode::Text),
                        _ => {}
                    }
                }
            }
        });
    });

    use_drop(move || {
        let _ = document::eval(
            r"
                if (window.__seshat_canvas_keyboard_cleanup) {
                    window.__seshat_canvas_keyboard_cleanup();
                    window.__seshat_canvas_keyboard_cleanup = null;
                }
                if (window.__seshat_canvas_resize_cleanup) {
                    window.__seshat_canvas_resize_cleanup();
                    window.__seshat_canvas_resize_cleanup = null;
                }
                if (window.__seshat_canvas_middle_pan_cleanup) {
                    window.__seshat_canvas_middle_pan_cleanup();
                    window.__seshat_canvas_middle_pan_cleanup = null;
                }
                if (window.__seshat_canvas_pointer_raf_cleanup) {
                    window.__seshat_canvas_pointer_raf_cleanup();
                    window.__seshat_canvas_pointer_raf_cleanup = null;
                }
                if (window.__seshat_canvas_pointer_global_cleanup) {
                    window.__seshat_canvas_pointer_global_cleanup();
                    window.__seshat_canvas_pointer_global_cleanup = null;
                }
                if (window.__seshat_canvas_touch_guard_cleanup) {
                    window.__seshat_canvas_touch_guard_cleanup();
                    window.__seshat_canvas_touch_guard_cleanup = null;
                }
            ",
        );
    });

    use_effect(move || {
        let mut eval = document::eval(
            r"
                if (window.__seshat_canvas_touch_guard_cleanup) {
                    window.__seshat_canvas_touch_guard_cleanup();
                }

                const reportTouches = (event) => {
                    const target = event.target;
                    const inCanvas = target && target.closest && target.closest('.canvas-container');
                    if (!inCanvas) {
                        return;
                    }
                    const touches = event.touches ? event.touches.length : 0;
                    dioxus.send({ type: 'touchmeta', touches });
                };

                const onTouchStart = (event) => reportTouches(event);
                const onTouchMove = (event) => reportTouches(event);
                const onTouchEnd = (event) => reportTouches(event);
                const onTouchCancel = (event) => reportTouches(event);

                window.addEventListener('touchstart', onTouchStart, { passive: true, capture: true });
                window.addEventListener('touchmove', onTouchMove, { passive: true, capture: true });
                window.addEventListener('touchend', onTouchEnd, { passive: true, capture: true });
                window.addEventListener('touchcancel', onTouchCancel, { passive: true, capture: true });

                window.__seshat_canvas_touch_guard_cleanup = () => {
                    window.removeEventListener('touchstart', onTouchStart, true);
                    window.removeEventListener('touchmove', onTouchMove, true);
                    window.removeEventListener('touchend', onTouchEnd, true);
                    window.removeEventListener('touchcancel', onTouchCancel, true);
                };
            ",
        );

        spawn(async move {
            while let Ok(json) = eval.recv::<serde_json::Value>().await {
                if json["type"].as_str() != Some("touchmeta") {
                    continue;
                }

                let touch_count = json["touches"].as_u64().map_or(0_u64, |v| v);
                let is_multi_touch = touch_count >= 2;
                multi_touch_active.set(is_multi_touch);

                if is_multi_touch {
                    pending_pointer_sample.set(None);
                    pending_wheel_sample.set(None);
                    space_pan_active.set(false);
                    interaction_mode.set(InteractionMode::Select);
                }
            }
        });
    });

    use_effect(move || {
        let _ = document::eval(
            r"
                if (window.__seshat_canvas_middle_pan_cleanup) {
                    window.__seshat_canvas_middle_pan_cleanup();
                }

                const preventMiddleAutoScroll = (event) => {
                    const target = event.target;
                    const inCanvas = target && target.closest && target.closest('.canvas-container');
                    if (event.button === 1 && inCanvas) {
                        event.preventDefault();
                    }
                };

                window.addEventListener('mousedown', preventMiddleAutoScroll, { capture: true });
                window.__seshat_canvas_middle_pan_cleanup = () => {
                    window.removeEventListener('mousedown', preventMiddleAutoScroll, { capture: true });
                };
            ",
        );
    });

    use_effect(move || {
        let mut eval = document::eval(
            r"
                if (window.__seshat_canvas_pointer_raf_cleanup) {
                    window.__seshat_canvas_pointer_raf_cleanup();
                }

                let rafId = 0;
                const onFrame = () => {
                    dioxus.send({ type: 'raf' });
                    rafId = window.requestAnimationFrame(onFrame);
                };

                rafId = window.requestAnimationFrame(onFrame);
                window.__seshat_canvas_pointer_raf_cleanup = () => {
                    if (rafId !== 0) {
                        window.cancelAnimationFrame(rafId);
                    }
                };
            ",
        );

        spawn(async move {
            while let Ok(json) = eval.recv::<serde_json::Value>().await {
                if json["type"].as_str() == Some("raf") {
                    if pending_pointer_sample.read().is_some() {
                        flush_pending_pointer_update(
                            doc_signal,
                            history_signal,
                            interaction_mode,
                            pending_pointer_sample,
                        );
                    }
                    if pending_wheel_sample.read().is_some() {
                        flush_pending_wheel_update(doc_signal, pending_wheel_sample);
                    }
                }
            }
        });
    });

    use_effect(move || {
        let mut eval = document::eval(
            r"
                if (window.__seshat_canvas_resize_cleanup) {
                    window.__seshat_canvas_resize_cleanup();
                }

                const target = document.querySelector('.canvas-container');
                if (target) {
                    let rafId = 0;
                    let lastLeft = Number.NaN;
                    let lastTop = Number.NaN;
                    let lastWidth = Number.NaN;
                    let lastHeight = Number.NaN;
                    const notify = (left, top, width, height) => {
                        if (
                            Math.abs(left - lastLeft) < 0.5 &&
                            Math.abs(top - lastTop) < 0.5 &&
                            Math.abs(width - lastWidth) < 0.5 &&
                            Math.abs(height - lastHeight) < 0.5
                        ) {
                            return;
                        }
                        lastLeft = left;
                        lastTop = top;
                        lastWidth = width;
                        lastHeight = height;
                        dioxus.send({ type: 'resize', left, top, width, height });
                    };

                    const scheduleNotify = () => {
                        if (rafId !== 0) {
                            return;
                        }
                        rafId = window.requestAnimationFrame(() => {
                            rafId = 0;
                            const r = target.getBoundingClientRect();
                            notify(r.left, r.top, r.width, r.height);
                        });
                    };

                    const ro = new ResizeObserver(() => scheduleNotify());
                    ro.observe(target);

                    // Use requestAnimationFrame loop to continuously update canvas origin.
                    // This catches scroll events from nested containers that don't bubble to window.
                    // Send update every frame to ensure we always have current position.
                    const pollOrigin = () => {
                        const rect = target.getBoundingClientRect();
                        dioxus.send({ type: 'resize', left: rect.left, top: rect.top, width: rect.width, height: rect.height });
                        rafId = window.requestAnimationFrame(pollOrigin);
                    };
                    rafId = window.requestAnimationFrame(pollOrigin);

                    // Send immediate update on scroll to minimize race condition
                    // between scroll and pointerdown
                    const onScroll = () => {
                        const rect = target.getBoundingClientRect();
                        dioxus.send({ type: 'resize', left: rect.left, top: rect.top, width: rect.width, height: rect.height });
                    };
                    window.addEventListener('scroll', onScroll, { passive: true, capture: true });
                    document.addEventListener('scroll', onScroll, { passive: true, capture: true });

                    window.addEventListener('resize', scheduleNotify, { passive: true });
                    // Listen in capture phase to catch scroll events from nested scrollable containers
                    // since scroll events do not bubble.
                    window.addEventListener('scroll', scheduleNotify, { passive: true, capture: true });
                    document.addEventListener('scroll', scheduleNotify, { passive: true, capture: true });
                    window.__seshat_canvas_resize_cleanup = () => {
                        ro.disconnect();
                        if (rafId !== 0) {
                            window.cancelAnimationFrame(rafId);
                        }
                        window.removeEventListener('scroll', onScroll, true);
                        document.removeEventListener('scroll', onScroll, true);
                        window.removeEventListener('resize', scheduleNotify);
                        window.removeEventListener('scroll', scheduleNotify, true);
                        document.removeEventListener('scroll', scheduleNotify, true);
                    };

                    scheduleNotify();
                }
            ",
        );

        spawn(async move {
            while let Ok(json) = eval.recv::<serde_json::Value>().await {
                if json["type"].as_str() == Some("resize") {
                    canvas_origin.set((
                        json["left"].as_f64().map_or(0.0, |v| v),
                        json["top"].as_f64().map_or(0.0, |v| v),
                    ));
                    let next = normalize_viewport(
                        json["width"].as_f64().map_or(1200.0, |v| v),
                        json["height"].as_f64().map_or(800.0, |v| v),
                    );
                    let current = *viewport_size.read();
                    if viewport_changed(current, next) {
                        viewport_size.set(next);
                    }
                }
            }
        });
    });

    use_effect(move || {
        let mut eval = document::eval(
            r"
                if (window.__seshat_canvas_pointer_global_cleanup) {
                    window.__seshat_canvas_pointer_global_cleanup();
                }

                // Global function to get current canvas origin - can be called from Rust
                window.__seshat_get_canvas_origin = () => {
                    const target = document.querySelector('.canvas-container');
                    if (!target) return { x: 0, y: 0 };
                    const rect = target.getBoundingClientRect();
                    return { x: rect.left, y: rect.top };
                };

                const getCanvasOrigin = () => {
                    const target = document.querySelector('.canvas-container');
                    if (!target) return { x: 0, y: 0 };
                    const rect = target.getBoundingClientRect();
                    return { x: rect.left, y: rect.top };
                };

                const onPointerMove = (event) => {
                    const origin = getCanvasOrigin();
                    dioxus.send({ type: 'pointermove', x: event.clientX, y: event.clientY, originX: origin.x, originY: origin.y });
                };

                const onPointerUp = (event) => {
                    const origin = getCanvasOrigin();
                    dioxus.send({ type: 'pointerup', x: event.clientX, y: event.clientY, originX: origin.x, originY: origin.y });
                };

                const onPointerDown = (event) => {
                    // Get fresh origin before anything else
                    const origin = getCanvasOrigin();
                    // Store in global for Rust to read via any means necessary
                    window.__seshat_current_origin = { x: origin.x, y: origin.y };
                    // Mark that pointerdown was handled so onMouseDownCapture can stop Dioxus handler
                    window.__seshat_pointerdown_handled = true;
                    // Send pointerdown with ALL the information Rust needs
                    // Rust will handle the actual logic
                    dioxus.send({
                        type: 'pointerdown',
                        x: event.clientX,
                        y: event.clientY,
                        originX: origin.x,
                        originY: origin.y,
                        button: event.button.toString(),
                        // Include current tool and modifier state
                        tool: window.__seshat_current_tool || 'select',
                        shiftKey: event.shiftKey,
                        ctrlKey: event.ctrlKey,
                        metaKey: event.metaKey,
                    });
                };

                // Capture-phase mousedown handler to stop Dioxus onmousedown from running
                // when pointerdown has already handled the event with fresh coordinates
                const onMouseDownCapture = (event) => {
                    if (window.__seshat_pointerdown_handled) {
                        window.__seshat_pointerdown_handled = false;
                        // DO NOT stop propagation here! Dioxus relies on onmousedown
                        // for elements like nodes and resize handles.
                        // event.preventDefault();
                        // event.stopPropagation();
                        // event.stopImmediatePropagation();
                    }
                };

                // Reset the flag on pointerup in case pointerdown was canceled
                const onPointerUpReset = (event) => {
                    window.__seshat_pointerdown_handled = false;
                };

                window.addEventListener('pointermove', onPointerMove, { passive: true });
                window.addEventListener('pointerup', onPointerUp, { passive: true });
                window.addEventListener('pointerup', onPointerUpReset, { passive: true });
                window.addEventListener('pointerdown', onPointerDown, { passive: true });
                // Use capture phase to intercept before Dioxus's handler
                window.addEventListener('mousedown', onMouseDownCapture, { capture: true, passive: false });

                window.__seshat_canvas_pointer_global_cleanup = () => {
                    window.removeEventListener('pointermove', onPointerMove);
                    window.removeEventListener('pointerup', onPointerUp);
                    window.removeEventListener('pointerup', onPointerUpReset);
                    window.removeEventListener('pointerdown', onPointerDown);
                    window.removeEventListener('mousedown', onMouseDownCapture, true);
                };
            ",
        );

        spawn(async move {
            while let Ok(json) = eval.recv::<serde_json::Value>().await {
                let event_type = json["type"].as_str().map_or("", |s| s);

                // Also handle resize messages from pointerdown handler to update canvas_origin
                if event_type == "resize" {
                    canvas_origin.set((
                        json["left"].as_f64().map_or(0.0, |v| v),
                        json["top"].as_f64().map_or(0.0, |v| v),
                    ));
                    continue;
                }

                // Get client coordinates
                let client_x = json["x"].as_f64().map_or(0.0, |v| v);
                let client_y = json["y"].as_f64().map_or(0.0, |v| v);
                // Compute local coordinates from message - uses origin from the message directly
                let origin_x = json["originX"].as_f64().map_or(0.0, |v| v);
                let origin_y = json["originY"].as_f64().map_or(0.0, |v| v);
                let local_x = client_x - origin_x;
                let local_y = client_y - origin_y;

                // Handle pointerdown - use fresh origin from the message directly
                if event_type == "pointerdown" {
                    // Update canvas_origin with the fresh origin from this message
                    canvas_origin.set((origin_x, origin_y));

                    // Handle editing commit
                    if editing_node.read().is_some() || editing_edge.read().is_some() {
                        commit_inline_edit(
                            doc_signal,
                            history_signal,
                            editing_node,
                            editing_edge,
                            edit_value,
                        );
                    }

                    let button = json["button"].as_str().map_or("0", |s| s);
                    let is_middle = button == "1";
                    let is_right = button == "2";
                    let tool = *tool_signal.read();
                    let shift = json["shiftKey"].as_bool().unwrap_or(false);
                    let ctrl = json["ctrlKey"].as_bool().unwrap_or(false);
                    let meta = json["metaKey"].as_bool().unwrap_or(false);

                    if *space_pressed.read() || is_middle || is_right || tool == ToolMode::Pan {
                        space_pan_active.set(
                            *space_pressed.read()
                                && !is_middle
                                && !is_right
                                && tool != ToolMode::Pan,
                        );
                        interaction_mode.set(InteractionMode::Panning {
                            last_pos: (local_x, local_y),
                        });
                        continue;
                    }

                    // Only handle primary button (button "0")
                    if button != "0" {
                        continue;
                    }

                    let pos = {
                        let doc = doc_signal.read();
                        to_canvas_coords(
                            local_x,
                            local_y,
                            doc.editor_state.camera_x.0,
                            doc.editor_state.camera_y.0,
                            doc.editor_state.zoom.0,
                        )
                    };

                    if tool == ToolMode::Select {
                        let doc = doc_signal.read().clone();
                        if let Some(edge_id) = find_edge_at(&doc, pos.0, pos.1) {
                            let additive = shift || ctrl || meta;
                            doc_signal.with_mut(|d| {
                                d.editor_state.selected_items = if additive {
                                    toggle_selection(
                                        &d.editor_state.selected_items,
                                        &edge_id.to_string(),
                                    )
                                } else {
                                    select_single(edge_id.to_string())
                                };
                            });
                            interaction_mode.set(InteractionMode::Select);
                            continue;
                        }
                    }

                    if tool == ToolMode::Text {
                        let id = NodeId::new(Uuid::new_v4().to_string());
                        let current = doc_signal.read().clone();
                        let history = history_signal.read().clone();
                        *history_signal.write() = history.push(current);
                        doc_signal.with_mut(|doc| {
                            let (x, y) = snap_point(
                                pos,
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
                                    locked: false,
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
                    } else {
                        // For other tools, let the Dioxus handler deal with it
                    }
                    continue;
                }

                let client_x = json["x"].as_f64().map_or(0.0, |v| v);
                let client_y = json["y"].as_f64().map_or(0.0, |v| v);
                // Use origin from the message (synchronously fetched from DOM) instead of cached
                // signal
                let origin_x = json["originX"].as_f64().map_or(0.0, |v| v);
                let origin_y = json["originY"].as_f64().map_or(0.0, |v| v);
                let local_x = client_x - origin_x;
                let local_y = client_y - origin_y;

                if *multi_touch_active.read() {
                    continue;
                }

                if event_type == "pointermove" {
                    interaction_mode.with_mut(|mode| match mode {
                        InteractionMode::DrawingEdge { current_pos, .. } => {
                            let doc = doc_signal.read();
                            *current_pos = to_canvas_coords(
                                local_x,
                                local_y,
                                doc.editor_state.camera_x.0,
                                doc.editor_state.camera_y.0,
                                doc.editor_state.zoom.0,
                            );
                        }
                        InteractionMode::RubberBand { current, .. }
                        | InteractionMode::DrawingSubgraph { current, .. } => {
                            let doc = doc_signal.read();
                            let raw = to_canvas_coords(
                                local_x,
                                local_y,
                                doc.editor_state.camera_x.0,
                                doc.editor_state.camera_y.0,
                                doc.editor_state.zoom.0,
                            );
                            *current = snap_point(
                                raw,
                                doc.editor_state.snap_to_grid,
                                doc.editor_state.grid_size,
                            );
                        }
                        InteractionMode::DraggingSelection { .. }
                        | InteractionMode::ResizingSelection { .. }
                        | InteractionMode::Panning { .. } => {
                            pending_pointer_sample.set(Some((local_x, local_y)));
                        }
                        InteractionMode::DraggingSelection { .. }
                        | InteractionMode::ResizingSelection { .. } => {
                            pending_pointer_sample.set(Some((local_x, local_y)));
                        }
                        InteractionMode::Select => {}
                    });
                    continue;
                }

                if event_type == "pointerup" {
                    flush_pending_pointer_update(
                        doc_signal,
                        history_signal,
                        interaction_mode,
                        pending_pointer_sample,
                    );

                    interaction_mode.with_mut(|mode| match mode {
                        InteractionMode::DrawingEdge { from_node, .. } => {
                            let doc = doc_signal.read().clone();
                            let pos = to_canvas_coords(
                                local_x,
                                local_y,
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
                                        bend_points: im::Vector::new(),
                                        tags: im::Vector::new(),
                                        metadata: HashMap::new(),
                                        font_size: None,
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
                            let additive = *shift_pressed.read()
                                || *ctrl_pressed.read()
                                || *meta_pressed.read();
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
                                subgraph_release_bounds(*start, *current, snap, grid)
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
                                            locked: true,
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
                            doc_signal.with_mut(|doc| {
                                let _ = finalize_motion_release(mode, doc);
                            });
                        }
                        InteractionMode::Panning { .. } => {
                            *mode = InteractionMode::Select;
                        }
                        InteractionMode::DraggingSelection { .. }
                        | InteractionMode::ResizingSelection { .. } => {
                            *mode = InteractionMode::Select;
                        }
                        InteractionMode::Select => {}
                    });
                    space_pan_active.set(false);
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
                let image_data_url = payload.image_data_url;
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
                    let origin = sync_canvas_origin().unwrap_or_else(|| *canvas_origin.read());
                    let local_x = coords.x - origin.0;
                    let local_y = coords.y - origin.1;
                    let (x, y) = to_canvas_coords(
                        local_x,
                        local_y,
                        doc.editor_state.camera_x.0,
                        doc.editor_state.camera_y.0,
                        doc.editor_state.zoom.0,
                    );
                    let (x, y) = snap_point(
                        (x - 32.0, y - 32.0),
                        doc.editor_state.snap_to_grid,
                        doc.editor_state.grid_size,
                    );
                    let metadata = image_data_url.clone().map_or_else(HashMap::new, |image| {
                        HashMap::new().update("icon_data_url".to_string(), Value::String(image))
                    });
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
                            locked: false,
                            parent: None,
                            dag_rank: None,
                            tags: tags.into(),
                            metadata,
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
                    local_x,
                    local_y,
                    doc.editor_state.camera_x.0,
                    doc.editor_state.camera_y.0,
                    doc.editor_state.zoom.0,
                );

                let hit_node = ordered_node_cache
                    .read()
                    .iter()
                    .rev()
                    .find_map(|id| {
                        doc.document.nodes.get(id).and_then(|node| {
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
                            pos,
                            d.editor_state.snap_to_grid,
                            d.editor_state.grid_size,
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
                                locked: false,
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
                    );
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
                        local_x,
                        local_y,
                        doc.editor_state.camera_x.0,
                        doc.editor_state.camera_y.0,
                        doc.editor_state.zoom.0,
                    )
                };

                if tool == ToolMode::Select {
                    let doc = doc_signal.read().clone();
                    if let Some(edge_id) = find_edge_at(&doc, pos.0, pos.1) {
                        let additive = *shift_pressed.read() || *ctrl_pressed.read() || *meta_pressed.read();
                        doc_signal.with_mut(|d| {
                            d.editor_state.selected_items = if additive {
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
                                    locked: false,
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
                            pos,
                            doc.editor_state.snap_to_grid,
                            doc.editor_state.grid_size,
                        );
                        interaction_mode.set(InteractionMode::DrawingSubgraph {
                            start: snapped_start,
                            current: snapped_start,
                        });
                    }
                    ToolMode::Select => {
                        interaction_mode.set(InteractionMode::RubberBand { start: pos, current: pos });
                    }
                    ToolMode::Edge => {
                        let doc = doc_signal.read().clone();
                        if let Some(from_node) = find_node_at(&doc, pos.0, pos.1) {
                            interaction_mode.set(InteractionMode::DrawingEdge {
                                from_node,
                                current_pos: pos,
                            });
                        }
                    }
                    ToolMode::Pan => {}
                    ToolMode::Draw => {}
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
                            *current_pos = to_canvas_coords(
                                local_x,
                                local_y,
                                doc.editor_state.camera_x.0,
                                doc.editor_state.camera_y.0,
                                doc.editor_state.zoom.0,
                            );
                        }
                        InteractionMode::RubberBand { current, .. }
                        | InteractionMode::DrawingSubgraph { current, .. } => {
                            let doc = doc_signal.read();
                            let raw = to_canvas_coords(
                                local_x,
                                local_y,
                                doc.editor_state.camera_x.0,
                                doc.editor_state.camera_y.0,
                                doc.editor_state.zoom.0,
                            );
                            *current = snap_point(
                                raw,
                                doc.editor_state.snap_to_grid,
                                doc.editor_state.grid_size,
                            );
                        }
                        InteractionMode::DraggingSelection { .. }
                        | InteractionMode::ResizingSelection { .. }
                        | InteractionMode::Panning { .. } => {
                            pending_pointer_sample.set(Some((local_x, local_y)));
                        }
                        InteractionMode::DraggingSelection { .. }
                        | InteractionMode::ResizingSelection { .. } => {
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
                );
                interaction_mode.with_mut(|mode| {
                    match mode {
                        InteractionMode::DrawingEdge { from_node, .. } => {
                            let coords = evt.data.coordinates().client();
                            let origin = sync_canvas_origin().unwrap_or_else(|| *canvas_origin.read());
                            let local_x = coords.x - origin.0;
                            let local_y = coords.y - origin.1;
                            let doc = doc_signal.read().clone();
                            let pos = to_canvas_coords(
                                local_x,
                                local_y,
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
                                        bend_points: im::Vector::new(),
                                        tags: im::Vector::new(),
                                        metadata: HashMap::new(),
                                        font_size: None,
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
                                subgraph_release_bounds(*start, *current, snap, grid)
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
                                            locked: true,
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
                            doc_signal.with_mut(|doc| {
                                let _ = finalize_motion_release(mode, doc);
                            });
                        }
                        InteractionMode::Panning { .. } => {
                            *mode = InteractionMode::Select;
                        }
                        InteractionMode::DraggingSelection { .. }
                        | InteractionMode::ResizingSelection { .. } => {
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

                {
                    let doc = doc_signal.read();
                    let s = &doc.editor_state;
                    let (vw, vh) = *viewport_size.read();
                    let pattern_step = (s.grid_size.inner().max(8.0) * s.zoom.0).max(4.0);
                    let pattern_x = (-s.camera_x.0 * s.zoom.0).rem_euclid(pattern_step);
                    let pattern_y = (-s.camera_y.0 * s.zoom.0).rem_euclid(pattern_step);
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
                    let doc = doc_signal.read().clone();
                    let s = doc.editor_state.clone();
                    let selected_items =
                        s.selected_items.iter().cloned().collect::<std::collections::HashSet<_>>();
                    let camera_x = s.camera_x.0;
                    let camera_y = s.camera_y.0;
                    let zoom = s.zoom.0;
                    let (viewport_w, viewport_h) = *viewport_size.read();
                    let margin_x = (viewport_w / zoom).max(100.0) * 0.5;
                    let margin_y = (viewport_h / zoom).max(100.0) * 0.5;
                    let culling_min_x = camera_x - margin_x;
                    let culling_min_y = camera_y - margin_y;
                    let culling_max_x = camera_x + (viewport_w / zoom) + margin_x;
                    let culling_max_y = camera_y + (viewport_h / zoom) + margin_y;

                    #[allow(clippy::needless_collect)]
                    let edge_rows = doc.document.edges.iter().filter_map(|(id, edge)| {
                        doc.document.nodes
                            .get(&edge.source)
                            .zip(doc.document.nodes.get(&edge.target))
                            .and_then(|(src, tgt)| {
                                let src_min_x = src.x.0;
                                let src_min_y = src.y.0;
                                let src_max_x = src.x.0 + src.width.0;
                                let src_max_y = src.y.0 + src.height.0;

                                let tgt_min_x = tgt.x.0;
                                let tgt_min_y = tgt.y.0;
                                let tgt_max_x = tgt.x.0 + tgt.width.0;
                                let tgt_max_y = tgt.y.0 + tgt.height.0;

                                let min_x = src_min_x.min(tgt_min_x);
                                let min_y = src_min_y.min(tgt_min_y);
                                let max_x = src_max_x.max(tgt_max_x);
                                let max_y = src_max_y.max(tgt_max_y);

                                let visible = max_x >= culling_min_x
                                    && min_x <= culling_max_x
                                    && max_y >= culling_min_y
                                    && min_y <= culling_max_y;

                                visible.then(|| (id.clone(), edge.clone(), src.clone(), tgt.clone()))
                            })
                    }).collect::<Vec<_>>();
                    edge_rows.into_iter().map(move |(id, edge, src, tgt)| {
                                let (sx, sy) = to_screen_coords(src.x.0 + src.width.0 / 2.0, src.y.0 + src.height.0 / 2.0, camera_x, camera_y, zoom);
                                let (tx, ty) = to_screen_coords(tgt.x.0 + tgt.width.0 / 2.0, tgt.y.0 + tgt.height.0 / 2.0, camera_x, camera_y, zoom);
                                let d = edge_path(sx, sy, tx, ty, &edge);
                                let (mid_x, mid_y) = edge_label_position(sx, sy, tx, ty, &edge);
                                let is_selected = selected_items.contains(id.as_str());
                                let stroke_color = if is_selected {
                                    EDGE_SELECTED
                                } else {
                                    EDGE_DEFAULT
                                };
                                let stroke_width = if is_selected { 2.5 } else { 1.5 };
                                let marker = edge_marker_ref(is_selected);
                                let dash = if edge.style == EdgeStyle::Dashed {
                                    "8,4"
                                } else if edge.style == EdgeStyle::Dotted {
                                    "2,4"
                                } else {
                                    ""
                                };
                                let font_size = edge.font_size.map_or(10.0, |f| f.0) * zoom;
                                let is_editing_edge = editing_edge.read().as_ref() == Some(&id);
                                rsx! {
                                    path {
                                        key: "{id:?}",
                                        "data-node-kind": "edge",
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
                                    } else if !edge.label.is_empty() && zoom >= 0.3 {
                                        text {
                                            x: "{mid_x}",
                                            y: "{mid_y - 6.0}",
                                            text_anchor: "middle",
                                            style: "fill:{TEXT_MUTED}; font-size:{font_size}px;",
                                            "{edge.label}"
                                        }
                                    } else if is_selected && zoom >= 0.3 {
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
                let doc_for_nodes = doc_signal.read().clone();
                let editor_for_nodes = doc_for_nodes.editor_state.clone();
                let selected_items = editor_for_nodes
                    .selected_items
                    .iter()
                    .cloned()
                    .collect::<std::collections::HashSet<_>>();
                let hovered_now = hovered_node.read().clone();
                let camera_x = editor_for_nodes.camera_x.0;
                let camera_y = editor_for_nodes.camera_y.0;
                let zoom = editor_for_nodes.zoom.0;
                let (viewport_w, viewport_h) = *viewport_size.read();
                let margin_x = (viewport_w / zoom).max(100.0) * 0.5;
                let margin_y = (viewport_h / zoom).max(100.0) * 0.5;
                let culling_min_x = camera_x - margin_x;
                let culling_min_y = camera_y - margin_y;
                let culling_max_x = camera_x + (viewport_w / zoom) + margin_x;
                let culling_max_y = camera_y + (viewport_h / zoom) + margin_y;

                #[allow(clippy::needless_collect)]
                let node_rows = ordered_node_cache
                    .read()
                    .iter()
                    .filter_map(|id| {
                        doc_for_nodes
                            .document
                            .nodes
                            .get(id)
                            .and_then(|node| {
                                let node_min_x = node.x.0;
                                let node_min_y = node.y.0;
                                let node_max_x = node.x.0 + node.width.0;
                                let node_max_y = node.y.0 + node.height.0;

                                let visible = node_max_x >= culling_min_x
                                    && node_min_x <= culling_max_x
                                    && node_max_y >= culling_min_y
                                    && node_min_y <= culling_max_y;

                                visible.then(|| (id.clone(), node.clone()))
                            })
                    })
                    .collect::<Vec<_>>();
                node_rows.into_iter().map(move |(id, node)| {
                    let id_mousedown = id.clone();
                    let id_mouseup = id.clone();
                    let id_mouseenter = id.clone();
                    let id_mouseleave = id.clone();
                    let id_data_attr = id.to_string();
                    let id_edit_text = id.clone();
                    let id_edit_subgraph = id.clone();
                    let id_edit_node = id.clone();
                    let is_selected = selected_items.contains(id.as_str());
                    let (left, top) = to_screen_coords(
                        node.x.0,
                        node.y.0,
                        camera_x,
                        camera_y,
                        zoom,
                    );
                    let (width, height) = (
                        node.width.0 * zoom,
                        node.height.0 * zoom,
                    );
                    let is_hovered = hovered_now.as_ref() == Some(&id);
                    let border_width = if is_selected { "2" } else { "1" };
                    let border_base = if is_selected || is_hovered {
                        ACCENT
                    } else {
                        NODE_BORDER
                    };
                    let border_mix = if is_hovered && !is_selected { "50" } else { "100" };
                    let bg = if node.kind == NodeKind::Subgraph { NODE_BG_SUBGRAPH } else { NODE_BG };
                    let z_index = node.z_index + if node.kind == NodeKind::Subgraph { 0 } else { 1000 };
                    let is_editing_node = editing_node.read().as_ref() == Some(&id);
                    let font_px = node.font_size.map_or(11.0, |f| f.0) * zoom;
                    let fallback_provider = node.icon.split('/').next().map_or("generic", |p| p);
                    let provider = node.tags.front().map_or(fallback_provider, |p| p.as_str());
                    let provider_top = provider_color(provider);
                    let node_initials = initials(&node.label);

                    rsx! {
                        div {
                            key: "{id:?}",
                            "data-testid": "node",
                            "data-node-id": "{id_data_attr}",
                            "data-node-kind": match node.kind {
                                NodeKind::Node => "node",
                                NodeKind::Subgraph => "subgraph",
                                NodeKind::Text => "text",
                            },
                            style: "position: absolute; left: {left}px; top: {top}px; width: {width}px; height: {height}px; border: {border_width}px solid color-mix(in oklch, {border_base} {border_mix}%, transparent); border-radius: 10px; background: linear-gradient(180deg, color-mix(in oklch, {bg} 92%, {BG_BASE}) 0%, {bg} 100%); display: flex; flex-direction: column; align-items: center; justify-content: center; cursor: inherit; z-index: {z_index}; box-shadow: 0 6px 18px color-mix(in oklch, black 24%, transparent);",

                            onmouseenter: {
                                move |_| hovered_node.set(Some(id_mouseenter.clone()))
                            },
                            onmouseleave: move |_| {
                                if hovered_node.read().as_ref() == Some(&id_mouseleave) {
                                    hovered_node.set(None);
                                }
                            },

                            onmousedown: move |evt| {
                                if *multi_touch_active.read() {
                                    return;
                                }
                                evt.stop_propagation();
                                let tool = *tool_signal.read();
                                let doc = doc_signal.read().clone();
                                let additive = *shift_pressed.read() || *ctrl_pressed.read() || *meta_pressed.read();
                                let is_middle = evt.data.trigger_button() == Some(MouseButton::Auxiliary);
                                let is_right = evt.data.trigger_button() == Some(MouseButton::Secondary);
                                let is_primary = evt.data.trigger_button() == Some(MouseButton::Primary);
                                let coords = evt.data.coordinates().client();
                                let origin = sync_canvas_origin().unwrap_or_else(|| *canvas_origin.read());
                                let local_x = coords.x - origin.0;
                                let local_y = coords.y - origin.1;
                                let pos = to_canvas_coords(
                                    local_x,
                                    local_y,
                                    doc.editor_state.camera_x.0,
                                    doc.editor_state.camera_y.0,
                                    doc.editor_state.zoom.0,
                                );

                                if *space_pressed.read() || is_middle || is_right || tool == ToolMode::Pan {
                                    space_pan_active.set(*space_pressed.read() && !is_middle && !is_right && tool != ToolMode::Pan);
                                    interaction_mode.set(InteractionMode::Panning { last_pos: (local_x, local_y) });
                                    return;
                                }

                                if !is_primary {
                                    return;
                                }

                                if tool == ToolMode::Edge {
                                    let mode_now = interaction_mode.read().clone();
                                    if !matches!(mode_now, InteractionMode::DrawingEdge { .. }) {
                                        interaction_mode.set(InteractionMode::DrawingEdge {
                                            from_node: id_mousedown.clone(),
                                            current_pos: pos,
                                        });
                                    }
                                } else {
                                    let was_selected =
                                        doc.editor_state.selected_items.contains(id_mousedown.as_str());

                                    doc_signal.with_mut(|d| {
                                        let selected = if additive {
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
                                        anchor_canvas: pos,
                                        anchor_client: (local_x, local_y),
                                        original_positions,
                                        did_move: false,
                                    });
                                }
                            },

                            onmouseup: move |evt| {
                                evt.stop_propagation();
                                flush_pending_pointer_update(
                                    doc_signal,
                                    history_signal,
                                    interaction_mode,
                                    pending_pointer_sample,
                                );
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
                                                bend_points: im::Vector::new(),
                                                tags: im::Vector::new(),
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
                                            } else {
                                                let _ = toast.show(
                                                    crate::ui::toast::ToastIntent::Warning,
                                                    "Cannot create circular connection",
                                                    None,
                                                );
                                            }
                                        }
                                        if *tool_signal.read() == ToolMode::Edge {
                                            let doc_now = doc_signal.read().clone();
                                            let coords = evt.data.coordinates().client();
                                            let origin = sync_canvas_origin().unwrap_or_else(|| *canvas_origin.read());
                                            let local_x = coords.x - origin.0;
                                            let local_y = coords.y - origin.1;
                                            let pos = to_canvas_coords(
                                                local_x,
                                                local_y,
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
                                    InteractionMode::DraggingSelection { .. }
                                    | InteractionMode::ResizingSelection { .. } => {
                                        interaction_mode.with_mut(|mode_mut| {
                                            doc_signal.with_mut(|doc| {
                                                let _ = finalize_motion_release(mode_mut, doc);
                                            });
                                        });
                                    }
                                    _ => {}
                                }

                                if *tool_signal.read() != ToolMode::Edge {
                                    tool_signal.set(ToolMode::Select);
                                }
                            },

                            div {
                                "data-testid": "node-hitbox",
                                style: "position:absolute; inset:0; pointer-events:none; opacity:0;"
                            }

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
                                        "data-testid": "node-label",
                                        style: "font-size: {font_px}px; color: {TEXT_MAIN};",
                                        ondoubleclick: {
                                            let edit_label = node.label.clone();
                                            move |evt| {
                                                evt.stop_propagation();
                                                editing_edge.set(None);
                                                editing_node.set(Some(id_edit_text.clone()));
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
                                        "data-testid": "node-label",
                                        style: "position:absolute; top:8px; left:10px; font-size:{font_px}px; color:{TEXT_MUTED};",
                                        ondoubleclick: {
                                            let edit_label = node.label.clone();
                                            move |evt| {
                                                evt.stop_propagation();
                                                editing_edge.set(None);
                                                editing_node.set(Some(id_edit_subgraph.clone()));
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
                                                            let origin = sync_canvas_origin().unwrap_or_else(|| *canvas_origin.read());
                                                            let local_x = coords.x - origin.0;
                                                            let local_y = coords.y - origin.1;
                                                            let doc = doc_signal.read().clone();
                                                            let mouse_pos = to_canvas_coords(
                                                                local_x,
                                                                local_y,
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

                                {
                                    if zoom >= 0.3 {
                                        let icon_w = fit_icon_side(width);
                                        let icon_h = fit_icon_side(height);
                                        node_image_data_url(&node).map_or_else(
                                            || {
                                                rsx! {
                                                    span {
                                                        style: "font-size: {font_px * 1.1}px; color: {provider_top}; font-weight: 700; font-family: monospace;",
                                                        "{node_initials}"
                                                    }
                                                }
                                            },
                                            |icon_src| {
                                                rsx! {
                                                    img {
                                                        src: "{icon_src}",
                                                        width: "{icon_w}px",
                                                        height: "{icon_h}px",
                                                        style: "object-fit: contain; pointer-events: none; user-select: none;"
                                                    }
                                                }
                                            },
                                        )
                                    } else {
                                        rsx! { "" }
                                    }
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
                                } else if zoom >= 0.3 {
                                    span {
                                        "data-testid": "node-label",
                                        style: "position:absolute; left:0; right:0; bottom:-18px; text-align:center; font-size:{font_px}px; color:{TEXT_MAIN};",
                                        ondoubleclick: {
                                            let edit_label = node.label.clone();
                                            move |evt| {
                                                evt.stop_propagation();
                                                editing_edge.set(None);
                                                editing_node.set(Some(id_edit_node.clone()));
                                                edit_value.set(edit_label.clone());
                                            }
                                        },
                                        "{node.label}"
                                    }
                                }
                            }
                    }
                }
                    })
            }

            {
                let doc = doc_signal.read().clone();
                selection_handles_overlay(
                    &doc,
                    interaction_mode,
                    doc_signal,
                    canvas_origin,
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
                                    let mut next = current.clone();
                                    if scale_selected_nodes(&mut next, 0.8) {
                                        next.revision = next.revision.increment();
                                        let history = history_signal.read().clone();
                                        *history_signal.write() = history.push(current);
                                        *doc_signal.write() = next;
                                    }
                                },
                                "Shrink"
                            }
                            button {
                                style: "border:1px solid {BORDER}; border-radius:6px; background:{BG_BASE}; color:{TEXT_MAIN}; padding:0 8px; height:24px; cursor:{scale_cursor}; opacity:{scale_opacity};",
                                disabled: !can_scale,
                                onclick: move |_| {
                                    let current = doc_signal.read().clone();
                                    let mut next = current.clone();
                                    if scale_selected_nodes(&mut next, 1.25) {
                                        next.revision = next.revision.increment();
                                        let history = history_signal.read().clone();
                                        *history_signal.write() = history.push(current);
                                        *doc_signal.write() = next;
                                    }
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

#[cfg(test)]
mod tests {
    use im::HashMap;

    use super::{apply_rubber_band_release, fit_icon_side, subgraph_release_bounds};
    use crate::{
        models::document::{DiagramDocument, Node, NodeId, NodeKind, NodeStyle, OrderedFloat},
        ui::grid::GridSize,
    };

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
            locked: true,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        }
    }

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

    #[test]
    fn given_subgraph_release_bounds_when_drag_too_small_then_none() {
        let grid = GridSize::new(20.0).unwrap();
        let result = subgraph_release_bounds((0.0, 0.0), (10.0, 10.0), false, grid);
        assert!(result.is_none());
    }

    #[test]
    fn given_subgraph_release_bounds_when_drag_valid_then_bounds_returned() {
        let grid = GridSize::new(20.0).unwrap();
        let result = subgraph_release_bounds((5.0, 10.0), (60.0, 70.0), false, grid);
        assert_eq!(result, Some((5.0, 10.0, 55.0, 60.0)));
    }

    #[test]
    fn given_icon_side_when_too_small_then_fit_never_panics_and_stays_non_negative() {
        let result = fit_icon_side(19.68);
        assert!(result >= 0.0);
        assert!(result <= 11.68);
    }
}
