//! Z-order operations - bring forward, send backward, bring to front, send to back

use dioxus::prelude::*;

use crate::history::History;
use diagram_models::document::DiagramDocument;

/// Z-order operation types
#[derive(Clone, Copy)]
pub enum ZOrderOp {
    BringForward,
    SendBackward,
    BringToFront,
    SendToBack,
}

/// Applies the given z-order operation to the selected nodes.
fn apply_z_order_operation(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    op: ZOrderOp,
) -> bool {
    let current = doc_signal.read().clone();

    // Collect into Vec instead of BTreeSet - more efficient since we don't need set operations
    let selected: Vec<_> = current
        .editor_state
        .selected_items
        .iter()
        .filter_map(|id| diagram_models::document::NodeId::new(id.clone()).into())
        .collect();

    if selected.is_empty() {
        return false;
    }

    // Get current max z-index for bring-to-front/send-to-back operations
    let (max_z, min_z) = current
        .document
        .nodes
        .iter()
        .fold((i64::MIN, i64::MAX), |(max_z, min_z), (_, node)| {
            (max_z.max(node.z_index), min_z.min(node.z_index))
        });

    let mut changed = false;
    let mut next = current.clone();

    // Apply operation to each selected node
    for node_id in &selected {
        if let Some(node) = next.document.nodes.get_mut(node_id) {
            let new_z = match op {
                ZOrderOp::BringForward => node.z_index + 1,
                ZOrderOp::SendBackward => node.z_index.saturating_sub(1),
                ZOrderOp::BringToFront => max_z + 1,
                ZOrderOp::SendToBack => min_z.saturating_sub(1),
            };
            if new_z != node.z_index {
                node.z_index = new_z;
                changed = true;
            }
        }
    }

    if !changed {
        return false;
    }

    next.revision = next.revision.increment();
    let history = history_signal.read().clone();
    *history_signal.write() = history.push(current);
    *doc_signal.write() = next;
    true
}

/// Bring selected nodes forward by one z-index level
#[must_use]
pub fn apply_bring_forward(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
) -> bool {
    apply_z_order_operation(doc_signal, history_signal, ZOrderOp::BringForward)
}

/// Send selected nodes backward by one z-index level
#[must_use]
pub fn apply_send_backward(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
) -> bool {
    apply_z_order_operation(doc_signal, history_signal, ZOrderOp::SendBackward)
}

/// Bring selected nodes to the front (highest z-index)
#[must_use]
pub fn apply_bring_to_front(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
) -> bool {
    apply_z_order_operation(doc_signal, history_signal, ZOrderOp::BringToFront)
}

/// Send selected nodes to the back (lowest z-index)
#[must_use]
pub fn apply_send_to_back(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
) -> bool {
    apply_z_order_operation(doc_signal, history_signal, ZOrderOp::SendToBack)
}
