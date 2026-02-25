use crate::history::History;
use crate::layout::dag::{dag_layout, DagLayoutSettings};
use crate::models::document::DiagramDocument;
use crate::mutation::pipeline::run_mutation;
use crate::ui::commands::{apply_delete_selected, apply_redo, apply_undo};
use crate::ui::toast::ToastApi;
use dioxus::prelude::*;

fn zoom_to_center(doc: &mut DiagramDocument, factor: f64, viewport_size: (f64, f64)) {
    let old_zoom = doc.editor_state.zoom.0;
    let new_zoom = (old_zoom * factor).clamp(0.1, 4.0);
    if (new_zoom - old_zoom).abs() < f64::EPSILON {
        return;
    }

    let viewport_w = viewport_size.0.max(1.0);
    let viewport_h = viewport_size.1.max(1.0);
    let center_world_x = ((viewport_w / 2.0) - doc.editor_state.camera_x.0) / old_zoom;
    let center_world_y = ((viewport_h / 2.0) - doc.editor_state.camera_y.0) / old_zoom;

    doc.editor_state.camera_x.0 = (viewport_w / 2.0) - (center_world_x * new_zoom);
    doc.editor_state.camera_y.0 = (viewport_h / 2.0) - (center_world_y * new_zoom);
    doc.editor_state.zoom.0 = new_zoom;
}

pub fn auto_layout(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    toast: ToastApi,
) {
    let current_doc = doc_signal.read().clone();
    match run_mutation(&current_doc, |doc| {
        Ok(dag_layout(doc, &DagLayoutSettings::default()))
    }) {
        Ok(next_doc) => {
            let history = history_signal.read().clone();
            *history_signal.write() = history.push(current_doc);
            *doc_signal.write() = next_doc;
        }
        Err(err) => {
            let _ = toast.error(
                "Auto-arrange failed",
                Some(format!("Code: {}", super::mutation_error_code(&err))),
            );
        }
    }
}

pub fn undo(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>) {
    apply_undo(doc_signal, history_signal);
}

pub fn redo(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>) {
    apply_redo(doc_signal, history_signal);
}

pub fn zoom_in(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    viewport_size_signal: Signal<(f64, f64)>,
) {
    let viewport_size = *viewport_size_signal.read();
    let current = doc_signal.read().clone();
    let history = history_signal.read().clone();
    *history_signal.write() = history.push(current);
    doc_signal.with_mut(|doc| {
        zoom_to_center(doc, 1.25, viewport_size);
        doc.revision = doc.revision.increment();
    });
}

pub fn zoom_out(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    viewport_size_signal: Signal<(f64, f64)>,
) {
    let viewport_size = *viewport_size_signal.read();
    let current = doc_signal.read().clone();
    let history = history_signal.read().clone();
    *history_signal.write() = history.push(current);
    doc_signal.with_mut(|doc| {
        zoom_to_center(doc, 0.8, viewport_size);
        doc.revision = doc.revision.increment();
    });
}

pub fn zoom_reset(mut doc_signal: Signal<DiagramDocument>, mut history_signal: Signal<History>) {
    let current = doc_signal.read().clone();
    let history = history_signal.read().clone();
    *history_signal.write() = history.push(current);
    doc_signal.with_mut(|doc| {
        doc.editor_state.zoom.0 = 1.0;
        doc.revision = doc.revision.increment();
    });
}

pub fn delete_selection(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>) {
    let _ = apply_delete_selected(doc_signal, history_signal);
}
