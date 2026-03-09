use crate::core::routing::SUBGRAPH_PADDING;
use crate::models::document::{DiagramDocument, Node, NodeId, NodeKind, OrderedFloat};
use im::{HashMap, HashSet};
use std::collections::BTreeSet;
use thiserror::Error;

/// Maximum nesting depth for subgraphs
pub const MAX_SUBGRAPH_NESTING_DEPTH: usize = 5;

#[derive(Debug, Error, PartialEq)]
pub enum GroupingError {
    #[error("Selection is empty")]
    EmptySelection,
    #[error("Node {0} is locked")]
    LockedNode(NodeId),
    #[error("Subgraph too small: width={width}, height={height}")]
    SubgraphTooSmall { width: f64, height: f64 },
    #[error("Nested subgraph limit exceeded (max {0})")]
    NestedSubgraphLimitExceeded(usize),
}

/// Calculate bounding box from selected node IDs using functional style
fn calculate_bounding_box(
    doc: &DiagramDocument,
    selected: &HashSet<String>,
) -> Option<(f64, f64, f64, f64)> {
    selected
        .iter()
        .filter_map(|id_str| {
            let id = NodeId::new(id_str.clone());
            doc.document.nodes.get(&id).map(|node| {
                (
                    node.x.0,
                    node.y.0,
                    node.x.0 + node.width.0,
                    node.y.0 + node.height.0,
                )
            })
        })
        .reduce(|(min_x, min_y, max_x, max_y), (x, y, w, h)| {
            (min_x.min(x), min_y.min(y), max_x.max(w), max_y.max(h))
        })
}

/// Validate nodes are not locked - returns first locked node ID if any
fn find_locked_node(doc: &DiagramDocument, selected: &HashSet<String>) -> Option<NodeId> {
    selected.iter().find_map(|id_str| {
        let id = NodeId::new(id_str.clone());
        doc.document
            .nodes
            .get(&id)
            .and_then(|node| node.locked.then_some(id))
    })
}

/// Validate coordinates are valid (not NaN or Infinity)
fn validate_coordinates(min_x: f64, min_y: f64, width: f64, height: f64) -> bool {
    min_x.is_finite()
        && min_y.is_finite()
        && width.is_finite()
        && height.is_finite()
        && width > 0.0
        && height > 0.0
}

/// Create a Subgraph node with validated bounds
fn create_subgraph_node(
    _group_id: &NodeId,
    min_x: f64,
    min_y: f64,
    width: f64,
    height: f64,
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
        locked: false,
        parent: None,
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: im::HashMap::new(),
        z_index: 0,
        style: None,
        collapsed: None,
    })
}

/// Check nesting depth doesn't exceed limit
fn check_nesting_depth(doc: &DiagramDocument, selected: &HashSet<String>) -> bool {
    selected.iter().all(|id_str| {
        let id = NodeId::new(id_str.clone());
        doc.document.nodes.get(&id).is_none_or(|node| {
            count_nesting_depth(doc, node.parent.as_ref()) < MAX_SUBGRAPH_NESTING_DEPTH
        })
    })
}

/// Count the nesting depth of a node's parent chain
fn count_nesting_depth(doc: &DiagramDocument, parent: Option<&NodeId>) -> usize {
    parent
        .and_then(|pid| doc.document.nodes.get(pid))
        .map_or(0, |node| 1 + count_nesting_depth(doc, node.parent.as_ref()))
}

/// Validate selection for grouping - returns error if invalid
fn validate_selection(
    doc: &DiagramDocument,
    selected: &HashSet<String>,
) -> Result<(), GroupingError> {
    if selected.is_empty() {
        return Err(GroupingError::EmptySelection);
    }

    if let Some(locked_id) = find_locked_node(doc, selected) {
        return Err(GroupingError::LockedNode(locked_id));
    }

    if !check_nesting_depth(doc, selected) {
        return Err(GroupingError::NestedSubgraphLimitExceeded(
            MAX_SUBGRAPH_NESTING_DEPTH,
        ));
    }

    Ok(())
}

/// Compute padded bounding box for the group
fn compute_padded_bounds(
    doc: &DiagramDocument,
    selected: &HashSet<String>,
) -> Result<(f64, f64, f64, f64), GroupingError> {
    let (min_x, min_y, max_x, max_y) =
        calculate_bounding_box(doc, selected).ok_or(GroupingError::EmptySelection)?;

    let padded_min_x = min_x - SUBGRAPH_PADDING;
    let padded_min_y = min_y - SUBGRAPH_PADDING;
    let padded_max_x = max_x + SUBGRAPH_PADDING;
    let padded_max_y = max_y + SUBGRAPH_PADDING;
    let width = padded_max_x - padded_min_x;
    let height = padded_max_y - padded_min_y;

    if !validate_coordinates(padded_min_x, padded_min_y, width, height) {
        return Err(GroupingError::SubgraphTooSmall { width, height });
    }

    Ok((padded_min_x, padded_min_y, width, height))
}

/// Assign parent to all selected nodes
fn assign_parent_to_selected(
    doc: &mut DiagramDocument,
    selected: &HashSet<String>,
    parent_id: &NodeId,
) {
    for id_str in selected {
        let id = NodeId::new(id_str.clone());
        if let Some(node) = doc.document.nodes.get_mut(&id) {
            node.parent = Some(parent_id.clone());
        }
    }
}

/// Finalize group: insert node and update selection
fn finalize_group(doc: &mut DiagramDocument, group_id: &NodeId, group_node: Node) {
    doc.document.nodes.insert(group_id.clone(), group_node);
    doc.editor_state.selected_items.clear();
    doc.editor_state
        .selected_items
        .insert(group_id.as_str().to_string());
}

/// Creates a new Subgraph node that encompasses all selected nodes,
/// then assigns the selected nodes as children of the new Subgraph.
///
/// # Errors
///
/// Returns `GroupingError::EmptySelection` if no nodes are selected.
/// Returns `GroupingError::LockedNode` if any selected node is locked.
pub fn group_selection(doc: &mut DiagramDocument, group_id: &NodeId) -> Result<(), GroupingError> {
    let selected = doc.editor_state.selected_items.clone();
    validate_selection(doc, &selected)?;

    let (padded_min_x, padded_min_y, width, height) = compute_padded_bounds(doc, &selected)?;

    let group_node = create_subgraph_node(group_id, padded_min_x, padded_min_y, width, height)
        .ok_or(GroupingError::SubgraphTooSmall { width, height })?;

    assign_parent_to_selected(doc, &selected, group_id);
    finalize_group(doc, group_id, group_node);

    Ok(())
}

/// Find all Subgraph nodes in the selection
fn find_target_subgraphs(
    doc: &DiagramDocument,
    selected_items: &HashSet<String>,
) -> BTreeSet<NodeId> {
    selected_items
        .iter()
        .map(|id| NodeId::new(id.clone()))
        .filter(|id| {
            doc.document
                .nodes
                .get(id)
                .is_some_and(|node| node.kind == NodeKind::Subgraph)
        })
        .collect()
}

/// Build parent mapping for subgraphs
fn build_subgraph_parent_map(
    doc: &DiagramDocument,
    subgraphs: &BTreeSet<NodeId>,
) -> HashMap<NodeId, Option<NodeId>> {
    subgraphs
        .iter()
        .filter_map(|id| {
            doc.document
                .nodes
                .get(id)
                .map(|node| (id.clone(), node.parent.clone()))
        })
        .collect()
}

/// Calculate new parent for a node being removed from a subgraph
fn calculate_new_parent(
    node: &Node,
    target_subgraphs: &BTreeSet<NodeId>,
    subgraph_parents: &HashMap<NodeId, Option<NodeId>>,
) -> Option<NodeId> {
    node.parent.as_ref().and_then(|parent| {
        target_subgraphs
            .contains(parent)
            .then(|| subgraph_parents.get(parent).cloned().flatten())
            .flatten()
    })
}

/// Check if a node becomes orphaned after subgraph removal
fn is_orphaned(node: &Node, target_subgraphs: &BTreeSet<NodeId>) -> bool {
    node.parent
        .as_ref()
        .is_some_and(|p| target_subgraphs.contains(p))
}

/// Reposition a single node within the fold operation
fn reposition_node(
    _id: &NodeId,
    node: &Node,
    target_subgraphs: &BTreeSet<NodeId>,
    subgraph_parents: &HashMap<NodeId, Option<NodeId>>,
) -> (Node, bool) {
    let new_parent = calculate_new_parent(node, target_subgraphs, subgraph_parents);
    let orphaned = is_orphaned(node, target_subgraphs);
    (
        Node {
            parent: new_parent,
            ..node.clone()
        },
        orphaned,
    )
}

/// Remove subgraphs and reparent their children using functional style
fn remove_subgraphs_and_reparent(
    doc: &mut DiagramDocument,
    target_subgraphs: &BTreeSet<NodeId>,
    subgraph_parents: &im::HashMap<NodeId, Option<NodeId>>,
) -> BTreeSet<NodeId> {
    let (new_nodes, orphaned): (im::HashMap<NodeId, Node>, BTreeSet<NodeId>) =
        doc.document.nodes.iter()
        .filter(|(id, _)| !target_subgraphs.contains(id))
        .fold((im::HashMap::new(), BTreeSet::new()),
            |(mut nodes, mut orphans), (id, node)| {
                let (new_node, is_orphaned) =
                    reposition_node(id, node, target_subgraphs, subgraph_parents);
                if is_orphaned { orphans.insert(id.clone()); }
                nodes.insert(id.clone(), new_node);
                (nodes, orphans)
            });
    doc.document.nodes = new_nodes;
    orphaned
}

/// Remove edges connected to deleted subgraphs
fn remove_orphan_edges(doc: &mut DiagramDocument, target_subgraphs: &BTreeSet<NodeId>) {
    doc.document.edges = doc
        .document
        .edges
        .iter()
        .filter(|(_, edge)| {
            !target_subgraphs.contains(&edge.source) && !target_subgraphs.contains(&edge.target)
        })
        .map(|(id, edge)| (id.clone(), edge.clone()))
        .collect();
}

/// Finds all selected Subgraph nodes, collects their children, sets their children's
/// parent to the subgraph's parent (or None), deletes the subgraph node, and selects
/// all the newly orphaned children.
///
/// # Errors
///
/// Returns `GroupingError::EmptySelection` if no nodes are selected.
pub fn ungroup_selection(doc: &mut DiagramDocument) -> Result<(), GroupingError> {
    let selected_items = doc.editor_state.selected_items.clone();
    let target_subgraphs = find_target_subgraphs(doc, &selected_items);

    if target_subgraphs.is_empty() {
        return Err(GroupingError::EmptySelection);
    }

    let subgraph_parents = build_subgraph_parent_map(doc, &target_subgraphs);
    let orphaned_children =
        remove_subgraphs_and_reparent(doc, &target_subgraphs, &subgraph_parents);
    remove_orphan_edges(doc, &target_subgraphs);

    doc.editor_state.selected_items.clear();
    for child_id in orphaned_children {
        doc.editor_state
            .selected_items
            .insert(child_id.as_str().to_string());
    }

    Ok(())
}

#[cfg(test)]
#[path = "grouping_tests.rs"]
mod tests;
