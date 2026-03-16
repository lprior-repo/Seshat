#[cfg(kani)]
#[kani::proof]
#[test]
fn given_full_drag_workflow_from_raw_inputs_when_executed_then_yields_correct_final_state() {
    let mut dsl = CanvasTestDsl::new();

    // Idle -> Dragging
    dsl = dsl.when_raw_event(RawEvent {
        event_type: "mouse_down_target".to_string(),
        x: 10.0,
        y: 10.0,
        dx: 0.0,
        dy: 0.0,
        is_additive: false,
    });

    assert!(matches!(
        dsl.state.as_ref().unwrap(),
        InteractionState::Dragging { .. }
    ));

    // Dragging -> Dragging
    dsl = dsl.when_raw_event(RawEvent {
        event_type: "drag_move".to_string(),
        x: 0.0,
        y: 0.0,
        dx: 5.0,
        dy: 5.0,
        is_additive: false,
    });

    if let InteractionState::Dragging { drag } = dsl.state.as_ref().unwrap() {
        assert_eq!(drag.cumulative_offset, CanvasVector::new(5.0, 5.0).unwrap());
        assert_eq!(drag.current, CanvasPoint::new(15.0, 15.0).unwrap());
    } else {
        panic!("Expected Dragging state");
    }

    // Dragging -> Dragging
    dsl = dsl.when_raw_event(RawEvent {
        event_type: "drag_move".to_string(),
        x: 0.0,
        y: 0.0,
        dx: 2.0,
        dy: -1.0,
        is_additive: false,
    });

    if let InteractionState::Dragging { drag } = dsl.state.as_ref().unwrap() {
        assert_eq!(drag.cumulative_offset, CanvasVector::new(7.0, 4.0).unwrap());
        assert_eq!(drag.current, CanvasPoint::new(17.0, 14.0).unwrap());
    } else {
        panic!("Expected Dragging state");
    }

    // Dragging -> Idle
    dsl = dsl.when_raw_event(RawEvent {
        event_type: "mouse_up".to_string(),
        x: 0.0,
        y: 0.0,
        dx: 0.0,
        dy: 0.0,
        is_additive: false,
    });

    assert!(matches!(
        dsl.state.as_ref().unwrap(),
        InteractionState::Idle
    ));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_full_selection_workflow_from_raw_inputs_when_executed_then_yields_correct_selection_bounds(
) {
    let mut dsl = CanvasTestDsl::new();

    // Idle -> Selecting
    dsl = dsl.when_raw_event(RawEvent {
        event_type: "mouse_down_background".to_string(),
        x: 0.0,
        y: 0.0,
        dx: 0.0,
        dy: 0.0,
        is_additive: false,
    });

    assert!(matches!(
        dsl.state.as_ref().unwrap(),
        InteractionState::Selecting { .. }
    ));

    // Selecting -> Selecting
    dsl = dsl.when_raw_event(RawEvent {
        event_type: "mouse_move".to_string(),
        x: 50.0,
        y: 50.0,
        dx: 0.0,
        dy: 0.0,
        is_additive: false,
    });

    if let InteractionState::Selecting { start, current, .. } = dsl.state.as_ref().unwrap() {
        assert_eq!(start.x, 0.0);
        assert_eq!(start.y, 0.0);
        assert_eq!(current.x, 50.0);
        assert_eq!(current.y, 50.0);
    } else {
        panic!("Expected Selecting state");
    }

    // Selecting -> Idle
    dsl = dsl.when_raw_event(RawEvent {
        event_type: "mouse_up".to_string(),
        x: 0.0,
        y: 0.0,
        dx: 0.0,
        dy: 0.0,
        is_additive: false,
    });

    assert!(matches!(
        dsl.state.as_ref().unwrap(),
        InteractionState::Idle
    ));
}
