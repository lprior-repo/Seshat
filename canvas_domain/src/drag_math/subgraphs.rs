use canvas_math::within;
use diagram_models::document::NodeId;
use im::HashMap;
use std::collections::HashSet;

#[must_use]
pub fn calculate_resize_target_ids(
    selected_ids: &[NodeId],
    node_geometry: &HashMap<NodeId, (f64, f64, f64, f64, bool)>, // (x, y, w, h, is_subgraph)
) -> Vec<NodeId> {
    let mut selected_set = HashSet::new();
    let mut selected_subgraphs = Vec::new();

    for id in selected_ids {
        selected_set.insert(id.clone());
        if let Some(&(x, y, w, h, is_subgraph)) = node_geometry.get(id) {
            if is_subgraph {
                selected_subgraphs.push((x, y, w, h));
            }
        }
    }

    if selected_subgraphs.is_empty() {
        return selected_ids.to_vec();
    }

    for (id, &(x, y, w, h, _)) in node_geometry {
        let node_rect = (x, y, w, h);
        let included = selected_subgraphs
            .iter()
            .any(|subgraph_rect| within(*subgraph_rect, node_rect));

        if included {
            selected_set.insert(id.clone());
        }
    }

    selected_set.into_iter().collect()
}

