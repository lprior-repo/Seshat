use super::validation::{validate_coordinates, GroupingError};
use crate::models::document::{Edge, EdgeId, LockState, Node, NodeId, NodeKind, OrderedFloat};
use im::{HashMap, HashSet};
use std::collections::BTreeSet;

/// Constant for subgraph padding
pub const SUBGRAPH_PADDING_NEW: f64 = 24.0;

/// Get the parent chain of a node including itself
fn get_parent_chain(nodes: &HashMap<NodeId, Node>, id: &NodeId) -> Vec<Option<NodeId>> {
    let mut chain = vec![Some(id.clone())];
    let mut current = id.clone();
    while let Some(node) = nodes.get(&current) {
        if let Some(parent) = &node.parent {
            chain.push(Some(parent.clone()));
            current = parent.clone();
        } else {
            chain.push(None);
            break;
        }
    }
    chain.into_iter().rev().collect()
}

/// Find Lowest Common Ancestor of selected nodes
#[must_use]
pub fn find_lca(nodes: &HashMap<NodeId, Node>, selected: &HashSet<NodeId>) -> Option<NodeId> {
    let chains: Vec<Vec<Option<NodeId>>> = selected
        .iter()
        .map(|id| get_parent_chain(nodes, id))
        .collect();

    if chains.is_empty() {
        return None;
    }

    let mut lca = None;
    let mut i = 0;
    loop {
        let current_val = chains.first().and_then(|c| c.get(i));
        if current_val.is_none() {
            break;
        }

        let all_same = chains.iter().all(|c| c.get(i) == current_val);
        if all_same {
            if let Some(val) = current_val {
                lca = val.clone();
            }
            i += 1;
        } else {
            break;
        }
    }

    // We must ensure the LCA is not one of the selected nodes.
    // If it is, we take its parent.
    while let Some(id) = &lca {
        if selected.contains(id) {
            lca = nodes.get(id).and_then(|n| n.parent.clone());
        } else {
            break;
        }
    }

    lca
}

/// Calculate bounding box from selected node IDs
#[must_use]
pub fn calculate_bounding_box(
    nodes: &HashMap<NodeId, Node>,
    selected: &HashSet<NodeId>,
) -> Option<(f64, f64, f64, f64)> {
    selected
        .iter()
        .filter_map(|id| {
            nodes.get(id).map(|node| {
                (
                    node.x.0,
                    node.y.0,
                    node.x.0 + node.width.0,
                    node.y.0 + node.height.0,
                )
            })
        })
        .reduce(|acc: (f64, f64, f64, f64), cur: (f64, f64, f64, f64)| {
            (
                acc.0.min(cur.0),
                acc.1.min(cur.1),
                acc.2.max(cur.2),
                acc.3.max(cur.3),
            )
        })
}

/// Create a Subgraph node with validated bounds
#[must_use]
pub fn create_subgraph_node(
    min_x: f64,
    min_y: f64,
    width: f64,
    height: f64,
    z_index: i64,
    parent: Option<NodeId>,
) -> Option<Node> {
    let x = OrderedFloat::new(min_x).ok()?;
    let y = OrderedFloat::new(min_y).ok()?;
    let w = OrderedFloat::new(width).ok()?;
    let h = OrderedFloat::new(height).ok()?;

    Some(Node {
        kind: NodeKind::Subgraph,
        icon: String::new(),
        label: "Group".to_string(),
        x,
        y,
        width: w,
        height: h,
        font_size: None,
        font_weight: None,
        lock_state: LockState::Unlocked,
        parent,
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: im::HashMap::new(),
        z_index,
        style: Some(crate::models::document::NodeStyle::Box),
        collapsed: Some(false),
    })
}

/// Compute padded bounding box for the group
pub fn compute_padded_bounds(
    nodes: &HashMap<NodeId, Node>,
    selected: &HashSet<NodeId>,
) -> Result<(f64, f64, f64, f64), GroupingError> {
    let (min_x, min_y, max_x, max_y) =
        calculate_bounding_box(nodes, selected).ok_or(GroupingError::EmptySelection)?;

    let padded_min_x = min_x - SUBGRAPH_PADDING_NEW;
    let padded_min_y = min_y - SUBGRAPH_PADDING_NEW;
    let padded_max_x = max_x + SUBGRAPH_PADDING_NEW;
    let padded_max_y = max_y + SUBGRAPH_PADDING_NEW;
    let width = padded_max_x - padded_min_x;
    let height = padded_max_y - padded_min_y;

    if !validate_coordinates(padded_min_x, padded_min_y, width, height) {
        return Err(GroupingError::InvalidCoordinates);
    }

    Ok((padded_min_x, padded_min_y, width, height))
}

/// Pure calculation: Remove subgraphs and reparent their children
#[must_use]
pub fn calculate_ungroup(
    nodes: &HashMap<NodeId, Node>,
    target_subgraphs: &BTreeSet<NodeId>,
) -> (HashMap<NodeId, Node>, BTreeSet<NodeId>) {
    let subgraph_parents: HashMap<NodeId, Option<NodeId>> = target_subgraphs
        .iter()
        .filter_map(|id| nodes.get(id).map(|node| (id.clone(), node.parent.clone())))
        .collect();

    let (new_nodes, orphaned): (HashMap<NodeId, Node>, BTreeSet<NodeId>) = nodes
        .iter()
        .filter(|(id, _)| !target_subgraphs.contains(id))
        .fold(
            (HashMap::new(), BTreeSet::new()),
            |(mut acc_nodes, mut orphans): (HashMap<NodeId, Node>, BTreeSet<NodeId>),
             (id, node): (&NodeId, &Node)| {
                let new_parent = node.parent.as_ref().and_then(|parent| {
                    if target_subgraphs.contains(parent) {
                        subgraph_parents.get(parent).cloned().flatten()
                    } else {
                        Some(parent.clone())
                    }
                });

                let is_orphaned = node
                    .parent
                    .as_ref()
                    .is_some_and(|p| target_subgraphs.contains(p));

                if is_orphaned {
                    orphans.insert(id.clone());
                }

                acc_nodes.insert(
                    id.clone(),
                    Node {
                        parent: new_parent,
                        ..node.clone()
                    },
                );
                (acc_nodes, orphans)
            },
        );

    (new_nodes, orphaned)
}

/// Pure calculation: Remove edges connected to deleted subgraphs
#[must_use]
pub fn calculate_edge_cleanup(
    edges: &HashMap<EdgeId, Edge>,
    deleted_subgraphs: &BTreeSet<NodeId>,
) -> HashMap<EdgeId, Edge> {
    edges
        .iter()
        .filter(|(_, edge)| {
            !deleted_subgraphs.contains(&edge.source) && !deleted_subgraphs.contains(&edge.target)
        })
        .map(|(id, edge): (&EdgeId, &Edge)| (id.clone(), edge.clone()))
        .collect()
}
