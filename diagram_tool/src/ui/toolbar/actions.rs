use crate::history::History;
use crate::layout::dag::{dag_layout, DagLayoutSettings};
use crate::models::document::DiagramDocument;
use crate::mutation::pipeline::run_mutation;
use crate::ui::commands::{
    apply_delete_selected, apply_redo, apply_undo, apply_zoom_in, apply_zoom_out, apply_zoom_reset,
};
use crate::ui::toast::ToastApi;
use dioxus::prelude::*;

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
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    viewport_size_signal: Signal<(f64, f64)>,
) {
    let viewport_size = *viewport_size_signal.read();
    let _ = apply_zoom_in(doc_signal, history_signal, viewport_size);
}

pub fn zoom_out(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    viewport_size_signal: Signal<(f64, f64)>,
) {
    let viewport_size = *viewport_size_signal.read();
    let _ = apply_zoom_out(doc_signal, history_signal, viewport_size);
}

pub fn zoom_reset(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>) {
    let _ = apply_zoom_reset(doc_signal, history_signal);
}

pub fn delete_selection(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>) {
    let _ = apply_delete_selected(doc_signal, history_signal);
}
