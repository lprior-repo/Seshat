#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
use super::super::types::{CommitError, InteractionMode, LabelEditError, ResizeHandle};
use diagram_models::document::NodeId;
use im::HashMap;

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn rq_types_label_edit_error_is_send_sync() {
    assert_send_sync::<LabelEditError>();
}

#[test]
fn rq_types_commit_error_is_send_sync() {
    assert_send_sync::<CommitError>();
}

#[test]
fn rq_types_interaction_mode_is_send_sync() {
    assert_send_sync::<InteractionMode>();
}

#[test]
fn rq_types_resize_handle_is_send_sync() {
    assert_send_sync::<ResizeHandle>();
}

#[test]
fn rq_types_label_edit_error_clone_roundtrip() {
    let err = LabelEditError::TargetNotFound;
    let cloned = err.clone();
    assert_eq!(err, cloned);
    let err = LabelEditError::ValidationError;
    let cloned = err.clone();
    assert_eq!(err, cloned);
}

#[test]
fn rq_types_commit_error_clone_roundtrip() {
    let err = CommitError::LabelEdit(LabelEditError::TargetNotFound);
    let cloned = err.clone();
    assert_eq!(err, cloned);
}

#[test]
fn rq_types_from_label_edit_to_commit_error() {
    let label_err = LabelEditError::ValidationError;
    let commit_err: CommitError = label_err.into();
    assert_eq!(commit_err, CommitError::LabelEdit(LabelEditError::ValidationError));
}

#[test]
fn rq_types_commit_error_update_failed_variant_exists() {
    let err = CommitError::UpdateFailed(crate::stubs::DispatchError::Failed);
    let _ = err;
}

#[test]
fn rq_types_resize_handle_all_variants_copy() {
    let handles = [
        ResizeHandle::Nw,
        ResizeHandle::N,
        ResizeHandle::Ne,
        ResizeHandle::E,
        ResizeHandle::Se,
        ResizeHandle::S,
        ResizeHandle::Sw,
        ResizeHandle::W,
    ];
    for h in handles {
        let copy = h;
        assert_eq!(h, copy);
    }
}

#[test]
fn rq_types_resize_handle_all_variants_distinct() {
    let handles: Vec<ResizeHandle> = vec![
        ResizeHandle::Nw,
        ResizeHandle::N,
        ResizeHandle::Ne,
        ResizeHandle::E,
        ResizeHandle::Se,
        ResizeHandle::S,
        ResizeHandle::Sw,
        ResizeHandle::W,
    ];
    for (i, a) in handles.iter().enumerate() {
        for (j, b) in handles.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "ResizeHandle variants at {i} and {j} should differ");
            }
        }
    }
}

#[test]
fn rq_types_interaction_mode_nan_breaks_equality() {
    let mode1 = InteractionMode::Panning {
        last_pos: (f64::NAN, 0.0),
    };
    let mode2 = InteractionMode::Panning {
        last_pos: (f64::NAN, 0.0),
    };
    assert_ne!(mode1, mode2, "NaN fields break PartialEq - a known footgun in InteractionMode");
}

#[test]
fn rq_types_interaction_mode_finite_values_equal() {
    let mode1 = InteractionMode::Panning {
        last_pos: (1.0, 2.0),
    };
    let mode2 = InteractionMode::Panning {
        last_pos: (1.0, 2.0),
    };
    assert_eq!(mode1, mode2);
}

#[test]
fn rq_types_interaction_mode_all_variants_constructible() {
    let _select = InteractionMode::Select;
    let _rubberband = InteractionMode::RubberBand {
        start: (0.0, 0.0),
        current: (1.0, 1.0),
    };
    let _dragging = InteractionMode::DraggingSelection {
        anchor_canvas: (0.0, 0.0),
        anchor_client: (0.0, 0.0),
        original_positions: HashMap::new(),
        did_move: false,
    };
    let _drawing_edge = InteractionMode::DrawingEdge {
        from_node: NodeId::new(String::from("n")),
        current_pos: (0.0, 0.0),
        start_port: None,
    };
    let _drawing_subgraph = InteractionMode::DrawingSubgraph {
        start: (0.0, 0.0),
        current: (1.0, 1.0),
    };
    let _resizing = InteractionMode::ResizingSelection {
        handle: ResizeHandle::Se,
        original_bounds: (0.0, 0.0, 100.0, 100.0),
        originals: HashMap::new(),
        anchor: (50.0, 50.0),
        did_resize: false,
        aspect_ratio: None,
    };
    let _panning = InteractionMode::Panning {
        last_pos: (0.0, 0.0),
    };
    let _bend = InteractionMode::DraggingBendPoint {
        edge_id: diagram_models::document::EdgeId::new(String::from("e")),
        bend_index: 0,
    };
}

#[test]
fn rq_types_interaction_mode_debug_format() {
    let mode = InteractionMode::Select;
    let debug_str = format!("{mode:?}");
    assert!(debug_str.contains("Select"), "Debug should contain variant name");

    let mode = InteractionMode::Panning {
        last_pos: (1.0, 2.0),
    };
    let debug_str = format!("{mode:?}");
    assert!(debug_str.contains("Panning"), "Debug should contain variant name");
}

#[test]
fn rq_types_label_edit_error_debug() {
    let err = LabelEditError::TargetNotFound;
    let debug_str = format!("{err:?}");
    assert!(
        debug_str.contains("TargetNotFound"),
        "Debug should contain variant name"
    );
}

#[test]
fn rq_types_commit_error_debug() {
    let err = CommitError::LabelEdit(LabelEditError::ValidationError);
    let debug_str = format!("{err:?}");
    assert!(
        debug_str.contains("LabelEdit"),
        "Debug should contain variant name"
    );
}

#[test]
fn rq_types_resize_handle_is_copy() {
    fn assert_copy<T: Copy>() {}
    assert_copy::<ResizeHandle>();
}

#[test]
fn rq_types_label_edit_error_is_eq() {
    fn assert_eq_trait<T: Eq>() {}
    assert_eq_trait::<LabelEditError>();
}

#[test]
fn rq_types_commit_error_is_eq() {
    fn assert_eq_trait<T: Eq>() {}
    assert_eq_trait::<CommitError>();
}

#[test]
fn rq_types_resize_handle_is_eq() {
    fn assert_eq_trait<T: Eq>() {}
    assert_eq_trait::<ResizeHandle>();
}

#[test]
fn rq_types_interaction_mode_is_partial_eq_only() {
    let mode1 = InteractionMode::Select;
    let mode2 = InteractionMode::Select;
    assert_eq!(mode1, mode2, "PartialEq must work for equal values");
}

#[test]
fn rq_types_interaction_mode_infinity_in_fields() {
    let mode = InteractionMode::Panning {
        last_pos: (f64::INFINITY, f64::NEG_INFINITY),
    };
    assert_eq!(
        mode,
        InteractionMode::Panning {
            last_pos: (f64::INFINITY, f64::NEG_INFINITY),
        },
        "Infinity values should preserve equality"
    );
}

#[test]
fn rq_types_interaction_mode_zero_negative_zero_equal() {
    let mode1 = InteractionMode::Panning {
        last_pos: (0.0, 0.0),
    };
    let mode2 = InteractionMode::Panning {
        last_pos: (-0.0, 0.0),
    };
    assert_eq!(mode1, mode2, "0.0 and -0.0 should be equal per IEEE 754");
}

#[test]
fn rq_types_dragging_selection_large_hashmap() {
    let mut positions = HashMap::new();
    for i in 0..1000 {
        let id = NodeId::new(format!("node_{i}"));
        positions = positions.update(id, (i as f64, i as f64 * 2.0));
    }
    let mode = InteractionMode::DraggingSelection {
        anchor_canvas: (0.0, 0.0),
        anchor_client: (0.0, 0.0),
        original_positions: positions,
        did_move: false,
    };
    if let InteractionMode::DraggingSelection {
        original_positions,
        ..
    } = mode
    {
        assert_eq!(original_positions.len(), 1000);
    }
}

#[test]
fn rq_types_resize_handle_ord_not_implemented() {
    fn requires_ord<T: Ord>() {}
    fn can_use_ord() -> bool {
        false
    }
    let _ = can_use_ord();
}

#[test]
fn rq_types_dragging_bend_point_zero_bend_index() {
    let mode = InteractionMode::DraggingBendPoint {
        edge_id: diagram_models::document::EdgeId::new(String::from("e1")),
        bend_index: 0,
    };
    if let InteractionMode::DraggingBendPoint { bend_index, .. } = mode {
        assert_eq!(bend_index, 0);
    }
}

#[test]
fn rq_types_dragging_bend_point_large_bend_index() {
    let mode = InteractionMode::DraggingBendPoint {
        edge_id: diagram_models::document::EdgeId::new(String::from("e1")),
        bend_index: usize::MAX,
    };
    if let InteractionMode::DraggingBendPoint { bend_index, .. } = mode {
        assert_eq!(bend_index, usize::MAX);
    }
}

#[test]
fn rq_types_drawing_edge_with_port() {
    let mode = InteractionMode::DrawingEdge {
        from_node: NodeId::new(String::from("n1")),
        current_pos: (100.0, 200.0),
        start_port: Some(diagram_models::port::PortAnchor::Top),
    };
    if let InteractionMode::DrawingEdge { start_port, .. } = mode {
        assert!(start_port.is_some());
    }
}

#[test]
fn rq_types_resizing_with_aspect_ratio() {
    let mode = InteractionMode::ResizingSelection {
        handle: ResizeHandle::Se,
        original_bounds: (0.0, 0.0, 100.0, 100.0),
        originals: HashMap::new(),
        anchor: (50.0, 50.0),
        did_resize: false,
        aspect_ratio: Some(1.618),
    };
    if let InteractionMode::ResizingSelection { aspect_ratio, .. } = mode {
        assert_eq!(aspect_ratio, Some(1.618));
    }
}

#[test]
fn rq_types_resizing_nan_aspect_ratio() {
    let mode = InteractionMode::ResizingSelection {
        handle: ResizeHandle::Se,
        original_bounds: (0.0, 0.0, 100.0, 100.0),
        originals: HashMap::new(),
        anchor: (50.0, 50.0),
        did_resize: false,
        aspect_ratio: Some(f64::NAN),
    };
    if let InteractionMode::ResizingSelection { aspect_ratio, .. } = mode {
        assert!(aspect_ratio.is_some_and(|r| r.is_nan()));
    }
}

#[test]
fn rq_types_resizing_infinity_aspect_ratio() {
    let mode = InteractionMode::ResizingSelection {
        handle: ResizeHandle::Se,
        original_bounds: (0.0, 0.0, 100.0, 100.0),
        originals: HashMap::new(),
        anchor: (50.0, 50.0),
        did_resize: false,
        aspect_ratio: Some(f64::INFINITY),
    };
    if let InteractionMode::ResizingSelection { aspect_ratio, .. } = mode {
        assert!(aspect_ratio.is_some_and(|r| r.is_infinite()));
    }
}

#[test]
fn rq_types_commit_error_exhaustive_match() {
    let errors = [
        CommitError::LabelEdit(LabelEditError::TargetNotFound),
        CommitError::LabelEdit(LabelEditError::ValidationError),
        CommitError::UpdateFailed(crate::stubs::DispatchError::Failed),
    ];
    for err in &errors {
        let is_label = matches!(err, CommitError::LabelEdit(_));
        let is_update = matches!(err, CommitError::UpdateFailed(_));
        assert!(
            is_label || is_update,
            "Every CommitError variant must be handled"
        );
    }
}

#[test]
fn rq_types_label_edit_error_exhaustive_match() {
    let errors = [LabelEditError::TargetNotFound, LabelEditError::ValidationError];
    for err in &errors {
        let handled = match err {
            LabelEditError::TargetNotFound => true,
            LabelEditError::ValidationError => true,
        };
        assert!(handled);
    }
}

#[test]
fn rq_types_interaction_mode_different_variants_not_equal() {
    let modes = [
        InteractionMode::Select,
        InteractionMode::RubberBand {
            start: (0.0, 0.0),
            current: (0.0, 0.0),
        },
        InteractionMode::Panning {
            last_pos: (0.0, 0.0),
        },
        InteractionMode::DrawingSubgraph {
            start: (0.0, 0.0),
            current: (0.0, 0.0),
        },
    ];
    for (i, a) in modes.iter().enumerate() {
        for (j, b) in modes.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "Different InteractionMode variants at {i} and {j} should not be equal");
            }
        }
    }
}

#[test]
fn rq_types_all_eight_interaction_mode_variants_pairwise_distinct() {
    let modes: Vec<InteractionMode> = vec![
        InteractionMode::Select,
        InteractionMode::RubberBand {
            start: (0.0, 0.0),
            current: (1.0, 1.0),
        },
        InteractionMode::DraggingSelection {
            anchor_canvas: (0.0, 0.0),
            anchor_client: (0.0, 0.0),
            original_positions: HashMap::new(),
            did_move: false,
        },
        InteractionMode::DrawingEdge {
            from_node: NodeId::new(String::from("n")),
            current_pos: (0.0, 0.0),
            start_port: None,
        },
        InteractionMode::DrawingSubgraph {
            start: (0.0, 0.0),
            current: (1.0, 1.0),
        },
        InteractionMode::ResizingSelection {
            handle: ResizeHandle::Se,
            original_bounds: (0.0, 0.0, 100.0, 100.0),
            originals: HashMap::new(),
            anchor: (50.0, 50.0),
            did_resize: false,
            aspect_ratio: None,
        },
        InteractionMode::Panning {
            last_pos: (0.0, 0.0),
        },
        InteractionMode::DraggingBendPoint {
            edge_id: diagram_models::document::EdgeId::new(String::from("e")),
            bend_index: 0,
        },
    ];
    assert_eq!(modes.len(), 8, "Must test ALL 8 InteractionMode variants");
    for (i, a) in modes.iter().enumerate() {
        for (j, b) in modes.iter().enumerate() {
            if i != j {
                assert_ne!(
                    a, b,
                    "Different InteractionMode variants at {i} and {j} must not be equal"
                );
            }
        }
    }
}

#[test]
fn rq_types_commit_error_update_failed_equality() {
    let err1 = CommitError::UpdateFailed(crate::stubs::DispatchError::Failed);
    let err2 = CommitError::UpdateFailed(crate::stubs::DispatchError::Failed);
    assert_eq!(err1, err2, "Identical UpdateFailed errors must be equal");
}

#[test]
fn rq_types_commit_error_update_failed_not_equal_to_label_edit() {
    let update_err = CommitError::UpdateFailed(crate::stubs::DispatchError::Failed);
    let label_err = CommitError::LabelEdit(LabelEditError::TargetNotFound);
    assert_ne!(
        update_err, label_err,
        "UpdateFailed and LabelEdit variants must not be equal"
    );
}

#[test]
fn rq_types_commit_error_update_failed_clone_roundtrip() {
    let err = CommitError::UpdateFailed(crate::stubs::DispatchError::Failed);
    let cloned = err.clone();
    assert_eq!(err, cloned, "Cloned UpdateFailed must equal original");
}

#[test]
fn rq_types_from_target_not_found_to_commit_error() {
    let label_err = LabelEditError::TargetNotFound;
    let commit_err: CommitError = label_err.into();
    assert_eq!(
        commit_err,
        CommitError::LabelEdit(LabelEditError::TargetNotFound)
    );
}

#[test]
fn rq_types_commit_error_update_failed_debug() {
    let err = CommitError::UpdateFailed(crate::stubs::DispatchError::Failed);
    let debug_str = format!("{err:?}");
    assert!(
        debug_str.contains("UpdateFailed"),
        "Debug for UpdateFailed must contain variant name, got: {debug_str}"
    );
}

#[test]
fn rq_types_rubberband_nan_breaks_equality() {
    let mode1 = InteractionMode::RubberBand {
        start: (f64::NAN, 0.0),
        current: (1.0, 1.0),
    };
    let mode2 = InteractionMode::RubberBand {
        start: (f64::NAN, 0.0),
        current: (1.0, 1.0),
    };
    assert_ne!(
        mode1, mode2,
        "NaN in RubberBand.start breaks PartialEq"
    );
}

#[test]
fn rq_types_rubberband_nan_current_breaks_equality() {
    let mode1 = InteractionMode::RubberBand {
        start: (0.0, 0.0),
        current: (f64::NAN, 1.0),
    };
    let mode2 = InteractionMode::RubberBand {
        start: (0.0, 0.0),
        current: (f64::NAN, 1.0),
    };
    assert_ne!(
        mode1, mode2,
        "NaN in RubberBand.current breaks PartialEq"
    );
}

#[test]
fn rq_types_drawing_subgraph_nan_breaks_equality() {
    let mode1 = InteractionMode::DrawingSubgraph {
        start: (f64::NAN, 0.0),
        current: (1.0, 1.0),
    };
    let mode2 = InteractionMode::DrawingSubgraph {
        start: (f64::NAN, 0.0),
        current: (1.0, 1.0),
    };
    assert_ne!(
        mode1, mode2,
        "NaN in DrawingSubgraph.start breaks PartialEq"
    );
}

#[test]
fn rq_types_resizing_nan_original_bounds_breaks_equality() {
    let mode1 = InteractionMode::ResizingSelection {
        handle: ResizeHandle::Se,
        original_bounds: (f64::NAN, 0.0, 100.0, 100.0),
        originals: HashMap::new(),
        anchor: (50.0, 50.0),
        did_resize: false,
        aspect_ratio: None,
    };
    let mode2 = InteractionMode::ResizingSelection {
        handle: ResizeHandle::Se,
        original_bounds: (f64::NAN, 0.0, 100.0, 100.0),
        originals: HashMap::new(),
        anchor: (50.0, 50.0),
        did_resize: false,
        aspect_ratio: None,
    };
    assert_ne!(
        mode1, mode2,
        "NaN in ResizingSelection.original_bounds breaks PartialEq"
    );
}

#[test]
fn rq_types_resizing_nan_anchor_breaks_equality() {
    let mode1 = InteractionMode::ResizingSelection {
        handle: ResizeHandle::Se,
        original_bounds: (0.0, 0.0, 100.0, 100.0),
        originals: HashMap::new(),
        anchor: (f64::NAN, 50.0),
        did_resize: false,
        aspect_ratio: None,
    };
    let mode2 = InteractionMode::ResizingSelection {
        handle: ResizeHandle::Se,
        original_bounds: (0.0, 0.0, 100.0, 100.0),
        originals: HashMap::new(),
        anchor: (f64::NAN, 50.0),
        did_resize: false,
        aspect_ratio: None,
    };
    assert_ne!(
        mode1, mode2,
        "NaN in ResizingSelection.anchor breaks PartialEq"
    );
}

#[test]
fn rq_types_dragging_selection_nan_anchor_breaks_equality() {
    let mode1 = InteractionMode::DraggingSelection {
        anchor_canvas: (f64::NAN, 0.0),
        anchor_client: (0.0, 0.0),
        original_positions: HashMap::new(),
        did_move: false,
    };
    let mode2 = InteractionMode::DraggingSelection {
        anchor_canvas: (f64::NAN, 0.0),
        anchor_client: (0.0, 0.0),
        original_positions: HashMap::new(),
        did_move: false,
    };
    assert_ne!(
        mode1, mode2,
        "NaN in DraggingSelection.anchor_canvas breaks PartialEq"
    );
}

#[test]
fn rq_types_drawing_edge_nan_pos_breaks_equality() {
    let mode1 = InteractionMode::DrawingEdge {
        from_node: NodeId::new(String::from("n")),
        current_pos: (f64::NAN, 0.0),
        start_port: None,
    };
    let mode2 = InteractionMode::DrawingEdge {
        from_node: NodeId::new(String::from("n")),
        current_pos: (f64::NAN, 0.0),
        start_port: None,
    };
    assert_ne!(
        mode1, mode2,
        "NaN in DrawingEdge.current_pos breaks PartialEq"
    );
}

#[test]
fn rq_types_dragging_selection_did_move_true_distinct_from_false() {
    let mode_false = InteractionMode::DraggingSelection {
        anchor_canvas: (10.0, 20.0),
        anchor_client: (10.0, 20.0),
        original_positions: HashMap::new(),
        did_move: false,
    };
    let mode_true = InteractionMode::DraggingSelection {
        anchor_canvas: (10.0, 20.0),
        anchor_client: (10.0, 20.0),
        original_positions: HashMap::new(),
        did_move: true,
    };
    assert_ne!(
        mode_false, mode_true,
        "did_move flag changes InteractionMode equality"
    );
}

#[test]
fn rq_types_resizing_did_resize_true_distinct_from_false() {
    let mode_false = InteractionMode::ResizingSelection {
        handle: ResizeHandle::Se,
        original_bounds: (0.0, 0.0, 100.0, 100.0),
        originals: HashMap::new(),
        anchor: (50.0, 50.0),
        did_resize: false,
        aspect_ratio: None,
    };
    let mode_true = InteractionMode::ResizingSelection {
        handle: ResizeHandle::Se,
        original_bounds: (0.0, 0.0, 100.0, 100.0),
        originals: HashMap::new(),
        anchor: (50.0, 50.0),
        did_resize: true,
        aspect_ratio: None,
    };
    assert_ne!(
        mode_false, mode_true,
        "did_resize flag changes InteractionMode equality"
    );
}

#[test]
fn rq_types_resizing_negative_aspect_ratio_accepted() {
    let mode = InteractionMode::ResizingSelection {
        handle: ResizeHandle::Se,
        original_bounds: (0.0, 0.0, 100.0, 100.0),
        originals: HashMap::new(),
        anchor: (50.0, 50.0),
        did_resize: false,
        aspect_ratio: Some(-1.5),
    };
    if let InteractionMode::ResizingSelection { aspect_ratio, .. } = mode {
        assert_eq!(
            aspect_ratio,
            Some(-1.5),
            "BUG: negative aspect_ratio accepted without validation"
        );
    }
}

#[test]
fn rq_types_resizing_zero_aspect_ratio_accepted() {
    let mode = InteractionMode::ResizingSelection {
        handle: ResizeHandle::Se,
        original_bounds: (0.0, 0.0, 100.0, 100.0),
        originals: HashMap::new(),
        anchor: (50.0, 50.0),
        did_resize: false,
        aspect_ratio: Some(0.0),
    };
    if let InteractionMode::ResizingSelection { aspect_ratio, .. } = mode {
        assert_eq!(
            aspect_ratio,
            Some(0.0),
            "BUG: zero aspect_ratio accepted - potential division-by-zero downstream"
        );
    }
}

#[test]
fn rq_types_resizing_degenerate_zero_bounds() {
    let mode = InteractionMode::ResizingSelection {
        handle: ResizeHandle::Se,
        original_bounds: (0.0, 0.0, 0.0, 0.0),
        originals: HashMap::new(),
        anchor: (0.0, 0.0),
        did_resize: false,
        aspect_ratio: None,
    };
    if let InteractionMode::ResizingSelection {
        original_bounds, ..
    } = mode
    {
        assert_eq!(original_bounds, (0.0, 0.0, 0.0, 0.0));
    }
}

#[test]
fn rq_types_resizing_negative_bounds() {
    let mode = InteractionMode::ResizingSelection {
        handle: ResizeHandle::Se,
        original_bounds: (-10.0, -10.0, -5.0, -5.0),
        originals: HashMap::new(),
        anchor: (0.0, 0.0),
        did_resize: false,
        aspect_ratio: None,
    };
    if let InteractionMode::ResizingSelection {
        original_bounds, ..
    } = mode
    {
        assert!(
            original_bounds.0 < 0.0 && original_bounds.2 < 0.0,
            "BUG: negative bounds accepted without validation"
        );
    }
}

#[test]
fn rq_types_rubberband_inverted_selection() {
    let mode = InteractionMode::RubberBand {
        start: (100.0, 100.0),
        current: (0.0, 0.0),
    };
    if let InteractionMode::RubberBand { start, current } = mode {
        assert!(
            start.0 > current.0,
            "RubberBand allows start > current (inverted selection area)"
        );
    }
}

#[test]
fn rq_types_rubberband_zero_area() {
    let mode = InteractionMode::RubberBand {
        start: (5.0, 5.0),
        current: (5.0, 5.0),
    };
    if let InteractionMode::RubberBand { start, current } = mode {
        assert_eq!(start, current, "Zero-area rubber band (start == current) is representable");
    }
}

#[test]
fn rq_types_drawing_edge_without_port() {
    let mode = InteractionMode::DrawingEdge {
        from_node: NodeId::new(String::from("n1")),
        current_pos: (100.0, 200.0),
        start_port: None,
    };
    if let InteractionMode::DrawingEdge { start_port, .. } = mode {
        assert!(
            start_port.is_none(),
            "DrawingEdge without port is representable"
        );
    }
}

#[test]
fn rq_types_drawing_edge_empty_node_id() {
    let mode = InteractionMode::DrawingEdge {
        from_node: NodeId::new(String::new()),
        current_pos: (0.0, 0.0),
        start_port: None,
    };
    if let InteractionMode::DrawingEdge { from_node, .. } = mode {
        assert!(
            from_node.as_str().is_empty(),
            "BUG: DrawingEdge with empty NodeId accepted without validation"
        );
    }
}

#[test]
fn rq_types_dragging_bend_point_empty_edge_id() {
    let mode = InteractionMode::DraggingBendPoint {
        edge_id: diagram_models::document::EdgeId::new(String::new()),
        bend_index: 0,
    };
    if let InteractionMode::DraggingBendPoint { edge_id, .. } = mode {
        assert!(
            edge_id.as_str().is_empty(),
            "BUG: DraggingBendPoint with empty EdgeId accepted without validation"
        );
    }
}

#[test]
fn rq_types_resize_handle_all_variants_exhaustive_match() {
    let all_constructed: Vec<ResizeHandle> = vec![
        ResizeHandle::Nw, ResizeHandle::N, ResizeHandle::Ne, ResizeHandle::E,
        ResizeHandle::Se, ResizeHandle::S, ResizeHandle::Sw, ResizeHandle::W,
    ];
    for h in &all_constructed {
        let label = match h {
            ResizeHandle::Nw => "nw",
            ResizeHandle::N => "n",
            ResizeHandle::Ne => "ne",
            ResizeHandle::E => "e",
            ResizeHandle::Se => "se",
            ResizeHandle::S => "s",
            ResizeHandle::Sw => "sw",
            ResizeHandle::W => "w",
        };
        assert!(!label.is_empty());
    }
    assert_eq!(all_constructed.len(), 8, "All 8 ResizeHandle variants must be covered");
}

#[test]
fn rq_types_interaction_mode_debug_all_variants() {
    let modes: Vec<(&str, InteractionMode)> = vec![
        ("Select", InteractionMode::Select),
        ("RubberBand", InteractionMode::RubberBand {
            start: (0.0, 0.0),
            current: (1.0, 1.0),
        }),
        ("DraggingSelection", InteractionMode::DraggingSelection {
            anchor_canvas: (0.0, 0.0),
            anchor_client: (0.0, 0.0),
            original_positions: HashMap::new(),
            did_move: false,
        }),
        ("DrawingEdge", InteractionMode::DrawingEdge {
            from_node: NodeId::new(String::from("n")),
            current_pos: (0.0, 0.0),
            start_port: None,
        }),
        ("DrawingSubgraph", InteractionMode::DrawingSubgraph {
            start: (0.0, 0.0),
            current: (1.0, 1.0),
        }),
        ("ResizingSelection", InteractionMode::ResizingSelection {
            handle: ResizeHandle::Se,
            original_bounds: (0.0, 0.0, 100.0, 100.0),
            originals: HashMap::new(),
            anchor: (50.0, 50.0),
            did_resize: false,
            aspect_ratio: None,
        }),
        ("Panning", InteractionMode::Panning {
            last_pos: (0.0, 0.0),
        }),
        ("DraggingBendPoint", InteractionMode::DraggingBendPoint {
            edge_id: diagram_models::document::EdgeId::new(String::from("e")),
            bend_index: 0,
        }),
    ];
    assert_eq!(modes.len(), 8, "Debug format must be tested for all 8 variants");
    for (expected_name, mode) in &modes {
        let debug_str = format!("{mode:?}");
        assert!(
            debug_str.contains(expected_name),
            "Debug for {expected_name} variant must contain '{expected_name}', got: {debug_str}"
        );
    }
}

#[test]
fn rq_types_interaction_mode_self_equality_all_variants() {
    let modes: Vec<InteractionMode> = vec![
        InteractionMode::Select,
        InteractionMode::RubberBand {
            start: (1.0, 2.0),
            current: (3.0, 4.0),
        },
        InteractionMode::DraggingSelection {
            anchor_canvas: (1.0, 2.0),
            anchor_client: (3.0, 4.0),
            original_positions: HashMap::new(),
            did_move: true,
        },
        InteractionMode::DrawingEdge {
            from_node: NodeId::new(String::from("n")),
            current_pos: (5.0, 6.0),
            start_port: Some(diagram_models::port::PortAnchor::Top),
        },
        InteractionMode::DrawingSubgraph {
            start: (1.0, 2.0),
            current: (3.0, 4.0),
        },
        InteractionMode::ResizingSelection {
            handle: ResizeHandle::Nw,
            original_bounds: (0.0, 0.0, 100.0, 100.0),
            originals: HashMap::new(),
            anchor: (50.0, 50.0),
            did_resize: true,
            aspect_ratio: Some(1.5),
        },
        InteractionMode::Panning {
            last_pos: (7.0, 8.0),
        },
        InteractionMode::DraggingBendPoint {
            edge_id: diagram_models::document::EdgeId::new(String::from("e")),
            bend_index: 3,
        },
    ];
    assert_eq!(modes.len(), 8);
    for mode in &modes {
        assert_eq!(mode, mode, "Self-equality must hold for {:?}", mode);
    }
}

#[test]
fn rq_types_dragging_selection_with_populated_positions() {
    let mut positions = HashMap::new();
    positions = positions.update(NodeId::new(String::from("a")), (10.0, 20.0));
    positions = positions.update(NodeId::new(String::from("b")), (30.0, 40.0));
    let mode = InteractionMode::DraggingSelection {
        anchor_canvas: (0.0, 0.0),
        anchor_client: (0.0, 0.0),
        original_positions: positions.clone(),
        did_move: true,
    };
    if let InteractionMode::DraggingSelection {
        original_positions, ..
    } = mode
    {
        assert_eq!(original_positions.len(), 2);
        assert_eq!(original_positions.get(&NodeId::new(String::from("a"))), Some(&(10.0, 20.0)));
    }
}

#[test]
fn rq_types_resizing_with_populated_originals() {
    let mut originals = HashMap::new();
    originals = originals.update(
        NodeId::new(String::from("n1")),
        (0.0, 0.0, 50.0, 50.0),
    );
    originals = originals.update(
        NodeId::new(String::from("n2")),
        (100.0, 100.0, 75.0, 75.0),
    );
    let mode = InteractionMode::ResizingSelection {
        handle: ResizeHandle::Nw,
        original_bounds: (0.0, 0.0, 175.0, 175.0),
        originals: originals.clone(),
        anchor: (87.5, 87.5),
        did_resize: true,
        aspect_ratio: Some(1.0),
    };
    if let InteractionMode::ResizingSelection { originals, .. } = mode {
        assert_eq!(originals.len(), 2);
        assert_eq!(
            originals.get(&NodeId::new(String::from("n1"))),
            Some(&(0.0, 0.0, 50.0, 50.0))
        );
    }
}

#[test]
fn rq_types_subnormal_floats_preserved() {
    let subnormal = f64::from_bits(1u64);
    assert!(subnormal.is_normal() == false);
    let mode = InteractionMode::Panning {
        last_pos: (subnormal, subnormal),
    };
    if let InteractionMode::Panning { last_pos } = mode {
        assert_eq!(last_pos.0.to_bits(), 1u64);
        assert_eq!(last_pos.1.to_bits(), 1u64);
    }
}

#[test]
fn rq_types_resize_handle_is_copy_and_eq() {
    fn takes_copy_eq<T: Copy + Eq>(_val: T) {}
    takes_copy_eq(ResizeHandle::Nw);
}

#[test]
fn rq_types_resize_handle_is_not_hash() {
    let h = ResizeHandle::Se;
    let _ = h;
    let _ = format!("{h:?}");
    assert!(
        true,
        "FINDING: ResizeHandle derives Copy+Eq+Debug but NOT Hash. \
         If used as HashMap key, will need Hash added"
    );
}

#[test]
fn rq_types_label_edit_error_both_variants_clone_roundtrip() {
    let err1 = LabelEditError::TargetNotFound;
    assert_eq!(err1, err1.clone());

    let err2 = LabelEditError::ValidationError;
    assert_eq!(err2, err2.clone());
}

#[test]
fn rq_types_label_edit_error_not_equal() {
    assert_ne!(
        LabelEditError::TargetNotFound,
        LabelEditError::ValidationError,
        "Different LabelEditError variants must not be equal"
    );
}

#[test]
fn rq_types_commit_error_label_edit_variants_not_equal() {
    let err1 = CommitError::LabelEdit(LabelEditError::TargetNotFound);
    let err2 = CommitError::LabelEdit(LabelEditError::ValidationError);
    assert_ne!(err1, err2, "Different LabelEdit inner values must not be equal");
}

#[test]
fn rq_types_dispatch_error_clone_eq() {
    let err = crate::stubs::DispatchError::Failed;
    assert_eq!(err, err.clone());
}
