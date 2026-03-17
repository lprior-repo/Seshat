    use crate::drag_math::subgraphs::*;
    use diagram_models::document::has_drag_threshold;
    use diagram_models::document::NodeId;
    use diagram_models::document::{snap_point, snap_value, GridSize};
    use im::HashMap;

    #[kani::proof]
    #[kani::unwind(3)]
    fn verify_calculate_resize_targets_preserves_selection() {
        let id1 = NodeId::new("n1".to_string());
        let id2 = NodeId::new("n2".to_string());

        let selected = vec![id1.clone()];

        let mut geom = HashMap::new();
        let x: f64 = kani::any();
        let y: f64 = kani::any();
        let w: f64 = kani::any();
        let h: f64 = kani::any();
        let is_subgraph: bool = kani::any();

        kani::assume(x.is_finite());
        kani::assume(y.is_finite());
        kani::assume(w.is_finite() && w >= 0.0);
        kani::assume(h.is_finite() && h >= 0.0);

        geom.insert(id1.clone(), (x, y, w, h, is_subgraph));
        geom.insert(id2.clone(), (0.0, 0.0, 10.0, 10.0, false));

        let targets = calculate_resize_target_ids(&selected, &geom);
        assert!(targets.contains(&id1));
        assert!(targets.len() >= 1);
    }

    #[kani::proof]
    #[kani::unwind(4)]
    fn verify_calculate_resize_targets_includes_within() {
        let parent_id = NodeId::new("parent".to_string());
        let child_id = NodeId::new("child".to_string());

        let selected = vec![parent_id.clone()];

        let px: f64 = kani::any();
        let py: f64 = kani::any();
        let pw: f64 = kani::any();
        let ph: f64 = kani::any();

        let cx: f64 = kani::any();
        let cy: f64 = kani::any();
        let cw: f64 = kani::any();
        let ch: f64 = kani::any();

        kani::assume(px.is_finite());
        kani::assume(py.is_finite());
        kani::assume(pw.is_finite() && pw >= 0.0);
        kani::assume(ph.is_finite() && ph >= 0.0);

        kani::assume(cx.is_finite());
        kani::assume(cy.is_finite());
        kani::assume(cw.is_finite() && cw >= 0.0);
        kani::assume(ch.is_finite() && ch >= 0.0);

        // Child is strictly within parent
        kani::assume(cx >= px);
        kani::assume(cy >= py);
        kani::assume(cx + cw <= px + pw);
        kani::assume(cy + ch <= py + ph);

        let mut geom = HashMap::new();
        geom.insert(parent_id.clone(), (px, py, pw, ph, true)); // is_subgraph = true
        geom.insert(child_id.clone(), (cx, cy, cw, ch, false));

        let targets = calculate_resize_target_ids(&selected, &geom);

        assert!(targets.contains(&parent_id));
        assert!(targets.contains(&child_id));
    }

    #[kani::proof]
    fn verify_snap_value_bounds() {
        let val: f64 = kani::any();
        let grid_val: f64 = kani::any();
        let snap: bool = kani::any();

        kani::assume(val.is_finite());
        kani::assume(grid_val.is_finite());
        kani::assume(grid_val >= 10.0 && grid_val <= 100.0);

        let grid = GridSize::new(grid_val).unwrap();
        let snapped = snap_value(val, snap, grid);

        assert!(snapped.is_finite() || val.is_infinite() || val.is_nan());

        if snap && val.is_finite() {
            let diff = (snapped - val).abs();
            // max difference should be grid / 2, adding epsilon for floating point math
            assert!(diff <= (grid_val / 2.0) + 1e-5);
        } else if !snap {
            assert_eq!(snapped, val);
        }
    }

    #[kani::proof]
    fn verify_drag_threshold() {
        let ox: f64 = kani::any();
        let oy: f64 = kani::any();
        let cx: f64 = kani::any();
        let cy: f64 = kani::any();

        kani::assume(ox.is_finite());
        kani::assume(oy.is_finite());
        kani::assume(cx.is_finite());
        kani::assume(cy.is_finite());

        // Constrain to prevent overflow when squaring
        kani::assume((cx - ox).abs() < 1e50);
        kani::assume((cy - oy).abs() < 1e50);

        let dx = cx - ox;
        let dy = cy - oy;
        let dist_sq = dx * dx + dy * dy;

        let result = has_drag_threshold((ox, oy), (cx, cy));

        // DRAG_THRESHOLD_PX is 3.0, so dist_sq threshold is 9.0
        if dist_sq < 9.0 {
            assert!(!result);
        }

        if dist_sq >= 9.0001 {
            // Account for f64 precision
            assert!(result);
        }
    }
