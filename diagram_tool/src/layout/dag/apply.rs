use diagram_models::document::{Node, NodeId, OrderedFloat};
use im::HashMap;

/// Compute the new node position:
/// - If it has an entry in `new_positions` → use that (unlocked root node).
/// - Else if it is a child → apply parent delta if parent was moved.
/// - Else (locked root node) → leave unchanged.
pub fn apply_position(
    id: &NodeId,
    node: &Node,
    new_positions: &HashMap<NodeId, (f64, f64)>,
    deltas: &HashMap<NodeId, (f64, f64)>,
    all_nodes: &HashMap<NodeId, Node>,
) -> Node {
    if let Some(&(nx, ny)) = new_positions.get(id) {
        return Node {
            x: OrderedFloat(nx),
            y: OrderedFloat(ny),
            ..node.clone()
        };
    }

    let Some(pid) = node.parent.as_ref() else {
        return node.clone(); // locked root → unchanged
    };

    let inherited_delta = std::iter::successors(Some(pid.clone()), |parent_id| {
        all_nodes
            .get(parent_id)
            .and_then(|parent| parent.parent.clone())
    })
    .take(all_nodes.len())
    .fold(None, |acc: Option<(f64, f64)>, parent_id| {
        deltas.get(&parent_id).map_or(acc, |&(dx, dy)| {
            Some(match acc {
                Some((adx, ady)) => (adx + dx, ady + dy),
                None => (dx, dy),
            })
        })
    });

    let Some((dx, dy)) = inherited_delta else {
        return node.clone(); // parent chain not moved → unchanged
    };

    Node {
        x: OrderedFloat(node.x.0 + dx),
        y: OrderedFloat(node.y.0 + dy),
        ..node.clone()
    }
}
