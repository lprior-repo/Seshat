#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use super::selection_geometry::{selected_node_ids, selection_bounds};
use crate::history::History;
use crate::models::document::{DiagramDocument, EdgeId, NodeId, NodeKind};
use dioxus::prelude::*;
use im::HashMap;
use std::collections::HashSet;

fn safe_zoom(zoom: f64) -> Option<f64> {
    (zoom.is_finite() && zoom > f64::EPSILON).then_some(zoom)
}

fn within(subgraph: (f64, f64, f64, f64), node: (f64, f64, f64, f64)) -> bool {
    let (sx, sy, sw, sh) = subgraph;
    let (nx, ny, nw, nh) = node;
    nx >= sx && ny >= sy && nx + nw <= sx + sw && ny + nh <= sy + sh
}

fn resize_target_ids(doc: &DiagramDocument) -> Vec<NodeId> {
    let selected = selected_node_ids(doc);
    let selected_set = selected.iter().cloned().collect::<HashSet<_>>();

    let selected_subgraphs = selected
        .iter()
        .filter_map(|id| doc.document.nodes.get(id).map(|node| (id, node)))
        .filter(|(_, node)| node.kind == NodeKind::Subgraph)
        .map(|(_, node)| (node.x.0, node.y.0, node.width.0, node.height.0))
        .collect::<Vec<_>>();

    if selected_subgraphs.is_empty() {
        return selected;
    }

    doc.document
        .nodes
        .iter()
        .fold(selected_set, |acc, (id, node)| {
            let node_rect = (node.x.0, node.y.0, node.width.0, node.height.0);
            let included = selected_subgraphs
                .iter()
                .any(|subgraph_rect| within(*subgraph_rect, node_rect));
            if included {
                let mut updated = acc;
                let _ = updated.insert(id.clone());
                updated
            } else {
                acc
            }
        })
        .into_iter()
        .collect::<Vec<_>>()
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum InteractionMode {
    Select,
    RubberBand {
        start: (f64, f64),
        current: (f64, f64),
    },
    DraggingSelection {
        anchor_canvas: (f64, f64),
        anchor_client: (f64, f64),
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
        let Some(zoom) = safe_zoom(doc.editor_state.zoom.0) else {
            return;
        };
        let cx = (client_x / zoom) + doc.editor_state.camera_x.0;
        let cy = (client_y / zoom) + doc.editor_state.camera_y.0;

        let originals = resize_target_ids(&doc)
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

pub(super) fn finalize_motion_release(
    mode: &mut InteractionMode,
    doc: &mut DiagramDocument,
) -> bool {
    let should_increment = match mode {
        InteractionMode::DraggingSelection { did_move, .. } => Some(*did_move),
        InteractionMode::ResizingSelection { did_resize, .. } => Some(*did_resize),
        _ => None,
    };

    if let Some(increment) = should_increment {
        if increment {
            doc.revision = doc.revision.increment();
        }
        *mode = InteractionMode::Select;
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::{finalize_motion_release, resize_target_ids, InteractionMode, ResizeHandle};
    use crate::models::document::{
        DiagramDocument, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
    };
    use im::HashMap;

    fn node(kind: NodeKind, x: f64, y: f64, w: f64, h: f64) -> Node {
        Node {
            kind,
            icon: String::new(),
            label: String::from("n"),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(w),
            height: OrderedFloat(h),
            font_size: None,
            font_weight: None,
            locked: true,
            parent: None,
            dag_rank: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        }
    }

    #[test]
    fn given_drag_end_when_finalized_twice_then_revision_bumps_once() {
        let mut doc = DiagramDocument::default();
        let mut mode = InteractionMode::DraggingSelection {
            anchor_canvas: (0.0, 0.0),
            anchor_client: (0.0, 0.0),
            original_positions: HashMap::new(),
            did_move: true,
        };

        let first = finalize_motion_release(&mut mode, &mut doc);
        let second = finalize_motion_release(&mut mode, &mut doc);

        assert!(first);
        assert!(!second);
        assert_eq!(
            doc.revision,
            DiagramDocument::default().revision.increment()
        );
        assert_eq!(mode, InteractionMode::Select);
    }

    #[test]
    fn given_resize_end_without_resize_when_finalized_then_no_revision_bump() {
        let mut doc = DiagramDocument::default();
        let mut mode = InteractionMode::ResizingSelection {
            handle: ResizeHandle::Se,
            original_bounds: (0.0, 0.0, 10.0, 10.0),
            originals: HashMap::new(),
            anchor: (0.0, 0.0),
            did_resize: false,
        };

        let finalized = finalize_motion_release(&mut mode, &mut doc);

        assert!(finalized);
        assert_eq!(doc.revision, DiagramDocument::default().revision);
        assert_eq!(mode, InteractionMode::Select);
    }

    #[test]
    fn given_selected_subgraph_when_collecting_resize_targets_then_interior_nodes_included() {
        let mut doc = DiagramDocument::default();
        let subgraph = NodeId::new(String::from("sub"));
        let inside = NodeId::new(String::from("inside"));
        let outside = NodeId::new(String::from("outside"));

        doc.document.nodes = doc
            .document
            .nodes
            .update(
                subgraph.clone(),
                node(NodeKind::Subgraph, 100.0, 100.0, 300.0, 220.0),
            )
            .update(
                inside.clone(),
                node(NodeKind::Node, 140.0, 140.0, 80.0, 60.0),
            )
            .update(
                outside.clone(),
                node(NodeKind::Node, 450.0, 300.0, 80.0, 60.0),
            );
        let _ = doc.editor_state.selected_items.insert(subgraph.to_string());

        let targets = resize_target_ids(&doc);
        assert!(targets.contains(&subgraph));
        assert!(targets.contains(&inside));
        assert!(!targets.contains(&outside));
    }
}
