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
                    deltas.get(pid).map_or_else(
                        || (id.clone(), node.clone()),
                        |&(dx, dy)| {
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
