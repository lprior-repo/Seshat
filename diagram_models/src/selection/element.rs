use crate::document::DiagramDocument;
use crate::selection::types::{ElementId, SelectModifiers, SelectionError};

fn is_element_visible(metadata: &im::HashMap<String, serde_json::Value>) -> bool {
    metadata
        .get("visibility")
        .and_then(serde_json::Value::as_str)
        != Some("hidden")
}

pub fn select_element(
    state: &mut im::HashSet<String>,
    document: &DiagramDocument,
    id: &ElementId,
    modifiers: &SelectModifiers,
) -> Result<(), SelectionError> {
    match id {
        ElementId::Node(node_id) => {
            let node = document
                .document
                .nodes
                .get(node_id)
                .ok_or(SelectionError::ElementNotFound)?;

            if node.lock_state.is_locked() {
                return Err(SelectionError::ElementLocked);
            }

            if !is_element_visible(&node.metadata) {
                return Err(SelectionError::ElementHidden);
            }

            if modifiers.alt {
                if let Some(parent_id) = &node.parent {
                    state.clear();
                    state.insert(parent_id.to_string());
                } else {
                    return Err(SelectionError::NoParentContainer);
                }
            } else {
                state.clear();
                state.insert(node_id.to_string());
            }
        }
        ElementId::Edge(edge_id) => {
            let edge = document
                .document
                .edges
                .get(edge_id)
                .ok_or(SelectionError::ElementNotFound)?;

            if !is_element_visible(&edge.metadata) {
                return Err(SelectionError::ElementHidden);
            }

            state.clear();
            state.insert(edge_id.to_string());
        }
    }
    Ok(())
}
