use diagram_tool::ui::canvas::math::{
    canvas_to_screen, safe_zoom, sanitize_zoom, screen_to_canvas, within,
};

#[test]
fn test_safe_zoom_valid() {
    assert!(safe_zoom(1.0).is_some());
    assert!(safe_zoom(0.5).is_some());
    assert!(safe_zoom(100.0).is_some());
}

#[test]
fn test_safe_zoom_invalid() {
    assert!(safe_zoom(0.0).is_none());
    assert!(safe_zoom(-1.0).is_none());
    assert!(safe_zoom(f64::NAN).is_none());
    assert!(safe_zoom(f64::INFINITY).is_none());
    assert!(safe_zoom(f64::NEG_INFINITY).is_none());
    assert!(safe_zoom(f64::EPSILON).is_none());
}

#[test]
fn test_within_basic() {
    let subgraph = (0.0, 0.0, 100.0, 100.0);
    let node = (10.0, 10.0, 50.0, 50.0);
    assert!(within(subgraph, node));
}

#[test]
fn test_within_outside() {
    let subgraph = (0.0, 0.0, 100.0, 100.0);
    let node = (90.0, 90.0, 50.0, 50.0);
    assert!(!within(subgraph, node));
}

#[test]
fn test_within_on_edge() {
    let subgraph = (0.0, 0.0, 100.0, 100.0);
    let node = (0.0, 0.0, 100.0, 100.0);
    assert!(within(subgraph, node));
}

#[test]
fn test_screen_to_canvas_basic() {
    let result = screen_to_canvas(100.0, 200.0, 0.0, 0.0, 2.0);
    assert!(result.is_some());
    let (cx, cy) = result.unwrap();
    assert!((cx - 50.0).abs() < 1e-6);
    assert!((cy - 100.0).abs() < 1e-6);
}

#[test]
fn test_screen_to_canvas_with_camera() {
    let result = screen_to_canvas(100.0, 200.0, 50.0, 75.0, 2.0);
    assert!(result.is_some());
    let (cx, cy) = result.unwrap();
    assert!((cx - 100.0).abs() < 1e-6);
    assert!((cy - 175.0).abs() < 1e-6);
}

#[test]
fn test_screen_to_canvas_invalid_zoom() {
    assert!(screen_to_canvas(100.0, 100.0, 0.0, 0.0, 0.0).is_none());
    assert!(screen_to_canvas(100.0, 100.0, 0.0, 0.0, f64::NAN).is_none());
}

#[test]
fn test_canvas_to_screen_basic() {
    let result = canvas_to_screen(50.0, 100.0, 0.0, 0.0, 2.0);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), (100.0, 200.0));
}

#[test]
fn test_canvas_to_screen_with_camera() {
    let result = canvas_to_screen(100.0, 175.0, 50.0, 75.0, 2.0);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), (100.0, 200.0));
}

#[test]
fn test_canvas_to_screen_roundtrip() {
    let (cx, cy) = (100.0, 200.0);
    let (camera_x, camera_y) = (50.0, 75.0);
    let zoom = 2.0;

    let screen_result = canvas_to_screen(cx, cy, camera_x, camera_y, zoom);
    assert!(screen_result.is_some());
    let (sx, sy) = screen_result.unwrap();

    let back_result = screen_to_canvas(sx, sy, camera_x, camera_y, zoom);
    assert!(back_result.is_some());
    let (rx, ry) = back_result.unwrap();

    assert!((rx - cx).abs() < 1e-6);
    assert!((ry - cy).abs() < 1e-6);
}

#[test]
fn test_sanitize_zoom_basic() {
    let result = sanitize_zoom(0.5, 0.1, 4.0);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), 0.5);
}

#[test]
fn test_safe_zoom_clamp_below_min() {
    let result = sanitize_zoom(0.05, 0.1, 4.0);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), 0.1);
}

#[test]
fn test_safe_zoom_clamp_above_max() {
    let result = sanitize_zoom(10.0, 0.1, 4.0);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), 4.0);
}

#[test]
fn test_sanitize_zoom_rejects_invalid() {
    assert!(sanitize_zoom(0.0, 0.1, 4.0).is_none());
    assert!(sanitize_zoom(-1.0, 0.1, 4.0).is_none());
    assert!(sanitize_zoom(f64::NAN, 0.1, 4.0).is_none());
}

#[test]
fn test_sanitize_zoom_valid_in_range() {
    let result = sanitize_zoom(1.0, 0.1, 4.0);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), 1.0);
}

#[test]
fn test_sanitize_zoom_below_min() {
    let result = sanitize_zoom(0.05, 0.1, 4.0);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), 0.1);
}

#[test]
fn test_sanitize_zoom_above_max() {
    let result = sanitize_zoom(10.0, 0.1, 4.0);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), 4.0);
}
