//! Label validation for diagram elements (nodes and edges).
//!
//! This module provides a single canonical source of truth for label validation
//! across the entire codebase.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

/// Maximum allowed length for labels (node and edge).
///
/// This limit is chosen to:
/// - Allow reasonably long labels for complex diagrams
/// - Prevent memory exhaustion from maliciously long inputs
/// - Keep database storage efficient
pub const MAX_LABEL_LENGTH: usize = 4096;

/// Validates a label (node or edge) for security and sanity constraints.
///
/// # Validation Rules
///
/// - Length must not exceed `MAX_LABEL_LENGTH` (4096 characters)
/// - No null bytes (`\0`)
/// - No control characters except newline (`\n`), carriage return (`\r`), and tab (`\t`)
/// - No zero-width spaces (U+200B, U+200C, U+200D, U+FEFF) - visual spoofing protection
/// - No bi-directional text overrides (U+202A..U+202E, U+2066..U+2069) - visual spoofing protection
///
/// # Examples
///
/// ```
/// use diagram_models::validation::label::{is_valid_label, MAX_LABEL_LENGTH};
///
/// assert!(is_valid_label("Hello, World!"));
/// assert!(is_valid_label("Line 1\nLine 2"));
/// assert!(is_valid_label("\tIndented"));
/// assert!(!is_valid_label("Hello\0World")); // null byte
/// assert!(!is_valid_label(&"x".repeat(MAX_LABEL_LENGTH + 1))); // too long
/// ```
#[must_use]
pub fn is_valid_label(label: &str) -> bool {
    if label.len() > MAX_LABEL_LENGTH {
        return false;
    }

    label.chars().all(is_valid_char)
}

/// Validates a single character for inclusion in a label.
#[inline]
fn is_valid_char(c: char) -> bool {
    // Reject null bytes
    if c == '\0' {
        return false;
    }

    // Reject control characters except safe whitespace
    if c.is_control() && !matches!(c, '\n' | '\r' | '\t') {
        return false;
    }

    // Reject zero-width spaces (visual spoofing)
    if matches!(c, '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}') {
        return false;
    }

    // Reject bi-directional overrides (visual spoofing)
    if matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}') {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_simple_text() {
        assert!(is_valid_label("Hello, World!"));
        assert!(is_valid_label(""));
        assert!(is_valid_label("A"));
    }

    #[test]
    fn accepts_allowed_whitespace() {
        assert!(is_valid_label("Line 1\nLine 2"));
        assert!(is_valid_label("Line 1\r\nLine 2"));
        assert!(is_valid_label("\tIndented text"));
        assert!(is_valid_label("\n\r\t"));
    }

    #[test]
    fn rejects_null_byte() {
        assert!(!is_valid_label("Hello\0World"));
        assert!(!is_valid_label("\0"));
    }

    #[test]
    fn rejects_control_characters() {
        assert!(!is_valid_label("Hello\x01World")); // SOH
        assert!(!is_valid_label("Hello\x08World")); // Backspace
        assert!(!is_valid_label("Hello\x7FWorld")); // DEL
    }

    #[test]
    fn rejects_zero_width_spaces() {
        assert!(!is_valid_label("Hello\u{200B}World")); // Zero-width space
        assert!(!is_valid_label("Hello\u{200C}World")); // Zero-width non-joiner
        assert!(!is_valid_label("Hello\u{200D}World")); // Zero-width joiner
        assert!(!is_valid_label("Hello\u{FEFF}World")); // BOM
    }

    #[test]
    fn rejects_bidi_overrides() {
        assert!(!is_valid_label("Hello\u{202A}World")); // LRE
        assert!(!is_valid_label("Hello\u{202E}World")); // RLO
        assert!(!is_valid_label("Hello\u{2066}World")); // LRI
        assert!(!is_valid_label("Hello\u{2069}World")); // PDI
    }

    #[test]
    fn rejects_too_long_labels() {
        let long_label = "x".repeat(MAX_LABEL_LENGTH + 1);
        assert!(!is_valid_label(&long_label));
    }

    #[test]
    fn accepts_max_length_label() {
        let max_label = "x".repeat(MAX_LABEL_LENGTH);
        assert!(is_valid_label(&max_label));
    }

    #[test]
    fn accepts_unicode() {
        assert!(is_valid_label("こんにちは世界")); // Japanese
        assert!(is_valid_label("🌍🌎🌏")); // Emoji
        assert!(is_valid_label("Привет мир")); // Russian
    }
}
