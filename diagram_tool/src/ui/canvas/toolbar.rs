use dioxus::prelude::*;

use crate::{
    history::History,
    ui::theme::{BORDER, TOOLBAR_BG},
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
        let _scale_cursor = if can_scale { "pointer" } else { "not-allowed" };
        let _scale_opacity = if can_scale { "1" } else { "0.5" };
        let first_node = selected_nodes
            .first()
            .and_then(|id| doc.document.nodes.get(id));
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
                class: "absolute -translate-x-1/2 flex items-center gap-[6px] px-[8px] py-[6px] rounded-[8px] backdrop-blur-sm",
                style: "left: {screen_x}px; top: {top}px; z-index: 25; border: 1px solid {BORDER}; background: {TOOLBAR_BG}f2;",
                onmousedown: move |evt| evt.stop_propagation(),
                onclick: move |evt| evt.stop_propagation(),
                ondoubleclick: move |evt| evt.stop_propagation(),

                button {
                    "data-testid": "selection-font-decrease",
                    class: "border border-border rounded-md bg-[var(--bg-base)] text-foreground w-6 h-6 cursor-pointer flex items-center justify-center",
                    title: "Decrease font size",
                    "aria-label": "Decrease selected font size",
                    onmousedown: move |evt| evt.stop_propagation(),
                    ondoubleclick: move |evt| evt.stop_propagation(),
                    onclick: move |evt| {
                        evt.stop_propagation();
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
                span { class: "text-[11px] text-muted-foreground min-w-[26px] text-center", "{font_size.round()}" }
                button {
                    "data-testid": "selection-font-increase",
                    class: "border border-border rounded-md bg-[var(--bg-base)] text-foreground w-6 h-6 cursor-pointer flex items-center justify-center",
                    title: "Increase font size",
                    "aria-label": "Increase selected font size",
                    onmousedown: move |evt| evt.stop_propagation(),
                    ondoubleclick: move |evt| evt.stop_propagation(),
                    onclick: move |evt| {
                        evt.stop_propagation();
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
                class: "absolute left-[12px] bottom-[12px] z-[20] rounded-[8px] text-[11px] px-[9px] py-[5px] backdrop-blur-sm border border-border bg-[var(--toolbar-bg)]/90 text-muted-foreground shadow-lg",
                "{selected_count} item{plural} selected"
            }
        }
    } else {
        rsx! {}
    }
}
