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
use diagram_models::document::{DiagramDocument, EdgeId, Node, NodeId, NodeKind};

#[derive(Props, Clone, PartialEq)]
pub struct NodeElementProps {
    pub id: NodeId,
    pub node: Node,
    pub is_selected: bool,
    pub is_hovered: bool,
    pub is_editing: bool,
    pub doc_signal: Signal<DiagramDocument>,
    pub history_signal: Signal<History>,
    pub tool_signal: Signal<ToolMode>,
    pub interaction_mode: Signal<InteractionMode>,
    pub editing_node: Signal<Option<NodeId>>,
    pub editing_edge: Signal<Option<EdgeId>>,
    pub edit_value: Signal<String>,
    pub hovered_node: Signal<Option<NodeId>>,
    pub canvas_origin: Signal<(f64, f64)>,
    pub shift_pressed: Signal<bool>,
    pub ctrl_pressed: Signal<bool>,
    pub meta_pressed: Signal<bool>,
    pub space_pressed: Signal<bool>,
    pub multi_touch_active: Signal<bool>,
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
    let editing_node = props.editing_node;
    let editing_edge = props.editing_edge;
    let edit_value = props.edit_value;
    let mut hovered_node = props.hovered_node;
    let canvas_origin = props.canvas_origin;
    let shift_pressed = props.shift_pressed;
    let ctrl_pressed = props.ctrl_pressed;
    let meta_pressed = props.meta_pressed;
    let space_pressed = props.space_pressed;
    let multi_touch_active = props.multi_touch_active;
    let space_pan_active = props.space_pan_active;
    let db_tx = props.db_tx;
    let pending_pointer_sample = props.pending_pointer_sample;

    let edge_style_default = use_context::<Signal<diagram_models::document::EdgeStyle>>();
    let arrow_type_default = use_context::<Signal<diagram_models::document::ArrowType>>();
    let toast = crate::ui::toast::use_toast();

    let node = props.node;
    let id = props.id;
    let id_mousedown = id.clone();
    let id_mouseup = id.clone();
    let id_mouseenter = id.clone();
    let id_mouseleave = id.clone();
    let id_data_attr = id.to_string();
    let is_selected = props.is_selected;
    let is_hovered = props.is_hovered;
    let is_editing_node = props.is_editing;
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
            style: "position: absolute; left: {left}px; top: {top}px; width: {width}px; height: {height}px; border: {border_width}px solid color-mix(in oklch, {border_base} {border_mix}%, transparent); border-radius: 10px; background: linear-gradient(180deg, color-mix(in oklch, {bg} 92%, {BG_BASE}) 0%, {bg} 100%); display: flex; flex-direction: column; align-items: center; justify-content: center; cursor: inherit; z-index: {z_index}; box-shadow: 0 6px 18px color-mix(in oklch, black 24%, transparent);",

            onmouseenter: move |_| hovered_node.set(Some(id_mouseenter.clone())),
            onmouseleave: move |_| {
                if hovered_node.read().as_ref() == Some(&id_mouseleave) {
                    hovered_node.set(None);
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
                style: "position:absolute; inset:0; pointer-events:none; opacity:0;"
            }

            {
                let content_props = super::node_content::NodeContentProps {
                    node: node.clone(),
                    id: id.clone(),
                    is_editing_node,
                    font_px,
                    provider_top: provider_top.to_string(),
                    node_initials,
                    width,
                    height,
                    zoom,
                    doc_signal,
                    history_signal,
                    editing_node,
                    editing_edge,
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
