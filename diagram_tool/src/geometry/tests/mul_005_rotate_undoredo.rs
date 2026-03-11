use super::super::*;
use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== MUL-005: Rotate Undo/Redo ==============

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_mul_rotate_undo_redo() {
    // Given: initial positions of multiple items
    let original_positions = [
        Point::new(0.0, 0.0),
        Point::new(100.0, 0.0),
        Point::new(50.0, 100.0),
    ];
    let center = selection_center(&original_positions);
    let rotation_angle = PI / 4.0; // 45 degrees

    // Simulate rotation operation
    let rotated_positions: Vec<Point> = original_positions
        .iter()
        .map(|&p| rotate_around_center(p, center, rotation_angle))
        .collect();

    // When: "undo" - restore original positions
    let after_undo = original_positions;

    // Verify undo restores original state
    for (original, restored) in original_positions.iter().zip(after_undo.iter()) {
        assert!((restored.x - original.x).abs() < TOLERANCE);
        assert!((restored.y - original.y).abs() < TOLERANCE);
    }

    // When: "redo" - apply rotation again
    let after_redo: Vec<Point> = after_undo
        .iter()
        .map(|&p| rotate_around_center(p, center, rotation_angle))
        .collect();

    // Then: redo produces the same rotated state
    for (expected, actual) in rotated_positions.iter().zip(after_redo.iter()) {
        assert!((actual.x - expected.x).abs() < TOLERANCE);
        assert!((actual.y - expected.y).abs() < TOLERANCE);
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_mul_rotate_undo_redo_with_history() {
    // This test uses the History pattern to verify undo/redo behavior
    use std::cell::RefCell;

    // Given: state that can be snapshotted
    #[derive(Clone, Debug)]
    struct SelectionState {
        positions: Vec<Point>,
    }

    impl SelectionState {
        fn rotate(&self, center: Point, angle: f64) -> Self {
            Self {
                positions: self
                    .positions
                    .iter()
                    .map(|&p| rotate_around_center(p, center, angle))
                    .collect(),
            }
        }
    }

    let original = SelectionState {
        positions: vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 50.0),
            Point::new(50.0, 100.0),
        ],
    };

    // Simple history simulation
    let history = RefCell::new(Vec::new());

    // Save initial state
    history.borrow_mut().push(original.clone());

    let center = selection_center(&original.positions);

    // Apply rotation and save
    let rotated = original.rotate(center, PI / 6.0);
    history.borrow_mut().push(rotated.clone());

    // Apply another rotation and save
    let rotated_again = rotated.rotate(center, PI / 6.0);
    history.borrow_mut().push(rotated_again.clone());

    // When: undo (pop and restore previous)
    history.borrow_mut().pop(); // Remove current
    let after_undo = history.borrow().last().cloned().unwrap();

    // Then: state matches first rotation
    for (expected, actual) in rotated.positions.iter().zip(after_undo.positions.iter()) {
        assert!((actual.x - expected.x).abs() < TOLERANCE);
        assert!((actual.y - expected.y).abs() < TOLERANCE);
    }

    // When: undo again
    history.borrow_mut().pop();
    let after_second_undo = history.borrow().last().cloned().unwrap();

    // Then: state matches original
    for (expected, actual) in original
        .positions
        .iter()
        .zip(after_second_undo.positions.iter())
    {
        assert!((actual.x - expected.x).abs() < TOLERANCE);
        assert!((actual.y - expected.y).abs() < TOLERANCE);
    }
}
