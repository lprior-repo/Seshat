#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::cast_possible_truncation)]
#![forbid(unsafe_code)]

use crate::models::document::{DiagramDocument, NodeId, OrderedFloat};
use im::HashMap;
use itertools::Itertools;

fn accumulated_parent_delta(
    parent_id: &NodeId,
    deltas: &HashMap<NodeId, (f64, f64)>,
    nodes: &HashMap<NodeId, crate::models::document::Node>,
) -> Option<(f64, f64)> {
    std::iter::successors(Some(parent_id.clone()), |id| {
        nodes.get(id).and_then(|node| node.parent.clone())
    })
    .take(nodes.len())
    .fold(None, |acc: Option<(f64, f64)>, id| {
        deltas.get(&id).map_or(acc, |&(dx, dy)| {
            Some(match acc {
                Some((adx, ady)) => (adx + dx, ady + dy),
                None => (dx, dy),
            })
        })
    })
}

/// Pure calculation to determine grid layout.
/// Returns a new document with updated positions for unlocked nodes.
#[must_use]
pub fn calculate_grid_layout(doc: &DiagramDocument, cell_size: f64) -> DiagramDocument {
    let occupied_cells = doc
        .document
        .nodes
        .values()
        .filter(|node| node.locked && node.parent.is_none())
        .map(|node| {
            (
                (node.x.0 / cell_size).round() as i32,
                (node.y.0 / cell_size).round() as i32,
            )
        })
        .collect::<im::HashSet<(i32, i32)>>();

    let unlocked_ids = doc
        .document
        .nodes
        .iter()
        .filter(|(_, n)| !n.locked && n.parent.is_none())
        .map(|(id, _)| id.clone())
        .sorted()
        .collect::<Vec<NodeId>>();

    if unlocked_ids.is_empty() {
        return doc.clone();
    }

    #[allow(clippy::cast_precision_loss)]
    let cols_target = (unlocked_ids.len() as f64).sqrt().ceil() as i32;

    let (_, _, _, positions) = unlocked_ids.iter().fold(
        (occupied_cells, 0_i32, 0_i32, HashMap::new()),
        |(mut occupied, mut col, mut row, pos_map), id| {
            let (new_col, new_row) = (0..)
                .find_map(|_| {
                    if occupied.contains(&(col, row)) {
                        col += 1;
                        if col >= cols_target.max(1) {
                            col = 0;
                            row += 1;
                        }
                        None
                    } else {
                        Some((col, row))
                    }
                })
                .map_or((0, 0), |p| p);

            let new_pos = (
                f64::from(new_col) * cell_size,
                f64::from(new_row) * cell_size,
            );
            let next_pos_map = pos_map.update(id.clone(), new_pos);
            let _ = occupied.insert((new_col, new_row));

            (occupied, new_col + 1, new_row, next_pos_map)
        },
    );

    let deltas: HashMap<NodeId, (f64, f64)> = positions
        .iter()
        .filter_map(|(id, (nx, ny))| {
            doc.document
                .nodes
                .get(id)
                .map(|node| (id.clone(), (nx - node.x.0, ny - node.y.0)))
        })
        .collect();

    let next_nodes = doc
        .document
        .nodes
        .iter()
        .map(|(id, node)| match positions.get(id) {
            Some(&(nx, ny)) => {
                let mut next_node = node.clone();
                next_node.x = OrderedFloat(nx);
                next_node.y = OrderedFloat(ny);
                (id.clone(), next_node)
            }
            None => node.parent.as_ref().map_or_else(
                || (id.clone(), node.clone()),
                |pid| {
                    accumulated_parent_delta(pid, &deltas, &doc.document.nodes).map_or_else(
                        || (id.clone(), node.clone()),
                        |(dx, dy)| {
                            let mut next_node = node.clone();
                            next_node.x = OrderedFloat(node.x.0 + dx);
                            next_node.y = OrderedFloat(node.y.0 + dy);
                            (id.clone(), next_node)
                        },
                    )
                },
            ),
        })
        .collect::<HashMap<NodeId, _>>();

    let mut next_doc = doc.clone();
    next_doc.document.nodes = next_nodes;
    next_doc
}

#[cfg(test)]
mod tests {
    use super::calculate_grid_layout;
    use crate::models::document::{
        DiagramDocument, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
    };
    use im::HashMap;

    fn node(x: f64, y: f64, locked: bool, parent: Option<NodeId>) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: String::new(),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(100.0),
            height: OrderedFloat(60.0),
            font_size: None,
            font_weight: None,
            locked,
            parent,
            dag_rank: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        }
    }

    #[test]
    fn given_nested_children_when_grid_layout_moves_root_then_descendants_follow() {
        let root = NodeId::new(String::from("root"));
        let child = NodeId::new(String::from("child"));
        let grandchild = NodeId::new(String::from("grandchild"));

        let mut doc = DiagramDocument::default();
        doc.document.nodes = HashMap::new()
            .update(root.clone(), node(40.0, 40.0, false, None))
            .update(child.clone(), node(50.0, 50.0, true, Some(root.clone())))
            .update(
                grandchild.clone(),
                node(60.0, 60.0, true, Some(child.clone())),
            );

        let next = calculate_grid_layout(&doc, 100.0);

        let root_before = doc
            .document
            .nodes
            .get(&root)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
        let root_after = next
            .document
            .nodes
            .get(&root)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
        let delta = (root_after.0 - root_before.0, root_after.1 - root_before.1);

        let child_before = doc
            .document
            .nodes
            .get(&child)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
        let child_after = next
            .document
            .nodes
            .get(&child)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
        let grand_before = doc
            .document
            .nodes
            .get(&grandchild)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
        let grand_after = next
            .document
            .nodes
            .get(&grandchild)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));

        assert!((child_after.0 - (child_before.0 + delta.0)).abs() < f64::EPSILON);
        assert!((child_after.1 - (child_before.1 + delta.1)).abs() < f64::EPSILON);
        assert!((grand_after.0 - (grand_before.0 + delta.0)).abs() < f64::EPSILON);
        assert!((grand_after.1 - (grand_before.1 + delta.1)).abs() < f64::EPSILON);
    }
}
