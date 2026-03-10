use crate::geometry::primitives::{Point, Rectangle, AABB};

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum BoundsError {
    #[error("Invalid coordinate: NaN or Infinity")]
    InvalidCoordinate,
}

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

    let (final_min_x, final_max_x) = if min_x <= max_x {
        (min_x, max_x)
    } else {
        (max_x, min_x)
    };
    let (final_min_y, final_max_y) = if min_y <= max_y {
        (min_y, max_y)
    } else {
        (max_y, min_y)
    };

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
        pointer.x + (view_center.x - pointer.x) * factor,
        pointer.y + (view_center.y - pointer.y) * factor,
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

#[derive(Debug, Clone, PartialEq)]
pub struct OrthogonalRoute {
    pub points: Vec<Point>,
}

#[must_use]
pub fn orthogonal_route(from: Point, to: Point) -> OrthogonalRoute {
    let tolerance = 1e-10;
    if (from.x - to.x).abs() < tolerance {
        OrthogonalRoute {
            points: vec![from, to],
        }
    } else if (from.y - to.y).abs() < tolerance {
        OrthogonalRoute {
            points: vec![from, to],
        }
    } else {
        let mid = Point::new(to.x, from.y);
        OrthogonalRoute {
            points: vec![from, mid, to],
        }
    }
}

pub fn segment_intersects_aabb(p1: Point, p2: Point, aabb: &AABB) -> bool {
    let tolerance = 1e-10;
    if (p1.y - p2.y).abs() < tolerance {
        let min_x = p1.x.min(p2.x);
        let max_x = p1.x.max(p2.x);
        let y = p1.y;
        y >= aabb.min_y && y <= aabb.max_y && max_x >= aabb.min_x && min_x <= aabb.max_x
    } else if (p1.x - p2.x).abs() < tolerance {
        let x = p1.x;
        let min_y = p1.y.min(p2.y);
        let max_y = p1.y.max(p2.y);
        x >= aabb.min_x && x <= aabb.max_x && max_y >= aabb.min_y && min_y <= aabb.max_y
    } else {
        false
    }
}

#[must_use]
pub fn orthogonal_route_avoiding(from: Point, to: Point, obstacle: &AABB) -> OrthogonalRoute {
    let direct = orthogonal_route(from, to);

    if !direct
        .points
        .windows(2)
        .any(|seg| segment_intersects_aabb(seg[0], seg[1], obstacle))
    {
        return direct;
    }

    let detour_y = if from.y < obstacle.max_y && to.y < obstacle.max_y {
        obstacle.min_y - 10.0
    } else {
        obstacle.max_y + 10.0
    };

    OrthogonalRoute {
        points: vec![
            from,
            Point::new(obstacle.min_x - 10.0, from.y),
            Point::new(obstacle.min_x - 10.0, detour_y),
            Point::new(obstacle.max_x + 10.0, detour_y),
            Point::new(obstacle.max_x + 10.0, to.y),
            to,
        ],
    }
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

pub fn world_to_screen(world: Point, camera: Point, zoom: f64) -> Point {
    Point::new((world.x - camera.x) * zoom, (world.y - camera.y) * zoom)
}

pub fn screen_to_world(screen: Point, camera: Point, zoom: f64) -> Point {
    Point::new(screen.x / zoom + camera.x, screen.y / zoom + camera.y)
}

pub fn selection_center(points: &[Point]) -> Point {
    if points.is_empty() {
        return Point::origin();
    }
    let sum_x: f64 = points.iter().map(|p| p.x).sum();
    let sum_y: f64 = points.iter().map(|p| p.y).sum();
    let count = points.len() as f64;
    Point::new(sum_x / count, sum_y / count)
}
