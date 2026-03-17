use dioxus::prelude::*;

use crate::ui::canvas::document_ops::scale_selected_nodes;
use crate::{
    history::History,
    ui::theme::{BG_BASE, BORDER, TEXT_MAIN, TEXT_MUTED, TOOLBAR_BG},
};
use canvas_domain::interaction_reducer::InteractionMode;
use canvas_domain::perf::to_screen_coords;
use canvas_domain::selection_geometry::{selected_node_ids, selection_bounds};
use diagram_models::document::{DiagramDocument, EdgeId, OrderedFloat};

#[component]
pub fn Toolbar(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    interaction_mode: Signal<InteractionMode>,
) -> Element {
    let doc = doc_signal.read().clone();
    let mode = interaction_mode.read().clone();
    let hide_toolbar = matches!(
        mode,
        InteractionMode::DraggingSelection { .. }
            | InteractionMode::ResizingSelection { .. }
            | InteractionMode::RubberBand { .. }
    );
    let selected_nodes = selected_node_ids(&doc);
    let selected_edges = doc
        .editor_state
        .selected_items
        .iter()
        .filter(|id| doc.document.edges.contains_key(&EdgeId::new((*id).clone())))
        .count();

    if hide_toolbar {
        return rsx! {};
    }

    if let Some((bx, by, bw, _bh)) = selection_bounds(&doc) {
        let s = &doc.editor_state;
        let canvas_domain::ScreenCoord(screen_x, screen_y) = to_screen_coords(
            canvas_domain::CanvasCoord(bx + (bw / 2.0), by),
            canvas_domain::CanvasCoord(s.camera_x.0, s.camera_y.0),
            s.zoom.0,
        );
        let top: f64 = (screen_y - 48.0).max(4.0);
        let can_scale = !selected_nodes.is_empty();
        let scale_cursor = if can_scale { "pointer" } else { "not-allowed" };
        let scale_opacity = if can_scale { "1" } else { "0.5" };
        let first_node = selected_nodes
            .first()
            .and_then(|id| doc.document.nodes.get(&id));
        let first_edge = if selected_nodes.is_empty() && selected_edges == 1 {
            doc.editor_state
                .selected_items
                .iter()
                .find_map(|id| doc.document.edges.get(&EdgeId::new(id.clone())))
        } else {
            None
        };
        let font_size = first_node
            .and_then(|n| n.font_size)
            .or_else(|| first_edge.and_then(|e| e.font_size))
            .map_or(11.0, |v| v.0);
        rsx! {
            div {
                style: "position:absolute; left:{screen_x}px; top:{top}px; transform: translateX(-50%); z-index:25; display:flex; align-items:center; gap:6px; padding:6px 8px; border:1px solid {BORDER}; border-radius:8px; background:{TOOLBAR_BG}f2; backdrop-filter: blur(6px);",
                onmousedown: move |evt| evt.stop_propagation(),

                button {
                    style: "border:1px solid {BORDER}; border-radius:6px; background:{BG_BASE}; color:{TEXT_MAIN}; width:24px; height:24px; cursor:pointer;",
                    onclick: move |_| {
                        let selected = doc_signal.read().editor_state.selected_items.clone();
                        let current = doc_signal.read().clone();
                        let history = history_signal.read().clone();
                        *history_signal.write() = history.push(current);
                        doc_signal.with_mut(|d| {
                            for selected_id in selected.iter() {
                                let nid = diagram_models::document::NodeId::new(selected_id.clone());
                                if let Some(node) = d.document.nodes.get_mut(&nid) {
                                    let next = node.font_size.map_or(10.0, |v| v.0) - 1.0;
                                    node.font_size = Some(OrderedFloat(next.clamp(8.0, 72.0)));
                                } else {
                                    let eid = EdgeId::new(selected_id.clone());
                                    if let Some(edge) = d.document.edges.get_mut(&eid) {
                                        let next = edge.font_size.map_or(10.0, |v| v.0) - 1.0;
                                        edge.font_size = Some(OrderedFloat(next.clamp(8.0, 72.0)));
                                    }
                                }
                            }
                            d.revision = d.revision.increment();
                        });
                    },
                    "-"
                }
                span { style: "font-size:11px; color:{TEXT_MUTED}; min-width:26px; text-align:center;", "{font_size.round()}" }
                button {
                    style: "border:1px solid {BORDER}; border-radius:6px; background:{BG_BASE}; color:{TEXT_MAIN}; width:24px; height:24px; cursor:pointer;",
                    onclick: move |_| {
                        let selected = doc_signal.read().editor_state.selected_items.clone();
                        let current = doc_signal.read().clone();
                        let history = history_signal.read().clone();
                        *history_signal.write() = history.push(current);
                        doc_signal.with_mut(|d| {
                            for selected_id in selected.iter() {
                                let nid = diagram_models::document::NodeId::new(selected_id.clone());
                                if let Some(node) = d.document.nodes.get_mut(&nid) {
                                    let next = node.font_size.map_or(10.0, |v| v.0) + 1.0;
                                    node.font_size = Some(OrderedFloat(next.clamp(8.0, 72.0)));
                                } else {
                                    let eid = EdgeId::new(selected_id.clone());
                                    if let Some(edge) = d.document.edges.get_mut(&eid) {
                                        let next = edge.font_size.map_or(10.0, |v| v.0) + 1.0;
                                        edge.font_size = Some(OrderedFloat(next.clamp(8.0, 72.0)));
                                    }
                                }
                            }
                            d.revision = d.revision.increment();
                        });
                    },
                    "+"
                }
            }
        }
    } else {
        rsx! {}
    }
}

#[component]
pub fn SelectionPill(doc_signal: Signal<DiagramDocument>) -> Element {
    let selected_count = doc_signal.read().editor_state.selected_items.len();
    let plural = if selected_count == 1 { "" } else { "s" };
    if selected_count > 0 {
        rsx! {
            div {
                style: "position:absolute; left:12px; bottom:12px; z-index:20; border:1px solid {BORDER}; border-radius:8px; background:{TOOLBAR_BG}e8; color:{TEXT_MUTED}; font-size:11px; padding:5px 9px; backdrop-filter: blur(6px); box-shadow: 0 4px 12px color-mix(in oklch, black 24%, transparent);",
                "{selected_count} item{plural} selected"
            }
        }
    } else {
        rsx! {}
    }
}
