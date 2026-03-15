use crate::geometry::primitives::AABB;
use crate::models::document::DiagramDocument;
use crate::models::spatial_index::{build_spatial_index, gather_candidates};
use crate::ui::interaction::types::{SelectionMode, DRAG_THRESHOLD_PX};
use im::HashSet;

#[must_use]
pub const fn selection_mode_from_drag(start: (f64, f64), current: (f64, f64)) -> SelectionMode {
    if current.0 < start.0 {
        SelectionMode::Intersect
    } else {
        SelectionMode::Contain
    }
}

#[must_use]
pub fn has_drag_threshold(origin: (f64, f64), current: (f64, f64)) -> bool {
    let dx = current.0 - origin.0;
    let dy = current.1 - origin.1;
    (dx.mul_add(dx, dy * dy)).sqrt() >= DRAG_THRESHOLD_PX
}

#[must_use]
pub fn select_single(item_id: String) -> HashSet<String> {
    HashSet::new().update(item_id)
}

#[must_use]
pub fn toggle_selection(current: &HashSet<String>, item_id: &str) -> HashSet<String> {
    if current.contains(item_id) {
        current.without(item_id)
    } else {
        current.update(item_id.to_string())
    }
}

#[must_use]
pub fn node_ids_in_rect(
    doc: &DiagramDocument,
    start: (f64, f64),
    current: (f64, f64),
) -> HashSet<String> {
    node_ids_in_rect_with_mode(
        doc,
        start,
        current,
        selection_mode_from_drag(start, current),
    )
}

#[must_use]
pub fn node_ids_in_rect_with_mode(
    doc: &DiagramDocument,
    start: (f64, f64),
    current: (f64, f64),
    mode: SelectionMode,
) -> HashSet<String> {
    let min_x = start.0.min(current.0);
    let min_y = start.1.min(current.1);
    let max_x = start.0.max(current.0);
    let max_y = start.1.max(current.1);

    let index = build_spatial_index(&doc.document.nodes);
    let marquee_aabb = AABB::new(min_x, min_y, max_x, max_y);
    let candidates = gather_candidates(&index, &marquee_aabb);

    candidates
        .iter()
        .filter_map(|id| doc.document.nodes.get(id).map(|n| (id, n)))
        .filter(|(_, n)| !n.lock_state.is_locked())
        .filter(|(_, n)| match mode {
            SelectionMode::Contain => {
                n.x.0 >= min_x
                    && n.y.0 >= min_y
                    && n.x.0 + n.width.0 <= max_x
                    && n.y.0 + n.height.0 <= max_y
            }
            SelectionMode::Intersect => {
                let node_max_x = n.x.0 + n.width.0;
                let node_max_y = n.y.0 + n.height.0;
                n.x.0 < max_x && node_max_x > min_x && n.y.0 < max_y && node_max_y > min_y
            }
        })
        .map(|(id, _)| id.to_string())
        .collect()
}
