use crate::document::DiagramDocument;
use crate::geometry::Point;
use crate::selection::types::{ElementId, SelectionError};

fn is_element_visible(metadata: &im::HashMap<String, serde_json::Value>) -> bool {
    metadata
        .get("visibility")
        .and_then(serde_json::Value::as_str)
        != Some("hidden")
}

pub fn hit_test(
    point: &Point,
    document: &DiagramDocument,
) -> Result<Option<ElementId>, SelectionError> {
    use itertools::Itertools;

    let px = point.x;
    let py = point.y;

    let hit = document
        .document
        .nodes
        .iter()
        .sorted_by_key(|(_, n)| -n.z_index)
        .find(|(_, n)| {
            if !is_element_visible(&n.metadata) {
                return false;
            }

            let nx = n.x.0;
            let ny = n.y.0;
            let nw = n.width.0;
            let nh = n.height.0;

            px >= nx && px <= nx + nw && py >= ny && py <= ny + nh
        })
        .map(|(id, _)| ElementId::Node(id.clone()));

    Ok(hit)
}
