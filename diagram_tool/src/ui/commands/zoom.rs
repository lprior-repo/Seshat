//! Zoom and undo/redo operations

use dioxus::prelude::*;

use crate::history::History;
use diagram_models::document::{DiagramDocument, OrderedFloat};

/// Zoom in by 25%
#[must_use]
pub fn apply_zoom_in(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    viewport_size: (f64, f64),
) -> bool {
    let changed = {
        let doc = doc_signal.read();
        ((doc.editor_state.zoom.0 * 1.25).clamp(0.1, 4.0) - doc.editor_state.zoom.0).abs()
            >= f64::EPSILON
    };
    if !changed {
        return false;
    }

    let history = history_signal.read().clone();
    *history_signal.write() = history.push(doc_signal.read().clone());

    doc_signal.with_mut(|doc| {
        let _ = zoom_to_center(doc, 1.25, viewport_size);
        doc.revision = doc.revision.increment();
    });
    true
}

/// Zoom out by 20%
#[must_use]
pub fn apply_zoom_out(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    viewport_size: (f64, f64),
) -> bool {
    let changed = {
        let doc = doc_signal.read();
        ((doc.editor_state.zoom.0 * 0.8).clamp(0.1, 4.0) - doc.editor_state.zoom.0).abs()
            >= f64::EPSILON
    };
    if !changed {
        return false;
    }
    let history = history_signal.read().clone();
    *history_signal.write() = history.push(doc_signal.read().clone());

    doc_signal.with_mut(|doc| {
        let _ = zoom_to_center(doc, 0.8, viewport_size);
        doc.revision = doc.revision.increment();
    });
    true
}

/// Reset zoom to 100%
#[must_use]
pub fn apply_zoom_reset(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    viewport_size: (f64, f64),
) -> bool {
    let changed = (doc_signal.read().editor_state.zoom.0 - 1.0).abs() >= f64::EPSILON;
    if !changed {
        return false;
    }

    let history = history_signal.read().clone();
    *history_signal.write() = history.push(doc_signal.read().clone());

    doc_signal.with_mut(|doc| {
        let _ = set_zoom_centered(doc, 1.0, viewport_size);
        doc.revision = doc.revision.increment();
    });
    true
}

/// Undo the last operation
pub fn apply_undo(mut doc_signal: Signal<DiagramDocument>, mut history_signal: Signal<History>) {
    let current = doc_signal.read().clone();
    let history = history_signal.read().clone();
    if let Some((doc, next_history)) = history.undo(current) {
        *doc_signal.write() = doc;
        *history_signal.write() = next_history;
    }
}

/// Redo the last undone operation
pub fn apply_redo(mut doc_signal: Signal<DiagramDocument>, mut history_signal: Signal<History>) {
    let current = doc_signal.read().clone();
    let history = history_signal.read().clone();
    if let Some((doc, next_history)) = history.redo(current) {
        *doc_signal.write() = doc;
        *history_signal.write() = next_history;
    }
}

// Private helper functions

/// Zooms the viewport by a factor, keeping the center point stable.
fn zoom_to_center(doc: &mut DiagramDocument, factor: f64, viewport_size: (f64, f64)) -> bool {
    let (viewport_width, viewport_height) = viewport_size;
    let center_x = viewport_width / 2.0;
    let center_y = viewport_height / 2.0;

    // Get the canvas coordinates of the viewport center before zoom
    let canvas_x = (center_x - doc.editor_state.camera_x.0) / doc.editor_state.zoom.0;
    let canvas_y = (center_y - doc.editor_state.camera_y.0) / doc.editor_state.zoom.0;

    // Calculate new zoom
    let new_zoom = (doc.editor_state.zoom.0 * factor).clamp(0.1, 4.0);

    // If zoom didn't change, nothing to do
    if (new_zoom - doc.editor_state.zoom.0).abs() < f64::EPSILON {
        return false;
    }

    doc.editor_state.zoom = OrderedFloat(new_zoom);

    // Calculate new camera position to keep the center point stable
    doc.editor_state.camera_x = OrderedFloat(center_x - canvas_x * new_zoom);
    doc.editor_state.camera_y = OrderedFloat(center_y - canvas_y * new_zoom);

    true
}

/// Sets the zoom to a specific value, adjusting camera to keep viewport centered.
fn set_zoom_centered(doc: &mut DiagramDocument, zoom: f64, viewport_size: (f64, f64)) -> bool {
    let (viewport_width, viewport_height) = viewport_size;
    let center_x = viewport_width / 2.0;
    let center_y = viewport_height / 2.0;

    let zoom = zoom.clamp(0.1, 4.0);

    if (zoom - doc.editor_state.zoom.0).abs() < f64::EPSILON {
        return false;
    }

    // Get canvas coordinates of center before zoom change
    let canvas_x = (center_x - doc.editor_state.camera_x.0) / doc.editor_state.zoom.0;
    let canvas_y = (center_y - doc.editor_state.camera_y.0) / doc.editor_state.zoom.0;

    doc.editor_state.zoom = OrderedFloat(zoom);

    // Adjust camera to keep center point stable
    doc.editor_state.camera_x = OrderedFloat(center_x - canvas_x * zoom);
    doc.editor_state.camera_y = OrderedFloat(center_y - canvas_y * zoom);

    true
}
