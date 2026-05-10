use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

use crate::ui::canvas::document_ops::{snap_edge_port_toward, sync_canvas_origin};
use crate::ui::editor::ToolMode;
use crate::ui::theme::{ACCENT, BG_BASE};
use canvas_domain::interaction_reducer::InteractionMode;
use canvas_domain::perf::to_canvas_coords;
use diagram_models::document::{DiagramDocument, NodeId};

#[derive(Props, Clone, PartialEq)]
pub struct ConnectionDotsProps {
    pub id: NodeId,
    pub width: f64,
    pub height: f64,
    pub is_selected: bool,
    pub is_hovered: bool,
    pub is_editing: bool,
    pub tool_signal: Signal<ToolMode>,
    pub interaction_mode: Signal<InteractionMode>,
    pub canvas_origin: ReadSignal<(f64, f64)>,
    pub doc_signal: Signal<DiagramDocument>,
}

#[component]
pub fn ConnectionDots(props: ConnectionDotsProps) -> Element {
    let drawing_from_this_node = {
        let mode = props.interaction_mode.read();
        matches!(&*mode, InteractionMode::DrawingEdge { from_node, .. } if from_node == &props.id)
    };
    let edge_tool_active = *props.tool_signal.read() == ToolMode::Edge;
    let broad_hit_zones_active = edge_tool_active || drawing_from_this_node;
    let show_connection_dots = !props.is_editing
        && (props.is_selected || props.is_hovered || edge_tool_active || drawing_from_this_node);

    if !show_connection_dots {
        return rsx! {};
    }

    let cx = props.width / 2.0;
    let cy = props.height / 2.0;
    let dot_specs = [
        ("top", cx, 0.0),
        ("bottom", cx, props.height),
        ("left", 0.0, cy),
        ("right", props.width, cy),
    ];
    let hit_zones = [
        (
            "top",
            format!(
                "position: absolute; width: {}px; height: 16px; left: 0px; top: -8px; background: transparent; cursor: crosshair;",
                props.width
            ),
        ),
        (
            "bottom",
            format!(
                "position: absolute; width: {}px; height: 16px; left: 0px; top: {}px; background: transparent; cursor: crosshair;",
                props.width,
                props.height - 8.0
            ),
        ),
        (
            "left",
            format!(
                "position: absolute; width: 16px; height: {}px; left: -8px; top: 0px; background: transparent; cursor: crosshair;",
                props.height
            ),
        ),
        (
            "right",
            format!(
                "position: absolute; width: 16px; height: {}px; left: {}px; top: 0px; background: transparent; cursor: crosshair;",
                props.height,
                props.width - 8.0
            ),
        ),
    ];

    let id = props.id.clone();
    let canvas_origin = props.canvas_origin;
    let doc_signal = props.doc_signal;
    let interaction_mode = props.interaction_mode;

    rsx! {
        if broad_hit_zones_active {
            for (side, zone_style) in hit_zones {
                div {
                    key: "hit-{side}",
                    style: "{zone_style}",
                    "data-testid": "connection-edge-hit-zone",
                    "data-side": "{side}",
                    onmousedown: {
                        let current_id = id.clone();
                        move |evt| start_edge_from_pointer(&evt, &current_id, canvas_origin, doc_signal, interaction_mode)
                    },
                    ondoubleclick: {
                        let current_id = id.clone();
                        move |evt| start_edge_from_pointer(&evt, &current_id, canvas_origin, doc_signal, interaction_mode)
                    }
                }
            }
        }
        for (side, dot_x, dot_y) in dot_specs {
            div {
                key: "{side}",
                style: "position: absolute; width: 20px; height: 20px; border-radius: 999px; background: transparent; cursor: crosshair; pointer-events: auto; left: {dot_x - 10.0}px; top: {dot_y - 10.0}px;",
                "data-testid": "connection-dot",
                "data-side": "{side}",
                title: "Create connection",
                "aria-label": "Create connection from {side} handle",
                onmousedown: {
                    let current_id = id.clone();
                    move |evt| {
                        if broad_hit_zones_active {
                            start_edge_from_pointer(&evt, &current_id, canvas_origin, doc_signal, interaction_mode);
                        }
                    }
                },
                ondoubleclick: {
                    let current_id = id.clone();
                    move |evt| start_edge_from_pointer(&evt, &current_id, canvas_origin, doc_signal, interaction_mode)
                },
                div {
                    style: "position: absolute; left: 5px; top: 5px; width: 10px; height: 10px; border-radius: 999px; opacity: 0.9; pointer-events: none; background:{ACCENT}; border:1px solid {BG_BASE};"
                }
            }
        }
    }
}

fn start_edge_from_pointer(
    evt: &Event<MouseData>,
    current_id: &NodeId,
    canvas_origin: ReadSignal<(f64, f64)>,
    doc_signal: Signal<DiagramDocument>,
    mut interaction_mode: Signal<InteractionMode>,
) {
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
        doc.editor_state.zoom.0,
    );
    let start_port = doc
        .document
        .nodes
        .get(current_id)
        .map(|src| snap_edge_port_toward(src, mouse_pos.0, mouse_pos.1));
    interaction_mode.set(InteractionMode::DrawingEdge {
        from_node: current_id.clone(),
        current_pos: (mouse_pos.0, mouse_pos.1),
        start_port,
    });
}
