#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::history::History;
use crate::models::document::{
    DiagramDocument, Edge, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
};
use dioxus::prelude::*;
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use uuid::Uuid;

#[derive(Clone, Copy)]
enum ZOrderOp {
    BringForward,
    SendBackward,
    BringToFront,
    SendToBack,
}

#[derive(Clone)]
struct ClipboardState {
    nodes: Vec<(NodeId, Node)>,
    edges: Vec<Edge>,
    paste_serial: u32,
}

thread_local! {
    static CLIPBOARD: RefCell<Option<ClipboardState>> = const { RefCell::new(None) };
}

fn selected_node_ids(doc: &DiagramDocument) -> BTreeSet<NodeId> {
    doc.editor_state
        .selected_items
        .iter()
        .map(|id| NodeId::new(id.clone()))
        .filter(|id| doc.document.nodes.contains_key(id))
        .collect()
}

fn push_history(mut history_signal: Signal<History>, current: DiagramDocument) {
    let history = history_signal.read().clone();
    *history_signal.write() = history.push(current);
}

fn reparent_if_deleted(parent: Option<NodeId>, deleted_ids: &BTreeSet<NodeId>) -> Option<NodeId> {
    parent.and_then(|parent_id| {
        if deleted_ids.contains(&parent_id) {
            None
        } else {
            Some(parent_id)
        }
    })
}

fn remap_pasted_parent(parent: Option<NodeId>, id_map: &HashMap<NodeId, NodeId>) -> Option<NodeId> {
    parent.and_then(|parent_id| id_map.get(&parent_id).cloned().or(Some(parent_id)))
}

fn selected_nodes_from_selection(
    selected: &im::HashSet<String>,
    nodes: &im::HashMap<NodeId, Node>,
) -> BTreeSet<NodeId> {
    selected
        .iter()
        .map(|id| NodeId::new(id.clone()))
        .filter(|id| nodes.contains_key(id))
        .collect()
}

fn ordered_layer_node_ids(doc: &DiagramDocument, subgraph_layer: bool) -> Vec<NodeId> {
    let mut node_ids = doc
        .document
        .nodes
        .iter()
        .filter_map(|(id, node)| {
            let is_subgraph = node.kind == NodeKind::Subgraph;
            if is_subgraph == subgraph_layer {
                Some(id.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    node_ids.sort_by(|a, b| {
        doc.document
            .nodes
            .get(a)
            .zip(doc.document.nodes.get(b))
            .map_or(std::cmp::Ordering::Equal, |(na, nb)| {
                (na.z_index, a.to_string()).cmp(&(nb.z_index, b.to_string()))
            })
    });

    node_ids
}

fn apply_z_order_to_ids(ids: &mut Vec<NodeId>, selected: &BTreeSet<NodeId>, op: ZOrderOp) {
    if ids.len() < 2 {
        return;
    }

    match op {
        ZOrderOp::BringForward => {
            for idx in (0..(ids.len() - 1)).rev() {
                let current_selected = selected.contains(&ids[idx]);
                let next_selected = selected.contains(&ids[idx + 1]);
                if current_selected && !next_selected {
                    ids.swap(idx, idx + 1);
                }
            }
        }
        ZOrderOp::SendBackward => {
            for idx in 1..ids.len() {
                let current_selected = selected.contains(&ids[idx]);
                let previous_selected = selected.contains(&ids[idx - 1]);
                if current_selected && !previous_selected {
                    ids.swap(idx - 1, idx);
                }
            }
        }
        ZOrderOp::BringToFront => {
            let mut reordered = ids
                .iter()
                .filter(|id| !selected.contains(*id))
                .cloned()
                .collect::<Vec<_>>();
            reordered.extend(ids.iter().filter(|id| selected.contains(*id)).cloned());
            *ids = reordered;
        }
        ZOrderOp::SendToBack => {
            let mut reordered = ids
                .iter()
                .filter(|id| selected.contains(*id))
                .cloned()
                .collect::<Vec<_>>();
            reordered.extend(ids.iter().filter(|id| !selected.contains(*id)).cloned());
            *ids = reordered;
        }
    }
}

fn apply_z_order_operation(
    mut doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    op: ZOrderOp,
) -> bool {
    let current = doc_signal.read().clone();
    let selected = selected_node_ids(&current)
        .into_iter()
        .filter(|id| {
            current
                .document
                .nodes
                .get(id)
                .is_some_and(|node| !node.locked || node.kind == NodeKind::Subgraph)
        })
        .collect::<BTreeSet<_>>();
    if selected.is_empty() {
        return false;
    }

    let mut next = current.clone();
    let mut changed = false;

    for is_subgraph_layer in [false, true] {
        let ordered = ordered_layer_node_ids(&next, is_subgraph_layer);
        if ordered.len() < 2 {
            continue;
        }
        let mut reordered = ordered.clone();
        apply_z_order_to_ids(&mut reordered, &selected, op);
        if reordered == ordered {
            continue;
        }

        let min_z = ordered
            .iter()
            .filter_map(|id| next.document.nodes.get(id).map(|node| node.z_index))
            .min()
            .unwrap_or(0);

        for (idx, id) in reordered.iter().enumerate() {
            if let Some(node) = next.document.nodes.get_mut(id) {
                node.z_index = min_z + idx as i64;
            }
        }

        changed = true;
    }

    if !changed {
        return false;
    }

    next.revision = next.revision.increment();
    push_history(history_signal, current);
    *doc_signal.write() = next;
    true
}

pub fn apply_bring_forward(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
) -> bool {
    apply_z_order_operation(doc_signal, history_signal, ZOrderOp::BringForward)
}

pub fn apply_send_backward(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
) -> bool {
    apply_z_order_operation(doc_signal, history_signal, ZOrderOp::SendBackward)
}

pub fn apply_bring_to_front(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
) -> bool {
    apply_z_order_operation(doc_signal, history_signal, ZOrderOp::BringToFront)
}

pub fn apply_send_to_back(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
) -> bool {
    apply_z_order_operation(doc_signal, history_signal, ZOrderOp::SendToBack)
}

pub fn apply_select_all(mut doc_signal: Signal<DiagramDocument>) {
    doc_signal.with_mut(|doc| {
        doc.editor_state.selected_items = doc
            .document
            .nodes
            .keys()
            .map(ToString::to_string)
            .chain(doc.document.edges.keys().map(ToString::to_string))
            .collect();
    });
}

pub fn apply_clear_selection(mut doc_signal: Signal<DiagramDocument>) {
    doc_signal.with_mut(|doc| {
        doc.editor_state.selected_items.clear();
    });
}

pub fn apply_delete_selected(
    mut doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
) -> bool {
    let selected = doc_signal.read().editor_state.selected_items.clone();
    if selected.is_empty() {
        return false;
    }

    push_history(history_signal, doc_signal.read().clone());
    doc_signal.with_mut(|doc| {
        let deleted_node_ids =
            selected_nodes_from_selection(&doc.editor_state.selected_items, &doc.document.nodes);
        doc.document.nodes = doc
            .document
            .nodes
            .iter()
            .filter(|(id, _)| !selected.contains(&id.to_string()))
            .map(|(id, node)| {
                let mut next = node.clone();
                next.parent = reparent_if_deleted(next.parent, &deleted_node_ids);
                (id.clone(), next)
            })
            .collect();

        let node_ids: im::HashSet<NodeId> = doc.document.nodes.keys().cloned().collect();
        doc.document.edges = doc
            .document
            .edges
            .iter()
            .filter(|(id, edge)| {
                node_ids.contains(&edge.source)
                    && node_ids.contains(&edge.target)
                    && !selected.contains(&id.to_string())
            })
            .map(|(id, edge)| (id.clone(), edge.clone()))
            .collect();

        doc.editor_state.selected_items.clear();
        doc.revision = doc.revision.increment();
    });
    true
}

pub fn apply_nudge_selection(
    mut doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    dx: f64,
    dy: f64,
    push_undo: bool,
) -> bool {
    let selected_nodes = {
        let doc = doc_signal.read();
        selected_node_ids(&doc)
    };
    if selected_nodes.is_empty() || (dx == 0.0 && dy == 0.0) {
        return false;
    }

    if push_undo {
        push_history(history_signal, doc_signal.read().clone());
    }
    doc_signal.with_mut(|doc| {
        for node_id in selected_nodes {
            if let Some(node) = doc.document.nodes.get_mut(&node_id) {
                if node.locked && node.kind != NodeKind::Subgraph {
                    continue;
                }
                node.x = OrderedFloat(node.x.0 + dx);
                node.y = OrderedFloat(node.y.0 + dy);
            }
        }
        doc.revision = doc.revision.increment();
    });
    true
}

pub fn clipboard_has_content() -> bool {
    CLIPBOARD.with(|slot| {
        slot.borrow()
            .as_ref()
            .is_some_and(|state| !state.nodes.is_empty())
    })
}

pub fn apply_copy_selection(doc_signal: Signal<DiagramDocument>) -> bool {
    let doc = doc_signal.read();
    copy_selection_to_clipboard(&doc)
}

fn copy_selection_to_clipboard(doc: &DiagramDocument) -> bool {
    let selected_nodes = selected_node_ids(doc);
    if selected_nodes.is_empty() {
        return false;
    }

    let nodes = selected_nodes
        .iter()
        .filter_map(|id| {
            doc.document
                .nodes
                .get(id)
                .map(|node| (id.clone(), node.clone()))
        })
        .collect::<Vec<_>>();

    let edges = doc
        .document
        .edges
        .iter()
        .filter(|(_, edge)| {
            selected_nodes.contains(&edge.source) && selected_nodes.contains(&edge.target)
        })
        .map(|(_, edge)| edge.clone())
        .collect::<Vec<_>>();

    CLIPBOARD.with(|slot| {
        *slot.borrow_mut() = Some(ClipboardState {
            nodes,
            edges,
            paste_serial: 0,
        });
    });
    true
}

fn paste_from(
    nodes: Vec<(NodeId, Node)>,
    edges: Vec<Edge>,
    serial: u32,
    doc: &mut DiagramDocument,
) {
    let offset = 20.0 * f64::from(serial.max(1));
    let id_map = nodes
        .iter()
        .map(|(old_id, _)| (old_id.clone(), NodeId::new(Uuid::new_v4().to_string())))
        .collect::<HashMap<_, _>>();
    let mut selected = im::HashSet::new();

    for (old_id, node) in nodes {
        let Some(new_id) = id_map.get(&old_id).cloned() else {
            continue;
        };
        let mut next = node;
        next.x = OrderedFloat(next.x.0 + offset);
        next.y = OrderedFloat(next.y.0 + offset);
        next.parent = remap_pasted_parent(next.parent, &id_map);
        let _ = selected.insert(new_id.to_string());
        let _ = doc.document.nodes.insert(new_id, next);
    }

    for edge in edges {
        if let (Some(new_source), Some(new_target)) =
            (id_map.get(&edge.source), id_map.get(&edge.target))
        {
            let mut next = edge;
            next.source = new_source.clone();
            next.target = new_target.clone();
            let new_edge_id = crate::models::document::EdgeId::new(Uuid::new_v4().to_string());
            let _ = doc.document.edges.insert(new_edge_id, next);
        }
    }

    doc.editor_state.selected_items = selected;
    doc.revision = doc.revision.increment();
}

pub fn apply_paste_selection(
    mut doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
) -> bool {
    let current = doc_signal.read().clone();
    let did_paste = doc_signal.with_mut(paste_from_clipboard);
    if did_paste {
        push_history(history_signal, current);
    }
    did_paste
}

fn paste_from_clipboard(doc: &mut DiagramDocument) -> bool {
    let clipboard = CLIPBOARD.with(|slot| {
        let mut state = slot.borrow_mut();
        state.as_mut().map(|clip| {
            clip.paste_serial = clip.paste_serial.saturating_add(1);
            (clip.nodes.clone(), clip.edges.clone(), clip.paste_serial)
        })
    });

    let Some((nodes, edges, serial)) = clipboard else {
        return false;
    };
    if nodes.is_empty() {
        return false;
    }

    paste_from(nodes, edges, serial, doc);
    true
}

pub fn apply_duplicate_selection(
    mut doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
) -> bool {
    let (nodes, edges) = {
        let doc = doc_signal.read();
        let selected_nodes = selected_node_ids(&doc);
        if selected_nodes.is_empty() {
            return false;
        }

        let nodes = selected_nodes
            .iter()
            .filter_map(|id| {
                doc.document
                    .nodes
                    .get(id)
                    .map(|node| (id.clone(), node.clone()))
            })
            .collect::<Vec<_>>();

        let edges = doc
            .document
            .edges
            .iter()
            .filter(|(_, edge)| {
                selected_nodes.contains(&edge.source) && selected_nodes.contains(&edge.target)
            })
            .map(|(_, edge)| edge.clone())
            .collect::<Vec<_>>();

        CLIPBOARD.with(|slot| {
            *slot.borrow_mut() = Some(ClipboardState {
                nodes: nodes.clone(),
                edges: edges.clone(),
                paste_serial: 1,
            });
        });

        (nodes, edges)
    };

    push_history(history_signal, doc_signal.read().clone());
    doc_signal.with_mut(|doc| {
        paste_from(nodes, edges, 1, doc);
    });
    true
}

pub fn apply_group_selection(
    mut doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
) -> bool {
    let selected_nodes = {
        let doc = doc_signal.read();
        selected_node_ids(&doc)
            .into_iter()
            .filter(|id| {
                doc.document
                    .nodes
                    .get(id)
                    .is_some_and(|node| node.kind != NodeKind::Subgraph)
            })
            .collect::<Vec<_>>()
    };
    if selected_nodes.len() < 2 {
        return false;
    }

    let (min_x, min_y, max_x, max_y) = {
        let doc = doc_signal.read();
        selected_nodes.iter().fold(
            (
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
            ),
            |(min_x, min_y, max_x, max_y), node_id| {
                doc.document
                    .nodes
                    .get(node_id)
                    .map_or((min_x, min_y, max_x, max_y), |node| {
                        (
                            min_x.min(node.x.0),
                            min_y.min(node.y.0),
                            max_x.max(node.x.0 + node.width.0),
                            max_y.max(node.y.0 + node.height.0),
                        )
                    })
            },
        )
    };

    if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
        return false;
    }

    push_history(history_signal, doc_signal.read().clone());
    let group_id = NodeId::new(Uuid::new_v4().to_string());
    let member_ids = selected_nodes;
    doc_signal.with_mut(|doc| {
        for node_id in &member_ids {
            if let Some(node) = doc.document.nodes.get_mut(node_id) {
                node.parent = Some(group_id.clone());
            }
        }

        let padding = 24.0;
        let _ = doc.document.nodes.insert(
            group_id.clone(),
            Node {
                kind: NodeKind::Subgraph,
                icon: String::new(),
                label: String::from("Group"),
                x: OrderedFloat(min_x - padding),
                y: OrderedFloat(min_y - padding),
                width: OrderedFloat((max_x - min_x) + (padding * 2.0)),
                height: OrderedFloat((max_y - min_y) + (padding * 2.0)),
                font_size: None,
                font_weight: None,
                locked: true,
                parent: None,
                dag_rank: None,
                tags: Vec::new(),
                metadata: im::HashMap::new(),
                z_index: -1,
                style: Some(NodeStyle::Box),
                collapsed: Some(false),
            },
        );
        doc.editor_state.selected_items.clear();
        let _ = doc.editor_state.selected_items.insert(group_id.to_string());
        doc.revision = doc.revision.increment();
    });
    true
}

pub fn apply_ungroup_selection(
    mut doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
) -> bool {
    let target_subgraphs = selected_subgraphs_for_ungroup(&doc_signal.read());

    if target_subgraphs.is_empty() {
        return false;
    }

    push_history(history_signal, doc_signal.read().clone());
    doc_signal.with_mut(|doc| {
        doc.document.nodes = doc
            .document
            .nodes
            .iter()
            .filter_map(|(id, node)| {
                if target_subgraphs.contains(id) {
                    None
                } else {
                    let mut next = node.clone();
                    if next
                        .parent
                        .as_ref()
                        .is_some_and(|parent| target_subgraphs.contains(parent))
                    {
                        next.parent = None;
                    }
                    Some((id.clone(), next))
                }
            })
            .collect();

        doc.document.edges = doc
            .document
            .edges
            .iter()
            .filter(|(_, edge)| {
                !target_subgraphs.contains(&edge.source) && !target_subgraphs.contains(&edge.target)
            })
            .map(|(id, edge)| (id.clone(), edge.clone()))
            .collect();

        doc.editor_state.selected_items.clear();
        doc.revision = doc.revision.increment();
    });
    true
}

fn selected_subgraphs_for_ungroup(doc: &DiagramDocument) -> BTreeSet<NodeId> {
    selected_node_ids(doc)
        .into_iter()
        .filter(|id| {
            doc.document
                .nodes
                .get(id)
                .is_some_and(|node| node.kind == NodeKind::Subgraph)
        })
        .collect::<BTreeSet<_>>()
}

fn set_zoom_centered(
    doc: &mut DiagramDocument,
    target_zoom: f64,
    viewport_size: (f64, f64),
) -> bool {
    let raw_old_zoom = doc.editor_state.zoom.0;
    let old_zoom = if raw_old_zoom.is_finite() && raw_old_zoom > f64::EPSILON {
        raw_old_zoom
    } else {
        1.0
    };
    let new_zoom = target_zoom.clamp(0.1, 4.0);
    if (new_zoom - old_zoom).abs() < f64::EPSILON {
        return false;
    }

    let viewport_w = viewport_size.0.max(1.0);
    let viewport_h = viewport_size.1.max(1.0);

    let cx = doc.editor_state.camera_x.0 + (viewport_w / old_zoom / 2.0);
    let cy = doc.editor_state.camera_y.0 + (viewport_h / old_zoom / 2.0);

    let factor = old_zoom / new_zoom;

    doc.editor_state.camera_x.0 = (cx - doc.editor_state.camera_x.0).mul_add(-factor, cx);
    doc.editor_state.camera_y.0 = (cy - doc.editor_state.camera_y.0).mul_add(-factor, cy);
    doc.editor_state.zoom.0 = new_zoom;
    true
}

fn zoom_to_center(doc: &mut DiagramDocument, factor: f64, viewport_size: (f64, f64)) -> bool {
    let raw_old_zoom = doc.editor_state.zoom.0;
    let old_zoom = if raw_old_zoom.is_finite() && raw_old_zoom > f64::EPSILON {
        raw_old_zoom
    } else {
        1.0
    };
    set_zoom_centered(doc, old_zoom * factor, viewport_size)
}

pub fn apply_zoom_in(
    mut doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    viewport_size: (f64, f64),
) -> bool {
    let changed = {
        let doc = doc_signal.read();
        ((doc.editor_state.zoom.0 * 1.25).clamp(0.1, 4.0) - doc.editor_state.zoom.0).abs()
            >= f64::EPSILON
    };
    if !changed {
        return false;
    }

    push_history(history_signal, doc_signal.read().clone());
    doc_signal.with_mut(|doc| {
        let _ = zoom_to_center(doc, 1.25, viewport_size);
        doc.revision = doc.revision.increment();
    });
    true
}

pub fn apply_zoom_out(
    mut doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    viewport_size: (f64, f64),
) -> bool {
    let changed = {
        let doc = doc_signal.read();
        ((doc.editor_state.zoom.0 * 0.8).clamp(0.1, 4.0) - doc.editor_state.zoom.0).abs()
            >= f64::EPSILON
    };
    if !changed {
        return false;
    }
    push_history(history_signal, doc_signal.read().clone());
    doc_signal.with_mut(|doc| {
        let _ = zoom_to_center(doc, 0.8, viewport_size);
        doc.revision = doc.revision.increment();
    });
    true
}

pub fn apply_zoom_reset(
    mut doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    viewport_size: (f64, f64),
) -> bool {
    let changed = (doc_signal.read().editor_state.zoom.0 - 1.0).abs() >= f64::EPSILON;
    if !changed {
        return false;
    }

    push_history(history_signal, doc_signal.read().clone());
    doc_signal.with_mut(|doc| {
        let _ = set_zoom_centered(doc, 1.0, viewport_size);
        doc.revision = doc.revision.increment();
    });
    true
}

pub fn apply_undo(mut doc_signal: Signal<DiagramDocument>, mut history_signal: Signal<History>) {
    let current = doc_signal.read().clone();
    let history = history_signal.read().clone();
    if let Some((doc, next_history)) = history.undo(current) {
        *doc_signal.write() = doc;
        *history_signal.write() = next_history;
    }
}

pub fn apply_redo(mut doc_signal: Signal<DiagramDocument>, mut history_signal: Signal<History>) {
    let current = doc_signal.read().clone();
    let history = history_signal.read().clone();
    if let Some((doc, next_history)) = history.redo(current) {
        *doc_signal.write() = doc;
        *history_signal.write() = next_history;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn make_doc_with_zoom(zoom: f64) -> DiagramDocument {
        let mut doc = DiagramDocument::default();
        doc.editor_state.zoom = OrderedFloat(zoom);
        doc
    }

    fn make_doc_with_camera(zoom: f64, cam_x: f64, cam_y: f64) -> DiagramDocument {
        let mut doc = DiagramDocument::default();
        doc.editor_state.zoom = OrderedFloat(zoom);
        doc.editor_state.camera_x = OrderedFloat(cam_x);
        doc.editor_state.camera_y = OrderedFloat(cam_y);
        doc
    }

    #[test]
    fn given_zoom_to_center_when_valid_zoom_then_zoom_clamped() {
        let mut doc = make_doc_with_zoom(1.0);
        let result = zoom_to_center(&mut doc, 2.0, (800.0, 600.0));
        assert!(result);
        assert!(doc.editor_state.zoom.0 >= 0.1 && doc.editor_state.zoom.0 <= 4.0);
    }

    #[test]
    fn given_zoom_to_center_when_nan_zoom_then_uses_default() {
        let mut doc = make_doc_with_zoom(f64::NAN);
        let result = zoom_to_center(&mut doc, 1.5, (800.0, 600.0));
        assert!(result);
        assert!(doc.editor_state.zoom.0.is_finite());
    }

    #[test]
    fn given_zoom_to_center_when_inf_zoom_then_uses_default() {
        let mut doc = make_doc_with_zoom(f64::INFINITY);
        let result = zoom_to_center(&mut doc, 0.5, (800.0, 600.0));
        assert!(result);
        assert!(doc.editor_state.zoom.0.is_finite());
    }

    #[test]
    fn given_zoom_to_center_when_zero_zoom_then_uses_default() {
        let mut doc = make_doc_with_zoom(0.0);
        let result = zoom_to_center(&mut doc, 2.0, (800.0, 600.0));
        assert!(result);
        assert!(doc.editor_state.zoom.0 >= 0.1);
    }

    #[test]
    fn given_zoom_to_center_when_negative_zoom_then_uses_default() {
        let mut doc = make_doc_with_zoom(-5.0);
        let result = zoom_to_center(&mut doc, 2.0, (800.0, 600.0));
        assert!(result);
        assert!(doc.editor_state.zoom.0 >= 0.1);
    }

    #[test]
    fn given_zoom_to_center_when_at_max_then_no_change() {
        let mut doc = make_doc_with_zoom(4.0);
        let result = zoom_to_center(&mut doc, 2.0, (800.0, 600.0));
        assert!(!result);
    }

    #[test]
    fn given_zoom_to_center_when_at_min_then_no_change() {
        let mut doc = make_doc_with_zoom(0.1);
        let result = zoom_to_center(&mut doc, 0.5, (800.0, 600.0));
        assert!(!result);
    }

    #[test]
    fn given_zoom_to_center_when_valid_then_camera_finite() {
        let mut doc = make_doc_with_camera(1.0, 100.0, 200.0);
        let _ = zoom_to_center(&mut doc, 1.5, (800.0, 600.0));
        assert!(doc.editor_state.camera_x.0.is_finite());
        assert!(doc.editor_state.camera_y.0.is_finite());
    }

    #[test]
    fn given_zoom_to_center_when_nan_camera_then_zoom_still_clamped() {
        let mut doc = make_doc_with_camera(1.0, f64::NAN, f64::NAN);
        let _ = zoom_to_center(&mut doc, 1.5, (800.0, 600.0));
        assert!(doc.editor_state.zoom.0.is_finite());
        assert!(doc.editor_state.zoom.0 >= 0.1 && doc.editor_state.zoom.0 <= 4.0);
    }

    #[test]
    fn given_zoom_to_center_when_tiny_viewport_then_no_panic() {
        let mut doc = make_doc_with_zoom(1.0);
        let result = zoom_to_center(&mut doc, 1.5, (0.0, 0.0));
        assert!(result);
        assert!(doc.editor_state.zoom.0.is_finite());
    }

    #[test]
    fn given_zoom_to_center_when_huge_viewport_then_no_panic() {
        let mut doc = make_doc_with_zoom(1.0);
        let result = zoom_to_center(&mut doc, 1.5, (1e10, 1e10));
        assert!(result);
        assert!(doc.editor_state.zoom.0.is_finite());
    }

    fn clear_clipboard() {
        CLIPBOARD.with(|s| *s.borrow_mut() = None);
    }

    fn make_node(label: &str, x: f64, y: f64) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: label.to_string(),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(100.0),
            height: OrderedFloat(50.0),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: None,
            dag_rank: None,
            tags: Vec::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        }
    }

    fn make_doc_with_node(id: &str, x: f64, y: f64) -> DiagramDocument {
        let mut doc = DiagramDocument::default();
        let node_id = NodeId::new(id.to_string());
        let _ = doc.document.nodes.insert(node_id, make_node(id, x, y));
        doc
    }

    fn make_doc_with_two_nodes_and_edge(
        id_a: &str,
        id_b: &str,
    ) -> (DiagramDocument, crate::models::document::EdgeId) {
        let mut doc = DiagramDocument::default();
        let node_a_id = NodeId::new(id_a.to_string());
        let node_b_id = NodeId::new(id_b.to_string());
        let _ = doc
            .document
            .nodes
            .insert(node_a_id.clone(), make_node(id_a, 0.0, 0.0));
        let _ = doc
            .document
            .nodes
            .insert(node_b_id.clone(), make_node(id_b, 200.0, 0.0));

        let edge_id = crate::models::document::EdgeId::new("edge-1".to_string());
        let edge = Edge {
            source: node_a_id,
            target: node_b_id,
            label: String::new(),
            style: crate::models::document::EdgeStyle::default(),
            arrow_type: crate::models::document::ArrowType::default(),
            label_offset_t: OrderedFloat(0.5),
            color: None,
            thickness: OrderedFloat(1.5),
            directed: true,
            bend_points: Vec::new(),
            tags: Vec::new(),
            metadata: im::HashMap::new(),
            font_size: None,
        };
        let _ = doc.document.edges.insert(edge_id.clone(), edge);
        (doc, edge_id)
    }

    #[test]
    fn given_empty_selection_when_copy_then_returns_false() {
        clear_clipboard();
        let doc = DiagramDocument::default();
        let result = copy_selection_to_clipboard(&doc);
        assert!(!result);
        CLIPBOARD.with(|s| assert!(s.borrow().is_none()));
    }

    #[test]
    fn given_single_node_selected_when_copy_then_succeeds() {
        clear_clipboard();
        let mut doc = make_doc_with_node("node-1", 100.0, 50.0);
        let _ = doc.editor_state.selected_items.insert("node-1".to_string());

        let result = copy_selection_to_clipboard(&doc);

        assert!(result);
        CLIPBOARD.with(|s| {
            let clip = s.borrow();
            let clip_ref = clip.as_ref();
            assert!(clip_ref.is_some());
            if let Some(c) = clip_ref {
                assert_eq!(c.nodes.len(), 1);
                assert!(c.edges.is_empty());
                assert_eq!(c.paste_serial, 0);
            }
        });
    }

    #[test]
    fn given_multiple_nodes_selected_when_copy_then_includes_edges() {
        clear_clipboard();
        let (mut doc, _edge_id) = make_doc_with_two_nodes_and_edge("node-a", "node-b");
        let _ = doc.editor_state.selected_items.insert("node-a".to_string());
        let _ = doc.editor_state.selected_items.insert("node-b".to_string());

        let result = copy_selection_to_clipboard(&doc);

        assert!(result);
        CLIPBOARD.with(|s| {
            let clip = s.borrow();
            let clip_ref = clip.as_ref();
            assert!(clip_ref.is_some());
            if let Some(c) = clip_ref {
                assert_eq!(c.nodes.len(), 2);
                assert_eq!(c.edges.len(), 1);
            }
        });
    }

    #[test]
    fn given_empty_clipboard_when_paste_then_returns_false() {
        clear_clipboard();
        let mut doc = DiagramDocument::default();
        let node_count_before = doc.document.nodes.len();

        let result = paste_from_clipboard(&mut doc);

        assert!(!result);
        assert_eq!(doc.document.nodes.len(), node_count_before);
    }

    #[test]
    fn given_copied_nodes_when_paste_then_creates_new_ids() {
        clear_clipboard();
        let mut doc = make_doc_with_node("original-node", 100.0, 50.0);
        let _ = doc
            .editor_state
            .selected_items
            .insert("original-node".to_string());

        let copy_result = copy_selection_to_clipboard(&doc);
        assert!(copy_result);

        let paste_result = paste_from_clipboard(&mut doc);
        assert!(paste_result);

        assert_eq!(doc.document.nodes.len(), 2);
        let original_id = NodeId::new("original-node".to_string());
        let pasted_ids: Vec<_> = doc
            .document
            .nodes
            .keys()
            .filter(|id| *id != &original_id)
            .collect();
        assert_eq!(pasted_ids.len(), 1);
    }

    #[test]
    fn given_copied_nodes_when_paste_then_applies_offset() {
        clear_clipboard();
        let mut doc = make_doc_with_node("original-node", 100.0, 50.0);
        let _ = doc
            .editor_state
            .selected_items
            .insert("original-node".to_string());

        let _ = copy_selection_to_clipboard(&doc);
        let _ = paste_from_clipboard(&mut doc);

        let original_id = NodeId::new("original-node".to_string());
        let pasted_node = doc
            .document
            .nodes
            .iter()
            .find(|(id, _)| *id != &original_id)
            .map(|(_, node)| node.clone());

        assert!(pasted_node.is_some());
        if let Some(ref p) = pasted_node {
            assert_eq!(p.x.0, 120.0);
            assert_eq!(p.y.0, 70.0);
        }
    }

    #[test]
    fn given_selected_middle_node_when_bring_to_front_then_relative_order_preserved() {
        let mut ids = vec![
            NodeId::new(String::from("a")),
            NodeId::new(String::from("b")),
            NodeId::new(String::from("c")),
        ];
        let mut selected = BTreeSet::new();
        let _ = selected.insert(NodeId::new(String::from("b")));

        apply_z_order_to_ids(&mut ids, &selected, ZOrderOp::BringToFront);

        assert_eq!(
            ids,
            vec![
                NodeId::new(String::from("a")),
                NodeId::new(String::from("c")),
                NodeId::new(String::from("b")),
            ]
        );
    }

    #[test]
    fn given_selected_middle_node_when_send_to_back_then_relative_order_preserved() {
        let mut ids = vec![
            NodeId::new(String::from("a")),
            NodeId::new(String::from("b")),
            NodeId::new(String::from("c")),
        ];
        let mut selected = BTreeSet::new();
        let _ = selected.insert(NodeId::new(String::from("b")));

        apply_z_order_to_ids(&mut ids, &selected, ZOrderOp::SendToBack);

        assert_eq!(
            ids,
            vec![
                NodeId::new(String::from("b")),
                NodeId::new(String::from("a")),
                NodeId::new(String::from("c")),
            ]
        );
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    prop_compose! {
        fn arb_finite_f64()(x in -1e6_f64..1e6_f64) -> f64 { x }
    }

    prop_compose! {
        fn arb_zoom_f64()(x in 0.001_f64..100.0_f64) -> f64 { x }
    }

    prop_compose! {
        fn arb_factor()(x in 0.1_f64..10.0_f64) -> f64 { x }
    }

    prop_compose! {
        fn arb_viewport()(w in 0.0_f64..5000.0_f64, h in 0.0_f64..5000.0_f64) -> (f64, f64) {
            (w, h)
        }
    }

    fn make_doc_for_prop(zoom: f64, cam_x: f64, cam_y: f64) -> DiagramDocument {
        let mut doc = DiagramDocument::default();
        doc.editor_state.zoom = OrderedFloat(zoom);
        doc.editor_state.camera_x = OrderedFloat(cam_x);
        doc.editor_state.camera_y = OrderedFloat(cam_y);
        doc
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn prop_zoom_to_center_zoom_always_clamped(
            zoom in arb_zoom_f64(),
            factor in arb_factor(),
            viewport in arb_viewport(),
        ) {
            let mut doc = make_doc_for_prop(zoom, 0.0, 0.0);
            let _ = zoom_to_center(&mut doc, factor, viewport);

            prop_assert!(doc.editor_state.zoom.0 >= 0.1);
            prop_assert!(doc.editor_state.zoom.0 <= 4.0);
        }

        #[test]
        fn prop_zoom_to_center_camera_stays_finite(
            zoom in arb_zoom_f64(),
            cam_x in arb_finite_f64(),
            cam_y in arb_finite_f64(),
            factor in arb_factor(),
            viewport in arb_viewport(),
        ) {
            let mut doc = make_doc_for_prop(zoom, cam_x, cam_y);
            let _ = zoom_to_center(&mut doc, factor, viewport);

            prop_assert!(doc.editor_state.camera_x.0.is_finite());
            prop_assert!(doc.editor_state.camera_y.0.is_finite());
        }

        #[test]
        fn prop_zoom_to_center_nan_zoom_recovered(
            factor in arb_factor(),
            viewport in arb_viewport(),
        ) {
            let mut doc = make_doc_for_prop(f64::NAN, 0.0, 0.0);
            let _ = zoom_to_center(&mut doc, factor, viewport);

            prop_assert!(doc.editor_state.zoom.0.is_finite());
            prop_assert!(doc.editor_state.zoom.0 >= 0.1);
            prop_assert!(doc.editor_state.zoom.0 <= 4.0);
        }

        #[test]
        fn prop_zoom_to_center_inf_zoom_recovered(
            factor in arb_factor(),
            viewport in arb_viewport(),
        ) {
            let mut doc = make_doc_for_prop(f64::INFINITY, 0.0, 0.0);
            let _ = zoom_to_center(&mut doc, factor, viewport);

            prop_assert!(doc.editor_state.zoom.0.is_finite());
        }

        #[test]
        fn prop_zoom_to_center_zero_zoom_recovered(
            factor in arb_factor(),
            viewport in arb_viewport(),
        ) {
            let mut doc = make_doc_for_prop(0.0, 0.0, 0.0);
            let _ = zoom_to_center(&mut doc, factor, viewport);

            prop_assert!(doc.editor_state.zoom.0 >= 0.1);
        }

        #[test]
        fn prop_zoom_to_center_negative_zoom_recovered(
            factor in arb_factor(),
            viewport in arb_viewport(),
        ) {
            let mut doc = make_doc_for_prop(-100.0, 0.0, 0.0);
            let _ = zoom_to_center(&mut doc, factor, viewport);

            prop_assert!(doc.editor_state.zoom.0 >= 0.1);
        }

        #[test]
        fn prop_zoom_increases_with_large_factor(
            zoom in 0.2_f64..2.0_f64,
            viewport in arb_viewport(),
        ) {
            let mut doc = make_doc_for_prop(zoom, 0.0, 0.0);
            let old_zoom = doc.editor_state.zoom.0;
            let changed = zoom_to_center(&mut doc, 2.0, viewport);

            if changed {
                prop_assert!(doc.editor_state.zoom.0 >= old_zoom);
            }
        }

        #[test]
        fn prop_zoom_decreases_with_small_factor(
            zoom in 0.5_f64..3.0_f64,
            viewport in arb_viewport(),
        ) {
            let mut doc = make_doc_for_prop(zoom, 0.0, 0.0);
            let old_zoom = doc.editor_state.zoom.0;
            let changed = zoom_to_center(&mut doc, 0.5, viewport);

            if changed {
                prop_assert!(doc.editor_state.zoom.0 <= old_zoom);
            }
        }
    }
}
