use dioxus::prelude::*;

use crate::ui::canvas::document_ops::{initials, provider_color};
use crate::{
    history::History,
    ui::{
        editor::ToolMode,
        theme::{ACCENT, BG_BASE, NODE_BG, NODE_BG_SUBGRAPH, NODE_BORDER},
    },
};
use canvas_domain::interaction_reducer::InteractionMode;
use canvas_domain::perf::to_screen_coords;
use diagram_models::document::{DiagramDocument, Node, NodeId, NodeKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NodeInteractionState {
    #[default]
    Normal,
    Hovered,
    Selected {
        hovered: bool,
    },
    Editing,
}

impl NodeInteractionState {
    pub fn is_selected(self) -> bool {
        matches!(self, Self::Selected { .. } | Self::Editing)
    }

    pub fn is_hovered(self) -> bool {
        matches!(self, Self::Hovered | Self::Selected { hovered: true })
    }

    pub fn is_editing(self) -> bool {
        matches!(self, Self::Editing)
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct NodeElementProps {
    pub id: NodeId,
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

    let app_state = use_context::<crate::app::AppState>();
    let edge_style_default = app_state.edge_style;
    let arrow_type_default = app_state.arrow_type;
    let toast = crate::ui::toast::use_toast();

    let node = props.node;
    let id = props.id;
    let id_mousedown = id.clone();
    let id_mouseup = id.clone();
    let id_mouseenter = id.clone();
    let id_mouseleave = id.clone();
    let id_data_attr = id.to_string();
    let interaction_state = props.interaction_state;
    let is_selected = interaction_state.is_selected();
    let is_hovered = interaction_state.is_hovered();
    let camera_x = props.camera_x;
    let camera_y = props.camera_y;
    let zoom = props.zoom;

    let canvas_domain::ScreenCoord(left, top) = to_screen_coords(
        canvas_domain::CanvasCoord(node.x.0, node.y.0),
        canvas_domain::CanvasCoord(camera_x, camera_y),
        zoom,
    );
    let (width, height) = (node.width.0 * zoom, node.height.0 * zoom);

    let border_width = if is_selected { "2" } else { "1" };
    let border_base = if is_selected || is_hovered {
        ACCENT
    } else {
        NODE_BORDER
    };
    let border_mix = if is_hovered && !is_selected {
        "50"
    } else {
        "100"
    };
    let bg = if node.kind == NodeKind::Subgraph {
        NODE_BG_SUBGRAPH
    } else {
        NODE_BG
    };
    let z_index = node.z_index
        + if node.kind == NodeKind::Subgraph {
            10
        } else {
            1000
        };
    let font_px = node.font_size.map_or(11.0, |f| f.0) * zoom;

    let fallback_provider = node.icon.split('/').next().map_or("generic", |p| p);
    let provider = node
        .tags
        .front()
        .map_or(fallback_provider, |p: &String| p.as_str());
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
            class: "absolute flex flex-col items-center justify-center cursor-inherit rounded-[10px]",
            style: "left: {left}px; top: {top}px; width: {width}px; height: {height}px; z-index: {z_index}; border: {border_width}px solid color-mix(in oklch, {border_base} {border_mix}%, transparent); background: linear-gradient(180deg, color-mix(in oklch, {bg} 92%, {BG_BASE}) 0%, {bg} 100%); box-shadow: 0 6px 18px color-mix(in oklch, black 24%, transparent);",

            onmouseenter: move |_| editor_state.set(crate::ui::canvas::state::EditorState::HoveringNode(id_mouseenter.clone())),
            onmouseleave: move |_| {
                let should_clear = if let crate::ui::canvas::state::EditorState::HoveringNode(ref hid) = *editor_state.read() {
                    hid == &id_mouseleave
                } else { false };
                if should_clear {
                    editor_state.set(crate::ui::canvas::state::EditorState::Idle);
                }
            },

            onmousedown: move |evt| {
                let tool = *tool_signal.read();
                let doc = doc_signal.read().clone();
                let additive = *shift_pressed.read() || *ctrl_pressed.read() || *meta_pressed.read();
                super::handlers::handle_mousedown(
                    evt,
                    id_mousedown.clone(),
                    *multi_touch_active.read(),
                    tool,
                    doc,
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
                    id_mouseup.clone(),
                    doc_signal,
                    history_signal,
                    interaction_mode,
                    pending_pointer_sample,
                    db_tx,
                    tool_signal,
                    *edge_style_default.read(),
                    *arrow_type_default.read(),
                    *canvas_origin.read(),
                    toast,
                );
            },

            div {
                "data-testid": "node-hitbox",
                class: "absolute opacity-0 inset-0 pointer-events-none"
            }

            {
                let content_props = super::node_content::NodeContentProps {
                    node: node.clone(),
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
                    id: id.clone(),
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
