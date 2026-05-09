use diagram_models::{
    dag::validate_dag,
    document::{DiagramDocument, Edge, EdgeId, Node, NodeId, NodeKind},
    port::PortAnchor,
};
use serde_json::Value;
use uuid::Uuid;

const SESHAT_BASE_PATH: &str = env!("SESHAT_BASE_PATH");

use crate::{
    geometry::hit_test_margin,
    icons::icon_index,
    ui::canvas::canvas_view::SCREEN_HIT_MARGIN,
    ui::grid::{snap_value, GridSize},
};

#[cfg(target_arch = "wasm32")]
pub fn sync_canvas_origin() -> Option<(f64, f64)> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let el = document
        .query_selector(".canvas-container")
        .ok()
        .flatten()?;
    let rect = el.get_bounding_client_rect();
    Some((rect.left(), rect.top()))
}

#[cfg(not(target_arch = "wasm32"))]
#[must_use]
pub const fn sync_canvas_origin() -> Option<(f64, f64)> {
    None
}

/// Fallback for provider color mapping
pub fn provider_color(provider: &str) -> &'static str {
    match provider {
        "aws" => "#FF9900",
        "gcp" => "#4285F4",
        "azure" => "#0078D4",
        "k8s" => "#326CE5",
        _ => "#6B7280",
    }
}

pub fn initials(label: &str) -> String {
    let parts = label
        .split(|ch: char| ch.is_whitespace() || ch == '/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    if parts.len() <= 1 {
        return label
            .chars()
            .take(3)
            .collect::<String>()
            .to_ascii_uppercase();
    }

    parts
        .iter()
        .filter_map(|part| part.chars().next())
        .take(3)
        .collect::<String>()
        .to_ascii_uppercase()
}

pub fn icon_tags(icon_key: &str) -> Vec<String> {
    let segments = icon_key.split('/').collect::<Vec<_>>();
    if segments.is_empty() {
        Vec::new()
    } else if segments.len() == 1 {
        vec![segments[0].to_string()]
    } else {
        vec![segments[0].to_string(), segments[1].to_string()]
    }
}

pub fn fallback_icon_label(icon_key: &str) -> String {
    icon_key.split('/').next_back().map_or_else(
        || String::from("Node"),
        |part| {
            let mut chars = part.chars();
            chars.next().map_or_else(String::new, |first| {
                let first_up = first.to_ascii_uppercase();
                format!("{first_up}{}", chars.as_str())
            })
        },
    )
}

pub fn icon_url_for_relpath(file_relpath: &str) -> Option<String> {
    if file_relpath.contains("..") || file_relpath.starts_with('/') {
        return None;
    }
    Some(format!("{SESHAT_BASE_PATH}/resources/{file_relpath}"))
}

pub fn icon_url(icon_key: &str) -> Option<String> {
    icon_index()
        .by_key
        .get(icon_key)
        .and_then(|meta| icon_url_for_relpath(&meta.file_relpath))
        .or_else(|| icon_url_for_relpath(icon_key))
}

pub fn node_image_url(node: &Node) -> Option<String> {
    node.metadata
        .get("icon_url")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| icon_url(&node.icon))
}

pub fn edge_preserves_dag(doc: &DiagramDocument, edge: &Edge) -> bool {
    let candidate_edges = doc
        .document
        .edges
        .update(EdgeId::new(Uuid::new_v4().to_string()), edge.clone());
    validate_dag(&doc.document.nodes, &candidate_edges).is_ok()
}

pub fn ordered_node_ids(doc: &DiagramDocument) -> Vec<NodeId> {
    let mut node_ids = doc.document.nodes.keys().cloned().collect::<Vec<_>>();
    node_ids.sort_by(|a_id, b_id| {
        doc.document
            .nodes
            .get(a_id)
            .zip(doc.document.nodes.get(b_id))
            .map_or(std::cmp::Ordering::Equal, |(a_node, b_node)| {
                let a_layer = i32::from(a_node.kind != NodeKind::Subgraph);
                let b_layer = i32::from(b_node.kind != NodeKind::Subgraph);
                (a_layer, a_node.z_index, a_id.to_string()).cmp(&(
                    b_layer,
                    b_node.z_index,
                    b_id.to_string(),
                ))
            })
    });

    node_ids
}

pub fn find_node_at(doc: &DiagramDocument, x: f64, y: f64) -> Option<NodeId> {
    let zoom = doc.editor_state.zoom.0;
    let hit_margin_world =
        hit_test_margin::screen_to_world_margin(SCREEN_HIT_MARGIN, zoom).unwrap_or(5.0);

    ordered_node_ids(doc)
        .iter()
        .rev()
        .find(|id| {
            doc.document.nodes.get(*id).is_some_and(|node| {
                x >= node.x.0 - hit_margin_world
                    && x <= node.x.0 + node.width.0 + hit_margin_world
                    && y >= node.y.0 - hit_margin_world
                    && y <= node.y.0 + node.height.0 + hit_margin_world
            })
        })
        .cloned()
}

#[must_use]
pub fn node_center(node: &Node) -> (f64, f64) {
    (
        node.x.0 + node.width.0 / 2.0,
        node.y.0 + node.height.0 / 2.0,
    )
}

#[must_use]
pub fn snap_edge_port_toward(node: &Node, toward_x: f64, toward_y: f64) -> PortAnchor {
    let (center_x, center_y) = node_center(node);
    let dx = toward_x - center_x;
    let dy = toward_y - center_y;

    if dx.abs() >= dy.abs() {
        if dx >= 0.0 {
            PortAnchor::Right
        } else {
            PortAnchor::Left
        }
    } else if dy >= 0.0 {
        PortAnchor::Bottom
    } else {
        PortAnchor::Top
    }
}

#[must_use]
pub fn snapped_edge_ports(source: &Node, target: &Node) -> (PortAnchor, PortAnchor) {
    let (source_center_x, source_center_y) = node_center(source);
    let (target_center_x, target_center_y) = node_center(target);
    (
        snap_edge_port_toward(source, target_center_x, target_center_y),
        snap_edge_port_toward(target, source_center_x, source_center_y),
    )
}

pub fn subgraph_release_bounds(
    start: (f64, f64),
    current: (f64, f64),
    snap: bool,
    grid: GridSize,
) -> Option<(f64, f64, f64, f64)> {
    let mut x = start.0.min(current.0);
    let mut y = start.1.min(current.1);
    let mut w = (start.0 - current.0).abs();
    let mut h = (start.1 - current.1).abs();
    let grid_inner = grid.inner();
    if snap {
        x = snap_value(x, true, grid);
        y = snap_value(y, true, grid);
        w = snap_value(w, true, grid).max(grid_inner.max(20.0));
        h = snap_value(h, true, grid).max(grid_inner.max(20.0));
    }

    (w > 20.0 && h > 20.0).then_some((x, y, w, h))
}

pub fn safe_zoom(zoom: f64) -> f64 {
    canvas_domain::math::safe_zoom(zoom).unwrap_or(1.0)
}

pub fn fit_icon_side(side: f64) -> f64 {
    if !side.is_finite() {
        return 0.0;
    }

    let max = (side - 8.0).max(0.0);
    let min = 20.0_f64.min(max);
    let preferred = side * 0.52;

    if !preferred.is_finite() {
        return min;
    }

    preferred.clamp(min, max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::grid::GridSize;

    #[cfg(kani)]
    #[kani::proof]
    fn given_subgraph_release_bounds_when_drag_too_small_then_none() {
        let grid = GridSize::new(20.0).unwrap();
        let result = subgraph_release_bounds((0.0, 0.0), (10.0, 10.0), false, grid);
        assert!(result.is_none());
    }

    #[cfg(kani)]
    #[kani::proof]
    fn given_subgraph_release_bounds_when_drag_valid_then_bounds_returned() {
        let grid = GridSize::new(20.0).unwrap();
        let result = subgraph_release_bounds((5.0, 10.0), (60.0, 70.0), false, grid);
        assert_eq!(result, Some((5.0, 10.0, 55.0, 60.0)));
    }

    #[cfg(kani)]
    #[kani::proof]
    fn given_icon_side_when_too_small_then_fit_never_panics_and_stays_non_negative() {
        let result = fit_icon_side(19.68);
        assert!(result >= 0.0);
        assert!(result <= 11.68);
    }
}

#[cfg(test)]
#[path = "queries_tests.rs"]
mod queries_tests;
