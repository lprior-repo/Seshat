#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! UI-friendly mutation helpers.
//!
//! This module provides utilities for performing validated document mutations
//! from UI code (components, commands, etc.).
//!
//! # Usage
//!
//! Instead of:
//! ```ignore
//! doc_signal.with_mut(|doc| {
//!     doc.editor_state.selected_items = ...;
//!     doc.revision = doc.revision.increment();
//! });
//! ```
//!
//! Use:
//! ```ignore
//! use crate::mutation::ui_helpers::{mutate_doc_signal, mutate_editor_signal};
//!
//! // For validated mutations (nodes/edges):
//! let result = mutate_doc_signal(&mut doc_signal, |doc| {
//!     doc.editor_state.selected_items = ...;
//! });
//!
//! // For non-validated mutations (editor state):
//! mutate_editor_signal(&mut doc_signal, |doc| {
//!     doc.editor_state.zoom = ...;
//! });
//! ```

use crate::history::History;
use crate::models::document::DiagramDocument;
use crate::mutation::error::MutationError;
use crate::mutation::pipeline::{mutate_document, mutate_editor_state};
use dioxus::prelude::*;

/// Applies a validated document mutation via a Dioxus Signal.
///
/// This is a convenience wrapper that:
/// 1. Clones the current document
/// 2. Applies the mutation
/// 3. Validates the result
/// 4. Sets the updated document on success
///
/// Returns `Ok(true)` on success, `Err(MutationError)` on validation failure.
///
/// # Example
/// ```ignore
/// use crate::mutation::ui_helpers::mutate_doc_signal;
///
/// fn delete_selected(mut doc_signal: Signal<DiagramDocument>) {
///     let result = mutate_doc_signal(&mut doc_signal, |doc| {
///         // Perform validated mutation
///         doc.document.nodes.remove(&node_id);
///     });
/// }
/// ```
///
/// # Errors
///
/// Returns a `MutationError` if the mutation fails validation.
pub fn mutate_doc_signal(
    mut doc_signal: Signal<DiagramDocument>,
    mutation: impl FnOnce(&mut DiagramDocument),
) -> Result<bool, MutationError> {
    let current = doc_signal.read().clone();
    match mutate_document(current, mutation) {
        Ok(updated) => {
            doc_signal.set(updated);
            Ok(true)
        }
        Err(e) => Err(e),
    }
}

/// Applies a non-validated editor state mutation via a Dioxus Signal.
///
/// Use this for editor state changes that don't need schema/semantic validation:
/// - Camera position (pan)
/// - Zoom level
/// - Selection state
/// - UI panel visibility
///
/// This is faster than `mutate_doc_signal` because it skips validation.
///
/// # Example
/// ```ignore
/// use crate::mutation::ui_helpers::mutate_editor_signal;
///
/// fn pan_canvas(mut doc_signal: Signal<DiagramDocument>, dx: f64, dy: f64) {
///     mutate_editor_signal(&mut doc_signal, |doc| {
///         doc.editor_state.camera_x = OrderedFloat(doc.editor_state.camera_x.0 + dx);
///         doc.editor_state.camera_y = OrderedFloat(doc.editor_state.camera_y.0 + dy);
///     });
/// }
/// ```
pub fn mutate_editor_signal(
    mut doc_signal: Signal<DiagramDocument>,
    mutation: impl FnOnce(&mut DiagramDocument),
) {
    let current = doc_signal.read().clone();
    let updated = mutate_editor_state(current, mutation);
    doc_signal.set(updated);
}

/// Applies a validated document mutation with history tracking.
///
/// This variant pushes the current state to history before applying the mutation,
/// enabling undo/redo support.
///
/// Returns `Ok(true)` on success, `Err(MutationError)` on validation failure.
///
/// # Example
/// ```ignore
/// use crate::mutation::ui_helpers::mutate_doc_with_history;
///
/// fn move_node(
///     mut doc_signal: Signal<DiagramDocument>,
///     mut history_signal: Signal<History>,
///     node_id: NodeId,
///     new_x: f64,
/// ) {
///     let result = mutate_doc_with_history(&mut doc_signal, &mut history_signal, |doc| {
///         if let Some(node) = doc.document.nodes.get_mut(&node_id) {
///             node.x = OrderedFloat(new_x);
///         }
///     });
/// }
/// ```
///
/// # Errors
///
/// Returns a `MutationError` if the mutation fails validation.
pub fn mutate_doc_with_history(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    mutation: impl FnOnce(&mut DiagramDocument),
) -> Result<bool, MutationError> {
    // Push current state to history for undo support
    let current = doc_signal.read().clone();
    let history = history_signal.read().clone();
    *history_signal.write() = history.push(current);

    // Apply validated mutation
    let current = doc_signal.read().clone();
    match mutate_document(current, mutation) {
        Ok(updated) => {
            doc_signal.set(updated);
            Ok(true)
        }
        Err(e) => Err(e),
    }
}
