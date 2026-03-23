use crate::history::History;
use diagram_models::document::DiagramDocument;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryError {
    NothingToUndo,
    NothingToRedo,
}

impl std::fmt::Display for HistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NothingToUndo => write!(f, "Nothing to undo"),
            Self::NothingToRedo => write!(f, "Nothing to redo"),
        }
    }
}

impl std::error::Error for HistoryError {}

/// Undoes the last document change.
///
/// # Errors
///
/// Returns `HistoryError::NothingToUndo` if there is no history to undo.
pub fn apply_undo(doc: &mut DiagramDocument, history: &mut History) -> Result<(), HistoryError> {
    if let Some((prev, new_history)) = history.undo(doc.clone()) {
        *doc = prev;
        *history = new_history;
        Ok(())
    } else {
        Err(HistoryError::NothingToUndo)
    }
}

/// Redoes the last undone document change.
///
/// # Errors
///
/// Returns `HistoryError::NothingToRedo` if there is no history to redo.
pub fn apply_redo(doc: &mut DiagramDocument, history: &mut History) -> Result<(), HistoryError> {
    if let Some((next, new_history)) = history.redo(doc.clone()) {
        *doc = next;
        *history = new_history;
        Ok(())
    } else {
        Err(HistoryError::NothingToRedo)
    }
}

#[cfg(test)]
#[path = "history_tests.rs"]
mod tests;
