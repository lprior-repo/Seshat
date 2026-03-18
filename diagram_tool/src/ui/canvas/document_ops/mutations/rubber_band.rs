use crate::ui::interaction::{
    has_drag_threshold, node_ids_in_rect, toggle_selection, with_auto_selected_edges,
};
use diagram_models::document::DiagramDocument;

pub fn apply_rubber_band_release(
    doc: &mut DiagramDocument,
    start: (f64, f64),
    current: (f64, f64),
    additive: bool,
) {
    if !has_drag_threshold(start, current) {
        return;
    }

    let boxed = node_ids_in_rect(doc, start, current);
    let selected = if additive {
        boxed
            .iter()
            .fold(doc.editor_state.selected_items.clone(), |acc, id| {
                toggle_selection(&acc, id)
            })
    } else {
        // Clear existing selection before applying new marquee selection
        doc.editor_state.selected_items.clear();
        boxed
    };
    doc.editor_state.selected_items = with_auto_selected_edges(doc, &selected);
}
