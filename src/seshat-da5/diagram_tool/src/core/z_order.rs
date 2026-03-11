use crate::models::document::{DiagramDocument, NodeId, NodeKind};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZOrderOp {
    BringForward,
    SendBackward,
    BringToFront,
    SendToBack,
}

fn selected_node_ids(doc: &DiagramDocument) -> BTreeSet<NodeId> {
    doc.editor_state
        .selected_items
        .iter()
        .map(|id| NodeId::new(id.clone()))
        .filter(|id| doc.document.nodes.contains_key(id))
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

pub fn apply_z_order_to_ids(ids: &mut Vec<NodeId>, selected: &BTreeSet<NodeId>, op: ZOrderOp) {
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

pub fn apply_z_order_operation(doc: &mut DiagramDocument, op: ZOrderOp) -> bool {
    let selected = selected_node_ids(doc)
        .into_iter()
        .filter(|id| {
            doc.document
                .nodes
                .get(id)
                .is_some_and(|node| !node.locked || node.kind == NodeKind::Subgraph)
        })
        .collect::<BTreeSet<_>>();

    if selected.is_empty() {
        return false;
    }

    let mut changed = false;

    for is_subgraph_layer in [false, true] {
        let ordered = ordered_layer_node_ids(doc, is_subgraph_layer);
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
            .filter_map(|id| doc.document.nodes.get(id).map(|node| node.z_index))
            .min()
            .unwrap_or(0);

        for (idx, id) in reordered.iter().enumerate() {
            if let Some(node) = doc.document.nodes.get_mut(id) {
                node.z_index = min_z + i64::try_from(idx).unwrap_or(min_z);
            }
        }

        changed = true;
    }

    changed
}

pub fn bring_forward(doc: &mut DiagramDocument) -> bool {
    apply_z_order_operation(doc, ZOrderOp::BringForward)
}

pub fn send_backward(doc: &mut DiagramDocument) -> bool {
    apply_z_order_operation(doc, ZOrderOp::SendBackward)
}

pub fn bring_to_front(doc: &mut DiagramDocument) -> bool {
    apply_z_order_operation(doc, ZOrderOp::BringToFront)
}

pub fn send_to_back(doc: &mut DiagramDocument) -> bool {
    apply_z_order_operation(doc, ZOrderOp::SendToBack)
}
