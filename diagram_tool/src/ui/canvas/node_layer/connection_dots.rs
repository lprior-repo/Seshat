use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

use crate::ui::canvas::document_ops::sync_canvas_origin;
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
    let show_connection_dots = !props.is_editing
        && (props.is_selected || props.is_hovered || *props.tool_signal.read() == ToolMode::Edge);

    if !show_connection_dots {
        return rsx! {};
    }

    let cx = props.width / 2.0;
    let cy = props.height / 2.0;
    let dot_specs = [(cx, 0.0), (cx, props.height), (0.0, cy), (props.width, cy)];

    let id = props.id.clone();
    let canvas_origin = props.canvas_origin;
    let doc_signal = props.doc_signal;
    let mut interaction_mode = props.interaction_mode;

    rsx! {
        for (dot_x, dot_y) in dot_specs {
            div {
                style: "position: absolute; width: 20px; height: 20px; border-radius: 999px; background: transparent; cursor: crosshair; left: {dot_x - 10.0}px; top: {dot_y - 10.0}px;",
                "data-testid": "connection-dot",
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
                    style: "position: absolute; left: 5px; top: 5px; width: 10px; height: 10px; border-radius: 999px; opacity: 0.9; pointer-events: none; background:{ACCENT}; border:1px solid {BG_BASE};"
                }
            }
        }
    }
}
