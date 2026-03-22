#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use crate::ui::canvas::domain::{
    transition, CanvasEvent, CanvasPoint, CanvasVector, DragState, InteractionState, SelectionMode,
};

fn pt(x: f64, y: f64) -> CanvasPoint {
    CanvasPoint::new(x, y).unwrap()
}

fn vec(dx: f64, dy: f64) -> CanvasVector {
    CanvasVector::new(dx, dy).unwrap()
}

fn drag_state(start: CanvasPoint) -> DragState {
    DragState {
        start,
        current: start,
        cumulative_offset: vec(0.0, 0.0),
    }
}

// -----------------------------------------------------------------------------
// Combinatorial Permutations: Idle State
// -----------------------------------------------------------------------------

#[test]
fn test_idle_to_hovering_via_mouse_move() {
    let result = transition(
        InteractionState::Idle,
        CanvasEvent::MouseMove {
            point: pt(10.0, 20.0),
        },
    );
    assert!(
        matches!(result.unwrap(), InteractionState::Hovering { point } if point.x == 10.0 && point.y == 20.0)
    );
}

#[test]
fn test_idle_remains_idle_via_touch_move() {
    let result = transition(
        InteractionState::Idle,
        CanvasEvent::TouchMove {
            point: pt(10.0, 20.0),
            delta: vec(1.0, 1.0),
        },
    );
    assert!(matches!(result.unwrap(), InteractionState::Idle));
}

#[test]
fn test_idle_to_dragging_via_mouse_down_target() {
    let result = transition(
        InteractionState::Idle,
        CanvasEvent::MouseDownTarget {
            point: pt(5.0, 5.0),
            mode: SelectionMode::Replace,
        },
    );
    assert!(
        matches!(result.unwrap(), InteractionState::Dragging { drag } if drag.start.x == 5.0 && drag.start.y == 5.0)
    );
}

#[test]
fn test_idle_to_dragging_via_touch_down_target() {
    let result = transition(
        InteractionState::Idle,
        CanvasEvent::TouchDownTarget {
            point: pt(5.0, 5.0),
            mode: SelectionMode::Replace,
        },
    );
    assert!(matches!(result.unwrap(), InteractionState::Dragging { drag } if drag.start.x == 5.0));
}

#[test]
fn test_idle_to_selecting_via_mouse_down_background() {
    let result = transition(
        InteractionState::Idle,
        CanvasEvent::MouseDownBackground {
            point: pt(0.0, 0.0),
            mode: SelectionMode::Replace,
        },
    );
    assert!(
        matches!(result.unwrap(), InteractionState::Selecting { start, current, mode: SelectionMode::Replace } if start.x == 0.0 && current.x == 0.0)
    );
}

#[test]
fn test_idle_to_selecting_via_touch_down_background() {
    let result = transition(
        InteractionState::Idle,
        CanvasEvent::TouchDownBackground {
            point: pt(0.0, 0.0),
            mode: SelectionMode::Replace,
        },
    );
    assert!(
        matches!(result.unwrap(), InteractionState::Selecting { start, current, mode: SelectionMode::Replace } if start.x == 0.0 && current.x == 0.0)
    );
}

#[test]
fn test_idle_remains_idle_via_mouse_up() {
    let result = transition(InteractionState::Idle, CanvasEvent::MouseUp);
    assert!(matches!(result.unwrap(), InteractionState::Idle));
}

#[test]
fn test_idle_remains_idle_via_touch_up() {
    let result = transition(InteractionState::Idle, CanvasEvent::TouchUp);
    assert!(matches!(result.unwrap(), InteractionState::Idle));
}

// -----------------------------------------------------------------------------
// Combinatorial Permutations: Hovering State
// -----------------------------------------------------------------------------

#[test]
fn test_hovering_updates_hovering_via_mouse_move() {
    let state = InteractionState::Hovering {
        point: pt(0.0, 0.0),
    };
    let result = transition(
        state,
        CanvasEvent::MouseMove {
            point: pt(10.0, 20.0),
        },
    );
    assert!(
        matches!(result.unwrap(), InteractionState::Hovering { point } if point.x == 10.0 && point.y == 20.0)
    );
}

#[test]
fn test_hovering_to_dragging_via_mouse_down_target() {
    let state = InteractionState::Hovering {
        point: pt(5.0, 5.0),
    };
    let result = transition(
        state,
        CanvasEvent::MouseDownTarget {
            point: pt(5.0, 5.0),
            mode: SelectionMode::Replace,
        },
    );
    assert!(matches!(result.unwrap(), InteractionState::Dragging { drag } if drag.start.x == 5.0));
}

#[test]
fn test_hovering_to_dragging_via_touch_down_target() {
    let state = InteractionState::Hovering {
        point: pt(5.0, 5.0),
    };
    let result = transition(
        state,
        CanvasEvent::TouchDownTarget {
            point: pt(5.0, 5.0),
            mode: SelectionMode::Replace,
        },
    );
    assert!(matches!(result.unwrap(), InteractionState::Dragging { drag } if drag.start.x == 5.0));
}

#[test]
fn test_hovering_to_selecting_via_mouse_down_background() {
    let state = InteractionState::Hovering {
        point: pt(5.0, 5.0),
    };
    let result = transition(
        state,
        CanvasEvent::MouseDownBackground {
            point: pt(10.0, 10.0),
            mode: SelectionMode::Replace,
        },
    );
    assert!(
        matches!(result.unwrap(), InteractionState::Selecting { start, .. } if start.x == 10.0)
    );
}

#[test]
fn test_hovering_to_idle_via_mouse_up() {
    let state = InteractionState::Hovering {
        point: pt(5.0, 5.0),
    };
    let result = transition(state, CanvasEvent::MouseUp);
    assert!(matches!(result.unwrap(), InteractionState::Idle));
}

// -----------------------------------------------------------------------------
// Combinatorial Permutations: Dragging State
// -----------------------------------------------------------------------------

#[test]
fn test_dragging_updates_via_drag_move() {
    let state = InteractionState::Dragging {
        drag: drag_state(pt(5.0, 5.0)),
    };
    let result = transition(
        state,
        CanvasEvent::DragMove {
            delta: vec(10.0, 15.0),
        },
    );
    assert!(
        matches!(result.unwrap(), InteractionState::Dragging { drag } if drag.cumulative_offset.dx == 10.0 && drag.cumulative_offset.dy == 15.0)
    );
}

#[test]
fn test_dragging_updates_via_touch_move() {
    let state = InteractionState::Dragging {
        drag: drag_state(pt(5.0, 5.0)),
    };
    let result = transition(
        state,
        CanvasEvent::TouchMove {
            point: pt(15.0, 20.0),
            delta: vec(10.0, 15.0),
        },
    );
    assert!(
        matches!(result.unwrap(), InteractionState::Dragging { drag } if drag.cumulative_offset.dx == 10.0 && drag.cumulative_offset.dy == 15.0)
    );
}

#[test]
fn test_dragging_to_idle_via_mouse_up() {
    let state = InteractionState::Dragging {
        drag: drag_state(pt(5.0, 5.0)),
    };
    let result = transition(state, CanvasEvent::MouseUp);
    assert!(matches!(result.unwrap(), InteractionState::Idle));
}

#[test]
fn test_dragging_remains_via_mouse_move() {
    let state = InteractionState::Dragging {
        drag: drag_state(pt(5.0, 5.0)),
    };
    let result = transition(
        state,
        CanvasEvent::MouseMove {
            point: pt(10.0, 10.0),
        },
    );
    assert!(matches!(result.unwrap(), InteractionState::Dragging { drag } if drag.start.x == 5.0));
}

// -----------------------------------------------------------------------------
// Combinatorial Permutations: Selecting State
// -----------------------------------------------------------------------------

#[test]
fn test_selecting_updates_via_mouse_move() {
    let state = InteractionState::Selecting {
        start: pt(0.0, 0.0),
        current: pt(0.0, 0.0),
        mode: SelectionMode::Replace,
    };
    let result = transition(
        state,
        CanvasEvent::MouseMove {
            point: pt(50.0, 50.0),
        },
    );
    assert!(
        matches!(result.unwrap(), InteractionState::Selecting { current, .. } if current.x == 50.0)
    );
}

#[test]
fn test_selecting_to_idle_via_mouse_up() {
    let state = InteractionState::Selecting {
        start: pt(0.0, 0.0),
        current: pt(50.0, 50.0),
        mode: SelectionMode::Replace,
    };
    let result = transition(state, CanvasEvent::MouseUp);
    assert!(matches!(result.unwrap(), InteractionState::Idle));
}

// -----------------------------------------------------------------------------
// Negative Path / Invalid Transitions
// -----------------------------------------------------------------------------

#[test]
fn test_invalid_transition_idle_drag_move() {
    let result = transition(
        InteractionState::Idle,
        CanvasEvent::DragMove {
            delta: vec(1.0, 1.0),
        },
    );
    assert!(result.is_err());
}
