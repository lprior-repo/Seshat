use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

    // ============== GEO-027: Path Simplification Tests ==============

    #[test]
    fn geo027_001_basic_simplification() {
        // Given: A path with 5 points in a rough line
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(2.0, 1.0),
            Point::new(3.0, 2.0),
            Point::new(4.0, 4.0),
        ];

        // When: Simplify is called with epsilon = 1.0
        let result = simplify_path(&points, PathSimplificationConfig::new(1.0, 2).unwrap());

        // Then: The output should have 2 points (start and end)
        assert!(result.is_ok());
        let simplified = result.unwrap();
        assert_eq!(simplified.len(), 2);
        assert_eq!(simplified[0], Point::new(0.0, 0.0));
        assert_eq!(simplified[1], Point::new(4.0, 4.0));
    }

    #[test]
    fn geo027_002_endpoint_preservation_start() {
        // Given: A path
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(20.0, 20.0),
            Point::new(30.0, 30.0),
        ];

        // When: Simplify is called
        let result = simplify_path(&points, PathSimplificationConfig::new(0.5, 2).unwrap());

        // Then: The first point MUST be (0,0)
        assert!(result.is_ok());
        let simplified = result.unwrap();
        assert_eq!(simplified[0], Point::new(0.0, 0.0));
    }

    #[test]
    fn geo027_003_endpoint_preservation_end() {
        // Given: A path
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(20.0, 20.0),
            Point::new(30.0, 30.0),
        ];

        // When: Simplify is called
        let result = simplify_path(&points, PathSimplificationConfig::new(0.5, 2).unwrap());

        // Then: The last point MUST be (30,30)
        assert!(result.is_ok());
        let simplified = result.unwrap();
        assert_eq!(simplified[simplified.len() - 1], Point::new(30.0, 30.0));
    }

    #[test]
    fn geo027_006_insufficient_points_zero() {
        // Given: An empty path
        let points: Vec<Point> = vec![];

        // When: Simplify is called
        let result = simplify_path(&points, PathSimplificationConfig::new(1.0, 2).unwrap());

        // Then: Returns InsufficientPoints error
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PathError::InsufficientPoints);
    }

    #[test]
    fn geo027_007_insufficient_points_one() {
        // Given: A path with one point
        let points = vec![Point::new(5.0, 5.0)];

        // When: Simplify is called
        let result = simplify_path(&points, PathSimplificationConfig::new(1.0, 2).unwrap());

        // Then: Returns InsufficientPoints error
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PathError::InsufficientPoints);
    }

    #[test]
    fn geo027_008_two_points_preserved() {
        // Given: A path with two points
        let points = vec![Point::new(0.0, 0.0), Point::new(10.0, 10.0)];

        // When: Simplify is called
        let result = simplify_path(&points, PathSimplificationConfig::new(1.0, 2).unwrap());

        // Then: The output MUST have exactly 2 points
        assert!(result.is_ok());
        let simplified = result.unwrap();
        assert_eq!(simplified.len(), 2);
    }

    #[test]
    fn geo027_009_invalid_point_nan() {
        // Given: A path with NaN
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(f64::NAN, 5.0),
            Point::new(10.0, 10.0),
        ];

        // When: Simplify is called
        let result = simplify_path(&points, PathSimplificationConfig::new(1.0, 2).unwrap());

        // Then: Returns InvalidPoint error
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PathError::InvalidPoint);
    }

    #[test]
    fn geo027_010_invalid_point_infinity() {
        // Given: A path with Infinity
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(f64::INFINITY, 5.0),
            Point::new(10.0, 10.0),
        ];

        // When: Simplify is called
        let result = simplify_path(&points, PathSimplificationConfig::new(1.0, 2).unwrap());

        // Then: Returns InvalidPoint error
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PathError::InvalidPoint);
    }

    #[test]
    fn geo027_011_epsilon_zero() {
        // Given: A path with points exactly on line
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(5.0, 0.0),
            Point::new(10.0, 0.0),
        ];

        // When: Simplify is called with epsilon = 0.0
        let result = simplify_path(&points, PathSimplificationConfig::new(0.0, 2).unwrap());

        // Then: With epsilon = 0, the path is returned as-is (no simplification)
        assert!(result.is_ok());
        let simplified = result.unwrap();
        assert_eq!(simplified.len(), 3);
    }

    #[test]
    fn geo027_012_epsilon_boundary_exactly_on() {
        // Given: A path where middle point is exactly at epsilon distance
        // Point (5, 1) distance from line y=0 is exactly 1.0
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(5.0, 1.0),
            Point::new(10.0, 0.0),
        ];

        // When: Simplify is called with epsilon = 1.0 (exactly on boundary)
        let result = simplify_path(&points, PathSimplificationConfig::new(1.0, 2).unwrap());

        // Then: The output should have 2 points (within epsilon)
        assert!(result.is_ok());
        let simplified = result.unwrap();
        assert_eq!(simplified.len(), 2);
    }

    #[test]
    fn geo027_016_straight_line_preserved() {
        // Given: A simple straight line
        let points = vec![Point::new(0.0, 0.0), Point::new(100.0, 0.0)];

        // When: Simplify is called
        let result = simplify_path(&points, PathSimplificationConfig::new(10.0, 2).unwrap());

        // Then: The output MUST be exactly [(0,0), (100,0)]
        assert!(result.is_ok());
        let simplified = result.unwrap();
        assert_eq!(simplified.len(), 2);
        assert_eq!(simplified[0], Point::new(0.0, 0.0));
        assert_eq!(simplified[1], Point::new(100.0, 0.0));
    }

    #[test]
    fn geo027_014_curved_path_simplification() {
        // Given: A curved path along diagonal
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(2.0, 2.0),
            Point::new(4.0, 4.0),
            Point::new(6.0, 6.0),
            Point::new(8.0, 8.0),
            Point::new(10.0, 10.0),
        ];

        // When: Simplify is called with epsilon = 1.0
        let result = simplify_path(&points, PathSimplificationConfig::new(1.0, 2).unwrap());

        // Then: The output should have 2 points
        assert!(result.is_ok());
        let simplified = result.unwrap();
        assert_eq!(simplified.len(), 2);
    }

    #[test]
    fn path_error_display() {
        assert_eq!(
            PathError::InsufficientPoints.to_string(),
            "Path has insufficient points"
        );
        assert_eq!(
            PathError::InvalidPoint.to_string(),
            "Path contains invalid point (NaN/Infinity)"
        );
        assert_eq!(
            PathError::SelfIntersection.to_string(),
            "Path has self-intersection"
        );
        assert_eq!(
            PathError::InvalidEpsilon.to_string(),
            "Epsilon must be non-negative"
        );
    }
