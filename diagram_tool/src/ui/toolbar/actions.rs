use crate::history::History;
use crate::layout::dag::{dag_layout, DagLayoutSettings};
use crate::models::document::DiagramDocument;
use crate::mutation::pipeline::run_mutation;
use crate::ui::commands::{
    apply_align_selection, apply_bring_forward, apply_bring_to_front, apply_copy_selection,
    apply_delete_selected, apply_distribute_selection, apply_paste_selection, apply_redo,
    apply_send_backward, apply_send_to_back, apply_undo, apply_zoom_in, apply_zoom_out,
    apply_zoom_reset, clipboard_has_content, AlignmentAxis, AlignmentMode, DistributionAxis,
    Clipboard,
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

pub fn zoom_reset(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
    viewport_size_signal: Signal<(f64, f64)>,
) {
    let viewport_size = *viewport_size_signal.read();
    let _ = apply_zoom_reset(doc_signal, history_signal, viewport_size);
}

pub fn delete_selection(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>) {
    let _ = apply_delete_selected(doc_signal, history_signal);
}

pub fn copy_selection(doc_signal: Signal<DiagramDocument>) {
    let clipboard_signal = use_context::<Signal<Option<Clipboard>>>();
    let _ = apply_copy_selection(doc_signal, clipboard_signal);
}

pub fn paste_selection(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>) {
    let clipboard_signal = use_context::<Signal<Option<Clipboard>>>();
    let _ = apply_paste_selection(doc_signal, clipboard_signal, history_signal);
}

pub fn can_paste() -> bool {
    let clipboard_signal = use_context::<Signal<Option<Clipboard>>>();
    let has_content = clipboard_has_content(&clipboard_signal.read());
    has_content
}

pub fn bring_forward(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>) {
    let _ = apply_bring_forward(doc_signal, history_signal);
}

pub fn send_backward(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>) {
    let _ = apply_send_backward(doc_signal, history_signal);
}

pub fn bring_to_front(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>) {
    let _ = apply_bring_to_front(doc_signal, history_signal);
}

pub fn send_to_back(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>) {
    let _ = apply_send_to_back(doc_signal, history_signal);
}

pub fn toggle_grid(mut doc_signal: Signal<DiagramDocument>) {
    doc_signal.with_mut(|doc| {
        doc.editor_state.show_grid = !doc.editor_state.show_grid;
    });
}

// Alignment actions

pub fn align_left(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>) {
    let _ = apply_align_selection(
        doc_signal,
        history_signal,
        AlignmentAxis::Horizontal,
        AlignmentMode::Start,
    );
}

pub fn align_center_horizontal(
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
) {
    let _ = apply_align_selection(
        doc_signal,
        history_signal,
        AlignmentAxis::Horizontal,
        AlignmentMode::Center,
    );
}

pub fn align_right(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>) {
    let _ = apply_align_selection(
        doc_signal,
        history_signal,
        AlignmentAxis::Horizontal,
        AlignmentMode::End,
    );
}

pub fn align_top(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>) {
    let _ = apply_align_selection(
        doc_signal,
        history_signal,
        AlignmentAxis::Vertical,
        AlignmentMode::Start,
    );
}

pub fn align_middle_vertical(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>) {
    let _ = apply_align_selection(
        doc_signal,
        history_signal,
        AlignmentAxis::Vertical,
        AlignmentMode::Center,
    );
}

pub fn align_bottom(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>) {
    let _ = apply_align_selection(
        doc_signal,
        history_signal,
        AlignmentAxis::Vertical,
        AlignmentMode::End,
    );
}

// Distribution actions

pub fn distribute_horizontal(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>) {
    let _ = apply_distribute_selection(doc_signal, history_signal, DistributionAxis::Horizontal);
}

pub fn distribute_vertical(doc_signal: Signal<DiagramDocument>, history_signal: Signal<History>) {
    let _ = apply_distribute_selection(doc_signal, history_signal, DistributionAxis::Vertical);
}
