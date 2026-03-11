use crate::history::History;
use crate::models::document::DiagramDocument;

/// Undoes the last document change.
///
/// # Errors
///
/// Returns `"Nothing to undo"` if there is no history to undo.
pub fn apply_undo(doc: &mut DiagramDocument, history: &mut History) -> Result<(), &'static str> {
    if let Some((prev, new_history)) = history.undo(doc.clone()) {
        *doc = prev;
        *history = new_history;
        Ok(())
    } else {
        Err("Nothing to undo")
    }
}

/// Redoes the last undone document change.
///
/// # Errors
///
/// Returns `"Nothing to redo"` if there is no history to redo.
pub fn apply_redo(doc: &mut DiagramDocument, history: &mut History) -> Result<(), &'static str> {
    if let Some((next, new_history)) = history.redo(doc.clone()) {
        *doc = next;
        *history = new_history;
        Ok(())
    } else {
        Err("Nothing to redo")
    }
}
