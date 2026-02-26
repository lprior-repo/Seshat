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
                node.x = OrderedFloat(node.x.0 + dx);
                node.y = OrderedFloat(node.y.0 + dy);
                node.locked = true;
            }
        }
        doc.revision = doc.revision.increment();
    });
    true
}

pub fn apply_copy_selection(doc_signal: Signal<DiagramDocument>) -> bool {
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
            nodes,
            edges,
            paste_serial: 0,
        })
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

    push_history(history_signal, doc_signal.read().clone());
    doc_signal.with_mut(|doc| {
        paste_from(nodes, edges, serial, doc);
    });
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
            })
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
    selected_node_ids(&doc)
        .into_iter()
        .filter(|id| {
            doc.document
                .nodes
                .get(id)
                .is_some_and(|node| node.kind == NodeKind::Subgraph)
        })
        .collect::<BTreeSet<_>>()
}

fn zoom_to_center(doc: &mut DiagramDocument, factor: f64, viewport_size: (f64, f64)) -> bool {
    let raw_old_zoom = doc.editor_state.zoom.0;
    let old_zoom = if raw_old_zoom.is_finite() && raw_old_zoom > f64::EPSILON {
        raw_old_zoom
    } else {
        1.0
    };
    let new_zoom = (old_zoom * factor).clamp(0.1, 4.0);
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
) -> bool {
    let changed = (doc_signal.read().editor_state.zoom.0 - 1.0).abs() >= f64::EPSILON;
    if !changed {
        return false;
    }

    push_history(history_signal, doc_signal.read().clone());
    doc_signal.with_mut(|doc| {
        doc.editor_state.zoom.0 = 1.0;
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
