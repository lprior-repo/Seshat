use diagram_models::document::DiagramDocument;
use dioxus::prelude::*;

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
    doc_signal: Signal<DiagramDocument>,
    viewport_size: Signal<(f64, f64)>,
) -> Element {
    let doc = doc_signal.read();
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
                        fill: "#444444",
                    }
                }
            }
            rect {
                x: "0",
                y: "0",
                width: "{vw.max(1.0)}",
                height: "{vh.max(1.0)}",
                fill: "#111111",
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
                fill: "#111111",
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
