pub mod calculations;
pub mod validation;

use crate::document::{DiagramDocument, NodeId, NodeKind};
pub use calculations::{
    calculate_bounding_box, calculate_edge_cleanup, calculate_ungroup, compute_padded_bounds,
    create_subgraph_node, find_lca,
};
use std::collections::BTreeSet;
pub use validation::{validate_coordinates, validate_selection, GroupingError, ValidatedSelection};

/// Action: Group selected items in a `DiagramDocument`
///
/// # Errors
///
/// Returns `GroupingError` if grouping fails.
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
///
/// # Errors
///
/// Returns `GroupingError` if ungrouping fails.
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

// #[cfg(test)]
// #[path = "../grouping_tests.rs"]
// mod tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{LockState, Node, OrderedFloat};

    fn create_test_node(x: f64, y: f64, width: f64, height: f64) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: "test".to_string(),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(width),
            height: OrderedFloat(height),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        }
    }

    #[test]
    fn given_empty_selection_when_grouped_then_returns_empty_selection_error() {
        let mut doc = DiagramDocument::default();
        let group_id = NodeId::new("g1".to_string());

        let result = group_selection(&mut doc, &group_id);
        assert!(matches!(result, Err(GroupingError::EmptySelection)));
    }

    #[test]
    fn given_valid_selection_when_grouped_then_creates_subgraph_and_updates_parents() {
        let mut doc = DiagramDocument::default();

        let n1_id = NodeId::new("n1".to_string());
        let n2_id = NodeId::new("n2".to_string());
        let group_id = NodeId::new("g1".to_string());

        doc.document
            .nodes
            .insert(n1_id.clone(), create_test_node(0.0, 0.0, 10.0, 10.0));
        doc.document
            .nodes
            .insert(n2_id.clone(), create_test_node(20.0, 0.0, 10.0, 10.0));

        doc.editor_state
            .selected_items
            .insert(n1_id.as_str().to_string());
        doc.editor_state
            .selected_items
            .insert(n2_id.as_str().to_string());

        let result = group_selection(&mut doc, &group_id);
        assert!(result.is_ok());

        assert!(doc.document.nodes.contains_key(&group_id));
        assert_eq!(
            doc.document.nodes.get(&n1_id).unwrap().parent,
            Some(group_id.clone())
        );
        assert_eq!(
            doc.document.nodes.get(&n2_id).unwrap().parent,
            Some(group_id.clone())
        );

        assert_eq!(doc.editor_state.selected_items.len(), 1);
        assert!(doc.editor_state.selected_items.contains(group_id.as_str()));
    }

    #[test]
    fn given_empty_selection_when_ungrouped_then_returns_empty_selection_error() {
        let mut doc = DiagramDocument::default();
        let result = ungroup_selection(&mut doc);
        assert!(matches!(result, Err(GroupingError::EmptySelection)));
    }

    #[test]
    fn given_subgraph_selection_when_ungrouped_then_removes_subgraph_and_clears_parents() {
        let mut doc = DiagramDocument::default();

        let n1_id = NodeId::new("n1".to_string());
        let group_id = NodeId::new("g1".to_string());

        let mut group_node = create_test_node(0.0, 0.0, 100.0, 100.0);
        group_node.kind = NodeKind::Subgraph;

        let mut child_node = create_test_node(10.0, 10.0, 10.0, 10.0);
        child_node.parent = Some(group_id.clone());

        doc.document.nodes.insert(group_id.clone(), group_node);
        doc.document.nodes.insert(n1_id.clone(), child_node);

        doc.editor_state
            .selected_items
            .insert(group_id.as_str().to_string());

        let result = ungroup_selection(&mut doc);
        assert!(result.is_ok());

        assert!(!doc.document.nodes.contains_key(&group_id));
        assert_eq!(doc.document.nodes.get(&n1_id).unwrap().parent, None);

        assert_eq!(doc.editor_state.selected_items.len(), 1);
        assert!(doc.editor_state.selected_items.contains(n1_id.as_str()));
    }
}
