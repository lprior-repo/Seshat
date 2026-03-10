use crate::geometry::primitives::{Point, Rectangle, AABB};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FitTransform {
    pub scale: f64,
    pub offset_x: f64,
    pub offset_y: f64,
}

#[must_use]
pub fn scale_around_anchor(point: Point, anchor: Point, factor: f64) -> Point {
    Point::new(
        (point.x - anchor.x).mul_add(factor, anchor.x),
        (point.y - anchor.y).mul_add(factor, anchor.y),
    )
}

#[must_use]
pub fn rotate_around_center(point: Point, center: Point, angle_radians: f64) -> Point {
    let cos = angle_radians.cos();
    let sin = angle_radians.sin();
    let dx = point.x - center.x;
    let dy = point.y - center.y;

    Point::new(
        dx.mul_add(cos, (-dy).mul_add(sin, center.x)),
        dx.mul_add(sin, dy.mul_add(cos, center.y)),
    )
}

#[must_use]
pub fn resize_with_aspect_lock(original_width: f64, original_height: f64, new_width: f64) -> f64 {
    if original_width <= 0.0 {
        return new_width;
    }
    let aspect_ratio = original_height / original_width;
    new_width * aspect_ratio
}

#[must_use]
pub fn scale_then_rotate(
    point: Point,
    anchor: Point,
    scale_factor: f64,
    angle_radians: f64,
) -> Point {
    let scaled = scale_around_anchor(point, anchor, scale_factor);
    rotate_around_center(scaled, anchor, angle_radians)
}

#[must_use]
pub fn fit_to_viewport(
    content: &AABB,
    viewport_width: f64,
    viewport_height: f64,
    padding: f64,
) -> FitTransform {
    let content_width = content.width();
    let content_height = content.height();

    if content_width <= 0.0 || content_height <= 0.0 {
        return FitTransform {
            scale: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
        };
    }

    let available_width = viewport_width - 2.0 * padding;
    let available_height = viewport_height - 2.0 * padding;

    let scale_x = available_width / content_width;
    let scale_y = available_height / content_height;
    let scale = scale_x.min(scale_y);

    let content_center = content.center();
    let offset_x = viewport_width / 2.0 - content_center.x * scale;
    let offset_y = viewport_height / 2.0 - content_center.y * scale;

    FitTransform {
        scale,
        offset_x,
        offset_y,
    }
}

pub fn clamp_to_min_size(width: f64, height: f64, min_size: f64) -> (f64, f64) {
    let clamped_width = width.max(min_size);
    let clamped_height = height.max(min_size);
    (clamped_width, clamped_height)
}

pub fn scale_with_flip(width: f64, height: f64, scale_x: f64, scale_y: f64) -> (f64, f64) {
    let new_width = (width * scale_x).abs();
    let new_height = (height * scale_y).abs();
    (new_width, new_height)
}

pub fn scale_with_clamp(
    width: f64,
    height: f64,
    scale_x: f64,
    scale_y: f64,
    min_size: f64,
) -> (f64, f64) {
    let new_width = if scale_x < 0.0 {
        min_size
    } else {
        (width * scale_x).max(min_size)
    };
    let new_height = if scale_y < 0.0 {
        min_size
    } else {
        (height * scale_y).max(min_size)
    };
    (new_width, new_height)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Corner {
    NorthWest,
    NorthEast,
    SouthEast,
    SouthWest,
}

pub fn get_corner_point(rect: &Rectangle, corner: Corner) -> Point {
    match corner {
        Corner::NorthWest => Point::new(rect.x, rect.y),
        Corner::NorthEast => Point::new(rect.x + rect.width, rect.y),
        Corner::SouthEast => Point::new(rect.x + rect.width, rect.y + rect.height),
        Corner::SouthWest => Point::new(rect.x, rect.y + rect.height),
    }
}

pub fn scale_rect_around_corner(rect: &Rectangle, corner: Corner, factor: f64) -> Rectangle {
    let anchor = get_corner_point(rect, corner);
    let nw = scale_around_anchor(get_corner_point(rect, Corner::NorthWest), anchor, factor);
    let se = scale_around_anchor(get_corner_point(rect, Corner::SouthEast), anchor, factor);

    let new_width = (se.x - nw.x).abs();
    let new_height = (se.y - nw.y).abs();

    let (new_x, new_y) = match corner {
        Corner::NorthWest => (anchor.x, anchor.y),
        Corner::NorthEast => (anchor.x - new_width, anchor.y),
        Corner::SouthEast => (anchor.x - new_width, anchor.y - new_height),
        Corner::SouthWest => (anchor.x, anchor.y - new_height),
    };

    Rectangle::new(new_x, new_y, new_width, new_height)
}
