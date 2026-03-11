use crate::geometry::primitives::Point;

#[must_use]
pub fn snap_to_grid(point: Point, grid_size: f64) -> Point {
    if grid_size <= 0.0 || !grid_size.is_finite() {
        return point;
    }

    Point::new(
        (point.x / grid_size).round() * grid_size,
        (point.y / grid_size).round() * grid_size,
    )
}

#[must_use]
pub fn is_on_grid(value: f64, grid_size: f64) -> bool {
    if grid_size <= 0.0 || !grid_size.is_finite() || !value.is_finite() {
        return false;
    }

    let remainder = (value % grid_size).abs();
    remainder < f64::EPSILON || (remainder - grid_size).abs() < f64::EPSILON
}
