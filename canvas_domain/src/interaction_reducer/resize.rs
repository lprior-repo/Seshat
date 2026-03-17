#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use dioxus::prelude::*;
use im::HashMap;

use super::geometry::{resize_target_ids, safe_zoom};
use super::types::{InteractionMode, ResizeHandle};
use crate::selection_geometry::selection_bounds;
use diagram_models::document::DiagramDocument;

pub fn start_resize_interaction(
    mut interaction_mode: Signal<InteractionMode>,
    doc_signal: Signal<DiagramDocument>,
    handle: ResizeHandle,
    client_x: f64,
    client_y: f64,
    aspect_lock_enabled: bool,
) {
    let doc = doc_signal.read().clone();
    if let Some(bounds) = selection_bounds(&doc) {
        let Some(zoom) = safe_zoom(doc.editor_state.zoom.0) else {
            return;
        };
        let cx = (client_x / zoom) + doc.editor_state.camera_x.0;
        let cy = (client_y / zoom) + doc.editor_state.camera_y.0;

        let originals = resize_target_ids(&doc)
            .into_iter()
            .fold(HashMap::new(), |acc, id| {
                if let Some(n) = doc.document.nodes.get(&id) {
                    acc.update(id, (n.x.0, n.y.0, n.width.0, n.height.0))
                } else {
                    acc
                }
            });

        let aspect_ratio = if aspect_lock_enabled && bounds.2 > 0.0 && bounds.3 > 0.0 {
            Some(bounds.2 / bounds.3)
        } else {
            None
        };

        interaction_mode.set(InteractionMode::ResizingSelection {
            handle,
            original_bounds: bounds,
            originals,
            anchor: (cx, cy),
            did_resize: false,
            aspect_ratio,
        });
    }
}
