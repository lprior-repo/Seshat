pub mod calculations;
pub mod validation;

use crate::models::document::{DiagramDocument, NodeId, NodeKind};
pub use calculations::{
    calculate_edge_cleanup, calculate_ungroup, compute_padded_bounds, create_subgraph_node,
    find_lca,
};
use std::collections::BTreeSet;
pub use validation::validate_selection;
pub use validation::GroupingError;
use validation::ValidatedSelection;

/// Action: Group selected items in a `DiagramDocument`
pub fn group_selection(doc: &mut DiagramDocument, group_id: &NodeId) -> Result<(), GroupingError> {
    // Parse/Validate at boundary
    let selected_ids: im::HashSet<NodeId> = doc
        .editor_state
        .selected_items
        .iter()
        .map(|id| NodeId::new(id.clone()))
        .collect();

    let validated = ValidatedSelection::try_new(&doc.document.nodes, &selected_ids)?;
    let selected = validated.inner();

    let (padded_min_x, padded_min_y, width, height) =
        compute_padded_bounds(&doc.document.nodes, selected)?;

    let min_z = selected
        .iter()
        .filter_map(|id| doc.document.nodes.get(id).map(|n| n.z_index))
        .min()
        .unwrap_or(0);

    let parent_id = calculations::find_lca(&doc.document.nodes, selected);

    let group_node = create_subgraph_node(
        padded_min_x,
        padded_min_y,
        width,
        height,
        min_z - 1,
        parent_id,
    )
    .ok_or(GroupingError::SubgraphTooSmall { width, height })?;

    // Update children
    for id in selected {
        if let Some(node) = doc.document.nodes.get_mut(id) {
            node.parent = Some(group_id.clone());
        }
    }

    doc.document.nodes.insert(group_id.clone(), group_node);
    doc.editor_state.selected_items.clear();
    doc.editor_state
        .selected_items
        .insert(group_id.as_str().to_string());

    Ok(())
}

/// Action: Ungroup selected subgraphs in a `DiagramDocument`
pub fn ungroup_selection(doc: &mut DiagramDocument) -> Result<(), GroupingError> {
    let selected_items = doc.editor_state.selected_items.clone();
    let target_subgraphs: BTreeSet<NodeId> = selected_items
        .iter()
        .map(|id| NodeId::new(id.clone()))
        .filter(|id| {
            doc.document
                .nodes
                .get(id)
                .is_some_and(|node| node.kind == NodeKind::Subgraph)
        })
        .collect();

    if target_subgraphs.is_empty() {
        return Err(GroupingError::EmptySelection);
    }

    let (new_nodes, orphaned_children) = calculate_ungroup(&doc.document.nodes, &target_subgraphs);
    doc.document.nodes = new_nodes;
    doc.document.edges = calculate_edge_cleanup(&doc.document.edges, &target_subgraphs);

    doc.editor_state.selected_items.clear();
    for child_id in orphaned_children {
        doc.editor_state
            .selected_items
            .insert(child_id.as_str().to_string());
    }

    Ok(())
}

#[cfg(test)]
#[path = "../grouping_tests.rs"]
mod tests;
