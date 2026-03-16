use super::*;
#[allow(unused_imports)]
use proptest::prelude::*;
#[allow(unused_imports)]
use std::f64::consts::*;

#[allow(dead_code)]
const TOLERANCE: f64 = 1e-10;

// ============== GEO-004: Text Bounds Calculation ==============

#[test]
fn test_text_bounds() {
    // Given: text at position with font size
    let text = Text::new(10.0, 20.0, "Hello", 16.0);

    // When: calculating bounds
    let bounds = text.bounds();

    // Then: bounds start at text position
    assert!((bounds.min_x - 10.0).abs() < TOLERANCE);
    assert!((bounds.min_y - 20.0).abs() < TOLERANCE);
    assert!((bounds.height() - 16.0).abs() < TOLERANCE);
    // Width = 0.6 * font_size * char_count = 0.6 * 16 * 5 = 48
    assert!((bounds.width() - 48.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_text_bounds_kani() {
    // Given: text at position with font size
    let text = Text::new(10.0, 20.0, "Hello", 16.0);

    // When: calculating bounds
    let bounds = text.bounds();

    // Then: bounds start at text position
    assert!((bounds.min_x - 10.0).abs() < TOLERANCE);
    assert!((bounds.min_y - 20.0).abs() < TOLERANCE);
    assert!((bounds.height() - 16.0).abs() < TOLERANCE);
    // Width = 0.6 * font_size * char_count = 0.6 * 16 * 5 = 48
    assert!((bounds.width() - 48.0).abs() < TOLERANCE);
}

#[test]
fn test_text_bounds_empty_string() {
    // Given: empty text
    let text = Text::new(10.0, 20.0, "", 16.0);

    // When: calculating bounds
    let bounds = text.bounds();

    // Then: bounds have zero width but maintain height
    assert!((bounds.min_x - 10.0).abs() < TOLERANCE);
    assert!((bounds.width() - 0.0).abs() < TOLERANCE);
    assert!((bounds.height() - 16.0).abs() < TOLERANCE);
}

#[cfg(kani)]
#[kani::proof]
fn test_text_bounds_empty_string_kani() {
    // Given: empty text
    let text = Text::new(10.0, 20.0, "", 16.0);

    // When: calculating bounds
    let bounds = text.bounds();

    // Then: bounds have zero width but maintain height
    assert!((bounds.min_x - 10.0).abs() < TOLERANCE);
    assert!((bounds.width() - 0.0).abs() < TOLERANCE);
    assert!((bounds.height() - 16.0).abs() < TOLERANCE);
}
