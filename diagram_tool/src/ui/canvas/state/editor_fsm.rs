//! Editor Finite State Machine
//!
//! Models the explicit state transitions for node/edge hover and edit modes.

use diagram_models::document::{DiagramDocument, EdgeId, NodeId};

#[derive(Clone, PartialEq, Debug)]
pub enum EditorState {
    Idle,
    HoveringNode(NodeId),
    EditingNode(NodeId),
    HoveringEdge(EdgeId),
    EditingEdge(EdgeId),
}

#[derive(Clone, PartialEq, Debug)]
pub enum EditorEvent {
    HoverNode(NodeId),
    HoverEdge(EdgeId),
    EditNode(NodeId),
    EditEdge(EdgeId),
    ClearHover,
    Escape,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum EditorError {
    #[error("Element {0} not found")]
    ElementNotFound(String),
    #[error("Invalid transition from {from:?} with event {to_event:?}")]
    InvalidTransition {
        from: EditorState,
        to_event: EditorEvent,
    },
    #[error("Inconsistent state")]
    InconsistentState,
}

/// Pure function that calculates the next editor state given current state and event.
/// This is the core of the state machine - total and deterministic.
#[allow(clippy::needless_pass_by_value)]
pub fn calculate_transition(
    current: &EditorState,
    event: EditorEvent,
    doc: &DiagramDocument,
) -> Result<EditorState, EditorError> {
    match event {
        EditorEvent::Escape => Ok(EditorState::Idle),
        EditorEvent::ClearHover => match current {
            EditorState::HoveringNode(_) | EditorState::HoveringEdge(_) | EditorState::Idle => {
                Ok(EditorState::Idle)
            }
            _ => Err(EditorError::InvalidTransition {
                from: current.clone(),
                to_event: event,
            }),
        },
        EditorEvent::HoverNode(ref id) => {
            if !doc.document.nodes.contains_key(id) {
                return Err(EditorError::ElementNotFound(id.to_string()));
            }
            match current {
                EditorState::HoveringNode(current_id) if current_id == id => Ok(current.clone()),
                EditorState::Idle | EditorState::HoveringNode(_) | EditorState::HoveringEdge(_) => {
                    Ok(EditorState::HoveringNode(id.clone()))
                }
                _ => Err(EditorError::InvalidTransition {
                    from: current.clone(),
                    to_event: event.clone(),
                }),
            }
        }
        EditorEvent::HoverEdge(ref id) => {
            if !doc.document.edges.contains_key(id) {
                return Err(EditorError::ElementNotFound(id.to_string()));
            }
            match current {
                EditorState::HoveringEdge(current_id) if current_id == id => Ok(current.clone()),
                EditorState::Idle | EditorState::HoveringNode(_) | EditorState::HoveringEdge(_) => {
                    Ok(EditorState::HoveringEdge(id.clone()))
                }
                _ => Err(EditorError::InvalidTransition {
                    from: current.clone(),
                    to_event: event.clone(),
                }),
            }
        }
        EditorEvent::EditNode(ref id) => {
            if !doc.document.nodes.contains_key(id) {
                return Err(EditorError::ElementNotFound(id.to_string()));
            }
            match current {
                EditorState::HoveringNode(hover_id) if hover_id == id => {
                    Ok(EditorState::EditingNode(id.clone()))
                }
                EditorState::EditingNode(edit_id) if edit_id == id => Ok(current.clone()),
                EditorState::EditingNode(_) => Ok(EditorState::EditingNode(id.clone())),
                _ => Err(EditorError::InvalidTransition {
                    from: current.clone(),
                    to_event: event.clone(),
                }),
            }
        }
        EditorEvent::EditEdge(ref id) => {
            if !doc.document.edges.contains_key(id) {
                return Err(EditorError::ElementNotFound(id.to_string()));
            }
            match current {
                EditorState::HoveringEdge(hover_id) if hover_id == id => {
                    Ok(EditorState::EditingEdge(id.clone()))
                }
                EditorState::EditingEdge(edit_id) if edit_id == id => Ok(current.clone()),
                EditorState::EditingEdge(_) => Ok(EditorState::EditingEdge(id.clone())),
                _ => Err(EditorError::InvalidTransition {
                    from: current.clone(),
                    to_event: event.clone(),
                }),
            }
        }
    }
}
