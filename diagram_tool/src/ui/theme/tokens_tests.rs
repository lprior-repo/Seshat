#![allow(clippy::unwrap_used)]

use super::*;

fn extract_lightness(oklch_str: &str) -> f64 {
    let start = oklch_str.find('(').expect("oklch( not found");
    let inner = &oklch_str[start + 1..oklch_str.len() - 1];
    let l_str = inner.split_whitespace().next().expect("no L* component");
    l_str.parse::<f64>().expect("L* not a float")
}

fn extract_chroma(oklch_str: &str) -> f64 {
    let start = oklch_str.find('(').expect("oklch( not found");
    let inner = &oklch_str[start + 1..oklch_str.len() - 1];
    let parts: Vec<&str> = inner.split_whitespace().collect();
    parts
        .get(1)
        .expect("no C component")
        .parse::<f64>()
        .expect("C not a float")
}

fn dark() -> ThemeTokens {
    dark_tokens()
}

fn light() -> ThemeTokens {
    light_tokens()
}

fn lightness(field_name: &str) -> f64 {
    let t = dark();
    extract_lightness(t.field(field_name))
}

fn lightness_light(field_name: &str) -> f64 {
    let t = light();
    extract_lightness(t.field(field_name))
}

#[test]
fn test_dark_border_subtle_lightness_is_0_30() {
    let l = lightness("border_subtle");
    assert!(
        (l - 0.30).abs() < 1e-6,
        "border_subtle L* = {l}, expected 0.30"
    );
}

#[test]
fn test_dark_node_bg_lightness_is_0_22() {
    let l = lightness("node_bg");
    assert!((l - 0.22).abs() < 1e-6, "node_bg L* = {l}, expected 0.22");
}

#[test]
fn test_dark_node_border_lightness_is_0_38() {
    let l = lightness("node_border");
    assert!(
        (l - 0.38).abs() < 1e-6,
        "node_border L* = {l}, expected 0.38"
    );
}

#[test]
fn test_dark_grid_dot_lightness_is_0_30() {
    let l = lightness("grid_dot");
    assert!((l - 0.30).abs() < 1e-6, "grid_dot L* = {l}, expected 0.30");
}

#[test]
fn test_dark_luminance_hierarchy() {
    let node_border = lightness("node_border");
    let border_subtle = lightness("border_subtle");
    let node_bg = lightness("node_bg");
    let bg_base = lightness("bg_base");
    assert!(
        bg_base < node_bg,
        "bg_base ({bg_base}) must be < node_bg ({node_bg})"
    );
    assert!(
        node_bg < border_subtle,
        "node_bg ({node_bg}) must be < border_subtle ({border_subtle})"
    );
    assert!(
        border_subtle < node_border,
        "border_subtle ({border_subtle}) must be < node_border ({node_border})"
    );
}

#[test]
fn test_light_tokens_unchanged() {
    let t = light();
    assert_eq!(t.field("border_subtle"), "oklch(0.88 0.01 260)");
    assert_eq!(t.field("node_bg"), "oklch(0.995 0.002 260)");
    assert_eq!(t.field("node_border"), "oklch(0.76 0.01 260)");
    assert_eq!(t.field("grid_dot"), "oklch(0.85 0.01 260)");
    assert_eq!(t.field("bg_base"), "oklch(0.96 0.005 260)");
    assert_eq!(t.field("bg_surface"), "oklch(0.985 0.004 260)");
    assert_eq!(t.field("bg_elevated"), "oklch(1 0 0)");
    assert_eq!(t.field("border"), "oklch(0.82 0.01 260)");
    assert_eq!(t.field("text_main"), "oklch(0.2 0.01 260)");
    assert_eq!(t.field("accent"), "oklch(0.62 0.16 192)");
}

#[test]
fn test_dark_chroma_preserved() {
    let t = dark();

    let neutral_fields = [
        "bg_base",
        "bg_surface",
        "bg_elevated",
        "border",
        "border_subtle",
        "node_bg",
        "grid_dot",
        "toolbar_bg",
    ];
    for &field in &neutral_fields {
        let c = extract_chroma(t.field(field));
        assert!((c - 0.005).abs() < 1e-6, "{field} C = {c}, expected 0.005");
    }

    let border_accented = ["node_border", "edge_default"];
    for &field in &border_accented {
        let c = extract_chroma(t.field(field));
        assert!((c - 0.01).abs() < 1e-6, "{field} C = {c}, expected 0.01");
    }
}
