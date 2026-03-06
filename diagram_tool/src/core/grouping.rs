use crate::geometry::AABB;
use crate::models::document::{DiagramDocument, Node, NodeId, NodeKind};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum GroupingError {
    #[error("Selection is empty")]
    EmptySelection,
    #[error("Node {0} is locked")]
    LockedNode(NodeId),
}

/// Creates a new Subgraph node that encompasses all selected nodes,
/// then assigns the selected nodes as children of the new Subgraph.
pub fn group_selection(doc: &mut DiagramDocument, group_id: NodeId) -> Result<(), GroupingError> {
    let selected = doc.editor_state.selected_items.clone();
    if selected.is_empty() {
        return Err(GroupingError::EmptySelection);
    }

    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    // 1. Calculate bounding box of selection
    for id_str in &selected {
        let id = NodeId::new(id_str.clone());
        if let Some(node) = doc.document.nodes.get(&id) {
            if node.locked {
                return Err(GroupingError::LockedNode(id));
            }

            min_x = min_x.min(node.x.0);
            min_y = min_y.min(node.y.0);
            max_x = max_x.max(node.x.0 + node.width.0);
            max_y = max_y.max(node.y.0 + node.height.0);
        }
    }

    let padding = 20.0;
    min_x -= padding;
    min_y -= padding;
    max_x += padding;
    max_y += padding;

    // 2. Create the Subgraph node
    let group_node = Node {
        kind: NodeKind::Subgraph,
        icon: String::new(),
        label: "Group".to_string(),
        x: crate::models::document::OrderedFloat::new_unchecked(min_x),
        y: crate::models::document::OrderedFloat::new_unchecked(min_y),
        width: crate::models::document::OrderedFloat::new_unchecked(max_x - min_x),
        height: crate::models::document::OrderedFloat::new_unchecked(max_y - min_y),
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
    };

    // 3. Update children to set their parent
    for id_str in &selected {
        let id = NodeId::new(id_str.clone());
        if let Some(node) = doc.document.nodes.get_mut(&id) {
            node.parent = Some(group_id.clone());
        }
    }

    // 4. Insert the new group and select it
    doc.document.nodes.insert(group_id.clone(), group_node);
    doc.editor_state.selected_items.clear();
    doc.editor_state
        .selected_items
        .insert(group_id.as_str().to_string());

    Ok(())
}

#[cfg(test)]
#[path = "grouping_tests.rs"]
mod tests;
