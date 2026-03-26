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

fn white() -> ThemeTokens {
    white_tokens()
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

// --- seshat-b36: White palette tests ---

#[test]
fn test_white_bg_base_is_pure_white() {
    let t = white();
    let val = t.bg_base;
    assert!(
        val.contains("oklch(1"),
        "bg_base must be pure white oklch(1 ...), got: {val}"
    );
}

#[test]
fn test_white_text_main_is_dark() {
    let t = white();
    let l = extract_lightness(t.text_main);
    assert!(
        l < 0.3,
        "text_main L* = {l}, must be < 0.3 for WCAG AA contrast on white"
    );
}

#[test]
fn test_white_node_border_visible() {
    let t = white();
    let l = extract_lightness(t.node_border);
    // On oklch(1 0 0) background, L* <= 0.60 gives > 3:1 contrast ratio.
    assert!(
        (0.30..=0.60).contains(&l),
        "node_border L* = {l}, must be in [0.30, 0.60] for 3:1 contrast on white"
    );
}

#[test]
fn test_white_palette_completeness() {
    let t = white();
    let fields = [
        t.bg_base,
        t.bg_surface,
        t.bg_elevated,
        t.border,
        t.border_subtle,
        t.text_main,
        t.text_muted,
        t.text_dim,
        t.accent,
        t.accent_soft,
        t.selection_rect_fill,
        t.subgraph_preview_fill,
        t.node_bg,
        t.node_bg_subgraph,
        t.node_border,
        t.grid_dot,
        t.edge_default,
        t.toolbar_bg,
        t.success,
        t.error,
        t.warning,
        t.chart_1,
        t.chart_2,
        t.chart_3,
        t.chart_4,
        t.chart_5,
    ];
    assert_eq!(fields.len(), 26, "expected 26 token fields");
    for (i, val) in fields.iter().enumerate() {
        assert!(!val.is_empty(), "token field index {i} must not be empty");
    }
}

#[test]
fn test_css_vars_for_white_produces_valid_output() {
    let css = super::super::css_vars_for(super::super::ThemeScheme::White);
    assert!(
        !css.is_empty(),
        "css_vars_for(White) must produce non-empty output"
    );
    // Every --var:value; segment must have a non-empty value before the semicolon.
    for segment in css.split(';') {
        let trimmed = segment.trim();
        if trimmed.is_empty() {
            continue;
        }
        assert!(
            trimmed.contains(':'),
            "CSS segment missing ':' delimiter: {trimmed}"
        );
        let colon_pos = trimmed.find(':').expect("colon checked above");
        let value = trimmed[colon_pos + 1..].trim();
        assert!(
            !value.is_empty(),
            "CSS variable has empty value in segment: {trimmed}"
        );
    }
}

#[test]
fn test_white_palette_differs_from_light() {
    let w = white();
    let l = light();
    assert_ne!(
        w.bg_base, l.bg_base,
        "White bg_base must differ from Light bg_base"
    );
}

#[test]
fn token_field_count_matches_match_arms() {
    let all_fields: [&str; TOKEN_FIELD_COUNT] = [
        "bg_base",
        "bg_surface",
        "bg_elevated",
        "border",
        "border_subtle",
        "text_main",
        "text_muted",
        "text_dim",
        "accent",
        "accent_soft",
        "selection_rect_fill",
        "subgraph_preview_fill",
        "node_bg",
        "node_bg_subgraph",
        "node_border",
        "grid_dot",
        "edge_default",
        "toolbar_bg",
        "success",
        "error",
        "warning",
        "chart_1",
        "chart_2",
        "chart_3",
        "chart_4",
        "chart_5",
    ];
    assert_eq!(
        all_fields.len(),
        TOKEN_FIELD_COUNT,
        "all_fields array length must equal TOKEN_FIELD_COUNT"
    );
    let t = dark();
    for name in &all_fields {
        let val = t.field(name);
        assert!(
            !val.is_empty(),
            "field({name}) returned empty — missing match arm?"
        );
    }
}
