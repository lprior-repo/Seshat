use crate::ui::canvas::canvas_view::edge::{edge_endpoints, edge_geometry, EdgeGeometry};
use crate::ui::canvas::canvas_view::geometry::{dist_to_segment, quadratic_bezier_point};
use diagram_models::document::{DiagramDocument, EdgeId};

const EDGE_SCREEN_HIT_RADIUS: f64 = 24.0;
const EDGE_ENDPOINT_HIT_RADIUS: f64 = 28.0;

pub fn find_edge_at(doc: &DiagramDocument, x: f64, y: f64) -> Option<EdgeId> {
    // Screen-consistent hit radius: 17.0 screen pixels scaled to world coordinates
    // This ensures hit testing behaves consistently regardless of zoom level
    let zoom = doc.editor_state.zoom.0;
    // Use safe_zoom to prevent division by zero or invalid zoom values
    let safe_zoom = canvas_domain::math::safe_zoom(zoom).unwrap_or(1.0);
    let hit_radius_world = EDGE_SCREEN_HIT_RADIUS / safe_zoom;
    let endpoint_hit_radius_world = EDGE_ENDPOINT_HIT_RADIUS / safe_zoom;
    doc.document
        .edges
        .iter()
        .filter_map(|(id, edge)| {
            doc.document
                .nodes
                .get(&edge.source)
                .zip(doc.document.nodes.get(&edge.target))
                .and_then(|(source, target)| {
                    let ((sx, sy), (tx, ty)) = edge_endpoints(edge, source, target);
                    let hit_distance = match edge_geometry(sx, sy, tx, ty, edge) {
                        EdgeGeometry::Quadratic { control: (cx, cy) } => {
                            (1_u32..=32)
                                .fold((f64::MAX, (sx, sy)), |(min_dist, prev), step| {
                                    let t = f64::from(step) / 32.0;
                                    let curr =
                                        quadratic_bezier_point((sx, sy), (cx, cy), (tx, ty), t);
                                    (
                                        min_dist.min(dist_to_segment(
                                            x, y, prev.0, prev.1, curr.0, curr.1,
                                        )),
                                        curr,
                                    )
                                })
                                .0
                        }
                        EdgeGeometry::Polyline(points) => points
                            .windows(2)
                            .map(|window| {
                                dist_to_segment(
                                    x,
                                    y,
                                    window[0].0,
                                    window[0].1,
                                    window[1].0,
                                    window[1].1,
                                )
                            })
                            .fold(f64::MAX, f64::min),
                    };
                    let endpoint_distance = dist_to_segment(x, y, sx, sy, sx, sy)
                        .min(dist_to_segment(x, y, tx, ty, tx, ty));
                    (hit_distance < hit_radius_world
                        || endpoint_distance < endpoint_hit_radius_world)
                        .then(|| (id.clone(), hit_distance))
                })
        })
        .min_by(|(a_id, a_dist), (b_id, b_dist)| {
            a_dist
                .total_cmp(b_dist)
                .then_with(|| a_id.as_str().cmp(b_id.as_str()))
        })
        .map(|(id, _)| id)
}
