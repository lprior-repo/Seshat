use super::*;

// --- B18–B21b: ThemeScheme::from_str (wasm32 only) ---
#[cfg(target_arch = "wasm32")]
#[test]
fn scheme_from_str_returns_light_for_light_string() {
    assert_eq!(ThemeScheme::from_str("light"), Some(ThemeScheme::Light));
}

#[cfg(target_arch = "wasm32")]
#[test]
fn scheme_from_str_returns_dark_for_dark_string() {
    assert_eq!(ThemeScheme::from_str("dark"), Some(ThemeScheme::Dark));
}

#[cfg(target_arch = "wasm32")]
#[test]
fn scheme_from_str_returns_white_for_white_string() {
    assert_eq!(ThemeScheme::from_str("white"), Some(ThemeScheme::White));
}

#[cfg(target_arch = "wasm32")]
#[test]
fn scheme_from_str_returns_none_for_empty_string() {
    assert_eq!(ThemeScheme::from_str(""), None);
}

#[cfg(target_arch = "wasm32")]
#[test]
fn scheme_from_str_returns_none_for_uppercase_white() {
    assert_eq!(ThemeScheme::from_str("White"), None);
}

#[cfg(target_arch = "wasm32")]
#[test]
fn scheme_from_str_returns_none_for_system_string() {
    assert_eq!(ThemeScheme::from_str("system"), None);
}

#[cfg(target_arch = "wasm32")]
#[test]
fn scheme_from_str_returns_none_for_partial_white() {
    assert_eq!(ThemeScheme::from_str("whit"), None);
}

#[cfg(target_arch = "wasm32")]
#[test]
fn scheme_from_str_returns_none_for_partial_dark() {
    assert_eq!(ThemeScheme::from_str("dar"), None);
}

#[cfg(target_arch = "wasm32")]
#[test]
fn scheme_from_str_returns_none_for_partial_light() {
    assert_eq!(ThemeScheme::from_str("ligh"), None);
}
