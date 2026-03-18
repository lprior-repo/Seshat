//! Grid layout algorithm.

use diagram_models::document::{DiagramDocument, NodeId, OrderedFloat};
use im::HashMap;
use itertools::Itertools;

pub(crate) fn accumulated_parent_delta(
    parent_id: &NodeId,
    deltas: &HashMap<NodeId, (f64, f64)>,
    nodes: &HashMap<NodeId, diagram_models::document::Node>,
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

    let unlocked_ids = doc
        .document
        .nodes
        .iter()
        .filter(|(_, n)| !n.lock_state.is_locked() && n.parent.is_none())
        .map(|(id, _)| id.clone())
        .sorted()
        .collect::<Vec<NodeId>>();

    if unlocked_ids.is_empty() {
        return doc.clone();
    }

    #[allow(clippy::cast_precision_loss)]
    let cols_target = (unlocked_ids.len() as f64).sqrt().ceil() as i32;
    let cols = cols_target.max(1);

    let next_free_cell = |occupied: &im::HashSet<(i32, i32)>, start_index: i32| {
        let max_rows_candidate = unlocked_ids
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

    let (_, _, positions) = unlocked_ids.iter().fold(
        (occupied_cells, 0_i32, HashMap::new()),
        |(mut occupied, cursor, pos_map), id| {
            let (new_col, new_row) = next_free_cell(&occupied, cursor);

            let new_pos = (
                f64::from(new_col) * cell_size,
                f64::from(new_row) * cell_size,
            );
            let next_pos_map = pos_map.update(id.clone(), new_pos);
            let _ = occupied.insert((new_col, new_row));
            let next_cursor = new_row * cols + new_col + 1;

            (occupied, next_cursor, next_pos_map)
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
                    if let Some((dx, dy)) =
                        accumulated_parent_delta(pid, &deltas, &doc.document.nodes)
                    {
                        let mut next_node = node.clone();
                        next_node.x = OrderedFloat(node.x.0 + dx);
                        next_node.y = OrderedFloat(node.y.0 + dy);
                        (id.clone(), next_node)
                    } else {
                        (id.clone(), node.clone())
                    }
                },
            ),
        })
        .collect::<HashMap<NodeId, _>>();

    let mut next_doc = doc.clone();
    next_doc.document.nodes = next_nodes;
    next_doc
}
