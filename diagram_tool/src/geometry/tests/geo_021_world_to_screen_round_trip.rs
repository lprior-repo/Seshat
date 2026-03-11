use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

    // ============== GEO-021: World-to-Screen Round-Trip ==============

    /// Transform world coordinates to screen coordinates
    fn world_to_screen(world: Point, camera: Point, zoom: f64) -> Point {
        Point::new((world.x - camera.x) * zoom, (world.y - camera.y) * zoom)
    }

    /// Transform screen coordinates back to world coordinates
    fn screen_to_world(screen: Point, camera: Point, zoom: f64) -> Point {
        Point::new(screen.x / zoom + camera.x, screen.y / zoom + camera.y)
    }

    #[test]
    fn test_world_to_screen_round_trip() {
        // Given: a world point, camera position, and zoom level
        let world = Point::new(100.0, 200.0);
        let camera = Point::new(50.0, 75.0);
        let zoom = 2.0;

        // When: transforming to screen and back to world
        let screen = world_to_screen(world, camera, zoom);
        let round_trip = screen_to_world(screen, camera, zoom);

        // Then: round-trip preserves original within tolerance
        assert!((round_trip.x - world.x).abs() < TOLERANCE);
        assert!((round_trip.y - world.y).abs() < TOLERANCE);
    }

    #[test]
    fn test_world_to_screen_round_trip_at_origin() {
        // Given: world point at origin
        let world = Point::origin();
        let camera = Point::new(100.0, 100.0);
        let zoom = 1.0;

        // When: transforming to screen and back
        let screen = world_to_screen(world, camera, zoom);
        let round_trip = screen_to_world(screen, camera, zoom);

        // Then: round-trip preserves origin
        assert!((round_trip.x - world.x).abs() < TOLERANCE);
        assert!((round_trip.y - world.y).abs() < TOLERANCE);
    }

    #[test]
    fn test_world_to_screen_round_trip_high_zoom() {
        // Given: high zoom level
        let world = Point::new(1000.0, 1000.0);
        let camera = Point::new(0.0, 0.0);
        let zoom = 10.0;

        // When: transforming to screen and back
        let screen = world_to_screen(world, camera, zoom);
        let round_trip = screen_to_world(screen, camera, zoom);

        // Then: round-trip preserves original
        assert!((round_trip.x - world.x).abs() < TOLERANCE);
        assert!((round_trip.y - world.y).abs() < TOLERANCE);
        // Verify screen coordinates are scaled
        assert!((screen.x - 10000.0).abs() < TOLERANCE);
    }

