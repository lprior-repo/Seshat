#[test]
fn test_scale_around_anchor_concrete() {
    let point = Point::new(10.0, 10.0);
    let anchor = Point::new(0.0, 0.0);
    assert_eq!(scale_around_anchor(point, anchor, 2.0), Point::new(20.0, 20.0));
}
#[test] fn test_rotate_around_center_concrete() { assert!(rotate_around_center(Point::new(10.0, 0.0), Point::new(0.0, 0.0), std::f64::consts::PI/2.0).y > 9.0); }
#[test] fn test_resize_with_aspect_lock_concrete() { assert_eq!(resize_with_aspect_lock(100.0, 50.0, 200.0), 100.0); }
#[test] fn test_scale_then_rotate_concrete() { assert!(scale_then_rotate(Point::new(10.0, 0.0), Point::new(0.0, 0.0), 2.0, std::f64::consts::PI/2.0).y > 19.0); }
#[test] fn test_fit_to_viewport_concrete() { assert_eq!(fit_to_viewport(&AABB::new(0.0, 0.0, 100.0, 100.0), 200.0, 200.0, 0.0).scale, 2.0); }
#[test] fn test_clamp_to_min_size_concrete() { assert_eq!(clamp_to_min_size(5.0, 10.0, 20.0), (20.0, 20.0)); }
#[test] fn test_scale_with_flip_concrete() { assert_eq!(scale_with_flip(10.0, 20.0, -1.0, 2.0), (10.0, 40.0)); }
#[test] fn test_scale_with_clamp_concrete() { assert_eq!(scale_with_clamp(10.0, 10.0, 0.5, 0.5, 20.0), (20.0, 20.0)); }
#[test] fn test_get_corner_point_concrete() { assert_eq!(get_corner_point(&Rectangle::new(10.0, 20.0, 100.0, 50.0), Corner::SouthEast), Point::new(110.0, 70.0)); }
#[test] fn test_scale_rect_around_corner_concrete() { assert_eq!(scale_rect_around_corner(&Rectangle::new(10.0, 10.0, 100.0, 100.0), Corner::NorthWest, 2.0).width, 200.0); }
