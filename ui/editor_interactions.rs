pub const NODE_WIDTH: f32 = 220.0;
pub const NODE_HEIGHT: f32 = 68.0;
pub type SelectionRect = (f32, f32, f32, f32);

#[must_use]
pub const fn normalize_rect(start: (f32, f32), end: (f32, f32)) -> SelectionRect {
    let min_x = start.0.min(end.0);
    let min_y = start.1.min(end.1);
    let max_x = start.0.max(end.0);
    let max_y = start.1.max(end.1);
    (min_x, min_y, max_x, max_y)
}

#[must_use]
#[allow(clippy::suspicious_operation_groupings)]
pub fn rect_contains(rect: SelectionRect, point: (f32, f32)) -> bool {
    point.0 >= rect.0 && point.0 <= rect.2 && point.1 >= rect.1 && point.1 <= rect.3
}

#[must_use]
pub fn node_intersects_rect(node_x: f32, node_y: f32, rect: SelectionRect) -> bool {
    let node_left = node_x;
    let node_top = node_y;
    let node_right = node_x + NODE_WIDTH;
    let node_bottom = node_y + NODE_HEIGHT;

    !(node_right < rect.0 || node_left > rect.2 || node_bottom < rect.1 || node_top > rect.3)
}

#[must_use]
pub fn snap_handle(
    nodes: &[crate::graph::Node],
    mx: f32,
    my: f32,
    viewport: &crate::graph::Viewport,
) -> Option<(crate::graph::NodeId, String, crate::ui::edges::Position)> {
    const SNAP_RADIUS: f32 = 24.0;
    let radius_sq = SNAP_RADIUS * SNAP_RADIUS;

    if !viewport.zoom.is_finite() || viewport.zoom.abs() <= f32::EPSILON {
        return None;
    }

    let canvas_x = (mx - viewport.x) / viewport.zoom;
    let canvas_y = (my - viewport.y) / viewport.zoom;

    let mut best: Option<(
        crate::graph::NodeId,
        String,
        crate::ui::edges::Position,
        f32,
    )> = None;

    for node in nodes {
        let handle_y = node.y + NODE_HEIGHT / 2.0;
        let candidates = [
            (
                "target",
                crate::ui::edges::Position {
                    x: node.x,
                    y: handle_y,
                },
            ),
            (
                "source",
                crate::ui::edges::Position {
                    x: node.x + NODE_WIDTH,
                    y: handle_y,
                },
            ),
        ];

        for (kind, pos) in candidates {
            let dx = canvas_x - pos.x;
            let dy = canvas_y - pos.y;
            let dist_sq = dx.mul_add(dx, dy * dy);

            if dist_sq > radius_sq {
                continue;
            }

            let should_replace = match &best {
                Some((_, _, _, best_dist)) => dist_sq < *best_dist,
                None => true,
            };
            if should_replace {
                best = Some((node.id, kind.to_string(), pos, dist_sq));
            }
        }
    }

    best.map(|(node_id, handle_kind, position, _)| (node_id, handle_kind, position))
}

#[cfg(test)]
mod tests {
    use super::{node_intersects_rect, normalize_rect, rect_contains, snap_handle};
    use crate::graph::{Viewport, Workflow};

    #[test]
    fn given_drag_points_when_normalizing_then_rect_bounds_are_ordered() {
        let rect = normalize_rect((120.0, 30.0), (20.0, 90.0));

        assert_eq!(rect, (20.0, 30.0, 120.0, 90.0));
    }

    #[test]
    fn given_rect_boundary_point_when_checking_contains_then_point_is_inside() {
        let rect = (10.0, 10.0, 20.0, 20.0);

        assert!(rect_contains(rect, (10.0, 20.0)));
    }

    #[test]
    fn given_node_overlapping_selection_when_checking_intersection_then_it_is_detected() {
        let intersects = node_intersects_rect(50.0, 50.0, (0.0, 0.0, 100.0, 100.0));

        assert!(intersects);
    }

    #[test]
    fn given_invalid_zoom_when_snapping_handle_then_no_handle_is_returned() {
        let mut workflow = Workflow::new();
        let _ = workflow.add_node("http-handler", 200.0, 200.0);

        let result = snap_handle(
            &workflow.nodes,
            200.0,
            200.0,
            &Viewport {
                x: 0.0,
                y: 0.0,
                zoom: 0.0,
            },
        );

        assert!(result.is_none());
    }

    #[test]
    fn given_cursor_near_closest_handle_when_snapping_then_closest_handle_is_selected() {
        let mut workflow = Workflow::new();
        let first_id = workflow.add_node("first", 100.0, 100.0);
        let _ = workflow.add_node("second", 300.0, 300.0);

        let viewport = Viewport {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
        };

        let snapped = snap_handle(&workflow.nodes, 318.0, 134.0, &viewport);

        assert!(snapped.is_some());

        match snapped {
            Some((node_id, handle_kind, _)) => {
                assert_eq!(node_id, first_id);
                assert_eq!(handle_kind, "source");
            }
            None => assert!(false, "closest handle should be detected"),
        }
    }
}
