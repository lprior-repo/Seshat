use crate::history::History;
use crate::ui::canvas::canvas_view::{edge_label_position, edge_marker_ref, edge_path};
use crate::ui::theme::{ACCENT, BG_BASE, EDGE_DEFAULT, EDGE_SELECTED, TEXT_MAIN, TEXT_MUTED};
use canvas_domain::interaction_reducer::commit_inline_edit;
use canvas_domain::perf::to_screen_coords;
use diagram_models::document::{DiagramDocument, Edge, EdgeId, EdgeStyle, Node, NodeId};
use dioxus::html::input_data::keyboard_types::Key;
use dioxus::prelude::*;

#[component]
pub fn EdgeLayer(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    mut editing_node: Signal<Option<NodeId>>,
    mut editing_edge: Signal<Option<EdgeId>>,
    mut edit_value: Signal<String>,
    viewport_size: Signal<(f64, f64)>,
    db_tx: Option<Coroutine<diagram_models::envelope::EventEnvelope>>,
) -> Element {
    let doc = doc_signal.read().clone();
    let s = doc.editor_state.clone();
    let selected_items = s
        .selected_items
        .iter()
        .cloned()
        .collect::<std::collections::HashSet<_>>();
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
    let edge_rows = doc
        .document
        .edges
        .iter()
        .filter_map(|(id, edge): (&EdgeId, &Edge)| {
            doc.document
                .nodes
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
        })
        .collect::<Vec<_>>();

    rsx! {
        {
            edge_rows.into_iter().map(move |(id, edge, src, tgt): (EdgeId, Edge, Node, Node)| {
                let canvas_domain::ScreenCoord(sx, sy) = to_screen_coords(canvas_domain::CanvasCoord(src.x.0 + src.width.0 / 2.0, src.y.0 + src.height.0 / 2.0), canvas_domain::CanvasCoord(camera_x, camera_y), zoom);
                let canvas_domain::ScreenCoord(tx, ty) = to_screen_coords(canvas_domain::CanvasCoord(tgt.x.0 + tgt.width.0 / 2.0, tgt.y.0 + tgt.height.0 / 2.0), canvas_domain::CanvasCoord(camera_x, camera_y), zoom);
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
    }
}
