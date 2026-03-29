use super::*;

/// seshat-feo: ACCENT must reference the CSS custom property --accent, not a hardcoded hex.
#[test]
fn accent_uses_css_custom_property() {
    assert_eq!(ACCENT, "var(--accent)");
}

/// seshat-feo: `ACCENT_DASH_BORDER` must use the accent CSS variable, not a hardcoded color.
#[test]
fn accent_dash_border_uses_accent_variable() {
    assert_eq!(ACCENT_DASH_BORDER, "2px dashed var(--accent)");
}

/// seshat-feo: `ACCENT_DASH_BORDER` must not contain any hardcoded hex color.
#[test]
fn accent_dash_border_has_no_hardcoded_hex() {
    let contains_hex = ACCENT_DASH_BORDER.contains('#')
        && ACCENT_DASH_BORDER
            .split('#')
            .nth(1)
            .map_or(false, |after_hash| {
                after_hash
                    .chars()
                    .take_while(char::is_ascii_hexdigit)
                    .count()
                    >= 3
            });
    assert!(
        !contains_hex,
        "ACCENT_DASH_BORDER must not contain hardcoded hex: {ACCENT_DASH_BORDER}"
    );
}
