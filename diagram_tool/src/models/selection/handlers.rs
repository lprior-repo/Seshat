use crate::models::document::{DiagramDocument, NodeId};
use crate::models::selection::types::SelectionError;

pub fn handle_long_press(
    doc: &mut DiagramDocument,
    target: NodeId,
    movement: f64,
) -> Result<(), SelectionError> {
    if movement >= 5.0 {
        return Err(SelectionError::MovementExceededDragThreshold);
    }

    // Must ensure node exists to not add invalid node IDs to selection
    if !doc.document.nodes.contains_key(&target) {
        return Err(SelectionError::NodeNotFound);
    }

    doc.editor_state.selected_items.insert(target.to_string());
    Ok(())
}

pub fn handle_double_click(
    doc: &mut DiagramDocument,
    target: NodeId,
) -> Result<(), SelectionError> {
    let node = doc
        .document
        .nodes
        .get(&target)
        .ok_or(SelectionError::NodeNotFound)?;

    if node.lock_state.is_locked() {
        return Err(SelectionError::NodeNotEditable);
    }

    doc.editor_state.edit_mode_target = Some(target.to_string());
    Ok(())
}
