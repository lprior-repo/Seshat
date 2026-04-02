use dioxus::prelude::*;

use crate::ui::canvas::document_ops::{initials, provider_color};
use crate::{history::History, ui::editor::ToolMode};
use canvas_domain::interaction_reducer::InteractionMode;
use canvas_domain::perf::to_screen_coords;
use diagram_models::document::{DiagramDocument, Node, NodeId, NodeKind};

use super::render_data::{NodeInteractionState, NodeRenderData};

#[derive(Props, Clone)]
pub struct NodeElementProps {
    pub id: NodeId,
    pub render_data: NodeRenderData,
    /// Full node reference for event handlers (not used in `PartialEq` diff path).
    pub node: Node,
    pub interaction_state: NodeInteractionState,
    pub doc_signal: Signal<DiagramDocument>,
    pub history_signal: Signal<History>,
    pub tool_signal: Signal<ToolMode>,
    pub interaction_mode: Signal<InteractionMode>,
    pub editor_state: Signal<crate::ui::canvas::state::EditorState>,
    pub edit_value: Signal<String>,
    pub canvas_origin: ReadSignal<(f64, f64)>,
    pub shift_pressed: ReadSignal<bool>,
    pub ctrl_pressed: ReadSignal<bool>,
    pub meta_pressed: ReadSignal<bool>,
    pub space_pressed: ReadSignal<bool>,
    pub multi_touch_active: ReadSignal<bool>,
    pub space_pan_active: Signal<bool>,
    pub db_tx: Option<Coroutine<diagram_models::envelope::EventEnvelope>>,
    pub pending_pointer_sample: Signal<Option<(f64, f64)>>,
    pub geometry_render_tick: Signal<u64>,
    pub camera_x: f64,
    pub camera_y: f64,
    pub zoom: f64,
}

#[component]
pub fn NodeElement(props: NodeElementProps) -> Element {
    let doc_signal = props.doc_signal;
    let history_signal = props.history_signal;
    let tool_signal = props.tool_signal;
    let interaction_mode = props.interaction_mode;
    let mut editor_state = props.editor_state;
    let edit_value = props.edit_value;
    let canvas_origin = props.canvas_origin;
    let shift_pressed = props.shift_pressed;
    let ctrl_pressed = props.ctrl_pressed;
    let meta_pressed = props.meta_pressed;
    let space_pressed = props.space_pressed;
    let multi_touch_active = props.multi_touch_active;
    let space_pan_active = props.space_pan_active;
    let db_tx = props.db_tx;
    let pending_pointer_sample = props.pending_pointer_sample;
    let geometry_render_tick = props.geometry_render_tick;

    let app_state = use_context::<crate::app::AppState>();
    let edge_style_default = app_state.edge_style;
    let arrow_type_default = app_state.arrow_type;
    let toast = crate::ui::toast::use_toast();

    let rd = &props.render_data;
    let node = props.node;
    let id = props.id;
    let id_data_attr = id.to_string();
    // Clone id separately for each event handler to avoid moving id multiple times
    let id_for_enter = id.clone();
    let id_for_leave = id.clone();
    let id_for_down = id.clone();
    let id_for_up = id.clone();
    let interaction_state = props.interaction_state;
    let is_selected = interaction_state.is_selected();
    let is_hovered = interaction_state.is_hovered();
    let camera_x = props.camera_x;
    let camera_y = props.camera_y;
    let zoom = props.zoom;

    let canvas_domain::ScreenCoord(left, top) = to_screen_coords(
        canvas_domain::CanvasCoord(rd.x, rd.y),
        canvas_domain::CanvasCoord(camera_x, camera_y),
        zoom,
    );
    let (width, height) = (rd.width * zoom, rd.height * zoom);

    let z_index = rd.z_index
        + if rd.kind == NodeKind::Subgraph {
            10
        } else {
            1000
        };
    let font_px = rd.font_size.map_or(11.0, |f| f) * zoom;

    let fallback_provider = rd.icon.split('/').next().map_or("generic", |p| p);
    let provider = rd
        .tags
        .front()
        .map_or(fallback_provider, |p: &String| p.as_str());
    let provider_top = provider_color(provider);
    let node_initials = initials(&rd.label);

    // CSS class determines border/background/box-shadow state.
    // Using pre-defined classes avoids per-node color-mix() evaluation at paint time.
    let node_state_class = if is_selected && is_hovered {
        "diagram-node-selected-hovered"
    } else if is_selected {
        "diagram-node-selected"
    } else if is_hovered {
        "diagram-node-hovered"
    } else {
        "diagram-node"
    };
    let bg_class = if rd.kind == NodeKind::Subgraph {
        "diagram-node-subgraph"
    } else {
        ""
    };

    rsx! {
            div {
                "data-testid": "node",
                "data-node-id": "{id_data_attr}",
                "data-node-kind": match rd.kind {
                    NodeKind::Node => "node",
                    NodeKind::Subgraph => "subgraph",
                    NodeKind::Text => "text",
                },
                class: "absolute flex flex-col items-center justify-center cursor-inherit rounded-[10px {node_state_class} {bg_class}",
                style: "left: 0; top: 0; transform: translate3d({left}px, {top}px, 0); width: {width}px; height: {height}px; z-index: {z_index}; contain: layout style; will-change: transform;",

            onmouseenter: move |_| editor_state.set(crate::ui::canvas::state::EditorState::HoveringNode(id_for_enter.clone())),
            onmouseleave: move |_| {
                let should_clear = if let crate::ui::canvas::state::EditorState::HoveringNode(ref hid) = *editor_state.read() {
                    hid == &id_for_leave.clone()
                } else { false };
                if should_clear {
                    editor_state.set(crate::ui::canvas::state::EditorState::Idle);
                }
            },

            onmousedown: move |evt| {
                let tool = *tool_signal.read();
                let camera = {
                    let doc = doc_signal.read();
                    (doc.editor_state.camera_x.0, doc.editor_state.camera_y.0, doc.editor_state.zoom.0)
                };
                // read lock dropped before handler call
                let additive = *shift_pressed.read() || *ctrl_pressed.read() || *meta_pressed.read();
                super::handlers::handle_mousedown(
                    evt,
                    id_for_down.clone(),
                    *multi_touch_active.read(),
                    tool,
                    camera,
                    additive,
                    *canvas_origin.read(),
                    interaction_mode,
                    doc_signal,
                    space_pan_active,
                    *space_pressed.read(),
                );
            },

            onmouseup: move |evt| {
                super::handlers::handle_mouseup(
                    evt,
                    id_for_up.clone(),
                    doc_signal,
                    history_signal,
                    interaction_mode,
                    pending_pointer_sample,
                    geometry_render_tick,
                    db_tx,
                    tool_signal,
                    *edge_style_default.read(),
                    *arrow_type_default.read(),
                    *canvas_origin.read(),
                    toast,
                );
            },

            {
                let content_props = super::node_content::NodeContentProps {
                    node,
                    id: id.clone(),
                    interaction_state,
                    font_px,
                    provider_top: provider_top.to_string(),
                    node_initials,
                    width,
                    height,
                    zoom,
                    doc_signal,
                    history_signal,
                    editor_state,
                    edit_value,
                    db_tx,
                };
                rsx! { super::node_content::NodeContent { ..content_props } }
            }

            {
                let dots_props = super::connection_dots::ConnectionDotsProps {
                    id,
                    width,
                    height,
                    is_selected,
                    is_hovered,
                    is_editing: interaction_state.is_editing(),
                    tool_signal,
                    interaction_mode,
                    canvas_origin,
                    doc_signal,
                };
                rsx! { super::connection_dots::ConnectionDots { ..dots_props } }
            }
        }
    }
}
