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

/// Axis for alignment operations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignmentAxis {
    Horizontal,
    Vertical,
}

/// Mode for alignment operations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignmentMode {
    Start,  // Left (Horizontal) or Top (Vertical)
    Center, // Center (Horizontal) or Middle (Vertical)
    End,    // Right (Horizontal) or Bottom (Vertical)
}

/// Axis for distribution operations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DistributionAxis {
    Horizontal,
    Vertical,
}

/// Pure clipboard data type - immutable state for clipboard operations.
///
/// This replaces the mutable thread_local RefCell-based clipboard with
/// a pure functional approach where clipboard state is passed explicitly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clipboard {
    /// The nodes that were copied to the clipboard
    pub nodes: Vec<(NodeId, Node)>,
    /// The edges that were copied to the clipboard
    pub edges: Vec<Edge>,
    /// Serial number for tracking paste operations (for offset calculation)
    pub paste_serial: u32,
}

impl Clipboard {
    /// Creates a new empty clipboard
    #[must_use]
    pub const fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            paste_serial: 0,
        }
    }

    /// Returns true if the clipboard has content that can be pasted
    #[must_use]
    pub const fn has_content(&self) -> bool {
        !self.nodes.is_empty()
    }

    /// Prepares the clipboard for a paste operation by incrementing the serial
    #[must_use]
    pub fn prepare_paste(mut self) -> Self {
        self.paste_serial = self.paste_serial.saturating_add(1);
        self
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}

/// Pure function: Checks if the given clipboard has pasteable content
#[must_use]
pub fn clipboard_has_content(clipboard: &Option<Clipboard>) -> bool {
    clipboard.as_ref().map_or(false, Clipboard::has_content)
}

/// Pure function: Creates a clipboard with the selected nodes and edges from the document.
///
/// Returns `None` if no nodes are selected, otherwise returns a new `Clipboard` with the
/// selected content.
#[must_use]
pub fn copy_selection(doc: &DiagramDocument) -> Option<Clipboard> {
    let selected_nodes = selected_node_ids(doc);
    if selected_nodes.is_empty() {
        return None;
    }

    let nodes = selected_nodes
        .iter()
        .filter_map(|id| {
            doc.document
                .nodes
                .get(id)
                .map(|node| (id.clone(), node.clone()))
        })
        .collect();

    let edges = doc
        .document
        .edges
        .iter()
        .filter(|(_, edge)| {
            selected_nodes.contains(&edge.source) && selected_nodes.contains(&edge.target)
        })
        .map(|(_, edge)| edge.clone())
        .collect();

    Some(Clipboard {
        nodes,
        edges,
        paste_serial: 0,
    })
}

/// Pure function: Creates a clipboard for duplicate operations.
///
/// Unlike `copy_selection`, this sets `paste_serial` to 1 to indicate
/// the content should be pasted with an offset.
#[must_use]
pub fn copy_selection_for_duplicate(doc: &DiagramDocument) -> Option<Clipboard> {
    let selected_nodes = selected_node_ids(doc);
    if selected_nodes.is_empty() {
        return None;
    }

    let nodes = selected_nodes
        .iter()
        .filter_map(|id| {
            doc.document
                .nodes
                .get(id)
                .map(|node| (id.clone(), node.clone()))
        })
        .collect();

    let edges = doc
        .document
        .edges
        .iter()
        .filter(|(_, edge)| {
            selected_nodes.contains(&edge.source) && selected_nodes.contains(&edge.target)
        })
        .map(|(_, edge)| edge.clone())
        .collect();

    Some(Clipboard {
        nodes,
        edges,
        paste_serial: 1,
    })
}

/// Pure function: Pastes clipboard content into the document.
///
/// Returns `None` if the clipboard is empty or has no nodes.
/// Otherwise returns a tuple of (updated_document, updated_clipboard).
#[must_use]
pub fn paste_contents(mut clipboard: Clipboard, doc: DiagramDocument) -> Option<(DiagramDocument, Clipboard)> {
    if clipboard.nodes.is_empty() {
        return None;
    }

    clipboard.paste_serial = clipboard.paste_serial.saturating_add(1);
    let serial = clipboard.paste_serial;

    let mut doc = doc;
    let offset = 20.0 * f64::from(serial.max(1));
    let id_map = clipboard
        .nodes
        .iter()
        .map(|(old_id, _)| (old_id.clone(), NodeId::new(Uuid::new_v4().to_string())))
        .collect::<HashMap<_, _>>();
    let mut selected = im::HashSet::new();

    for (old_id, node) in &clipboard.nodes {
        let Some(new_id) = id_map.get(old_id).cloned() else {
            continue;
        };
        let mut next = node.clone();
        next.x = OrderedFloat(next.x.0 + offset);
        next.y = OrderedFloat(next.y.0 + offset);
        next.parent = remap_pasted_parent(next.parent, &id_map);
        let _ = selected.insert(new_id.to_string());
        let _ = doc.document.nodes.insert(new_id, next);
    }

    for edge in &clipboard.edges {
        if let (Some(new_source), Some(new_target)) =
            (id_map.get(&edge.source), id_map.get(&edge.target))
        {
            let mut next = edge.clone();
            next.source = new_source.clone();
            next.target = new_target.clone();
            let new_edge_id = crate::models::document::EdgeId::new(Uuid::new_v4().to_string());
            let _ = doc.document.edges.insert(new_edge_id, next);
        }
    }

    doc.editor_state.selected_items = selected;
    doc.revision = doc.revision.increment();

    Some((doc, clipboard))
}

/// Public API: Applies copy operation using a clipboard signal.
///
/// This function maintains backward compatibility with the existing API
/// by using a Dioxus signal for clipboard state management.
pub fn apply_copy_selection(
    mut doc_signal: Signal<DiagramDocument>,
    mut clipboard_signal: Signal<Option<Clipboard>>,
) -> bool {
    let doc = doc_signal.read().clone();
    if let Some(clipboard) = copy_selection(&doc) {
        clipboard_signal.set(Some(clipboard));
        true
    } else {
        false
    }
}

/// Public API: Applies paste operation using a clipboard signal.
///
/// Returns true if paste was successful, false otherwise.
pub fn apply_paste_selection(
    mut doc_signal: Signal<DiagramDocument>,
    mut clipboard_signal: Signal<Option<Clipboard>>,
    history_signal: Signal<History>,
) -> bool {
    let current = doc_signal.read().clone();
    let clipboard = clipboard_signal.read().clone();

    let Some(clipboard) = clipboard else {
        return false;
    };

    let Some((new_doc, new_clipboard)) = paste_contents(clipboard, current) else {
        return false;
    };

    push_history(history_signal, doc_signal.read().clone());
    doc_signal.set(new_doc);
    clipboard_signal.set(Some(new_clipboard));
    true
}

/// Public API: Applies duplicate operation.
///
/// This is equivalent to copy followed by paste, but uses paste_serial=1
/// to ensure the duplicated content is offset from the original.
pub fn apply_duplicate_selection(
    mut doc_signal: Signal<DiagramDocument>,
    mut clipboard_signal: Signal<Option<Clipboard>>,
    history_signal: Signal<History>,
) -> bool {
    let doc = doc_signal.read().clone();
    let Some(clipboard) = copy_selection_for_duplicate(&doc) else {
        return false;
    };

    let Some((new_doc, _)) = paste_contents(clipboard, doc) else {
        return false;
    };

    // Update clipboard with the duplicated content (for subsequent pastes)
    let updated_clipboard = copy_selection_for_duplicate(&new_doc);

    push_history(history_signal, doc_signal.read().clone());
    doc_signal.set(new_doc);
    clipboard_signal.set(updated_clipboard);
    true
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
                node.z_index = min_z + i64::try_from(idx).unwrap_or(min_z);
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

/// Align selected nodes along the specified axis using the given mode.
///
/// Returns `true` if alignment was performed, `false` if:
/// - Fewer than 2 nodes are selected
/// - All selected nodes are locked
/// - Any selected node has non-finite coordinates
///
/// # Invariants
/// - Node dimensions (width, height) are never modified
/// - Z-order is preserved
/// - Locked nodes are skipped (unless they are Subgraphs)
pub fn apply_align_selection(
    mut doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    axis: AlignmentAxis,
    mode: AlignmentMode,
) -> bool {
    let current = doc_signal.read().clone();

    // Get selected nodes that are movable (not locked, or are subgraphs)
    let selected_nodes: Vec<NodeId> = selected_node_ids(&current)
        .into_iter()
        .filter(|id| {
            current.document.nodes.get(id).is_some_and(|node| {
                let coords_finite = node.x.0.is_finite() && node.y.0.is_finite();
                let movable = !node.locked || node.kind == NodeKind::Subgraph;
                coords_finite && movable
            })
        })
        .collect();

    // Need at least 2 nodes to align
    if selected_nodes.len() < 2 {
        return false;
    }

    // Calculate bounding box
    let (min_pos, max_pos, max_extent) = match axis {
        AlignmentAxis::Horizontal => {
            let positions: Vec<(f64, f64)> = selected_nodes
                .iter()
                .filter_map(|id| current.document.nodes.get(id))
                .map(|node| (node.x.0, node.x.0 + node.width.0))
                .collect();

            if positions
                .iter()
                .any(|(p, e)| !p.is_finite() || !e.is_finite())
            {
                return false;
            }

            let min_x = positions
                .iter()
                .map(|(p, _)| *p)
                .fold(f64::INFINITY, f64::min);
            let max_right = positions
                .iter()
                .map(|(_, e)| *e)
                .fold(f64::NEG_INFINITY, f64::max);

            if !min_x.is_finite() || !max_right.is_finite() {
                return false;
            }

            (min_x, max_right, max_right - min_x)
        }
        AlignmentAxis::Vertical => {
            let positions: Vec<(f64, f64)> = selected_nodes
                .iter()
                .filter_map(|id| current.document.nodes.get(id))
                .map(|node| (node.y.0, node.y.0 + node.height.0))
                .collect();

            if positions
                .iter()
                .any(|(p, e)| !p.is_finite() || !e.is_finite())
            {
                return false;
            }

            let min_y = positions
                .iter()
                .map(|(p, _)| *p)
                .fold(f64::INFINITY, f64::min);
            let max_bottom = positions
                .iter()
                .map(|(_, e)| *e)
                .fold(f64::NEG_INFINITY, f64::max);

            if !min_y.is_finite() || !max_bottom.is_finite() {
                return false;
            }

            (min_y, max_bottom, max_bottom - min_y)
        }
    };

    push_history(history_signal, current);

    doc_signal.with_mut(|doc| {
        for node_id in &selected_nodes {
            if let Some(node) = doc.document.nodes.get_mut(node_id) {
                // Double-check movability (should be redundant but defensive)
                if node.locked && node.kind != NodeKind::Subgraph {
                    continue;
                }

                match (axis, mode) {
                    (AlignmentAxis::Horizontal, AlignmentMode::Start) => {
                        // Align Left: set x to min_x
                        node.x = OrderedFloat(min_pos);
                    }
                    (AlignmentAxis::Horizontal, AlignmentMode::Center) => {
                        // Align Center H: center the node within the bounding box
                        let center_x = min_pos + max_extent / 2.0;
                        node.x = OrderedFloat(center_x - node.width.0 / 2.0);
                    }
                    (AlignmentAxis::Horizontal, AlignmentMode::End) => {
                        // Align Right: set x so right edge aligns with max_right
                        node.x = OrderedFloat(max_pos - node.width.0);
                    }
                    (AlignmentAxis::Vertical, AlignmentMode::Start) => {
                        // Align Top: set y to min_y
                        node.y = OrderedFloat(min_pos);
                    }
                    (AlignmentAxis::Vertical, AlignmentMode::Center) => {
                        // Align Middle V: center the node within the bounding box
                        let center_y = min_pos + max_extent / 2.0;
                        node.y = OrderedFloat(center_y - node.height.0 / 2.0);
                    }
                    (AlignmentAxis::Vertical, AlignmentMode::End) => {
                        // Align Bottom: set y so bottom edge aligns with max_bottom
                        node.y = OrderedFloat(max_pos - node.height.0);
                    }
                }
            }
        }
        doc.revision = doc.revision.increment();
    });

    true
}

/// Distribute selected nodes evenly along the specified axis.
///
/// # Preconditions
/// - At least 3 nodes must be selected (distribution requires 3+ to be meaningful)
/// - Selected nodes must have valid (finite) positions
/// - Nodes must be movable (not locked, or are subgraphs)
///
/// # Postconditions
/// - Outermost nodes remain at original bounds
/// - Interior nodes are repositioned to create equal spacing
/// - Node dimensions are preserved
/// - History is updated for undo support
///
/// # Invariants
/// - Distribution does not change node size
/// - Horizontal distribution preserves Y positions
/// - Vertical distribution preserves X positions
/// - Z-order is preserved
pub fn apply_distribute_selection(
    mut doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    axis: DistributionAxis,
) -> bool {
    let current = doc_signal.read().clone();

    // Get selected nodes that are movable (not locked, or are subgraphs)
    let selected_nodes: Vec<NodeId> = selected_node_ids(&current)
        .into_iter()
        .filter(|id| {
            current.document.nodes.get(id).is_some_and(|node| {
                let coords_finite = node.x.0.is_finite() && node.y.0.is_finite();
                let movable = !node.locked || node.kind == NodeKind::Subgraph;
                coords_finite && movable
            })
        })
        .collect();

    // Need at least 3 nodes to distribute
    if selected_nodes.len() < 3 {
        return false;
    }

    // Collect node data: (id, position, size) sorted by position along axis
    let mut node_data: Vec<(NodeId, f64, f64)> = selected_nodes
        .iter()
        .filter_map(|id| {
            current.document.nodes.get(id).map(|node| {
                let (pos, size) = match axis {
                    DistributionAxis::Horizontal => (node.x.0, node.width.0),
                    DistributionAxis::Vertical => (node.y.0, node.height.0),
                };
                (id.clone(), pos, size)
            })
        })
        .collect();

    // Check all positions and sizes are finite
    if node_data
        .iter()
        .any(|(_, pos, size)| !pos.is_finite() || !size.is_finite())
    {
        return false;
    }

    // Sort by position
    node_data.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    // Calculate bounds from outermost nodes
    let first_pos = node_data.first().map(|(_, p, _)| *p);

    let Some(first_pos) = first_pos else {
        return false;
    };
    let Some(last_node_end) = node_data.last().map(|(_, p, s)| p + s) else {
        return false;
    };

    if !first_pos.is_finite() || !last_node_end.is_finite() {
        return false;
    }

    // Calculate total size of all nodes and available space
    let total_node_size: f64 = node_data.iter().map(|(_, _, s)| *s).sum();
    let total_extent = last_node_end - first_pos;

    if total_extent <= f64::EPSILON {
        return false; // Nodes are stacked, no room to distribute
    }

    // Calculate equal spacing between nodes
    let node_count = node_data.len();
    let gap_count = node_count.saturating_sub(1);
    let available_space = total_extent - total_node_size;
    let spacing = if gap_count > 0 {
        available_space / f64::from(u32::try_from(gap_count).unwrap_or(1))
    } else {
        0.0
    };

    if !spacing.is_finite() {
        return false;
    }

    push_history(history_signal, current);

    doc_signal.with_mut(|doc| {
        // Position each node: first stays at first_pos, others distributed
        let mut current_pos = first_pos;
        for (node_id, _, node_size) in &node_data {
            if let Some(node) = doc.document.nodes.get_mut(node_id) {
                // Double-check movability
                if node.locked && node.kind != NodeKind::Subgraph {
                    continue;
                }

                match axis {
                    DistributionAxis::Horizontal => {
                        node.x = OrderedFloat(current_pos);
                    }
                    DistributionAxis::Vertical => {
                        node.y = OrderedFloat(current_pos);
                    }
                }
                current_pos += node_size + spacing;
            }
        }
        doc.revision = doc.revision.increment();
    });

    true
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

    // Pure clipboard function tests

    #[test]
    fn given_empty_selection_when_copy_then_returns_none() {
        let doc = DiagramDocument::default();
        let result = copy_selection(&doc);
        assert!(result.is_none());
    }

    #[test]
    fn given_single_node_selected_when_copy_then_succeeds() {
        let mut doc = make_doc_with_node("node-1", 100.0, 50.0);
        let _ = doc.editor_state.selected_items.insert("node-1".to_string());

        let result = copy_selection(&doc);

        assert!(result.is_some());
        let clipboard = result.unwrap();
        assert_eq!(clipboard.nodes.len(), 1);
        assert!(clipboard.edges.is_empty());
        assert_eq!(clipboard.paste_serial, 0);
    }

    #[test]
    fn given_multiple_nodes_selected_when_copy_then_includes_edges() {
        let (mut doc, _edge_id) = make_doc_with_two_nodes_and_edge("node-a", "node-b");
        let _ = doc.editor_state.selected_items.insert("node-a".to_string());
        let _ = doc.editor_state.selected_items.insert("node-b".to_string());

        let result = copy_selection(&doc);

        assert!(result.is_some());
        let clipboard = result.unwrap();
        assert_eq!(clipboard.nodes.len(), 2);
        assert_eq!(clipboard.edges.len(), 1);
    }

    #[test]
    fn given_empty_clipboard_when_paste_then_returns_none() {
        let clipboard = Clipboard::new();
        let doc = DiagramDocument::default();
        let node_count_before = doc.document.nodes.len();

        let result = paste_contents(clipboard, doc);

        assert!(result.is_none());
        // Document should not be modified
        let (_, returned_doc) = result.unwrap_or_default();
        assert_eq!(returned_doc.document.nodes.len(), node_count_before);
    }

    #[test]
    fn given_copied_nodes_when_paste_then_creates_new_ids() {
        let mut doc = make_doc_with_node("original-node", 100.0, 50.0);
        let _ = doc
            .editor_state
            .selected_items
            .insert("original-node".to_string());

        let Some(clipboard) = copy_selection(&doc) else {
            panic!("Copy should succeed");
        };
        assert!(clipboard.has_content());

        let Some((new_doc, _)) = paste_contents(clipboard, doc) else {
            panic!("Paste should succeed");
        };

        assert_eq!(new_doc.document.nodes.len(), 2);
        let original_id = NodeId::new("original-node".to_string());
        let pasted_ids: Vec<_> = new_doc
            .document
            .nodes
            .keys()
            .filter(|id| *id != &original_id)
            .collect();
        assert_eq!(pasted_ids.len(), 1);
    }

    #[test]
    fn given_copied_nodes_when_paste_then_applies_offset() {
        let mut doc = make_doc_with_node("original-node", 100.0, 50.0);
        let _ = doc
            .editor_state
            .selected_items
            .insert("original-node".to_string());

        let Some(clipboard) = copy_selection(&doc) else {
            panic!("Copy should succeed");
        };

        let Some((new_doc, _)) = paste_contents(clipboard, doc) else {
            panic!("Paste should succeed");
        };

        let original_id = NodeId::new("original-node".to_string());
        let pasted_node = new_doc
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

    // =============================================================================
    // Additional copy/paste tests (bd-2b4)
    // =============================================================================

    #[test]
    fn given_selection_with_nonexistent_ids_when_copy_then_returns_false() {
        clear_clipboard();
        let mut doc = DiagramDocument::default();
        // Add a node but select a different (non-existent) ID
        let real_id = NodeId::new("real-node".to_string());
        let _ = doc
            .document
            .nodes
            .insert(real_id, make_node("real-node", 0.0, 0.0));
        let _ = doc
            .editor_state
            .selected_items
            .insert("ghost-id".to_string());

        let result = copy_selection_to_clipboard(&doc);

        assert!(!result);
        CLIPBOARD.with(|s| assert!(s.borrow().is_none()));
    }

    #[test]
    fn given_three_nodes_selected_when_copy_then_copies_all() {
        clear_clipboard();
        let mut doc = DiagramDocument::default();
        let node_a = NodeId::new("node-a".to_string());
        let node_b = NodeId::new("node-b".to_string());
        let node_c = NodeId::new("node-c".to_string());
        let _ = doc
            .document
            .nodes
            .insert(node_a.clone(), make_node("node-a", 0.0, 0.0));
        let _ = doc
            .document
            .nodes
            .insert(node_b.clone(), make_node("node-b", 100.0, 0.0));
        let _ = doc
            .document
            .nodes
            .insert(node_c.clone(), make_node("node-c", 200.0, 0.0));
        let _ = doc.editor_state.selected_items.insert("node-a".to_string());
        let _ = doc.editor_state.selected_items.insert("node-b".to_string());
        let _ = doc.editor_state.selected_items.insert("node-c".to_string());

        let result = copy_selection_to_clipboard(&doc);

        assert!(result);
        CLIPBOARD.with(|s| {
            let clip = s.borrow();
            if let Some(c) = clip.as_ref() {
                assert_eq!(c.nodes.len(), 3);
            } else {
                panic!("clipboard should have content");
            }
        });
    }

    #[test]
    fn given_partial_edge_selection_when_copy_then_excludes_edge() {
        clear_clipboard();
        let (mut doc, _edge_id) = make_doc_with_two_nodes_and_edge("node-a", "node-b");
        // Only select source node, not target
        let _ = doc.editor_state.selected_items.insert("node-a".to_string());

        let result = copy_selection_to_clipboard(&doc);

        assert!(result);
        CLIPBOARD.with(|s| {
            let clip = s.borrow();
            if let Some(c) = clip.as_ref() {
                assert_eq!(c.nodes.len(), 1, "should copy one node");
                assert!(
                    c.edges.is_empty(),
                    "edge should be excluded when target not selected"
                );
            } else {
                panic!("clipboard should have content");
            }
        });
    }

    fn make_doc_with_parent_child(
        parent_id: &str,
        child_id: &str,
    ) -> (DiagramDocument, NodeId, NodeId) {
        let mut doc = DiagramDocument::default();
        let parent_node_id = NodeId::new(parent_id.to_string());
        let child_node_id = NodeId::new(child_id.to_string());

        let _ = doc
            .document
            .nodes
            .insert(parent_node_id.clone(), make_node(parent_id, 0.0, 0.0));

        let mut child_node = make_node(child_id, 50.0, 50.0);
        child_node.parent = Some(parent_node_id.clone());
        let _ = doc.document.nodes.insert(child_node_id.clone(), child_node);

        (doc, parent_node_id, child_node_id)
    }

    #[test]
    fn given_nested_nodes_selected_when_copy_then_preserves_parent_reference() {
        clear_clipboard();
        let (mut doc, parent_id, _child_id) = make_doc_with_parent_child("parent", "child");
        let _ = doc.editor_state.selected_items.insert("parent".to_string());
        let _ = doc.editor_state.selected_items.insert("child".to_string());

        let result = copy_selection_to_clipboard(&doc);

        assert!(result);
        CLIPBOARD.with(|s| {
            let clip = s.borrow();
            if let Some(c) = clip.as_ref() {
                assert_eq!(c.nodes.len(), 2);
                // Find the child node in clipboard
                let child_in_clipboard = c.nodes.iter().find(|(id, _)| id.to_string() == "child");
                if let Some((_, node)) = child_in_clipboard {
                    assert_eq!(
                        node.parent,
                        Some(parent_id),
                        "parent reference preserved during copy"
                    );
                } else {
                    panic!("child should be in clipboard");
                }
            } else {
                panic!("clipboard should have content");
            }
        });
    }

    #[test]
    fn given_clipboard_with_empty_nodes_when_paste_then_returns_false() {
        clear_clipboard();
        let mut doc = DiagramDocument::default();
        let node_count_before = doc.document.nodes.len();

        // Set clipboard with empty nodes vector
        CLIPBOARD.with(|s| {
            *s.borrow_mut() = Some(ClipboardState {
                nodes: vec![],
                edges: vec![],
                paste_serial: 0,
            });
        });

        let result = paste_from_clipboard(&mut doc);

        assert!(!result);
        assert_eq!(doc.document.nodes.len(), node_count_before);
    }

    #[test]
    fn given_second_paste_when_paste_then_applies_double_offset() {
        clear_clipboard();
        let mut doc = make_doc_with_node("original-node", 100.0, 50.0);
        let _ = doc
            .editor_state
            .selected_items
            .insert("original-node".to_string());

        let _ = copy_selection_to_clipboard(&doc);

        // First paste
        let _ = paste_from_clipboard(&mut doc);
        // Second paste
        let _ = paste_from_clipboard(&mut doc);

        let original_id = NodeId::new("original-node".to_string());
        // Find the second pasted node (there should be 2 pasted nodes now)
        let pasted_nodes: Vec<_> = doc
            .document
            .nodes
            .iter()
            .filter(|(id, _)| *id != &original_id)
            .collect();

        assert_eq!(pasted_nodes.len(), 2);

        // Second paste should have offset of 40.0 (20.0 * 2)
        let second_paste_node = pasted_nodes
            .iter()
            .find(|(_, node)| node.x.0 == 140.0 && node.y.0 == 90.0);
        assert!(
            second_paste_node.is_some(),
            "second paste should have offset of 40.0 (position 140.0, 90.0)"
        );
    }

    #[test]
    fn given_multiple_nodes_when_paste_then_all_ids_unique() {
        clear_clipboard();
        let mut doc = DiagramDocument::default();
        let node_a = NodeId::new("node-a".to_string());
        let node_b = NodeId::new("node-b".to_string());
        let node_c = NodeId::new("node-c".to_string());
        let _ = doc
            .document
            .nodes
            .insert(node_a.clone(), make_node("node-a", 0.0, 0.0));
        let _ = doc
            .document
            .nodes
            .insert(node_b.clone(), make_node("node-b", 100.0, 0.0));
        let _ = doc
            .document
            .nodes
            .insert(node_c.clone(), make_node("node-c", 200.0, 0.0));
        let _ = doc.editor_state.selected_items.insert("node-a".to_string());
        let _ = doc.editor_state.selected_items.insert("node-b".to_string());
        let _ = doc.editor_state.selected_items.insert("node-c".to_string());

        let _ = copy_selection_to_clipboard(&doc);
        let _ = paste_from_clipboard(&mut doc);

        // Should have 6 nodes now (3 original + 3 pasted)
        assert_eq!(doc.document.nodes.len(), 6);

        // All IDs should be unique
        let ids: std::collections::HashSet<_> = doc.document.nodes.keys().collect();
        assert_eq!(ids.len(), 6, "all node IDs should be unique");
    }

    #[test]
    fn given_edge_in_clipboard_when_paste_then_remapped_to_new_ids() {
        clear_clipboard();
        let (mut doc, _edge_id) = make_doc_with_two_nodes_and_edge("node-a", "node-b");
        let _ = doc.editor_state.selected_items.insert("node-a".to_string());
        let _ = doc.editor_state.selected_items.insert("node-b".to_string());

        let _ = copy_selection_to_clipboard(&doc);
        let _ = paste_from_clipboard(&mut doc);

        // Should have 2 edges now
        assert_eq!(doc.document.edges.len(), 2);

        // Find the pasted edge (the one whose source/target are not the originals)
        let original_a = NodeId::new("node-a".to_string());
        let original_b = NodeId::new("node-b".to_string());
        let pasted_edge = doc
            .document
            .edges
            .iter()
            .find(|(_, edge)| edge.source != original_a && edge.target != original_b);

        assert!(
            pasted_edge.is_some(),
            "should have a pasted edge with remapped IDs"
        );
        if let Some((_, edge)) = pasted_edge {
            // Verify the pasted edge's source and target are NOT the original IDs
            assert_ne!(edge.source, original_a);
            assert_ne!(edge.target, original_b);
            // And they should be actual node IDs in the document
            assert!(doc.document.nodes.contains_key(&edge.source));
            assert!(doc.document.nodes.contains_key(&edge.target));
        }
    }

    #[test]
    fn given_parent_also_pasted_when_paste_then_remapped() {
        clear_clipboard();
        let (mut doc, _parent_id, _child_id) = make_doc_with_parent_child("parent", "child");
        let _ = doc.editor_state.selected_items.insert("parent".to_string());
        let _ = doc.editor_state.selected_items.insert("child".to_string());

        let original_parent_id = NodeId::new("parent".to_string());

        let _ = copy_selection_to_clipboard(&doc);
        let _ = paste_from_clipboard(&mut doc);

        // Find the pasted child node
        let pasted_child = doc
            .document
            .nodes
            .iter()
            .find(|(id, node)| id.to_string() != "child" && node.label == "child");

        assert!(pasted_child.is_some());
        if let Some((_, child_node)) = pasted_child {
            // Parent should be remapped to the new parent ID, not the original
            if let Some(ref parent_ref) = child_node.parent {
                assert_ne!(
                    *parent_ref, original_parent_id,
                    "parent should be remapped to new pasted parent ID"
                );
                // The new parent should exist in the document
                assert!(
                    doc.document.nodes.contains_key(parent_ref),
                    "remapped parent should exist in document"
                );
            } else {
                panic!("pasted child should have a parent");
            }
        }
    }

    #[test]
    fn given_parent_not_pasted_when_paste_then_preserved() {
        clear_clipboard();
        let (mut doc, parent_id, _child_id) = make_doc_with_parent_child("parent", "child");
        // Only select and copy the child, not the parent
        let _ = doc.editor_state.selected_items.insert("child".to_string());

        let _ = copy_selection_to_clipboard(&doc);
        let _ = paste_from_clipboard(&mut doc);

        // Find the pasted child node
        let pasted_child = doc
            .document
            .nodes
            .iter()
            .find(|(id, node)| id.to_string() != "child" && node.label == "child");

        assert!(pasted_child.is_some());
        if let Some((_, child_node)) = pasted_child {
            // Parent should still point to the original parent (not remapped)
            assert_eq!(
                child_node.parent,
                Some(parent_id),
                "parent reference should be preserved when parent not pasted"
            );
        }
    }

    #[test]
    fn given_paste_successful_when_paste_then_selection_updated() {
        clear_clipboard();
        let mut doc = make_doc_with_node("original-node", 100.0, 50.0);
        let _ = doc
            .editor_state
            .selected_items
            .insert("original-node".to_string());

        let _ = copy_selection_to_clipboard(&doc);

        // Clear selection before paste to test that paste updates it
        doc.editor_state.selected_items.clear();
        let _ = paste_from_clipboard(&mut doc);

        // Selection should contain only the new pasted node
        assert_eq!(doc.editor_state.selected_items.len(), 1);

        // The selected item should NOT be the original node
        let selected_id = doc.editor_state.selected_items.iter().next();
        assert!(selected_id.is_some());
        if let Some(id) = selected_id {
            assert_ne!(
                id, "original-node",
                "selection should be the new pasted node, not original"
            );
        }
    }

    #[test]
    fn given_paste_successful_when_paste_then_revision_incremented() {
        clear_clipboard();
        let mut doc = make_doc_with_node("original-node", 100.0, 50.0);
        let _ = doc
            .editor_state
            .selected_items
            .insert("original-node".to_string());

        let _ = copy_selection_to_clipboard(&doc);

        let revision_before = doc.revision;
        let _ = paste_from_clipboard(&mut doc);

        assert_eq!(
            doc.revision,
            revision_before.increment(),
            "revision should be incremented after successful paste"
        );
    }

    #[test]
    fn given_clipboard_when_has_content_then_returns_true() {
        let clipboard = Clipboard {
            nodes: vec![(
                NodeId::new("test".to_string()),
                make_node("test", 0.0, 0.0),
            )],
            edges: Vec::new(),
            paste_serial: 0,
        };
        assert!(clipboard.has_content());
    }

    #[test]
    fn given_empty_clipboard_when_has_content_then_returns_false() {
        let clipboard = Clipboard::new();
        assert!(!clipboard.has_content());
    }

    #[test]
    fn given_none_clipboard_when_clipboard_has_content_then_returns_false() {
        assert!(!clipboard_has_content(&None));
    }

    #[test]
    fn given_some_clipboard_when_clipboard_has_content_then_reflects_content() {
        let empty = Clipboard::new();
        assert!(!clipboard_has_content(&Some(empty)));

        let with_content = Clipboard {
            nodes: vec![(
                NodeId::new("test".to_string()),
                make_node("test", 0.0, 0.0),
            )],
            edges: Vec::new(),
            paste_serial: 0,
        };
        assert!(clipboard_has_content(&Some(with_content)));
    }

    #[test]
    fn given_clipboard_when_prepare_paste_then_increments_serial() {
        let clipboard = Clipboard {
            nodes: vec![(
                NodeId::new("test".to_string()),
                make_node("test", 0.0, 0.0),
            )],
            edges: Vec::new(),
            paste_serial: 0,
        };

        let prepared = clipboard.prepare_paste();
        assert_eq!(prepared.paste_serial, 1);

        let prepared_again = prepared.prepare_paste();
        assert_eq!(prepared_again.paste_serial, 2);
    }

    #[test]
    fn given_selection_when_copy_selection_for_duplicate_then_sets_serial_to_1() {
        let mut doc = make_doc_with_node("node-1", 100.0, 50.0);
        let _ = doc.editor_state.selected_items.insert("node-1".to_string());

        let Some(clipboard) = copy_selection_for_duplicate(&doc) else {
            panic!("Copy should succeed");
        };

        assert_eq!(clipboard.paste_serial, 1);
        assert!(!clipboard.nodes.is_empty());
    }

    #[test]
    fn given_multiple_pastes_then_offset_increases() {
        let mut doc = make_doc_with_node("original-node", 100.0, 50.0);
        let _ = doc
            .editor_state
            .selected_items
            .insert("original-node".to_string());

        let Some(clipboard) = copy_selection(&doc) else {
            panic!("Copy should succeed");
        };

        // First paste
        let Some((doc1, clipboard1)) = paste_contents(clipboard, doc) else {
            panic!("First paste should succeed");
        };

        // Second paste
        let Some((doc2, _)) = paste_contents(clipboard1, doc1) else {
            panic!("Second paste should succeed");
        };

        assert_eq!(doc2.document.nodes.len(), 3); // Original + 2 pasted

        // Check that the second paste has more offset
        let original_id = NodeId::new("original-node".to_string());
        let pasted_nodes: Vec<_> = doc2
            .document
            .nodes
            .iter()
            .filter(|(id, _)| *id != &original_id)
            .map(|(_, node)| node.clone())
            .collect();

        assert_eq!(pasted_nodes.len(), 2);

        // First paste should be at (120, 70)
        // Second paste should be at (140, 90) - offset increases with serial
        let mut positions: Vec<_> = pasted_nodes
            .iter()
            .map(|n| (n.x.0, n.y.0))
            .collect();
        positions.sort_by_key(|&(x, y)| (x as i64, y as i64));

        assert_eq!(positions[0].0, 120.0);
        assert_eq!(positions[0].1, 70.0);
        assert_eq!(positions[1].0, 140.0);
        assert_eq!(positions[1].1, 90.0);
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

    // =============================================================================
    // SUB subgraph tests (bd-163)
    // =============================================================================

    fn make_subgraph_node(
        id: &str,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        parent: Option<NodeId>,
    ) -> Node {
        Node {
            kind: NodeKind::Subgraph,
            icon: String::new(),
            label: id.to_string(),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(w),
            height: OrderedFloat(h),
            font_size: None,
            font_weight: None,
            locked: true,
            parent,
            dag_rank: None,
            tags: Vec::new(),
            metadata: im::HashMap::new(),
            z_index: -1,
            style: Some(NodeStyle::Box),
            collapsed: Some(false),
        }
    }

    /// Pure function implementing group selection logic for testing
    fn perform_group_selection(doc: &mut DiagramDocument) -> bool {
        let selected_nodes: Vec<NodeId> = selected_node_ids(doc)
            .into_iter()
            .filter(|id| {
                doc.document
                    .nodes
                    .get(id)
                    .is_some_and(|node| node.kind != NodeKind::Subgraph)
            })
            .collect();

        if selected_nodes.len() < 2 {
            return false;
        }

        let (min_x, min_y, max_x, max_y) = selected_nodes.iter().fold(
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
        );

        if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
            return false;
        }

        let group_id = NodeId::new(Uuid::new_v4().to_string());
        let member_ids = selected_nodes;

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
        true
    }

    /// Pure function implementing ungroup selection logic for testing
    fn perform_ungroup_selection(doc: &mut DiagramDocument) -> bool {
        let target_subgraphs: BTreeSet<NodeId> = selected_node_ids(doc)
            .into_iter()
            .filter(|id| {
                doc.document
                    .nodes
                    .get(id)
                    .is_some_and(|node| node.kind == NodeKind::Subgraph)
            })
            .collect();

        if target_subgraphs.is_empty() {
            return false;
        }

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
        true
    }

    #[test]
    fn given_two_selected_nodes_when_group_selection_then_creates_subgraph_with_correct_bounds() {
        let mut doc = DiagramDocument::default();
        let node_a = NodeId::new("node-a".to_string());
        let node_b = NodeId::new("node-b".to_string());

        // Node A at (100, 100), size 100x50
        doc.document.nodes = doc
            .document
            .nodes
            .update(node_a.clone(), make_node("node-a", 100.0, 100.0))
            .update(node_b.clone(), make_node("node-b", 250.0, 200.0));

        // Select both nodes
        let _ = doc.editor_state.selected_items.insert("node-a".to_string());
        let _ = doc.editor_state.selected_items.insert("node-b".to_string());

        let initial_revision = doc.revision;

        let result = perform_group_selection(&mut doc);

        assert!(result, "group_selection should return true");

        // Revision should be incremented
        assert_eq!(
            doc.revision,
            initial_revision.increment(),
            "revision should be incremented"
        );

        // Find the created subgraph (should be the only subgraph)
        let subgraphs: Vec<_> = doc
            .document
            .nodes
            .iter()
            .filter(|(_, n)| n.kind == NodeKind::Subgraph)
            .collect();
        assert_eq!(subgraphs.len(), 1, "should have exactly one subgraph");

        let (group_id, group_node) = subgraphs[0].clone();

        // Verify bounds with 24px padding
        // Node A: (100, 100) to (200, 150)
        // Node B: (250, 200) to (350, 250)
        // Expected bounds: (100-24, 100-24) to (350+24, 250+24)
        // Width: 350 - 100 + 48 = 298
        // Height: 250 - 100 + 48 = 198
        assert_eq!(group_node.x.0, 76.0, "group x should be min_x - padding");
        assert_eq!(group_node.y.0, 76.0, "group y should be min_y - padding");
        assert_eq!(
            group_node.width.0, 298.0,
            "group width should cover all nodes + padding"
        );
        assert_eq!(
            group_node.height.0, 198.0,
            "group height should cover all nodes + padding"
        );

        // Verify parent relationships
        let node_a_after = doc
            .document
            .nodes
            .get(&node_a)
            .expect("node-a should exist");
        let node_b_after = doc
            .document
            .nodes
            .get(&node_b)
            .expect("node-b should exist");

        assert_eq!(
            node_a_after.parent,
            Some(group_id.clone()),
            "node-a parent should be the group"
        );
        assert_eq!(
            node_b_after.parent,
            Some(group_id.clone()),
            "node-b parent should be the group"
        );

        // Verify selection is now just the group
        assert_eq!(
            doc.editor_state.selected_items.len(),
            1,
            "selection should have exactly one item"
        );
        assert!(
            doc.editor_state
                .selected_items
                .contains(&group_id.to_string()),
            "selection should contain the group"
        );
    }

    #[test]
    fn given_selected_subgraph_with_children_when_ungroup_then_children_restored_to_root() {
        let mut doc = DiagramDocument::default();
        let group_id = NodeId::new("group-1".to_string());
        let child_a = NodeId::new("child-a".to_string());
        let child_b = NodeId::new("child-b".to_string());
        let outsider = NodeId::new("outsider".to_string());

        // Create subgraph
        doc.document.nodes = doc
            .document
            .nodes
            .update(
                group_id.clone(),
                make_subgraph_node("group-1", 50.0, 50.0, 300.0, 200.0, None),
            )
            // Children with parent pointing to group
            .update(child_a.clone(), {
                let mut n = make_node("child-a", 100.0, 100.0);
                n.parent = Some(group_id.clone());
                n
            })
            .update(child_b.clone(), {
                let mut n = make_node("child-b", 200.0, 150.0);
                n.parent = Some(group_id.clone());
                n
            })
            // Node outside the group with no parent
            .update(outsider.clone(), make_node("outsider", 500.0, 500.0));

        // Select the subgraph
        let _ = doc
            .editor_state
            .selected_items
            .insert("group-1".to_string());

        let initial_revision = doc.revision;

        let result = perform_ungroup_selection(&mut doc);

        assert!(result, "ungroup_selection should return true");

        // Revision should be incremented
        assert_eq!(
            doc.revision,
            initial_revision.increment(),
            "revision should be incremented"
        );

        // Subgraph should be removed
        assert!(
            !doc.document.nodes.contains_key(&group_id),
            "subgraph should be removed"
        );

        // Children should still exist with parent = None
        let child_a_after = doc
            .document
            .nodes
            .get(&child_a)
            .expect("child-a should exist");
        let child_b_after = doc
            .document
            .nodes
            .get(&child_b)
            .expect("child-b should exist");

        assert!(
            child_a_after.parent.is_none(),
            "child-a parent should be None after ungroup"
        );
        assert!(
            child_b_after.parent.is_none(),
            "child-b parent should be None after ungroup"
        );

        // Children should retain their absolute positions
        assert_eq!(child_a_after.x.0, 100.0, "child-a x position preserved");
        assert_eq!(child_a_after.y.0, 100.0, "child-a y position preserved");
        assert_eq!(child_b_after.x.0, 200.0, "child-b x position preserved");
        assert_eq!(child_b_after.y.0, 150.0, "child-b y position preserved");

        // Outsider should be unchanged
        let outsider_after = doc
            .document
            .nodes
            .get(&outsider)
            .expect("outsider should exist");
        assert!(
            outsider_after.parent.is_none(),
            "outsider parent should remain None"
        );

        // Selection should be cleared
        assert!(
            doc.editor_state.selected_items.is_empty(),
            "selection should be cleared after ungroup"
        );
    }

    #[test]
    fn given_nested_subgraphs_when_validated_then_parent_chain_correct() {
        let mut doc = DiagramDocument::default();
        let outer_id = NodeId::new("outer".to_string());
        let inner_id = NodeId::new("inner".to_string());
        let child_id = NodeId::new("child".to_string());

        // Outer subgraph at root level
        doc.document.nodes = doc
            .document
            .nodes
            .update(
                outer_id.clone(),
                make_subgraph_node("outer", 50.0, 50.0, 500.0, 400.0, None),
            )
            // Inner subgraph inside outer
            .update(inner_id.clone(), {
                let mut n =
                    make_subgraph_node("inner", 100.0, 100.0, 350.0, 250.0, Some(outer_id.clone()));
                n
            })
            // Regular node inside inner
            .update(child_id.clone(), {
                let mut n = make_node("child", 150.0, 150.0);
                n.parent = Some(inner_id.clone());
                n
            });

        // Verify the hierarchy
        let outer = doc
            .document
            .nodes
            .get(&outer_id)
            .expect("outer should exist");
        let inner = doc
            .document
            .nodes
            .get(&inner_id)
            .expect("inner should exist");
        let child = doc
            .document
            .nodes
            .get(&child_id)
            .expect("child should exist");

        assert!(outer.parent.is_none(), "outer should have no parent");
        assert_eq!(
            inner.parent.as_ref(),
            Some(&outer_id),
            "inner's parent should be outer"
        );
        assert_eq!(
            child.parent.as_ref(),
            Some(&inner_id),
            "child's parent should be inner"
        );

        // Verify all are subgraphs except child
        assert_eq!(outer.kind, NodeKind::Subgraph);
        assert_eq!(inner.kind, NodeKind::Subgraph);
        assert_eq!(child.kind, NodeKind::Node);
    }

    #[test]
    fn given_single_node_selected_when_group_selection_then_returns_false() {
        let mut doc = DiagramDocument::default();
        let node_id = NodeId::new("single-node".to_string());

        doc.document.nodes = doc
            .document
            .nodes
            .update(node_id, make_node("single-node", 100.0, 100.0));
        let _ = doc
            .editor_state
            .selected_items
            .insert("single-node".to_string());

        let result = perform_group_selection(&mut doc);

        assert!(
            !result,
            "group_selection should return false for single node"
        );
    }

    #[test]
    fn given_subgraph_selected_when_group_selection_then_subgraph_excluded() {
        let mut doc = DiagramDocument::default();
        let subgraph_id = NodeId::new("existing-subgraph".to_string());
        let node_id = NodeId::new("regular-node".to_string());

        doc.document.nodes = doc
            .document
            .nodes
            .update(
                subgraph_id.clone(),
                make_subgraph_node("existing-subgraph", 50.0, 50.0, 200.0, 150.0, None),
            )
            .update(node_id.clone(), make_node("regular-node", 100.0, 100.0));

        // Select both subgraph and regular node
        let _ = doc
            .editor_state
            .selected_items
            .insert("existing-subgraph".to_string());
        let _ = doc
            .editor_state
            .selected_items
            .insert("regular-node".to_string());

        let result = perform_group_selection(&mut doc);

        // Should fail because only one non-subgraph node is selected
        assert!(
            !result,
            "group_selection should return false when only one non-subgraph node is selected"
        );
    }

    // =============================================================================
    // Alignment tests (bd-2ng)
    // =============================================================================

    /// Pure function implementing alignment logic for testing
    fn perform_align_selection(
        doc: &mut DiagramDocument,
        axis: AlignmentAxis,
        mode: AlignmentMode,
    ) -> bool {
        // Get selected nodes that are movable
        let selected_nodes: Vec<NodeId> = selected_node_ids(doc)
            .into_iter()
            .filter(|id| {
                doc.document.nodes.get(id).is_some_and(|node| {
                    let coords_finite = node.x.0.is_finite() && node.y.0.is_finite();
                    let movable = !node.locked || node.kind == NodeKind::Subgraph;
                    coords_finite && movable
                })
            })
            .collect();

        if selected_nodes.len() < 2 {
            return false;
        }

        // Calculate bounding box
        let (min_pos, max_pos, max_extent) = match axis {
            AlignmentAxis::Horizontal => {
                let positions: Vec<(f64, f64)> = selected_nodes
                    .iter()
                    .filter_map(|id| doc.document.nodes.get(id))
                    .map(|node| (node.x.0, node.x.0 + node.width.0))
                    .collect();

                if positions
                    .iter()
                    .any(|(p, e)| !p.is_finite() || !e.is_finite())
                {
                    return false;
                }

                let min_x = positions
                    .iter()
                    .map(|(p, _)| *p)
                    .fold(f64::INFINITY, f64::min);
                let max_right = positions
                    .iter()
                    .map(|(_, e)| *e)
                    .fold(f64::NEG_INFINITY, f64::max);

                if !min_x.is_finite() || !max_right.is_finite() {
                    return false;
                }

                (min_x, max_right, max_right - min_x)
            }
            AlignmentAxis::Vertical => {
                let positions: Vec<(f64, f64)> = selected_nodes
                    .iter()
                    .filter_map(|id| doc.document.nodes.get(id))
                    .map(|node| (node.y.0, node.y.0 + node.height.0))
                    .collect();

                if positions
                    .iter()
                    .any(|(p, e)| !p.is_finite() || !e.is_finite())
                {
                    return false;
                }

                let min_y = positions
                    .iter()
                    .map(|(p, _)| *p)
                    .fold(f64::INFINITY, f64::min);
                let max_bottom = positions
                    .iter()
                    .map(|(_, e)| *e)
                    .fold(f64::NEG_INFINITY, f64::max);

                if !min_y.is_finite() || !max_bottom.is_finite() {
                    return false;
                }

                (min_y, max_bottom, max_bottom - min_y)
            }
        };

        for node_id in &selected_nodes {
            if let Some(node) = doc.document.nodes.get_mut(node_id) {
                if node.locked && node.kind != NodeKind::Subgraph {
                    continue;
                }

                match (axis, mode) {
                    (AlignmentAxis::Horizontal, AlignmentMode::Start) => {
                        node.x = OrderedFloat(min_pos);
                    }
                    (AlignmentAxis::Horizontal, AlignmentMode::Center) => {
                        let center_x = min_pos + max_extent / 2.0;
                        node.x = OrderedFloat(center_x - node.width.0 / 2.0);
                    }
                    (AlignmentAxis::Horizontal, AlignmentMode::End) => {
                        node.x = OrderedFloat(max_pos - node.width.0);
                    }
                    (AlignmentAxis::Vertical, AlignmentMode::Start) => {
                        node.y = OrderedFloat(min_pos);
                    }
                    (AlignmentAxis::Vertical, AlignmentMode::Center) => {
                        let center_y = min_pos + max_extent / 2.0;
                        node.y = OrderedFloat(center_y - node.height.0 / 2.0);
                    }
                    (AlignmentAxis::Vertical, AlignmentMode::End) => {
                        node.y = OrderedFloat(max_pos - node.height.0);
                    }
                }
            }
        }
        doc.revision = doc.revision.increment();
        true
    }

    #[test]
    fn given_two_nodes_when_align_left_then_both_share_min_x() {
        let mut doc = DiagramDocument::default();
        let node_a = NodeId::new("node-a".to_string());
        let node_b = NodeId::new("node-b".to_string());

        doc.document.nodes = doc
            .document
            .nodes
            .update(node_a.clone(), make_node("node-a", 100.0, 100.0))
            .update(node_b.clone(), make_node("node-b", 200.0, 150.0));

        let _ = doc.editor_state.selected_items.insert("node-a".to_string());
        let _ = doc.editor_state.selected_items.insert("node-b".to_string());

        let result =
            perform_align_selection(&mut doc, AlignmentAxis::Horizontal, AlignmentMode::Start);

        assert!(result, "align_left should return true");
        assert_eq!(doc.document.nodes.get(&node_a).map(|n| n.x.0), Some(100.0));
        assert_eq!(
            doc.document.nodes.get(&node_b).map(|n| n.x.0),
            Some(100.0),
            "node-b x should be aligned to min_x"
        );
    }

    #[test]
    fn given_two_nodes_when_align_right_then_both_share_max_right() {
        let mut doc = DiagramDocument::default();
        let node_a = NodeId::new("node-a".to_string());
        let node_b = NodeId::new("node-b".to_string());

        doc.document.nodes = doc
            .document
            .nodes
            .update(node_a.clone(), make_node("node-a", 100.0, 100.0))
            .update(node_b.clone(), make_node("node-b", 200.0, 150.0));

        let _ = doc.editor_state.selected_items.insert("node-a".to_string());
        let _ = doc.editor_state.selected_items.insert("node-b".to_string());

        let result =
            perform_align_selection(&mut doc, AlignmentAxis::Horizontal, AlignmentMode::End);

        assert!(result, "align_right should return true");
        assert_eq!(
            doc.document.nodes.get(&node_a).map(|n| n.x.0),
            Some(200.0),
            "node-a x should be 200"
        );
        assert_eq!(
            doc.document.nodes.get(&node_b).map(|n| n.x.0),
            Some(200.0),
            "node-b x should be 200"
        );
    }

    #[test]
    fn given_three_nodes_when_align_center_horizontal_then_centered() {
        let mut doc = DiagramDocument::default();
        let node_a = NodeId::new("node-a".to_string());
        let node_b = NodeId::new("node-b".to_string());
        let node_c = NodeId::new("node-c".to_string());

        doc.document.nodes = doc
            .document
            .nodes
            .update(node_a.clone(), make_node("node-a", 100.0, 100.0))
            .update(node_b.clone(), make_node("node-b", 250.0, 150.0))
            .update(node_c.clone(), make_node("node-c", 180.0, 200.0));

        let _ = doc.editor_state.selected_items.insert("node-a".to_string());
        let _ = doc.editor_state.selected_items.insert("node-b".to_string());
        let _ = doc.editor_state.selected_items.insert("node-c".to_string());

        let result =
            perform_align_selection(&mut doc, AlignmentAxis::Horizontal, AlignmentMode::Center);

        assert!(result, "align_center_horizontal should return true");
        assert_eq!(
            doc.document.nodes.get(&node_a).map(|n| n.x.0),
            Some(175.0),
            "node-a x should be centered"
        );
        assert_eq!(
            doc.document.nodes.get(&node_b).map(|n| n.x.0),
            Some(175.0),
            "node-b x should be centered"
        );
        assert_eq!(
            doc.document.nodes.get(&node_c).map(|n| n.x.0),
            Some(175.0),
            "node-c x should be centered"
        );
    }

    #[test]
    fn given_two_nodes_when_align_top_then_both_share_min_y() {
        let mut doc = DiagramDocument::default();
        let node_a = NodeId::new("node-a".to_string());
        let node_b = NodeId::new("node-b".to_string());

        doc.document.nodes = doc
            .document
            .nodes
            .update(node_a.clone(), make_node("node-a", 100.0, 100.0))
            .update(node_b.clone(), make_node("node-b", 200.0, 200.0));

        let _ = doc.editor_state.selected_items.insert("node-a".to_string());
        let _ = doc.editor_state.selected_items.insert("node-b".to_string());

        let result =
            perform_align_selection(&mut doc, AlignmentAxis::Vertical, AlignmentMode::Start);

        assert!(result, "align_top should return true");
        assert_eq!(doc.document.nodes.get(&node_a).map(|n| n.y.0), Some(100.0));
        assert_eq!(
            doc.document.nodes.get(&node_b).map(|n| n.y.0),
            Some(100.0),
            "node-b y should be aligned to min_y"
        );
    }

    #[test]
    fn given_two_nodes_when_align_bottom_then_both_share_max_bottom() {
        let mut doc = DiagramDocument::default();
        let node_a = NodeId::new("node-a".to_string());
        let node_b = NodeId::new("node-b".to_string());

        doc.document.nodes = doc
            .document
            .nodes
            .update(node_a.clone(), make_node("node-a", 100.0, 100.0))
            .update(node_b.clone(), make_node("node-b", 200.0, 200.0));

        let _ = doc.editor_state.selected_items.insert("node-a".to_string());
        let _ = doc.editor_state.selected_items.insert("node-b".to_string());

        let result = perform_align_selection(&mut doc, AlignmentAxis::Vertical, AlignmentMode::End);

        assert!(result, "align_bottom should return true");
        assert_eq!(
            doc.document.nodes.get(&node_a).map(|n| n.y.0),
            Some(200.0),
            "node-a y should be 200"
        );
        assert_eq!(
            doc.document.nodes.get(&node_b).map(|n| n.y.0),
            Some(200.0),
            "node-b y should be 200"
        );
    }

    #[test]
    fn given_three_nodes_when_align_middle_vertical_then_centered() {
        let mut doc = DiagramDocument::default();
        let node_a = NodeId::new("node-a".to_string());
        let node_b = NodeId::new("node-b".to_string());
        let node_c = NodeId::new("node-c".to_string());

        doc.document.nodes = doc
            .document
            .nodes
            .update(node_a.clone(), make_node("node-a", 100.0, 100.0))
            .update(node_b.clone(), make_node("node-b", 200.0, 200.0))
            .update(node_c.clone(), make_node("node-c", 150.0, 150.0));

        let _ = doc.editor_state.selected_items.insert("node-a".to_string());
        let _ = doc.editor_state.selected_items.insert("node-b".to_string());
        let _ = doc.editor_state.selected_items.insert("node-c".to_string());

        let result =
            perform_align_selection(&mut doc, AlignmentAxis::Vertical, AlignmentMode::Center);

        assert!(result, "align_middle_vertical should return true");
        assert_eq!(
            doc.document.nodes.get(&node_a).map(|n| n.y.0),
            Some(150.0),
            "node-a y should be centered"
        );
        assert_eq!(
            doc.document.nodes.get(&node_b).map(|n| n.y.0),
            Some(150.0),
            "node-b y should be centered"
        );
        assert_eq!(
            doc.document.nodes.get(&node_c).map(|n| n.y.0),
            Some(150.0),
            "node-c y should be centered"
        );
    }

    #[test]
    fn given_single_node_selected_when_align_then_returns_false() {
        let mut doc = DiagramDocument::default();
        let node_id = NodeId::new("single-node".to_string());

        doc.document.nodes = doc
            .document
            .nodes
            .update(node_id, make_node("single-node", 100.0, 100.0));
        let _ = doc
            .editor_state
            .selected_items
            .insert("single-node".to_string());

        let result =
            perform_align_selection(&mut doc, AlignmentAxis::Horizontal, AlignmentMode::Start);

        assert!(!result, "align should return false for single node");
    }

    #[test]
    fn given_empty_selection_when_align_then_returns_false() {
        let mut doc = DiagramDocument::default();
        let node_id = NodeId::new("node".to_string());

        doc.document.nodes = doc
            .document
            .nodes
            .update(node_id, make_node("node", 100.0, 100.0));

        let result =
            perform_align_selection(&mut doc, AlignmentAxis::Horizontal, AlignmentMode::Start);

        assert!(!result, "align should return false for empty selection");
    }

    #[test]
    fn given_locked_node_when_align_then_skips_locked() {
        let mut doc = DiagramDocument::default();
        let node_a = NodeId::new("node-a".to_string());
        let node_b = NodeId::new("node-b".to_string());
        let node_c = NodeId::new("node-c".to_string());

        let mut locked_node = make_node("node-b", 200.0, 100.0);
        locked_node.locked = true;

        doc.document.nodes = doc
            .document
            .nodes
            .update(node_a.clone(), make_node("node-a", 100.0, 100.0))
            .update(node_b.clone(), locked_node)
            .update(node_c.clone(), make_node("node-c", 150.0, 100.0));

        let _ = doc.editor_state.selected_items.insert("node-a".to_string());
        let _ = doc.editor_state.selected_items.insert("node-b".to_string());
        let _ = doc.editor_state.selected_items.insert("node-c".to_string());

        let result =
            perform_align_selection(&mut doc, AlignmentAxis::Horizontal, AlignmentMode::Start);

        assert!(result, "align should return true with 2 movable nodes");
        assert_eq!(
            doc.document.nodes.get(&node_a).map(|n| n.x.0),
            Some(100.0),
            "node-a should be aligned"
        );
        assert_eq!(
            doc.document.nodes.get(&node_b).map(|n| n.x.0),
            Some(200.0),
            "locked node-b should not move"
        );
        assert_eq!(
            doc.document.nodes.get(&node_c).map(|n| n.x.0),
            Some(100.0),
            "node-c should be aligned"
        );
    }

    #[test]
    fn given_selection_with_infinite_coords_when_align_then_returns_false() {
        let mut doc = DiagramDocument::default();
        let node_a = NodeId::new("node-a".to_string());
        let node_b = NodeId::new("node-b".to_string());

        let mut inf_node = make_node("node-b", 200.0, 100.0);
        inf_node.x = OrderedFloat(f64::INFINITY);

        doc.document.nodes = doc
            .document
            .nodes
            .update(node_a.clone(), make_node("node-a", 100.0, 100.0))
            .update(node_b.clone(), inf_node);

        let _ = doc.editor_state.selected_items.insert("node-a".to_string());
        let _ = doc.editor_state.selected_items.insert("node-b".to_string());

        let result =
            perform_align_selection(&mut doc, AlignmentAxis::Horizontal, AlignmentMode::Start);

        assert!(!result, "align should return false for non-finite coords");
    }

    #[test]
    fn given_alignment_when_successful_then_dimensions_unchanged() {
        let mut doc = DiagramDocument::default();
        let node_a = NodeId::new("node-a".to_string());
        let node_b = NodeId::new("node-b".to_string());

        doc.document.nodes = doc
            .document
            .nodes
            .update(node_a.clone(), make_node("node-a", 100.0, 100.0))
            .update(node_b.clone(), make_node("node-b", 200.0, 150.0));

        let _ = doc.editor_state.selected_items.insert("node-a".to_string());
        let _ = doc.editor_state.selected_items.insert("node-b".to_string());

        let width_before_a = doc.document.nodes.get(&node_a).map(|n| n.width.0);
        let height_before_a = doc.document.nodes.get(&node_a).map(|n| n.height.0);
        let width_before_b = doc.document.nodes.get(&node_b).map(|n| n.width.0);
        let height_before_b = doc.document.nodes.get(&node_b).map(|n| n.height.0);

        let _ = perform_align_selection(&mut doc, AlignmentAxis::Horizontal, AlignmentMode::Start);

        assert_eq!(
            doc.document.nodes.get(&node_a).map(|n| n.width.0),
            width_before_a
        );
        assert_eq!(
            doc.document.nodes.get(&node_a).map(|n| n.height.0),
            height_before_a
        );
        assert_eq!(
            doc.document.nodes.get(&node_b).map(|n| n.width.0),
            width_before_b
        );
        assert_eq!(
            doc.document.nodes.get(&node_b).map(|n| n.height.0),
            height_before_b
        );
    }

    #[test]
    fn given_alignment_when_successful_then_revision_incremented() {
        let mut doc = DiagramDocument::default();
        let node_a = NodeId::new("node-a".to_string());
        let node_b = NodeId::new("node-b".to_string());

        doc.document.nodes = doc
            .document
            .nodes
            .update(node_a.clone(), make_node("node-a", 100.0, 100.0))
            .update(node_b.clone(), make_node("node-b", 200.0, 150.0));

        let _ = doc.editor_state.selected_items.insert("node-a".to_string());
        let _ = doc.editor_state.selected_items.insert("node-b".to_string());

        let revision_before = doc.revision;

        let result =
            perform_align_selection(&mut doc, AlignmentAxis::Horizontal, AlignmentMode::Start);

        assert!(result);
        assert_eq!(
            doc.revision,
            revision_before.increment(),
            "revision should be incremented"
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

            prop_assert!(doc.editor_state.zoom.0.is_finite);
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

            prop_assert!(doc.editor_state.zoom.0.is_finite);
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

// =============================================================================
// Distribution tests (bd-51b)
// =============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod distribution_tests {
    use super::*;

    fn make_node_for_dist(label: &str, x: f64, y: f64, width: f64, height: f64) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: label.to_string(),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(width),
            height: OrderedFloat(height),
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

    fn make_doc_with_three_nodes_for_dist() -> DiagramDocument {
        let mut doc = DiagramDocument::default();
        let node_a = NodeId::new("node-a".to_string());
        let node_b = NodeId::new("node-b".to_string());
        let node_c = NodeId::new("node-c".to_string());

        let _ = doc
            .document
            .nodes
            .insert(node_a, make_node_for_dist("node-a", 0.0, 0.0, 100.0, 50.0));
        let _ = doc.document.nodes.insert(
            node_b,
            make_node_for_dist("node-b", 200.0, 0.0, 100.0, 50.0),
        );
        let _ = doc.document.nodes.insert(
            node_c,
            make_node_for_dist("node-c", 400.0, 0.0, 100.0, 50.0),
        );

        let _ = doc.editor_state.selected_items.insert("node-a".to_string());
        let _ = doc.editor_state.selected_items.insert("node-b".to_string());
        let _ = doc.editor_state.selected_items.insert("node-c".to_string());

        doc
    }

    /// Pure function for testing distribution logic
    fn perform_distribute(doc: &mut DiagramDocument, axis: DistributionAxis) -> bool {
        let selected_nodes: Vec<NodeId> = selected_node_ids(doc)
            .into_iter()
            .filter(|id| {
                doc.document.nodes.get(id).is_some_and(|node| {
                    let coords_finite = node.x.0.is_finite() && node.y.0.is_finite();
                    let movable = !node.locked || node.kind == NodeKind::Subgraph;
                    coords_finite && movable
                })
            })
            .collect();

        if selected_nodes.len() < 3 {
            return false;
        }

        let mut node_data: Vec<(NodeId, f64, f64)> = selected_nodes
            .iter()
            .filter_map(|id| {
                doc.document.nodes.get(id).map(|node| {
                    let (pos, size) = match axis {
                        DistributionAxis::Horizontal => (node.x.0, node.width.0),
                        DistributionAxis::Vertical => (node.y.0, node.height.0),
                    };
                    (id.clone(), pos, size)
                })
            })
            .collect();

        if node_data
            .iter()
            .any(|(_, pos, size)| !pos.is_finite() || !size.is_finite())
        {
            return false;
        }

        node_data.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let first_pos = node_data.first().map(|(_, p, _)| *p);
        let last_node_end = node_data.last().map(|(_, p, s)| p + s);

        let Some(first_pos) = first_pos else {
            return false;
        };
        let Some(last_node_end) = last_node_end else {
            return false;
        };

        if !first_pos.is_finite() || !last_node_end.is_finite() {
            return false;
        }

        let total_node_size: f64 = node_data.iter().map(|(_, _, s)| *s).sum();
        let total_extent = last_node_end - first_pos;

        if total_extent <= f64::EPSILON {
            return false;
        }

        let node_count = node_data.len();
        let gap_count = node_count.saturating_sub(1);
        let available_space = total_extent - total_node_size;
        let spacing = if gap_count > 0 {
            available_space / f64::from(u32::try_from(gap_count).unwrap_or(1))
        } else {
            0.0
        };

        if !spacing.is_finite() {
            return false;
        }

        let mut current_pos = first_pos;
        for (node_id, _, node_size) in &node_data {
            if let Some(node) = doc.document.nodes.get_mut(node_id) {
                if node.locked && node.kind != NodeKind::Subgraph {
                    continue;
                }

                match axis {
                    DistributionAxis::Horizontal => {
                        node.x = OrderedFloat(current_pos);
                    }
                    DistributionAxis::Vertical => {
                        node.y = OrderedFloat(current_pos);
                    }
                }
                current_pos += node_size + spacing;
            }
        }
        doc.revision = doc.revision.increment();
        true
    }

    #[test]
    fn test_distribute_horizontal_three_nodes() {
        let mut doc = make_doc_with_three_nodes_for_dist();
        let initial_revision = doc.revision;

        let result = perform_distribute(&mut doc, DistributionAxis::Horizontal);

        assert!(result, "distribute should return true for 3 nodes");
        assert_eq!(
            doc.revision,
            initial_revision.increment(),
            "revision should be incremented"
        );

        let node_a = doc
            .document
            .nodes
            .get(&NodeId::new("node-a".to_string()))
            .expect("node-a");
        let node_b = doc
            .document
            .nodes
            .get(&NodeId::new("node-b".to_string()))
            .expect("node-b");
        let node_c = doc
            .document
            .nodes
            .get(&NodeId::new("node-c".to_string()))
            .expect("node-c");

        assert_eq!(node_a.x.0, 0.0, "node-a x should be 0");
        assert_eq!(node_c.x.0, 400.0, "node-c x should be 400");
        assert_eq!(
            node_b.x.0, 200.0,
            "node-b x should be 200 for equal spacing"
        );
    }

    #[test]
    fn test_distribute_vertical_three_nodes() {
        let mut doc = DiagramDocument::default();
        let node_a = NodeId::new("node-a".to_string());
        let node_b = NodeId::new("node-b".to_string());
        let node_c = NodeId::new("node-c".to_string());

        let _ = doc
            .document
            .nodes
            .insert(node_a, make_node_for_dist("node-a", 0.0, 0.0, 100.0, 50.0));
        let _ = doc.document.nodes.insert(
            node_b,
            make_node_for_dist("node-b", 0.0, 200.0, 100.0, 50.0),
        );
        let _ = doc.document.nodes.insert(
            node_c,
            make_node_for_dist("node-c", 0.0, 400.0, 100.0, 50.0),
        );

        let _ = doc.editor_state.selected_items.insert("node-a".to_string());
        let _ = doc.editor_state.selected_items.insert("node-b".to_string());
        let _ = doc.editor_state.selected_items.insert("node-c".to_string());

        let result = perform_distribute(&mut doc, DistributionAxis::Vertical);

        assert!(result, "distribute vertical should return true");

        let node_a = doc
            .document
            .nodes
            .get(&NodeId::new("node-a".to_string()))
            .expect("node-a");
        let node_b = doc
            .document
            .nodes
            .get(&NodeId::new("node-b".to_string()))
            .expect("node-b");
        let node_c = doc
            .document
            .nodes
            .get(&NodeId::new("node-c".to_string()))
            .expect("node-c");

        assert_eq!(node_a.y.0, 0.0, "node-a y should be 0");
        assert_eq!(node_c.y.0, 400.0, "node-c y should be 400");
        assert_eq!(
            node_b.y.0, 200.0,
            "node-b y should be 200 for equal spacing"
        );
    }

    #[test]
    fn test_distribute_horizontal_preserves_y() {
        let mut doc = make_doc_with_three_nodes_for_dist();

        if let Some(node) = doc
            .document
            .nodes
            .get_mut(&NodeId::new("node-a".to_string()))
        {
            node.y = OrderedFloat(100.0);
        }
        if let Some(node) = doc
            .document
            .nodes
            .get_mut(&NodeId::new("node-b".to_string()))
        {
            node.y = OrderedFloat(200.0);
        }
        if let Some(node) = doc
            .document
            .nodes
            .get_mut(&NodeId::new("node-c".to_string()))
        {
            node.y = OrderedFloat(300.0);
        }

        let result = perform_distribute(&mut doc, DistributionAxis::Horizontal);
        assert!(result);

        let node_a = doc
            .document
            .nodes
            .get(&NodeId::new("node-a".to_string()))
            .expect("node-a");
        let node_b = doc
            .document
            .nodes
            .get(&NodeId::new("node-b".to_string()))
            .expect("node-b");
        let node_c = doc
            .document
            .nodes
            .get(&NodeId::new("node-c".to_string()))
            .expect("node-c");

        assert_eq!(node_a.y.0, 100.0, "node-a y should be unchanged");
        assert_eq!(node_b.y.0, 200.0, "node-b y should be unchanged");
        assert_eq!(node_c.y.0, 300.0, "node-c y should be unchanged");
    }

    #[test]
    fn test_distribute_vertical_preserves_x() {
        let mut doc = DiagramDocument::default();
        let node_a = NodeId::new("node-a".to_string());
        let node_b = NodeId::new("node-b".to_string());
        let node_c = NodeId::new("node-c".to_string());

        let _ = doc.document.nodes.insert(
            node_a.clone(),
            make_node_for_dist("node-a", 50.0, 0.0, 100.0, 50.0),
        );
        let _ = doc.document.nodes.insert(
            node_b.clone(),
            make_node_for_dist("node-b", 150.0, 200.0, 100.0, 50.0),
        );
        let _ = doc.document.nodes.insert(
            node_c.clone(),
            make_node_for_dist("node-c", 250.0, 400.0, 100.0, 50.0),
        );

        let _ = doc.editor_state.selected_items.insert("node-a".to_string());
        let _ = doc.editor_state.selected_items.insert("node-b".to_string());
        let _ = doc.editor_state.selected_items.insert("node-c".to_string());

        let result = perform_distribute(&mut doc, DistributionAxis::Vertical);
        assert!(result);

        let node_a = doc
            .document
            .nodes
            .get(&NodeId::new("node-a".to_string()))
            .expect("node-a");
        let node_b = doc
            .document
            .nodes
            .get(&NodeId::new("node-b".to_string()))
            .expect("node-b");
        let node_c = doc
            .document
            .nodes
            .get(&NodeId::new("node-c".to_string()))
            .expect("node-c");

        assert_eq!(node_a.x.0, 50.0, "node-a x should be unchanged");
        assert_eq!(node_b.x.0, 150.0, "node-b x should be unchanged");
        assert_eq!(node_c.x.0, 250.0, "node-c x should be unchanged");
    }

    #[test]
    fn test_distribute_less_than_three_nodes_returns_false() {
        let mut doc = DiagramDocument::default();
        let node_a = NodeId::new("node-a".to_string());
        let node_b = NodeId::new("node-b".to_string());

        let _ = doc
            .document
            .nodes
            .insert(node_a, make_node_for_dist("node-a", 0.0, 0.0, 100.0, 50.0));
        let _ = doc.document.nodes.insert(
            node_b,
            make_node_for_dist("node-b", 200.0, 0.0, 100.0, 50.0),
        );

        let _ = doc.editor_state.selected_items.insert("node-a".to_string());
        let _ = doc.editor_state.selected_items.insert("node-b".to_string());

        let result = perform_distribute(&mut doc, DistributionAxis::Horizontal);
        assert!(!result, "distribute should return false for 2 nodes");

        doc.editor_state.selected_items.clear();
        let _ = doc.editor_state.selected_items.insert("node-a".to_string());
        let result = perform_distribute(&mut doc, DistributionAxis::Horizontal);
        assert!(!result, "distribute should return false for 1 node");

        doc.editor_state.selected_items.clear();
        let result = perform_distribute(&mut doc, DistributionAxis::Horizontal);
        assert!(!result, "distribute should return false for 0 nodes");
    }

    #[test]
    fn test_distribute_outermost_nodes_at_bounds() {
        let mut doc = make_doc_with_three_nodes_for_dist();

        if let Some(node) = doc
            .document
            .nodes
            .get_mut(&NodeId::new("node-b".to_string()))
        {
            node.x = OrderedFloat(350.0);
        }

        let result = perform_distribute(&mut doc, DistributionAxis::Horizontal);
        assert!(result);

        let node_a = doc
            .document
            .nodes
            .get(&NodeId::new("node-a".to_string()))
            .expect("node-a");
        let node_c = doc
            .document
            .nodes
            .get(&NodeId::new("node-c".to_string()))
            .expect("node-c");

        assert_eq!(node_a.x.0, 0.0, "leftmost node should stay at min bound");
        assert_eq!(node_c.x.0, 400.0, "rightmost node should stay at max bound");
    }

    #[test]
    fn test_distribute_equal_spacing() {
        let mut doc = DiagramDocument::default();

        let node_a = NodeId::new("node-a".to_string());
        let node_b = NodeId::new("node-b".to_string());
        let node_c = NodeId::new("node-c".to_string());
        let node_d = NodeId::new("node-d".to_string());

        let _ = doc.document.nodes.insert(
            node_a.clone(),
            make_node_for_dist("node-a", 0.0, 0.0, 100.0, 50.0),
        );
        let _ = doc.document.nodes.insert(
            node_b.clone(),
            make_node_for_dist("node-b", 50.0, 0.0, 100.0, 50.0),
        );
        let _ = doc.document.nodes.insert(
            node_c.clone(),
            make_node_for_dist("node-c", 400.0, 0.0, 100.0, 50.0),
        );
        let _ = doc.document.nodes.insert(
            node_d.clone(),
            make_node_for_dist("node-d", 600.0, 0.0, 100.0, 50.0),
        );

        for id in ["node-a", "node-b", "node-c", "node-d"] {
            let _ = doc.editor_state.selected_items.insert(id.to_string());
        }

        let result = perform_distribute(&mut doc, DistributionAxis::Horizontal);
        assert!(result);

        let nodes: Vec<_> = ["node-a", "node-b", "node-c", "node-d"]
            .iter()
            .map(|id| {
                doc.document
                    .nodes
                    .get(&NodeId::new(id.to_string()))
                    .cloned()
            })
            .collect();

        let node0 = nodes[0].as_ref().expect("node-a should exist");
        let node1 = nodes[1].as_ref().expect("node-b should exist");
        let node2 = nodes[2].as_ref().expect("node-c should exist");
        let node3 = nodes[3].as_ref().expect("node-d should exist");

        let gap_ab = node1.x.0 - (node0.x.0 + node0.width.0);
        let gap_bc = node2.x.0 - (node1.x.0 + node1.width.0);
        let gap_cd = node3.x.0 - (node2.x.0 + node2.width.0);

        assert!(
            (gap_ab - gap_bc).abs() < f64::EPSILON,
            "gaps ab and bc should be equal: {} vs {}",
            gap_ab,
            gap_bc
        );
        assert!(
            (gap_bc - gap_cd).abs() < f64::EPSILON,
            "gaps bc and cd should be equal: {} vs {}",
            gap_bc,
            gap_cd
        );
    }

    #[test]
    fn test_distribute_preserves_node_size() {
        let mut doc = make_doc_with_three_nodes_for_dist();

        let widths_before: Vec<_> = ["node-a", "node-b", "node-c"]
            .iter()
            .map(|id| {
                doc.document
                    .nodes
                    .get(&NodeId::new(id.to_string()))
                    .map(|n| n.width.0)
            })
            .collect();

        let heights_before: Vec<_> = ["node-a", "node-b", "node-c"]
            .iter()
            .map(|id| {
                doc.document
                    .nodes
                    .get(&NodeId::new(id.to_string()))
                    .map(|n| n.height.0)
            })
            .collect();

        let result = perform_distribute(&mut doc, DistributionAxis::Horizontal);
        assert!(result);

        let widths_after: Vec<_> = ["node-a", "node-b", "node-c"]
            .iter()
            .map(|id| {
                doc.document
                    .nodes
                    .get(&NodeId::new(id.to_string()))
                    .map(|n| n.width.0)
            })
            .collect();

        let heights_after: Vec<_> = ["node-a", "node-b", "node-c"]
            .iter()
            .map(|id| {
                doc.document
                    .nodes
                    .get(&NodeId::new(id.to_string()))
                    .map(|n| n.height.0)
            })
            .collect();

        for (before, after) in widths_before.iter().zip(widths_after.iter()) {
            assert_eq!(before, after, "width should be preserved");
        }
        for (before, after) in heights_before.iter().zip(heights_after.iter()) {
            assert_eq!(before, after, "height should be preserved");
        }
    }

    #[test]
    fn test_distribute_locked_nodes_skipped() {
        let mut doc = DiagramDocument::default();

        let node_a = NodeId::new("node-a".to_string());
        let node_b = NodeId::new("node-b".to_string());
        let node_c = NodeId::new("node-c".to_string());
        let node_d = NodeId::new("node-d".to_string());

        let _ = doc.document.nodes.insert(
            node_a.clone(),
            make_node_for_dist("node-a", 0.0, 0.0, 100.0, 50.0),
        );
        let _ = doc.document.nodes.insert(
            node_b.clone(),
            make_node_for_dist("node-b", 50.0, 0.0, 100.0, 50.0),
        );
        let _ = doc.document.nodes.insert(
            node_c.clone(),
            make_node_for_dist("node-c", 400.0, 0.0, 100.0, 50.0),
        );
        let _ = doc.document.nodes.insert(
            node_d.clone(),
            make_node_for_dist("node-d", 600.0, 0.0, 100.0, 50.0),
        );

        if let Some(node) = doc.document.nodes.get_mut(&node_b) {
            node.locked = true;
        }

        for id in ["node-a", "node-b", "node-c", "node-d"] {
            let _ = doc.editor_state.selected_items.insert(id.to_string());
        }

        let original_b_x = doc
            .document
            .nodes
            .get(&NodeId::new("node-b".to_string()))
            .map(|n| n.x.0);

        let result = perform_distribute(&mut doc, DistributionAxis::Horizontal);
        assert!(
            result,
            "distribute should return true with 3+ movable nodes"
        );

        let new_b_x = doc
            .document
            .nodes
            .get(&NodeId::new("node-b".to_string()))
            .map(|n| n.x.0);
        assert_eq!(original_b_x, new_b_x, "locked node should not move");
    }

    #[test]
    fn test_distribute_updates_revision() {
        let mut doc = make_doc_with_three_nodes_for_dist();
        let revision_before = doc.revision;

        let result = perform_distribute(&mut doc, DistributionAxis::Horizontal);
        assert!(result);

        assert_eq!(
            doc.revision,
            revision_before.increment(),
            "revision should be incremented"
        );
    }
}
