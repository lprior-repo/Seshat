use diagram_models::document::{DiagramDocument, NodeId, NodeKind};
use diagram_models::z_order::{apply_z_order_reorder, ZOrderOp};
use std::collections::BTreeSet;

fn selected_node_ids(doc: &DiagramDocument) -> BTreeSet<NodeId> {
    doc.editor_state
        .selected_items
        .iter()
        .map(|id| diagram_models::document::NodeId::new(id.clone()))
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
                (na.z_index, a.as_str()).cmp(&(nb.z_index, b.as_str()))
            })
    });

    node_ids
}

pub fn apply_z_order_operation(doc: &mut DiagramDocument, op: ZOrderOp) -> bool {
    let selected = selected_node_ids(doc)
        .into_iter()
        .filter(|id| {
            doc.document
                .nodes
                .get(id)
                .is_some_and(|node| node.lock_state.is_movable(&node.kind))
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
        apply_z_order_reorder(&mut reordered, &selected, op);
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

#[cfg(test)]
#[path = "z_order_tests.rs"]
mod tests;
