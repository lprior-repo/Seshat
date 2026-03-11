use crate::geometry::primitives::{Point, Rectangle, AABB};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum BoundsError {
    #[error("Invalid coordinate: NaN or Infinity")]
    InvalidCoordinate,
}

/// Creates a bounding box safely.
///
/// # Errors
/// Returns an error if any coordinate is NaN or infinite.
pub fn safe_bounds(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Result<AABB, BoundsError> {
    if min_x.is_nan()
        || min_y.is_nan()
        || max_x.is_nan()
        || max_y.is_nan()
        || min_x.is_infinite()
        || min_y.is_infinite()
        || max_x.is_infinite()
        || max_y.is_infinite()
    {
        return Err(BoundsError::InvalidCoordinate);
    }

    let final_min_x = min_x.min(max_x);
    let final_max_x = min_x.max(max_x);
    let final_min_y = min_y.min(max_y);
    let final_max_y = min_y.max(max_y);

    Ok(AABB::new(
        final_min_x,
        final_min_y,
        final_max_x,
        final_max_y,
    ))
}

#[must_use]
pub fn zoom_at_pointer(view_center: Point, pointer: Point, factor: f64) -> Point {
    Point::new(
        (view_center.x - pointer.x).mul_add(factor, pointer.x),
        (view_center.y - pointer.y).mul_add(factor, pointer.y),
    )
}

#[must_use]
pub fn snap_horizontal(line_y: f64, targets: &[f64], tolerance: f64) -> Option<f64> {
    targets
        .iter()
        .map(|&t| (t, (line_y - t).abs()))
        .filter(|(_, dist)| *dist <= tolerance)
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(t, _)| t)
}

#[must_use]
pub fn snap_vertical(line_x: f64, targets: &[f64], tolerance: f64) -> Option<f64> {
    snap_horizontal(line_x, targets, tolerance)
}

#[must_use]
pub fn hit_test_rect(point: Point, rect: &Rectangle, margin: f64) -> bool {
    let aabb = rect.aabb();
    point.x >= aabb.min_x - margin
        && point.x <= aabb.max_x + margin
        && point.y >= aabb.min_y - margin
        && point.y <= aabb.max_y + margin
}

#[must_use]
pub fn hit_test_rotated_rect(point: Point, rect: &Rectangle) -> bool {
    if rect.rotation == 0.0 {
        return hit_test_rect(point, rect, 0.0);
    }
    let center = rect.aabb().center();
    let local_point =
        crate::geometry::transforms::rotate_around_center(point, center, -rect.rotation);
    let local_rect = Rectangle::new(rect.x, rect.y, rect.width, rect.height);
    hit_test_rect(local_point, &local_rect, 0.0)
}

#[must_use]
pub fn world_to_screen(world: Point, camera: Point, zoom: f64) -> Point {
    Point::new((world.x - camera.x) * zoom, (world.y - camera.y) * zoom)
}

#[must_use]
pub fn screen_to_world(screen: Point, camera: Point, zoom: f64) -> Point {
    Point::new(screen.x / zoom + camera.x, screen.y / zoom + camera.y)
}

#[must_use]
pub fn selection_center(points: &[Point]) -> Point {
    if points.is_empty() {
        return Point::origin();
    }
    let sum_x: f64 = points.iter().map(|p| p.x).sum();
    let sum_y: f64 = points.iter().map(|p| p.y).sum();
    let count = points.len() as f64;
    Point::new(sum_x / count, sum_y / count)
}

/// Computes the bounding box of a container based on its children's bounds.
///
/// # Parameters
/// - `children`: An iterator of (x, y, width, height) tuples representing child nodes
///
/// # Returns
/// - `Some((x, y, width, height))`: The computed bounds if children exist and are valid
/// - `None`: If there are no children or all children have invalid (NaN/Infinity) coordinates
///
/// # Contract (KIRK-001)
/// - Returns bounds that encompass ALL children geometrically
/// - Returns None if children list is empty
/// - Bounds are minimal (tight fit to children)
/// - All coordinate values are finite (not NaN/Infinity)
#[must_use]
pub fn compute_subgraph_bounds(
    children: impl IntoIterator<Item = (f64, f64, f64, f64)>,
) -> Option<(f64, f64, f64, f64)> {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut has_valid_child = false;

    for (x, y, width, height) in children {
        // Skip invalid child bounds (NaN or Infinity)
        if !x.is_finite() || !y.is_finite() || !width.is_finite() || !height.is_finite() {
            continue;
        }

        has_valid_child = true;
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x + width);
        max_y = max_y.max(y + height);
    }

    if !has_valid_child {
        return None;
    }

    // Verify final bounds are valid
    if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
        return None;
    }

    Some((min_x, min_y, max_x - min_x, max_y - min_y))
}

#[cfg(test)]
mod subgraph_bounds_tests {
    use super::*;

    // ============== GEO-025: Container Bounds Recomputation ==============

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_single_child_container() {
        // Given: a subgraph container with one child
        let children = vec![(100.0, 100.0, 50.0, 30.0)];

        // When: computing subgraph bounds
        let result = compute_subgraph_bounds(children);

        // Then: result should be the child's bounds
        assert_eq!(result, Some((100.0, 100.0, 50.0, 30.0)));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_multiple_children_horizontal_spread() {
        // Given: children spread horizontally
        let children = vec![
            (0.0, 0.0, 50.0, 50.0),
            (100.0, 0.0, 50.0, 50.0),
            (200.0, 0.0, 50.0, 50.0),
        ];

        // When: computing subgraph bounds
        let result = compute_subgraph_bounds(children);

        // Then: result should encompass all children
        assert_eq!(result, Some((0.0, 0.0, 250.0, 50.0)));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_multiple_children_vertical_spread() {
        // Given: children spread vertically
        let children = vec![
            (0.0, 0.0, 50.0, 50.0),
            (0.0, 100.0, 50.0, 50.0),
            (0.0, 200.0, 50.0, 50.0),
        ];

        // When: computing subgraph bounds
        let result = compute_subgraph_bounds(children);

        // Then: result should encompass all children
        assert_eq!(result, Some((0.0, 0.0, 50.0, 250.0)));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_empty_container() {
        // Given: no children
        let children: Vec<(f64, f64, f64, f64)> = vec![];

        // When: computing subgraph bounds
        let result = compute_subgraph_bounds(children);

        // Then: result should be None
        assert_eq!(result, None);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_child_with_negative_coordinates() {
        // Given: children with negative coordinates
        let children = vec![(-50.0, -50.0, 50.0, 50.0), (0.0, 0.0, 50.0, 50.0)];

        // When: computing subgraph bounds
        let result = compute_subgraph_bounds(children);

        // Then: result should handle negative coords
        assert_eq!(result, Some((-50.0, -50.0, 100.0, 100.0)));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_child_overlap() {
        // Given: overlapping children
        let children = vec![(0.0, 0.0, 100.0, 100.0), (50.0, 50.0, 100.0, 100.0)];

        // When: computing subgraph bounds
        let result = compute_subgraph_bounds(children);

        // Then: result should be the union
        assert_eq!(result, Some((0.0, 0.0, 150.0, 150.0)));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_invalid_child_nan() {
        // Given: one valid child and one with NaN coordinates
        let children = vec![(0.0, 0.0, 50.0, 50.0), (f64::NAN, 0.0, 50.0, 50.0)];

        // When: computing subgraph bounds
        let result = compute_subgraph_bounds(children);

        // Then: result should contain only valid child
        assert_eq!(result, Some((0.0, 0.0, 50.0, 50.0)));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_invalid_child_infinity() {
        // Given: one valid child and one with Infinity coordinates
        let children = vec![(0.0, 0.0, 50.0, 50.0), (f64::INFINITY, 0.0, 50.0, 50.0)];

        // When: computing subgraph bounds
        let result = compute_subgraph_bounds(children);

        // Then: result should contain only valid child
        assert_eq!(result, Some((0.0, 0.0, 50.0, 50.0)));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_single_point_child() {
        // Given: child with zero size
        let children = vec![(50.0, 50.0, 0.0, 0.0)];

        // When: computing subgraph bounds
        let result = compute_subgraph_bounds(children);

        // Then: result should be degenerate but valid
        assert_eq!(result, Some((50.0, 50.0, 0.0, 0.0)));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_all_children_at_same_position() {
        // Given: multiple children at same position with different sizes
        let children = vec![
            (50.0, 50.0, 30.0, 30.0),
            (50.0, 50.0, 50.0, 50.0),
            (50.0, 50.0, 20.0, 40.0),
        ];

        // When: computing subgraph bounds
        let result = compute_subgraph_bounds(children);

        // Then: result should be the union
        assert_eq!(result, Some((50.0, 50.0, 50.0, 50.0)));
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    use crate::geometry::primitives::{Point, Rectangle};

    #[kani::proof]
    fn verify_safe_bounds() {
        let min_x: f64 = kani::any();
        let min_y: f64 = kani::any();
        let max_x: f64 = kani::any();
        let max_y: f64 = kani::any();

        kani::assume(min_x.is_finite());
        kani::assume(min_y.is_finite());
        kani::assume(max_x.is_finite());
        kani::assume(max_y.is_finite());

        let result = safe_bounds(min_x, min_y, max_x, max_y);
        assert!(result.is_ok());
        let aabb = result.unwrap();

        // Invariant: min <= max
        assert!(aabb.min_x <= aabb.max_x);
        assert!(aabb.min_y <= aabb.max_y);

        // Invariant: coordinates are preserved
        assert!(aabb.min_x == min_x.min(max_x));
        assert!(aabb.max_x == min_x.max(max_x));
        assert!(aabb.min_y == min_y.min(max_y));
        assert!(aabb.max_y == min_y.max(max_y));
    }

    #[kani::proof]
    fn verify_zoom_at_pointer() {
        let cx: f64 = kani::any();
        let cy: f64 = kani::any();
        let px: f64 = kani::any();
        let py: f64 = kani::any();
        let factor: f64 = kani::any();

        kani::assume(cx.is_finite());
        kani::assume(cy.is_finite());
        kani::assume(px.is_finite());
        kani::assume(py.is_finite());
        kani::assume(factor.is_finite());

        let view_center = Point::new(cx, cy);
        let pointer = Point::new(px, py);

        let new_center = zoom_at_pointer(view_center, pointer, factor);

        // Invariant: if factor is 1, center doesn't change
        if factor == 1.0 {
            assert!(new_center.x == cx);
            assert!(new_center.y == cy);
        }

        // Invariant: if pointer is at center, center doesn't change
        if cx == px && cy == py {
            assert!(new_center.x == cx);
            assert!(new_center.y == cy);
        }
    }

    #[kani::proof]
    fn verify_snap_horizontal() {
        let line_y: f64 = kani::any();
        let target1: f64 = kani::any();
        let target2: f64 = kani::any();
        let tolerance: f64 = kani::any();

        kani::assume(line_y.is_finite());
        kani::assume(target1.is_finite());
        kani::assume(target2.is_finite());
        kani::assume(tolerance.is_finite());
        kani::assume(tolerance >= 0.0);

        let targets = [target1, target2];
        let result = snap_horizontal(line_y, &targets, tolerance);

        if let Some(snapped) = result {
            // Must have snapped to one of our targets
            assert!(snapped == target1 || snapped == target2);
            // Must be within tolerance
            assert!((line_y - snapped).abs() <= tolerance);
        }
    }

    #[kani::proof]
    fn verify_compute_subgraph_bounds() {
        let x1: f64 = kani::any();
        let y1: f64 = kani::any();
        let w1: f64 = kani::any();
        let h1: f64 = kani::any();

        let x2: f64 = kani::any();
        let y2: f64 = kani::any();
        let w2: f64 = kani::any();
        let h2: f64 = kani::any();

        kani::assume(x1.is_finite() && y1.is_finite() && w1.is_finite() && h1.is_finite());
        kani::assume(x2.is_finite() && y2.is_finite() && w2.is_finite() && h2.is_finite());
        kani::assume(w1 >= 0.0 && h1 >= 0.0);
        kani::assume(w2 >= 0.0 && h2 >= 0.0);

        let children = [(x1, y1, w1, h1), (x2, y2, w2, h2)];
        let result = compute_subgraph_bounds(children);

        assert!(result.is_some());
        if let Some((bx, by, bw, bh)) = result {
            // Bounds must encompass child 1
            assert!(bx <= x1);
            assert!(by <= y1);
            assert!(bx + bw >= x1 + w1);
            assert!(by + bh >= y1 + h1);

            // Bounds must encompass child 2
            assert!(bx <= x2);
            assert!(by <= y2);
            assert!(bx + bw >= x2 + w2);
            assert!(by + bh >= y2 + h2);

            // Dimensions must be non-negative
            assert!(bw >= 0.0);
            assert!(bh >= 0.0);
        }
    }

    #[kani::proof]
    fn verify_world_screen_conversion() {
        let world_x: f64 = kani::any();
        let world_y: f64 = kani::any();
        let cam_x: f64 = kani::any();
        let cam_y: f64 = kani::any();
        let zoom: f64 = kani::any();

        kani::assume(world_x.is_finite());
        kani::assume(world_y.is_finite());
        kani::assume(cam_x.is_finite());
        kani::assume(cam_y.is_finite());
        kani::assume(zoom.is_finite() && zoom > 0.0);

        let world = Point::new(world_x, world_y);
        let camera = Point::new(cam_x, cam_y);

        let screen = world_to_screen(world, camera, zoom);

        // Verify invariant: if zoom is 1 and camera is at origin, world == screen
        if zoom == 1.0 && cam_x == 0.0 && cam_y == 0.0 {
            assert!(screen.x == world_x);
            assert!(screen.y == world_y);
        }
    }

    #[kani::proof]
    fn verify_selection_center() {
        let x1: f64 = kani::any();
        let y1: f64 = kani::any();
        let x2: f64 = kani::any();
        let y2: f64 = kani::any();

        kani::assume(x1.is_finite() && y1.is_finite());
        kani::assume(x2.is_finite() && y2.is_finite());

        let points = [Point::new(x1, y1), Point::new(x2, y2)];
        let center = selection_center(&points);

        assert!(center.x.is_finite());
        assert!(center.y.is_finite());
        assert!(center.x == (x1 + x2) / 2.0);
        assert!(center.y == (y1 + y2) / 2.0);
    }

    #[kani::proof]
    fn verify_hit_test_rect() {
        let px: f64 = kani::any();
        let py: f64 = kani::any();
        let rx: f64 = kani::any();
        let ry: f64 = kani::any();
        let rw: f64 = kani::any();
        let rh: f64 = kani::any();
        let margin: f64 = kani::any();

        kani::assume(px.is_finite() && py.is_finite());
        kani::assume(rx.is_finite() && ry.is_finite());
        kani::assume(rw.is_finite() && rh.is_finite());
        kani::assume(rw >= 0.0 && rh >= 0.0);
        kani::assume(margin.is_finite() && margin >= 0.0);

        let point = Point::new(px, py);
        let rect = Rectangle::new(rx, ry, rw, rh);

        let is_hit = hit_test_rect(point, &rect, margin);

        // Invariant: if point is exactly the center, it should hit
        let cx = rx + rw / 2.0;
        let cy = ry + rh / 2.0;
        if px == cx && py == cy {
            assert!(is_hit);
        }
    }
}
