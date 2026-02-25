#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use super::selection_geometry::{selected_node_ids, selection_bounds};
use crate::history::History;
use crate::models::document::{DiagramDocument, EdgeId, NodeId};
use dioxus::prelude::*;
use im::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub(super) enum InteractionMode {
    Select,
    RubberBand {
        start: (f64, f64),
        current: (f64, f64),
    },
    DraggingSelection {
        anchor: (f64, f64),
        original_positions: HashMap<NodeId, (f64, f64)>,
        did_move: bool,
    },
    DrawingEdge {
        from_node: NodeId,
        current_pos: (f64, f64),
    },
    DrawingSubgraph {
        start: (f64, f64),
        current: (f64, f64),
    },
    ResizingSelection {
        handle: ResizeHandle,
        original_bounds: (f64, f64, f64, f64),
        originals: HashMap<NodeId, (f64, f64, f64, f64)>,
        anchor: (f64, f64),
        did_resize: bool,
    },
    Panning {
        last_pos: (f64, f64),
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ResizeHandle {
    Nw,
    N,
    Ne,
    E,
    Se,
    S,
    Sw,
    W,
}

pub(super) fn commit_inline_edit(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    mut editing_node: Signal<Option<NodeId>>,
    mut editing_edge: Signal<Option<EdgeId>>,
    edit_value: Signal<String>,
) {
    let node_target = editing_node.read().clone();
    if let Some(node_id) = node_target {
        let new_label = edit_value.read().clone();
        let target = node_id;
        let current_label = doc_signal
            .read()
            .document
            .nodes
            .get(&target)
            .map_or_else(String::new, |n| n.label.clone());
        if current_label != new_label {
            let current = doc_signal.read().clone();
            let history = history_signal.read().clone();
            *history_signal.write() = history.push(current);
            doc_signal.with_mut(|doc| {
                if let Some(n) = doc.document.nodes.get_mut(&target) {
                    n.label = new_label;
                    doc.revision = doc.revision.increment();
                }
            });
        }
        editing_node.set(None);
        return;
    }

    let edge_target = editing_edge.read().clone();
    if let Some(edge_id) = edge_target {
        let new_label = edit_value.read().clone();
        let target = edge_id;
        let current_label = doc_signal
            .read()
            .document
            .edges
            .get(&target)
            .map_or_else(String::new, |e| e.label.clone());
        if current_label != new_label {
            let current = doc_signal.read().clone();
            let history = history_signal.read().clone();
            *history_signal.write() = history.push(current);
            doc_signal.with_mut(|doc| {
                if let Some(e) = doc.document.edges.get_mut(&target) {
                    e.label = new_label;
                    doc.revision = doc.revision.increment();
                }
            });
        }
        editing_edge.set(None);
    }
}

pub(super) fn start_resize_interaction(
    mut interaction_mode: Signal<InteractionMode>,
    doc_signal: Signal<DiagramDocument>,
    handle: ResizeHandle,
    client_x: f64,
    client_y: f64,
) {
    let doc = doc_signal.read().clone();
    if let Some(bounds) = selection_bounds(&doc) {
        let zoom = doc.editor_state.zoom.0;
        let cx = (client_x - doc.editor_state.camera_x.0) / zoom;
        let cy = (client_y - doc.editor_state.camera_y.0) / zoom;

        let originals = selected_node_ids(&doc)
            .into_iter()
            .fold(HashMap::new(), |acc, id| {
                if let Some(n) = doc.document.nodes.get(&id) {
                    acc.update(id, (n.x.0, n.y.0, n.width.0, n.height.0))
                } else {
                    acc
                }
            });

        interaction_mode.set(InteractionMode::ResizingSelection {
            handle,
            original_bounds: bounds,
            originals,
            anchor: (cx, cy),
            did_resize: false,
        });
    }
}
