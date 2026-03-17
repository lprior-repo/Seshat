use dioxus::html::input_data::keyboard_types::Key;
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use im::HashMap;
use std::collections::HashSet;
use uuid::Uuid;

use crate::ui::canvas::canvas_view::selection_handles_overlay;
use crate::ui::canvas::document_ops::{
    edge_preserves_dag, fit_icon_side, flush_pending_pointer_update, initials, node_image_data_url,
    provider_color, scale_selected_nodes, sync_canvas_origin,
};
use crate::ui::interaction::with_auto_selected_edges;
use crate::{
    history::History,
    ui::{
        editor::ToolMode,
        grid::snap_point,
        interaction::{drag_original_positions, select_single, toggle_selection},
        theme::{
            ACCENT, BG_BASE, BORDER, NODE_BG, NODE_BG_SUBGRAPH, NODE_BORDER, TEXT_MAIN, TEXT_MUTED,
            TOOLBAR_BG,
        },
    },
};
use canvas_domain::interaction_reducer::{
    commit_inline_edit, finalize_motion_release, InteractionMode,
};
use canvas_domain::perf::{to_canvas_coords, to_screen_coords};
use canvas_domain::selection_geometry::{selected_node_ids, selection_bounds};
use diagram_models::document::{
    DiagramDocument, EdgeId, LockState, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
};

#[component]
pub fn NodeLayer(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    mut tool_signal: Signal<ToolMode>,
    mut interaction_mode: Signal<InteractionMode>,
    mut editing_node: Signal<Option<NodeId>>,
    mut editing_edge: Signal<Option<EdgeId>>,
    mut edit_value: Signal<String>,
    mut hovered_node: Signal<Option<NodeId>>,
    viewport_size: Signal<(f64, f64)>,
    ordered_node_cache: Memo<Vec<NodeId>>,
    mut canvas_origin: Signal<(f64, f64)>,
    shift_pressed: Signal<bool>,
    ctrl_pressed: Signal<bool>,
    meta_pressed: Signal<bool>,
    space_pressed: Signal<bool>,
    multi_touch_active: Signal<bool>,
    mut space_pan_active: Signal<bool>,
    db_tx: Option<Coroutine<diagram_models::envelope::EventEnvelope>>,
) -> Element {
    let mut pending_pointer_sample = use_signal(|| Option::<(f64, f64)>::None);
    let edge_style_default = use_context::<Signal<diagram_models::document::EdgeStyle>>();
    let arrow_type_default = use_context::<Signal<diagram_models::document::ArrowType>>();
    let toast = crate::ui::toast::use_toast();
    let doc_for_nodes = doc_signal.read().clone();
    let s = doc_for_nodes.editor_state.clone();
    let selected_items = s.selected_items.iter().cloned().collect::<HashSet<_>>();
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
    let hovered_now = hovered_node.read().clone();

    rsx! { {
    let node_rows = ordered_node_cache
                        .read()
                        .iter()
                        .filter_map(|id: &NodeId| {
                            doc_for_nodes
                                .document
                                .nodes
                                .get(&id)
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
                    node_rows.into_iter().map(move |(id, node): (NodeId, Node)| {
                        let id_mousedown = id.clone();
                        let id_mouseup = id.clone();
                        let id_mouseenter = id.clone();
                        let id_mouseleave = id.clone();
                        let id_data_attr = id.to_string();
                        let id_edit_text = id.clone();
                        let id_edit_subgraph = id.clone();
                        let id_edit_node = id.clone();
                        let is_selected = selected_items.contains(id.as_str());
                        let canvas_domain::ScreenCoord(left, top) = to_screen_coords(
        canvas_domain::CanvasCoord(node.x.0, node.y.0),
        canvas_domain::CanvasCoord(camera_x, camera_y),
        zoom
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
                        let z_index = node.z_index + if node.kind == NodeKind::Subgraph { 10 } else { 1000 };
                        let is_editing_node = editing_node.read().as_ref() == Some(&id);
                        let font_px = node.font_size.map_or(11.0, |f| f.0) * zoom;
                        let fallback_provider = node.icon.split('/').next().map_or("generic", |p| p);
                        let provider = node.tags.front().map_or(fallback_provider, |p: &String| p.as_str());
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
                                    canvas_domain::ScreenCoord(local_x, local_y),
                                    canvas_domain::CanvasCoord(doc.editor_state.camera_x.0, doc.editor_state.camera_y.0),
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
                                                current_pos: (pos.0, pos.1),
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
                                            anchor_canvas: (pos.0, pos.1),
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
                                        db_tx,
                                    );
                                    let mode = interaction_mode.read().clone();
                                    match mode {
                                        InteractionMode::DrawingEdge { from_node, .. } => {
                                            if from_node != id_mouseup {
                                                let doc_now = doc_signal.read().clone();
                                                let candidate_edge = diagram_models::document::Edge {
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
                source_port: None,
                target_port: None,
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
                                                let pos = to_canvas_coords(canvas_domain::ScreenCoord(local_x, local_y), canvas_domain::CanvasCoord(doc_now.editor_state.camera_x.0, doc_now.editor_state.camera_y.0), doc_now.editor_state.zoom.0);
                                                interaction_mode.set(InteractionMode::DrawingEdge {
                                                    from_node: id_mouseup.clone(),
                                                    current_pos: (pos.0, pos.1),
                                                });
                                            } else {
                                                interaction_mode.set(InteractionMode::Select);
                                            }
                                        }
                                        InteractionMode::DraggingSelection { .. }
                                        | InteractionMode::ResizingSelection { .. } => {
                                            let db_tx = db_tx;
                                            let mut doc_clone = doc_signal.read().clone();
                                            interaction_mode.with_mut(|mode_mut| {
                                                let did_change = finalize_motion_release(mode_mut, &mut doc_clone, &db_tx);
                                                if did_change {
                                                    doc_signal.set(doc_clone);
                                                }
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
                                                    db_tx,
                                                )
                                                .ok();
                                            },
                                            onkeydown: move |evt| {
                                                if evt.key() == Key::Enter {
                                                    commit_inline_edit(
                                                        doc_signal,
                                                        history_signal,
                                                        editing_node,
                                                        editing_edge,
                                                        edit_value,
                                                        db_tx,
                                                    )
                                                    .ok();
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
                                        style: "position: absolute; top: 0; left: 0; right: 0; height: 32px; border-bottom: 1px solid var(--border); display: flex; align-items: center; padding: 0 12px; background: color-mix(in oklch, var(--node-bg-subgraph) 80%, transparent); border-radius: 9px 9px 0 0; pointer-events: none;",
                                        span {
                                            style: "font-size: 11px; font-weight: 500; text-transform: uppercase; letter-spacing: 0.04em; color: var(--text-muted); pointer-events: none;",
                                            "{node.label}"
                                        }
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
                                                    db_tx,
                                                )
                                                .ok();
                                            },
                                            onkeydown: move |evt| {
                                                if evt.key() == Key::Enter {
                                                    commit_inline_edit(
                                                        doc_signal,
                                                        history_signal,
                                                        editing_node,
                                                        editing_edge,
                                                        edit_value,
                                                        db_tx,
                                                    )
                                                    .ok();
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
        canvas_domain::ScreenCoord(local_x, local_y),
        canvas_domain::CanvasCoord(doc.editor_state.camera_x.0, doc.editor_state.camera_y.0),
        doc.editor_state.zoom.0
                                                                );
                                                                interaction_mode.set(InteractionMode::DrawingEdge {
                                                                    from_node: current_id.clone(),
                                                                    current_pos: (mouse_pos.0, mouse_pos.1),
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
                                                    db_tx,
                                                )
                                                .ok();
                                            },
                                            onkeydown: move |evt| {
                                                if evt.key() == Key::Enter {
                                                    commit_inline_edit(
                                                        doc_signal,
                                                        history_signal,
                                                        editing_node,
                                                        editing_edge,
                                                        edit_value,
                                                        db_tx,
                                                    )
                                                    .ok();
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
        } }
}
