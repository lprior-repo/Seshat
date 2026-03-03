# Martin Fowler-Style BDD Tests: Viewport Operations (bd-1l3)

This document contains behavior-driven development tests following Martin Fowler's BDD style for the Viewport/Camera test category.

## Test Category: CAM (Viewport/Camera) - 12 Tests

---

## CAM-001: Pan Viewport Basic

### Scenario: Pan right and down
```gherkin
Given a viewport with camera at origin (0, 0) and zoom 1.0
When the user pans by screen delta (100, 50)
Then the camera position becomes (-100, -50)
And the world appears to move right and down
```

### Implementation
```rust
#[test]
fn cam_001_pan_viewport_basic() {
    // Given
    let viewport = ViewportState::new(800.0, 600.0);
    assert_eq!(viewport.camera_x(), 0.0);
    assert_eq!(viewport.camera_y(), 0.0);

    // When
    let result = viewport.pan(100.0, 50.0);

    // Then
    assert!(result.is_ok());
    assert_eq!(viewport.camera_x(), -100.0);
    assert_eq!(viewport.camera_y(), -50.0);
}
```

---

## CAM-002: Pan with Bounds Checking

### Scenario: Pan beyond maximum bounds
```gherkin
Given a viewport at camera (9500, 9500)
When the user pans by (1000, 1000)
Then the camera clamps to (10000, 10000)
And the pan operation returns successfully
```

### Scenario: Pan beyond minimum bounds
```gherkin
Given a viewport at camera (-9500, -9500)
When the user pans by (-1000, -1000)
Then the camera clamps to (-10000, -10000)
```

### Implementation
```rust
#[test]
fn cam_002_pan_with_bounds_checking() {
    // Given: near max bounds
    let mut viewport = ViewportState::new(800.0, 600.0);
    viewport.set_camera(9500.0, 9500.0);

    // When: pan beyond max
    let _ = viewport.pan(1000.0, 1000.0);

    // Then: clamped
    assert_eq!(viewport.camera_x(), 10000.0);
    assert_eq!(viewport.camera_y(), 10000.0);

    // Given: near min bounds
    viewport.set_camera(-9500.0, -9500.0);

    // When: pan beyond min
    let _ = viewport.pan(-1000.0, -1000.0);

    // Then: clamped
    assert_eq!(viewport.camera_x(), -10000.0);
    assert_eq!(viewport.camera_y(), -10000.0);
}
```

---

## CAM-003: Zoom In Operation

### Scenario: Zoom in from default
```gherkin
Given a viewport with zoom 1.0
When the user zooms in with factor 1.25
Then the zoom becomes 1.25
And the camera adjusts to keep viewport centered
```

### Implementation
```rust
#[test]
fn cam_003_zoom_in_operation() {
    // Given
    let mut viewport = ViewportState::new(800.0, 600.0);
    assert_eq!(viewport.zoom(), 1.0);

    // When
    let result = viewport.zoom_in();

    // Then
    assert!(result);
    assert!((viewport.zoom() - 1.25).abs() < f64::EPSILON);
}
```

---

## CAM-004: Zoom Out Operation

### Scenario: Zoom out from default
```gherkin
Given a viewport with zoom 1.0
When the user zooms out with factor 0.8
Then the zoom becomes 0.8
And the camera adjusts to keep viewport centered
```

### Implementation
```rust
#[test]
fn cam_004_zoom_out_operation() {
    // Given
    let mut viewport = ViewportState::new(800.0, 600.0);
    assert_eq!(viewport.zoom(), 1.0);

    // When
    let result = viewport.zoom_out();

    // Then
    assert!(result);
    assert!((viewport.zoom() - 0.8).abs() < f64::EPSILON);
}
```

---

## CAM-005: Zoom to Specific Level

### Scenario: Set zoom to exact value
```gherkin
Given a viewport with zoom 1.0
When the user sets zoom to 2.0
Then the zoom becomes 2.0
And the camera adjusts to keep viewport centered
```

### Implementation
```rust
#[test]
fn cam_005_zoom_to_specific_level() {
    // Given
    let mut viewport = ViewportState::new(800.0, 600.0);

    // When
    let result = viewport.set_zoom(2.0);

    // Then
    assert!(result);
    assert!((viewport.zoom() - 2.0).abs() < f64::EPSILON);
}
```

---

## CAM-006: Zoom with Bounds

### Scenario: Cannot zoom beyond maximum
```gherkin
Given a viewport with zoom 4.0 (at maximum)
When the user tries to zoom in
Then the zoom stays at 4.0
And the operation returns false (no change)
```

### Scenario: Cannot zoom below minimum
```gherkin
Given a viewport with zoom 0.1 (at minimum)
When the user tries to zoom out
Then the zoom stays at 0.1
And the operation returns false (no change)
```

### Implementation
```rust
#[test]
fn cam_006_zoom_with_bounds() {
    // Given: at max zoom
    let mut viewport = ViewportState::new(800.0, 600.0);
    viewport.set_zoom(4.0);

    // When: try to zoom in
    let result = viewport.zoom_in();

    // Then: no change
    assert!(!result);
    assert!((viewport.zoom() - 4.0).abs() < f64::EPSILON);

    // Given: at min zoom
    viewport.set_zoom(0.1);

    // When: try to zoom out
    let result = viewport.zoom_out();

    // Then: no change
    assert!(!result);
    assert!((viewport.zoom() - 0.1).abs() < f64::EPSILON);
}
```

---

## CAM-007: Screen to World Transform

### Scenario: Convert screen coordinates to world
```gherkin
Given a viewport with camera (100, 200) and zoom 2.0
When converting screen point (400, 300) to world
Then world point is (300, 350)
```

### Formula
```
world_x = camera_x + screen_x / zoom
world_y = camera_y + screen_y / zoom
```

### Implementation
```rust
#[test]
fn cam_007_screen_to_world_transform() {
    // Given
    let viewport = ViewportState::new(800.0, 600.0);
    viewport.set_camera(100.0, 200.0);
    viewport.set_zoom(2.0);

    // When
    let world = viewport.screen_to_world(400.0, 300.0);

    // Then
    // world_x = 100 + 400 / 2.0 = 300
    // world_y = 200 + 300 / 2.0 = 350
    assert!((world.x - 300.0).abs() < f64::EPSILON);
    assert!((world.y - 350.0).abs() < f64::EPSILON);
}
```

---

## CAM-008: World to Screen Transform

### Scenario: Convert world coordinates to screen
```gherkin
Given a viewport with camera (100, 200) and zoom 2.0
When converting world point (300, 350) to screen
Then screen point is (400, 300)
```

### Formula
```
screen_x = (world_x - camera_x) * zoom
screen_y = (world_y - camera_y) * zoom
```

### Implementation
```rust
#[test]
fn cam_008_world_to_screen_transform() {
    // Given
    let viewport = ViewportState::new(800.0, 600.0);
    viewport.set_camera(100.0, 200.0);
    viewport.set_zoom(2.0);

    // When
    let screen = viewport.world_to_screen(300.0, 350.0);

    // Then
    // screen_x = (300 - 100) * 2.0 = 400
    // screen_y = (350 - 200) * 2.0 = 300
    assert!((screen.x - 400.0).abs() < f64::EPSILON);
    assert!((screen.y - 300.0).abs() < f64::EPSILON);
}
```

---

## CAM-009: Fit Content to Viewport

### Scenario: Fit content with padding
```gherkin
Given content bounds AABB(0, 0, 500, 400)
And viewport size (800, 600) with padding 20
When fitting content to viewport
Then scale is calculated to fit with aspect ratio preserved
And content is centered in viewport
```

### Implementation
```rust
#[test]
fn cam_009_fit_content_to_viewport() {
    // Given
    let content = AABB::new(0.0, 0.0, 500.0, 400.0);
    let viewport = ViewportState::new(800.0, 600.0);

    // When
    let fit = viewport.fit_to_content(&content, 20.0);

    // Then
    assert!(fit.is_some());
    let fit = fit.unwrap();
    // Available: 760 x 560, Content: 500 x 400
    // Scale: min(760/500, 560/400) = min(1.52, 1.4) = 1.4
    assert!((fit.scale - 1.4).abs() < 0.01);
}
```

---

## CAM-010: Center on Specific Point

### Scenario: Center viewport on world point
```gherkin
Given a viewport at camera (0, 0) with zoom 1.0 and size (800, 600)
When centering on world point (250, 300)
Then camera moves to (-150, 0)
```

### Formula
```
camera_x = point_x - viewport_width / 2 / zoom
camera_y = point_y - viewport_height / 2 / zoom
```

### Implementation
```rust
#[test]
fn cam_010_center_on_specific_point() {
    // Given
    let mut viewport = ViewportState::new(800.0, 600.0);
    viewport.set_zoom(1.0);

    // When
    viewport.center_on(250.0, 300.0);

    // Then
    // camera_x = 250 - 800/2/1 = 250 - 400 = -150
    // camera_y = 300 - 600/2/1 = 300 - 300 = 0
    assert!((viewport.camera_x() - (-150.0)).abs() < f64::EPSILON);
    assert!((viewport.camera_y() - 0.0).abs() < f64::EPSILON);
}
```

---

## CAM-011: Zoom Around Point

### Scenario: Zoom keeping point under cursor
```gherkin
Given a viewport at zoom 1.0 with mouse at screen (400, 300)
And world point under mouse is (400, 300)
When zooming to 2.0x around that point
Then the world point under mouse remains at (400, 300)
And the camera adjusts to keep that point stationary
```

### Implementation
```rust
#[test]
fn cam_011_zoom_around_point() {
    // Given
    let mut viewport = ViewportState::new(800.0, 600.0);
    viewport.set_camera(0.0, 0.0);
    viewport.set_zoom(1.0);

    // Mouse at screen center (400, 300), world point is (400, 300)
    let screen_point = (400.0, 300.0);
    let world_before = viewport.screen_to_world(screen_point.0, screen_point.1);

    // When: zoom around that point
    viewport.zoom_around_point(2.0, screen_point.0, screen_point.1);

    // Then: world point under cursor is same
    let world_after = viewport.screen_to_world(screen_point.0, screen_point.1);
    assert!((world_before.x - world_after.x).abs() < f64::EPSILON);
    assert!((world_before.y - world_after.y).abs() < f64::EPSILON);
    assert!((viewport.zoom() - 2.0).abs() < f64::EPSILON);
}
```

---

## CAM-012: Viewport State Persistence

### Scenario: Serialize and deserialize viewport state
```gherkin
Given a viewport with camera (100, 200) and zoom 1.5
When serializing to JSON and deserializing back
Then the state is preserved exactly
```

### Implementation
```rust
#[test]
fn cam_012_viewport_state_persistence() {
    // Given
    let original = ViewportState::new(800.0, 600.0);
    original.set_camera(100.0, 200.0);
    original.set_zoom(1.5);

    // When
    let json = serde_json::to_string(&original).unwrap();
    let restored: ViewportState = serde_json::from_str(&json).unwrap();

    // Then
    assert!((restored.camera_x() - 100.0).abs() < f64::EPSILON);
    assert!((restored.camera_y() - 200.0).abs() < f64::EPSILON);
    assert!((restored.zoom() - 1.5).abs() < f64::EPSILON);
}
```

---

## Property-Based Tests

### Round-trip coordinate transform
```rust
proptest! {
    #[test]
    fn prop_coordinate_roundtrip(
        screen_x in 0.0_f64..1920.0,
        screen_y in 0.0_f64..1080.0,
        camera_x in -1000.0_f64..1000.0,
        camera_y in -1000.0_f64..1000.0,
        zoom in 0.1_f64..4.0
    ) {
        let viewport = ViewportState::new(1920.0, 1080.0);
        viewport.set_camera(camera_x, camera_y);
        viewport.set_zoom(zoom);

        let world = viewport.screen_to_world(screen_x, screen_y);
        let screen_back = viewport.world_to_screen(world.x, world.y);

        prop_assert!((screen_back.x - screen_x).abs() < 0.001);
        prop_assert!((screen_back.y - screen_y).abs() < 0.001);
    }
}
```

### Zoom bounds invariant
```rust
proptest! {
    #[test]
    fn prop_zoom_always_bounded(zoom_factor in 0.001_f64..1000.0) {
        let mut viewport = ViewportState::new(800.0, 600.0);
        viewport.set_zoom(1.0);

        // Apply arbitrary zoom factor
        let _ = viewport.zoom_by_factor(zoom_factor);

        // Invariant: zoom is always in [0.1, 4.0]
        prop_assert!(viewport.zoom() >= 0.1);
        prop_assert!(viewport.zoom() <= 4.0);
    }
}
```

---

## Test Summary

| Test ID | Description | Status |
|---------|-------------|--------|
| CAM-001 | Pan Viewport Basic | PENDING |
| CAM-002 | Pan with Bounds Checking | PENDING |
| CAM-003 | Zoom In Operation | PENDING |
| CAM-004 | Zoom Out Operation | PENDING |
| CAM-005 | Zoom to Specific Level | PENDING |
| CAM-006 | Zoom with Bounds | PENDING |
| CAM-007 | Screen to World Transform | PENDING |
| CAM-008 | World to Screen Transform | PENDING |
| CAM-009 | Fit Content to Viewport | PENDING |
| CAM-010 | Center on Specific Point | PENDING |
| CAM-011 | Zoom Around Point | PENDING |
| CAM-012 | Viewport State Persistence | PENDING |
