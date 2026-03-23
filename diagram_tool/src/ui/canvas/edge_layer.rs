use crate::history::History;
use crate::ui::canvas::canvas_view::markers::EdgeMarkers;
use crate::ui::canvas::canvas_view::{edge_label_position, edge_path};
use crate::ui::theme::{EDGE_DEFAULT, EDGE_SELECTED};
use canvas_domain::interaction_reducer::commit_inline_edit;
use canvas_domain::perf::to_screen_coords;
use diagram_models::document::{DiagramDocument, Edge, EdgeId, EdgeStyle, Node};
use dioxus::html::input_data::keyboard_types::Key;
use dioxus::prelude::*;
use serde_json::Value;

#[component]
pub fn EdgeLayer(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    mut editor_state: Signal<crate::ui::canvas::state::EditorState>,
    mut edit_value: Signal<String>,
    viewport_size: Signal<(f64, f64)>,
    db_tx: Option<Coroutine<diagram_models::envelope::EventEnvelope>>,
) -> Element {
    let doc = doc_signal.read();
    // Avoid cloning entire EditorState - read fields directly via reference
    let selected_items = &doc.editor_state.selected_items;
    let camera_x = doc.editor_state.camera_x.0;
    let camera_y = doc.editor_state.camera_y.0;
    let zoom = doc.editor_state.zoom.0;
    let (viewport_w, viewport_h) = *viewport_size.read();
    let edge_rows = get_visible_edges(&doc, camera_x, camera_y, zoom, viewport_w, viewport_h);

    rsx! {
        {
            edge_rows.map(move |(id, edge, src, tgt)| {
                let src_pt = edge.source_port.as_ref().map_or_else(
                    || diagram_models::geometry::Point::new(src.x.0 + src.width.0 / 2.0, src.y.0 + src.height.0 / 2.0),
                    |p| diagram_models::port::compute_port_absolute_position(src, p)
                );
                let tgt_pt = edge.target_port.as_ref().map_or_else(
                    || diagram_models::geometry::Point::new(tgt.x.0 + tgt.width.0 / 2.0, tgt.y.0 + tgt.height.0 / 2.0),
                    |p| diagram_models::port::compute_port_absolute_position(tgt, p)
                );
                let canvas_domain::ScreenCoord(sx, sy) = to_screen_coords(canvas_domain::CanvasCoord(src_pt.x, src_pt.y), canvas_domain::CanvasCoord(camera_x, camera_y), zoom);
                let canvas_domain::ScreenCoord(tx, ty) = to_screen_coords(canvas_domain::CanvasCoord(tgt_pt.x, tgt_pt.y), canvas_domain::CanvasCoord(camera_x, camera_y), zoom);
                let d = edge_path(sx, sy, tx, ty, edge);
                let (mid_x, mid_y) = edge_label_position(sx, sy, tx, ty, edge);
                let is_selected = selected_items.contains(id.as_str());
                let stroke_color = if is_selected {
                    EDGE_SELECTED
                } else {
                    EDGE_DEFAULT
                };
                let stroke_width = if is_selected { 2.5 } else { 1.5 };
                let is_bidirectional = edge
                    .metadata
                    .get("bidirectional")
                    .is_some_and(|v| v == &Value::Bool(true));
                let markers = EdgeMarkers::for_edge(edge.directed, is_bidirectional, is_selected);
                let dash = if edge.style == EdgeStyle::Dashed {
                    "8,4"
                } else if edge.style == EdgeStyle::Dotted {
                    "2,4"
                } else {
                    ""
                };
                let font_size = edge.font_size.map_or(10.0, |f| f.0) * zoom;
                let is_editing_edge = matches!(*editor_state.read(), crate::ui::canvas::state::EditorState::EditingEdge(ref edit_id) if edit_id == id);
                rsx! {
                    g {
                        key: "{id:?}",
                        path {
                            "data-node-kind": "edge",
                            "data-testid": "edge-{id}",
                            d: "{d}",
                            fill: "none",
                            stroke: "{stroke_color}",
                            stroke_width: "{stroke_width}",
                            stroke_dasharray: "{dash}",
                            marker_end: "{markers.marker_end}",
                            marker_start: markers.marker_start.as_deref().unwrap_or(""),
                        }
                        if is_editing_edge {
                            foreignObject {
                                x: "{mid_x - 50.0}",
                                y: "{mid_y - 12.0}",
                                width: "100",
                                height: "24",
                                input {
                                    value: "{edit_value}",
                                    class: "pointer-events-auto px-[6px] py-[2px] rounded border border-solid border-[var(--accent)] bg-[var(--bg-base)] text-[var(--text-main)] w-[100px] h-[22px] text-[11px]",
                                    onmousedown: move |evt| evt.stop_propagation(),
                                    oninput: move |evt| edit_value.set(evt.value()),
                                    onblur: move |_| {
                                        let (node_target, edge_target) = match *editor_state.read() {
                                            crate::ui::canvas::state::EditorState::EditingNode(ref id) => (Some(id.clone()), None),
                                            crate::ui::canvas::state::EditorState::EditingEdge(ref id) => (None, Some(id.clone())),
                                            _ => (None, None),
                                        };
                                        commit_inline_edit(
                                            doc_signal,
                                            history_signal,
                                            node_target,
                                            edge_target,
                                            edit_value,
                                            db_tx,
                                        )
                                        .ok();
                                        editor_state.set(crate::ui::canvas::state::EditorState::Idle);
                                    },
                                    onkeydown: move |evt| {
                                        if evt.key() == Key::Enter {
                                            let (node_target, edge_target) = match *editor_state.read() {
                                                crate::ui::canvas::state::EditorState::EditingNode(ref id) => (Some(id.clone()), None),
                                                crate::ui::canvas::state::EditorState::EditingEdge(ref id) => (None, Some(id.clone())),
                                                _ => (None, None),
                                            };
                                            commit_inline_edit(
                                                doc_signal,
                                                history_signal,
                                                node_target,
                                                edge_target,
                                                edit_value,
                                                db_tx,
                                            )
                                            .ok();
                                            editor_state.set(crate::ui::canvas::state::EditorState::Idle);
                                        } else if evt.key() == Key::Escape {
                                            editor_state.set(crate::ui::canvas::state::EditorState::Idle);
                                        }
                                    }
                                }
                            }
                        } else if !edge.label.is_empty() && zoom >= 0.3 {
                            text {
                                x: "{mid_x}",
                                y: "{mid_y - 6.0}",
                                text_anchor: "middle",
                                class: "fill-[var(--text-muted)]",
                                style: "font-size:{font_size}px;",
                                "{edge.label}"
                            }
                        } else if is_selected && zoom >= 0.3 {
                            text {
                                x: "{mid_x}",
                                y: "{mid_y - 6.0}",
                                text_anchor: "middle",
                                class: "fill-[var(--text-muted)] opacity-60 text-[9px]",
                                "label"
                            }
                        }
                    }
                }
            })
        }
    }
}

#[must_use]
pub fn get_visible_edges(
    doc: &DiagramDocument,
    camera_x: f64,
    camera_y: f64,
    zoom: f64,
    viewport_w: f64,
    viewport_h: f64,
) -> impl Iterator<Item = (&EdgeId, &Edge, &Node, &Node)> + use<'_> {
    let margin_x = (viewport_w / zoom).max(100.0) * 0.5;
    let margin_y = (viewport_h / zoom).max(100.0) * 0.5;
    let culling_min_x = camera_x - margin_x;
    let culling_min_y = camera_y - margin_y;
    let culling_max_x = camera_x + (viewport_w / zoom) + margin_x;
    let culling_max_y = camera_y + (viewport_h / zoom) + margin_y;

    doc.document.edges.iter().filter_map(move |(id, edge)| {
        doc.document
            .nodes
            .get(&edge.source)
            .and_then(|src| doc.document.nodes.get(&edge.target).map(|tgt| (src, tgt)))
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

                visible.then_some((id, edge, src, tgt))
            })
    })
}
