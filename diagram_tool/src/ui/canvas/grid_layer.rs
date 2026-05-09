use crate::ui::theme::{BG_BASE, GRID_DOT};
use dioxus::prelude::*;
use im::HashSet as ImHashSet;

#[must_use]
pub fn calculate_grid_pattern(
    grid_size: f64,
    zoom: f64,
    camera_x: f64,
    camera_y: f64,
) -> (f64, f64, f64, f64) {
    let pattern_step = (grid_size.max(8.0) * zoom).max(4.0);
    let pattern_x = (-camera_x * zoom - pattern_step / 2.0).rem_euclid(pattern_step);
    let pattern_y = (-camera_y * zoom - pattern_step / 2.0).rem_euclid(pattern_step);
    let dot_r = 1.5 * zoom;
    (pattern_step, pattern_x, pattern_y, dot_r)
}

#[component]
pub fn GridLayer(
    /// Lightweight trigger Memo for camera/selection data.
    /// `GridLayer` subscribes to this instead of the full `doc_signal`
    /// to avoid re-rendering on every node position change during drag.
    node_viewport_trigger: Memo<(u64, f64, f64, f64, ImHashSet<String>)>,
    doc_signal: Signal<diagram_models::document::DiagramDocument>,
    viewport_size: Signal<(f64, f64)>,
) -> Element {
    let (_revision, _camera_x, _camera_y, _zoom, _selected_items) =
        node_viewport_trigger.read().clone();
    // Peek at doc for grid_size and show_grid — does NOT subscribe to doc_signal.
    let doc = doc_signal.peek();
    let s = &doc.editor_state;
    let (vw, vh) = *viewport_size.read();
    let (pattern_step, pattern_x, pattern_y, dot_r) =
        calculate_grid_pattern(s.grid_size.0, s.zoom.0, s.camera_x.0, s.camera_y.0);

    if s.show_grid && s.zoom.0 >= 0.3 {
        rsx! {
            defs {
                pattern {
                    id: "canvas-grid-dot-pattern",
                    pattern_units: "userSpaceOnUse",
                    x: "{pattern_x}",
                    y: "{pattern_y}",
                    width: "{pattern_step}",
                    height: "{pattern_step}",
                    circle {
                        cx: "{pattern_step / 2.0}",
                        cy: "{pattern_step / 2.0}",
                        r: "{dot_r}",
                        style: "fill: {GRID_DOT};",
                    }
                }
            }
            rect {
                x: "0",
                y: "0",
                width: "{vw.max(1.0)}",
                height: "{vh.max(1.0)}",
                style: "fill: {BG_BASE};",
            }
            rect {
                x: "0",
                y: "0",
                width: "{vw.max(1.0)}",
                height: "{vh.max(1.0)}",
                fill: "url(#canvas-grid-dot-pattern)",
            }
        }
    } else {
        rsx! {
            rect {
                x: "0",
                y: "0",
                width: "{vw.max(1.0)}",
                height: "{vh.max(1.0)}",
                style: "fill: {BG_BASE};",
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::{BG_BASE, GRID_DOT};

    #[test]
    fn test_grid_pattern_alignment_with_nodes() {
        // Grid size 20, zoom 1.0, camera at origin
        let (step, px, py, r) = calculate_grid_pattern(20.0, 1.0, 0.0, 0.0);
        assert_eq!(step, 20.0);
        // Expect the pattern start to be offset by half a step to align the dot (at step/2) with the grid intersection
        assert_eq!(px, 10.0);
        assert_eq!(py, 10.0);
        assert_eq!(r, 1.5);
    }

    #[test]
    fn test_grid_pattern_alignment_with_camera_offset() {
        // Camera panned by 5 units right and down
        let (step, px, py, r) = calculate_grid_pattern(20.0, 1.0, 5.0, 5.0);
        assert_eq!(step, 20.0);
        // Original offset is -10. Pan by 5 means content moves left by 5.
        // So offset is (-5 - 10) mod 20 = -15 mod 20 = 5.0
        assert_eq!(px, 5.0);
        assert_eq!(py, 5.0);
        assert_eq!(r, 1.5);
    }

    #[test]
    fn test_grid_pattern_scales_with_zoom() {
        // Grid size 20, zoom 2.0 -> visual step is 40
        let (step, px, py, r) = calculate_grid_pattern(20.0, 2.0, 0.0, 0.0);
        assert_eq!(step, 40.0);
        assert_eq!(px, 20.0);
        assert_eq!(py, 20.0);
        assert_eq!(r, 3.0);
    }

    // ── Regression: calculate_grid_pattern pure function unchanged ──

    #[test]
    fn test_grid_pattern_minimum_step_clamp() {
        // Very small grid_size (2) and low zoom (0.5) should still produce >= 4.0 step
        let (step, _px, _py, _r) = calculate_grid_pattern(2.0, 0.5, 0.0, 0.0);
        assert!(step >= 4.0);
    }

    #[test]
    fn test_grid_pattern_negative_camera() {
        // Negative camera offset should wrap correctly via rem_euclid
        let (step, px, py, _r) = calculate_grid_pattern(20.0, 1.0, -15.0, -15.0);
        assert_eq!(step, 20.0);
        // rem_euclid always returns non-negative, so px/py in [0, step)
        assert!(px >= 0.0 && px < step);
        assert!(py >= 0.0 && py < step);
    }

    #[test]
    fn test_grid_pattern_zero_zoom_clamps() {
        // zoom=0 should still produce non-negative step via max(4.0)
        let (step, _px, _py, _r) = calculate_grid_pattern(20.0, 0.0, 0.0, 0.0);
        assert!(step >= 4.0);
    }

    // ── Contract: theme constants are CSS variable references ──

    #[test]
    fn test_grid_dot_is_css_variable() {
        assert!(
            GRID_DOT.starts_with("var("),
            "GRID_DOT must be a CSS custom property reference, got: {GRID_DOT}"
        );
    }

    #[test]
    fn test_bg_base_is_css_variable() {
        assert!(
            BG_BASE.starts_with("var("),
            "BG_BASE must be a CSS custom property reference, got: {BG_BASE}"
        );
    }

    #[test]
    fn test_grid_dot_references_grid_token() {
        assert!(
            GRID_DOT.contains("--grid-dot"),
            "GRID_DOT must reference --grid-dot CSS token, got: {GRID_DOT}"
        );
    }

    #[test]
    fn test_bg_base_references_bg_token() {
        assert!(
            BG_BASE.contains("--bg-base"),
            "BG_BASE must reference --bg-base CSS token, got: {BG_BASE}"
        );
    }

    #[test]
    fn grid_dot_and_bg_base_are_css_variables_not_hardcoded_hex() {
        assert!(
            !GRID_DOT.contains('#'),
            "GRID_DOT must not contain hardcoded hex, got: {GRID_DOT}"
        );
        assert!(
            !BG_BASE.contains('#'),
            "BG_BASE must not contain hardcoded hex, got: {BG_BASE}"
        );
    }

    #[test]
    fn grid_layer_compiles_with_style_attribute_css_variable_references() {
        let pattern_step = calculate_grid_pattern(20.0, 1.0, 0.0, 0.0).0;
        let dot_style = format!("fill: {GRID_DOT};");
        let bg_style = format!("fill: {BG_BASE};");
        assert!(dot_style.contains("var(--grid-dot)"));
        assert!(bg_style.contains("var(--bg-base)"));
        assert!(!dot_style.contains("fill: \""));
        assert!(!bg_style.contains("fill: \""));
    }
}
