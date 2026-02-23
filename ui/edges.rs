use crate::graph::{Connection, Node, NodeId};
use dioxus::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

const NODE_WIDTH: f32 = 220.0;
const NODE_HEIGHT: f32 = 68.0;
const BEND_CLAMP: f32 = 200.0;

#[derive(Clone, Copy, PartialEq)]
struct EdgeAnchor {
    from: Position,
    to: Position,
}

#[derive(Clone)]
struct DragState {
    edge_id: String,
    start_y: f32,
    start_bend: f32,
}

fn get_source_point(node: &Node) -> Position {
    Position {
        x: node.x + NODE_WIDTH,
        y: node.y + NODE_HEIGHT / 2.0,
    }
}

fn get_target_point(node: &Node) -> Position {
    Position {
        x: node.x,
        y: node.y + NODE_HEIGHT / 2.0,
    }
}

fn create_smooth_step_path(from: Position, to: Position, bend_y: f32) -> (String, Position) {
    let mid_y = f32::midpoint(from.y, to.y) + bend_y.clamp(-BEND_CLAMP, BEND_CLAMP);
    let radius: f32 = 8.0;

    let dx = to.x - from.x;
    let dy = to.y - from.y;

    if dx.abs() < 2.0 || !dx.is_finite() || !dy.is_finite() {
        return (
            format!("M {} {} L {} {}", from.x, from.y, to.x, to.y),
            Position {
                x: f32::midpoint(from.x, to.x),
                y: mid_y,
            },
        );
    }

    let sign_x = if dx > 0.0 { 1.0 } else { -1.0 };
    let r = radius.min(dx.abs() / 2.0).min(dy.abs() / 4.0);

    (
        format!(
            "M {fx} {fy} L {fx} {my_r} Q {fx} {my} {fx_r} {my} L {tx_r} {my} Q {tx} {my} {tx} {my_r2} L {tx} {ty}",
            fx = from.x,
            fy = from.y,
            my = mid_y,
            my_r = mid_y - r,
            my_r2 = mid_y + r,
            fx_r = from.x + sign_x * r,
            tx_r = to.x - sign_x * r,
            tx = to.x,
            ty = to.y
        ),
        Position {
            x: f32::midpoint(from.x, to.x),
            y: mid_y,
        },
    )
}

fn resolve_edge_anchors(edges: &[Connection], nodes: &[Node]) -> HashMap<String, EdgeAnchor> {
    let node_by_id: HashMap<_, _> = nodes.iter().map(|node| (node.id, node)).collect();

    edges
        .iter()
        .filter_map(|edge| {
            let source = node_by_id.get(&edge.source)?;
            let target = node_by_id.get(&edge.target)?;
            let from = get_source_point(source);
            let to = get_target_point(target);
            Some((edge.id.to_string(), EdgeAnchor { from, to }))
        })
        .collect()
}

fn sanitize_bend_input(input: f32, start_bend: f32) -> f32 {
    if !input.is_finite() {
        return start_bend;
    }
    input.clamp(-BEND_CLAMP, BEND_CLAMP)
}

#[cfg(test)]
mod tests {
    use super::{create_smooth_step_path, sanitize_bend_input, Position};

    #[test]
    fn given_non_finite_bend_when_sanitizing_then_start_bend_is_kept() {
        let sanitized = sanitize_bend_input(f32::NAN, 12.0);

        assert_eq!(sanitized, 12.0);
    }

    #[test]
    fn given_bend_over_clamp_when_sanitizing_then_it_is_clamped() {
        let sanitized = sanitize_bend_input(500.0, 0.0);

        assert_eq!(sanitized, 200.0);
    }

    #[test]
    fn given_non_finite_positions_when_building_path_then_path_is_straight() {
        let from = Position { x: 10.0, y: 20.0 };
        let to = Position {
            x: f32::NAN,
            y: 40.0,
        };

        let (path, _) = create_smooth_step_path(from, to, 0.0);

        assert!(path.starts_with("M "));
    }
}

#[component]
pub fn FlowEdges(
    edges: ReadSignal<Vec<Connection>>,
    nodes: ReadSignal<Vec<Node>>,
    temp_edge: ReadSignal<Option<(Position, Position)>>,
    running_node_ids: ReadSignal<Vec<NodeId>>,
) -> Element {
    let mut hovered_edge = use_signal(|| None::<String>);
    let mut bend_offsets = use_signal(HashMap::<String, f32>::new);
    let mut drag_state = use_signal(|| None::<DragState>);

    let edge_anchors = use_memo(move || {
        let node_list = nodes.read();
        let edge_list = edges.read();
        resolve_edge_anchors(&edge_list, &node_list)
    });

    let node_by_id = use_memo(move || {
        nodes
            .read()
            .iter()
            .cloned()
            .map(|node| (node.id, node))
            .collect::<HashMap<_, _>>()
    });

    let temp_path = use_memo(move || {
        (*temp_edge.read()).map(|(from, to)| create_smooth_step_path(from, to, 0.0).0)
    });

    let svg_pointer_class = if drag_state.read().is_some() {
        "pointer-events-auto"
    } else {
        "pointer-events-none"
    };

    rsx! {
        svg {
            class: "absolute inset-0 overflow-visible {svg_pointer_class}",
            style: "width: 100%; height: 100%; z-index: 0;",
            onmousemove: move |evt| {
                if let Some(state) = drag_state.read().clone() {
                    let coordinates = evt.page_coordinates();
                    #[allow(clippy::cast_possible_truncation)]
                    let page_y = coordinates.y as f32;
                    if !page_y.is_finite() {
                        return;
                    }
                    let next_bend = sanitize_bend_input(state.start_bend + (page_y - state.start_y), state.start_bend);
                    bend_offsets.write().insert(state.edge_id, next_bend);
                }
            },
            onmouseup: move |_| {
                drag_state.set(None);
            },
            onmouseleave: move |_| {
                drag_state.set(None);
            },
            defs {
                marker {
                    id: "arrowhead",
                    marker_width: "10",
                    marker_height: "8",
                    ref_x: "9",
                    ref_y: "4",
                    orient: "auto",
                    path {
                        d: "M 0 0 L 10 4 L 0 8 z",
                        class: "fill-slate-600"
                    }
                }
                marker {
                    id: "arrowhead-active",
                    marker_width: "10",
                    marker_height: "8",
                    ref_x: "9",
                    ref_y: "4",
                    orient: "auto",
                    path {
                        d: "M 0 0 L 10 4 L 0 8 z",
                        class: "fill-indigo-500"
                    }
                }
            }

            for edge in edges.read().iter() {
                {
                    let edge_id = edge.id.to_string();
                    let anchor = edge_anchors.read().get(&edge_id).copied();

                    #[allow(clippy::option_if_let_else)]
                    if let Some(anchor) = anchor {
                        let bend = bend_offsets
                            .read()
                            .get(&edge_id)
                            .copied()
                            .map_or(0.0, |value| value);
                        let (path, midpoint) = create_smooth_step_path(anchor.from, anchor.to, bend);
                        let dragging_this = drag_state
                            .read()
                            .as_ref()
                            .is_some_and(|state| state.edge_id == edge_id);
                        let hovered_this = hovered_edge
                            .read()
                            .as_ref()
                            .is_some_and(|id| *id == edge_id);
                        let handle_opacity = if hovered_this || dragging_this { "1" } else { "0" };
                        let source_status = node_by_id
                            .read()
                            .get(&edge.source)
                            .and_then(|node| node.config.get("status"))
                            .and_then(serde_json::Value::as_str)
                            .map_or_else(|| "pending".to_string(), std::string::ToString::to_string);
                        let target_is_running = running_node_ids
                            .read()
                            .contains(&edge.target);
                        let stroke_color = match source_status {
                            ref status if status == "running" => "rgba(37, 99, 235, 0.95)",
                            ref status if status == "completed" => "rgba(16, 185, 129, 0.85)",
                            ref status if status == "failed" => "rgba(244, 63, 94, 0.85)",
                            _ => "rgba(148, 163, 184, 0.9)",
                        };
                        let marker = if source_status == "running" || target_is_running {
                            "url(#arrowhead-active)"
                        } else {
                            "url(#arrowhead)"
                        };
                        let dash = if source_status == "running" || target_is_running { "6 4" } else { "0" };
                        let animation_class = if target_is_running { "edge-animated" } else { "" };

                        rsx! {
                            g { key: "{edge_id}",
                                path {
                                    d: "{path}",
                                    fill: "none",
                                    stroke: "transparent",
                                    stroke_width: "16",
                                    pointer_events: "stroke",
                                    class: "pointer-events-auto",
                                    onmouseenter: {
                                        let edge_id = edge_id.clone();
                                        move |_| hovered_edge.set(Some(edge_id.clone()))
                                    },
                                    onmouseleave: {
                                        let edge_id = edge_id.clone();
                                        move |_| {
                                            let is_dragging = drag_state
                                                .read()
                                                .as_ref()
                                                .is_some_and(|state| state.edge_id == edge_id);
                                            if !is_dragging {
                                                hovered_edge.set(None);
                                            }
                                        }
                                    }
                                }
                                path {
                                    d: "{path}",
                                    fill: "none",
                                    stroke: "{stroke_color}",
                                    stroke_width: "2",
                                    marker_end: "{marker}",
                                    stroke_dasharray: "{dash}",
                                    class: "transition-all duration-150 {animation_class}",
                                    style: if target_is_running { Some("animation: flow 0.5s linear infinite") } else { None }
                                }
                                circle {
                                    cx: "{midpoint.x}",
                                    cy: "{midpoint.y}",
                                    r: "5",
                                    fill: "rgba(99, 102, 241, 0.95)",
                                    stroke: "rgba(226, 232, 240, 0.95)",
                                    stroke_width: "1.5",
                                    opacity: "{handle_opacity}",
                                    class: "pointer-events-auto cursor-ns-resize transition-opacity duration-100",
                                    onmousedown: {
                                        #[allow(clippy::redundant_clone)]
                                        let edge_id = edge_id.clone();
                                        move |evt| {
                                            evt.stop_propagation();
                                            let coordinates = evt.page_coordinates();
                                            #[allow(clippy::cast_possible_truncation)]
                                            let page_y = coordinates.y as f32;
                                            if !page_y.is_finite() {
                                                return;
                                            }
                    let current_bend = bend_offsets
                        .read()
                        .get(&edge_id)
                        .copied()
                        .map_or(0.0, |value| value);
                                            drag_state.set(Some(DragState {
                                                edge_id: edge_id.clone(),
                                                start_y: page_y,
                                                start_bend: current_bend,
                                            }));
                                            hovered_edge.set(Some(edge_id.clone()));
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        rsx! {}
                    }
                }
            }

            if let Some(path) = temp_path.read().as_ref() {
                path {
                    d: "{path}",
                    fill: "none",
                    stroke: "rgba(99, 102, 241, 0.6)",
                    stroke_width: "2",
                    stroke_dasharray: "6 4"
                }
            }
        }
    }
}
