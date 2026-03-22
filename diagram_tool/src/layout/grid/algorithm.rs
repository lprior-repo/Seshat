//! Grid layout algorithm.

use diagram_models::document::{DiagramDocument, NodeId, OrderedFloat};
use im::HashMap;
use itertools::Itertools;

/// Pure calculation to determine grid layout.
/// Returns a new document with updated positions for unlocked nodes.
///
/// # Panics
/// Panics if `cell_size` is not positive or is not finite.
#[must_use]
pub fn calculate_grid_layout(doc: &DiagramDocument, cell_size: f64) -> DiagramDocument {
    debug_assert!(
        cell_size.is_finite() && cell_size > 0.0,
        "cell_size must be positive and finite, got {cell_size}",
    );

    let occupied_cells = doc
        .document
        .nodes
        .values()
        .filter(|node| node.lock_state.is_locked() && node.parent.is_none())
        .map(|node| {
            (
                (node.x.0 / cell_size).round() as i32,
                (node.y.0 / cell_size).round() as i32,
            )
        })
        .collect::<im::HashSet<(i32, i32)>>();

    let unlocked_iter = doc
        .document
        .nodes
        .iter()
        .filter(|(_, n)| !n.lock_state.is_locked() && n.parent.is_none())
        .map(|(id, _)| id.clone())
        .sorted();

    // Optimization 1: Use fold to compute both count and first accumulation,
    // then chain second fold directly without intermediate collect.
    let (sorted_ids, initial_acc) = unlocked_iter.fold(
        (Vec::new(), (occupied_cells, 0_i32, HashMap::new())),
        |(mut ids, (occupied, cursor, pos_map)), id| {
            ids.push(id);
            (ids, (occupied, cursor, pos_map))
        },
    );

    // Early return if no unlocked nodes — avoids the doc.clone() at the end
    if sorted_ids.is_empty() {
        return doc.clone();
    }

    #[allow(clippy::cast_precision_loss)]
    let cols_target = (sorted_ids.len() as f64).sqrt().ceil() as i32;
    let cols = cols_target.max(1);

    let next_free_cell = |occupied: &im::HashSet<(i32, i32)>, start_index: i32| {
        let max_rows_candidate = sorted_ids
            .len()
            .saturating_add(occupied.len())
            .saturating_add(1);
        let max_rows = i32::try_from(max_rows_candidate).unwrap_or(i32::MAX);
        let search_limit = (cols * max_rows).max(cols);

        (0..search_limit)
            .map(|step| start_index + step)
            .map(|index| (index.rem_euclid(cols), index.div_euclid(cols)))
            .find(|cell| !occupied.contains(cell))
            .unwrap_or_else(|| (start_index.rem_euclid(cols), start_index.div_euclid(cols)))
    };

    let (_, _, positions) =
        sorted_ids
            .iter()
            .fold(initial_acc, |(mut occupied, cursor, pos_map), id| {
                let (new_col, new_row) = next_free_cell(&occupied, cursor);

                let new_pos = (
                    f64::from(new_col) * cell_size,
                    f64::from(new_row) * cell_size,
                );
                let next_pos_map = pos_map.update(id.clone(), new_pos);
                let _ = occupied.insert((new_col, new_row));
                let next_cursor = new_row * cols + new_col + 1;

                (occupied, next_cursor, next_pos_map)
            });

    // Optimization 2 & 3: Compute deltas inline during traversal instead of
    // creating intermediate HashMap, and use into_iter to consume positions
    // directly without cloning NodeIds.
    //
    // First chain: nodes that have new positions (consume positions via into_iter)
    let positioned_nodes: HashMap<NodeId, _> = positions
        .into_iter()
        .filter_map(|(id, (nx, ny))| {
            doc.document.nodes.get(&id).map(|node| {
                let mut next_node = node.clone();
                next_node.x = OrderedFloat(nx);
                next_node.y = OrderedFloat(ny);
                (id, next_node)
            })
        })
        .collect();

    // Second chain: nodes without new positions, compute deltas inline
    let unpositioned_nodes: HashMap<NodeId, _> = doc
        .document
        .nodes
        .iter()
        .filter(|(id, _)| !positioned_nodes.contains_key(id))
        .map(|(id, node)| {
            let next_node = node.parent.as_ref().map_or_else(
                || node.clone(),
                |pid| {
                    // Compute accumulated parent delta inline using positioned_nodes
                    let (dx, dy) = std::iter::successors(Some(pid.clone()), |pid| {
                        doc.document.nodes.get(pid).and_then(|n| n.parent.clone())
                    })
                    .take_while(|pid| positioned_nodes.contains_key(pid) || *pid != pid.clone())
                    .filter(|pid| positioned_nodes.contains_key(pid))
                    .fold((0.0, 0.0), |(adx, ady), pid| {
                        positioned_nodes.get(&pid).map_or((adx, ady), |pn| {
                            let px = pn.x.0;
                            let py = pn.y.0;
                            doc.document
                                .nodes
                                .get(&pid)
                                .map_or((adx, ady), |n| (adx + px - n.x.0, ady + py - n.y.0))
                        })
                    });
                    if dx != 0.0 || dy != 0.0 {
                        let mut next = node.clone();
                        next.x = OrderedFloat(node.x.0 + dx);
                        next.y = OrderedFloat(node.y.0 + dy);
                        next
                    } else {
                        node.clone()
                    }
                },
            );
            (id.clone(), next_node)
        })
        .collect();

    let next_nodes = positioned_nodes
        .into_iter()
        .chain(unpositioned_nodes)
        .collect();

    let mut next_doc = doc.clone();
    next_doc.document.nodes = next_nodes;
    next_doc
}

#[cfg(test)]
#[path = "algorithm_tests.rs"]
mod tests;
