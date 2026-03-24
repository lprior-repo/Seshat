use diagram_models::document::{DiagramDocument, Node, NodeId};
use diagram_models::subgraph::recompute_affected_container_bounds;
use diagram_models::transform::{
    calculate_alignment, calculate_distribution, AlignmentAxis, AlignmentMode, TransformError,
    ValidTransform,
};
use im::HashMap;

fn apply_transform_to_doc(
    doc: &mut DiagramDocument,
    transform_fn: impl Fn(
        &[NodeId],
        &HashMap<NodeId, Node>,
    ) -> Result<Vec<(NodeId, Node)>, TransformError>,
) -> Result<(), TransformError> {
    let selected_ids = collect_selected_ids(doc);
    if selected_ids.is_empty() {
        return Err(TransformError::EmptySelection);
    }

    check_selection_locks(doc, &selected_ids)?;
    let updates = transform_fn(&selected_ids, &doc.document.nodes)?;

    perform_document_update(doc, &selected_ids, updates);
    Ok(())
}

fn collect_selected_ids(doc: &DiagramDocument) -> Vec<NodeId> {
    doc.editor_state
        .selected_items
        .iter()
        .map(|s| NodeId::new(s.clone()))
        .collect()
}

fn check_selection_locks(doc: &DiagramDocument, ids: &[NodeId]) -> Result<(), TransformError> {
    ids.iter().try_for_each(|id| {
        if let Some(node) = doc.document.nodes.get(id) {
            if !node.lock_state.is_movable(&node.kind) {
                return Err(TransformError::NodeLocked(id.clone()));
            }
        }
        Ok(())
    })
}

fn perform_document_update(
    doc: &mut DiagramDocument,
    ids: &[NodeId],
    updates: Vec<(NodeId, Node)>,
) {
    let mut new_nodes = doc.document.nodes.clone();
    for (id, node) in updates {
        new_nodes = new_nodes.update(id, node);
    }

    doc.document.nodes = recompute_affected_container_bounds(new_nodes, ids);
    doc.revision = doc.revision.increment();
}

/// Aligns selected nodes along the specified axis.
pub fn align_selection(
    doc: &mut DiagramDocument,
    axis: &AlignmentAxis,
    mode: &AlignmentMode,
) -> Result<(), TransformError> {
    apply_transform_to_doc(doc, |selected, nodes| {
        calculate_alignment(nodes, selected, *axis, *mode)
    })
}

/// Distributes selected nodes evenly along the specified axis.
pub fn distribute_selection(
    doc: &mut DiagramDocument,
    axis: &AlignmentAxis,
) -> Result<(), TransformError> {
    if doc.editor_state.selected_items.len() < 3 {
        return Err(TransformError::EmptySelection);
    }

    apply_transform_to_doc(doc, |selected, nodes| {
        calculate_distribution(nodes, selected, *axis)
    })
}

/// Translates selected nodes by `dx` and `dy`.
pub fn translate_selection(
    doc: &mut DiagramDocument,
    dx: f64,
    dy: f64,
) -> Result<(), TransformError> {
    let transform = ValidTransform::translate(dx, dy)?;

    apply_transform_to_doc(doc, |selected, nodes| {
        selected
            .iter()
            .map(|id| {
                nodes
                    .get(id)
                    .ok_or_else(|| TransformError::ItemNotFound(id.clone()))
                    .and_then(|node| {
                        diagram_models::transform::apply_transform_to_node(node, &transform)
                            .map(|updated| (id.clone(), updated))
                    })
            })
            .collect()
    })
}

#[cfg(test)]
#[path = "transform_tests.rs"]
mod tests;
