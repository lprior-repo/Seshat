use crate::models::document::{DiagramDocument, NodeId};
use crate::ui::grid::{snap_point as grid_snap_point, GridSize};
use im::{HashMap, HashSet};

#[must_use]
#[allow(dead_code)]
pub fn dragged_positions(
    originals: &HashMap<NodeId, (f64, f64)>,
    anchor: (f64, f64),
    current: (f64, f64),
) -> HashMap<NodeId, (f64, f64)> {
    dragged_positions_with_snap(originals, anchor, current, false, GridSize::default())
}

#[must_use]
pub fn dragged_positions_with_snap(
    originals: &HashMap<NodeId, (f64, f64)>,
    anchor: (f64, f64),
    current: (f64, f64),
    snap_to_grid: bool,
    grid_size: GridSize,
) -> HashMap<NodeId, (f64, f64)> {
    let dx = current.0 - anchor.0;
    let dy = current.1 - anchor.1;
    let (dx, dy) = grid_snap_point((dx, dy), snap_to_grid, grid_size);
    originals
        .iter()
        .fold(HashMap::new(), |acc, (id, (ox, oy))| {
            acc.update(id.clone(), (ox + dx, oy + dy))
        })
}

#[must_use]
pub fn drag_original_positions(
    doc: &DiagramDocument,
    selected_items: &HashSet<String>,
) -> HashMap<NodeId, (f64, f64)> {
    let selected_nodes = selected_items
        .iter()
        .map(|id| NodeId::new(id.clone()))
        .filter(|id| doc.document.nodes.contains_key(id))
        .collect::<HashSet<_>>();

    let with_children = std::iter::successors(Some(selected_nodes), |current| {
        let expanded = doc
            .document
            .nodes
            .iter()
            .fold(current.clone(), |acc, (id, node)| {
                if node
                    .parent
                    .as_ref()
                    .is_some_and(|parent| acc.contains(parent))
                {
                    acc.update(id.clone())
                } else {
                    acc
                }
            });

        (expanded.len() > current.len()).then_some(expanded)
    })
    .last()
    .unwrap_or_else(HashSet::new);

    with_children.iter().fold(HashMap::new(), |acc, id| {
        if let Some(node) = doc.document.nodes.get(id) {
            acc.update(id.clone(), (node.x.0, node.y.0))
        } else {
            acc
        }
    })
}
