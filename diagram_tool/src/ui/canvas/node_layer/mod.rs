pub mod connection_dots;
pub mod handlers;
pub mod inline_edit;
pub mod node_content;
pub mod node_element;

use std::collections::HashSet;

use canvas_domain::interaction_reducer::InteractionMode;
use diagram_models::document::{DiagramDocument, EdgeId, Node, NodeId};
use dioxus::prelude::*;

use crate::history::History;
use crate::ui::editor::ToolMode;

use self::node_element::{NodeElement, NodeElementProps};

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
    let pending_pointer_sample = use_signal(|| Option::<(f64, f64)>::None);
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

    let node_rows = ordered_node_cache
        .read()
        .iter()
        .filter_map(|id: &NodeId| {
            doc_for_nodes.document.nodes.get(id).and_then(|node| {
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

    rsx! {
        {
            node_rows.into_iter().map(move |(id, node): (NodeId, Node)| {
                let is_selected = selected_items.contains(id.as_str());
                let is_hovered = hovered_now.as_ref() == Some(&id);
                let is_editing = editing_node.read().as_ref() == Some(&id);

                let props = NodeElementProps {
                    id: id.clone(),
                    node,
                    is_selected,
                    is_hovered,
                    is_editing,
                    doc_signal,
                    history_signal,
                    tool_signal,
                    interaction_mode,
                    editing_node,
                    editing_edge,
                    edit_value,
                    hovered_node,
                    canvas_origin,
                    shift_pressed,
                    ctrl_pressed,
                    meta_pressed,
                    space_pressed,
                    multi_touch_active,
                    space_pan_active,
                    db_tx,
                    pending_pointer_sample,
                    camera_x,
                    camera_y,
                    zoom,
                };

                rsx! {
                    NodeElement { ..props }
                }
            })
        }
    }
}
