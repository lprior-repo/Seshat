use crate::ui::grid::snap_value;
use canvas_domain::selection_geometry::{selected_node_ids, selection_bounds};
use diagram_models::document::{DiagramDocument, OrderedFloat};

pub fn scale_selected_nodes(doc: &mut DiagramDocument, factor: f64) -> bool {
    let Some((bx, by, bw, bh)) = selection_bounds(doc) else {
        return false;
    };
    let selected = selected_node_ids(doc);
    if selected.is_empty() {
        return false;
    }

    let center_x = bx + (bw / 2.0);
    let center_y = by + (bh / 2.0);
    let snap = doc.editor_state.snap_to_grid;
    let grid = doc.editor_state.grid_size;
    let mut changed = false;

    for node_id in selected {
        if let Some(node) = doc.document.nodes.get_mut(&node_id) {
            if !node.lock_state.is_movable(&node.kind) {
                continue;
            }
            let rel_x = node.x.0 - center_x;
            let rel_y = node.y.0 - center_y;
            let mut next_x = center_x + (rel_x * factor);
            let mut next_y = center_y + (rel_y * factor);
            let mut next_w = (node.width.0 * factor).round().max(24.0);
            let mut next_h = (node.height.0 * factor).round().max(24.0);

            if snap {
                next_x = snap_value(next_x, true, grid);
                next_y = snap_value(next_y, true, grid);
                next_w = snap_value(next_w, true, grid).max(24.0);
                next_h = snap_value(next_h, true, grid).max(24.0);
            }

            node.x = OrderedFloat(next_x);
            node.y = OrderedFloat(next_y);
            node.width = OrderedFloat(next_w);
            node.height = OrderedFloat(next_h);
            changed = true;
        }
    }

    changed
}
