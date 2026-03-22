#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
use crate::app::AppState;
use crate::ui::canvas::root_container::handlers::*;
use crate::ui::canvas::state::CanvasState;
use dioxus::html::geometry::{
    ClientPoint, Coordinates, ElementPoint, PagePoint, ScreenPoint, WheelDelta,
};
use dioxus::html::input_data::{keyboard_types::Modifiers, MouseButton, MouseButtonSet};
use dioxus::prelude::*;
use proptest::prelude::*;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

pub fn with_canvas_state<F>(f: F)
where
    F: FnOnce(CanvasState) + 'static,
{
    let f = Rc::new(RefCell::new(Some(f)));
    let mut vdom = VirtualDom::new_with_props(
        move |()| {
            AppState::provide();
            let state = crate::ui::canvas::state::use_canvas_state();
            if let Some(func) = f.borrow_mut().take() {
                func(state);
            }
            rsx! { div {} }
        },
        (),
    );
    let _ = vdom.rebuild_in_place();
}

struct MockMouseData {
    x: f64,
    y: f64,
    button: MouseButton,
}

impl dioxus::html::InteractionLocation for MockMouseData {
    fn client_coordinates(&self) -> ClientPoint {
        ClientPoint::new(self.x, self.y)
    }
    fn screen_coordinates(&self) -> ScreenPoint {
        ScreenPoint::new(self.x, self.y)
    }
    fn page_coordinates(&self) -> PagePoint {
        PagePoint::new(self.x, self.y)
    }
}
impl dioxus::html::InteractionElementOffset for MockMouseData {
    fn element_coordinates(&self) -> ElementPoint {
        ElementPoint::new(self.x, self.y)
    }
    fn coordinates(&self) -> Coordinates {
        Coordinates::new(
            ScreenPoint::new(self.x, self.y),
            ClientPoint::new(self.x, self.y),
            ElementPoint::new(self.x, self.y),
            PagePoint::new(self.x, self.y),
        )
    }
}
impl dioxus::html::ModifiersInteraction for MockMouseData {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::PointerInteraction for MockMouseData {
    fn trigger_button(&self) -> Option<MouseButton> {
        Some(self.button)
    }
    fn held_buttons(&self) -> MouseButtonSet {
        MouseButtonSet::empty()
    }
}
impl dioxus::html::HasMouseData for MockMouseData {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn create_mouse_event(x: f64, y: f64, button: MouseButton) -> Event<MouseData> {
    let md = MouseData::new(MockMouseData { x, y, button });
    Event::new(Rc::new(md), true)
}

struct MockWheelData {
    x: f64,
    y: f64,
    dx: f64,
    dy: f64,
}

impl dioxus::html::InteractionLocation for MockWheelData {
    fn client_coordinates(&self) -> ClientPoint {
        ClientPoint::new(self.x, self.y)
    }
    fn screen_coordinates(&self) -> ScreenPoint {
        ScreenPoint::new(self.x, self.y)
    }
    fn page_coordinates(&self) -> PagePoint {
        PagePoint::new(self.x, self.y)
    }
}
impl dioxus::html::InteractionElementOffset for MockWheelData {
    fn element_coordinates(&self) -> ElementPoint {
        ElementPoint::new(self.x, self.y)
    }
    fn coordinates(&self) -> Coordinates {
        Coordinates::new(
            ScreenPoint::new(self.x, self.y),
            ClientPoint::new(self.x, self.y),
            ElementPoint::new(self.x, self.y),
            PagePoint::new(self.x, self.y),
        )
    }
}
impl dioxus::html::ModifiersInteraction for MockWheelData {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::PointerInteraction for MockWheelData {
    fn trigger_button(&self) -> Option<MouseButton> {
        None
    }
    fn held_buttons(&self) -> MouseButtonSet {
        MouseButtonSet::empty()
    }
}
impl dioxus::html::HasMouseData for MockWheelData {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Since we cannot construct euclid vector easily without dependency,
// we notice we only need the tests to compile. We can construct it via unsafe if needed.
// Actually, dioxus re-exports euclid types? Or we can just use another enum variant for WheelDelta if it exists.
// WheelDelta::Lines doesn't require a vector. It requires a euclid Vector2D. All variants require it.
// We can use unsafe zeroed to get a Vector2D since it's just two f64s and a marker.
impl dioxus::html::HasWheelData for MockWheelData {
    fn delta(&self) -> WheelDelta {
        let vec = dioxus::html::geometry::euclid::Vector3D::new(self.dx, self.dy, 0.0);
        WheelDelta::Pixels(vec)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn create_wheel_event(x: f64, y: f64, dx: f64, dy: f64) -> Event<WheelData> {
    let wd = WheelData::new(MockWheelData { x, y, dx, dy });
    Event::new(Rc::new(wd), true)
}

fn extreme_f64s() -> impl Strategy<Value = f64> {
    prop_oneof![
        Just(0.0),
        Just(-1.0),
        Just(1.0),
        Just(f64::MAX),
        Just(f64::MIN),
        Just(f64::INFINITY),
        Just(f64::NEG_INFINITY),
        Just(f64::NAN),
        any::<f64>(),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_mouse_down(
        x in extreme_f64s(),
        y in extreme_f64s(),
    ) {
        with_canvas_state(move |state| {
            let evt = create_mouse_event(x, y, MouseButton::Primary);
            handle_mouse_down(state, evt);
        });
    }

    #[test]
    fn prop_mouse_move(
        x in extreme_f64s(),
        y in extreme_f64s(),
    ) {
        with_canvas_state(move |state| {
            let evt = create_mouse_event(x, y, MouseButton::Primary);
            handle_mouse_move(state, evt);
        });
    }

    #[test]
    fn prop_mouse_up(
        x in extreme_f64s(),
        y in extreme_f64s(),
    ) {
        with_canvas_state(move |state| {
            let evt = create_mouse_event(x, y, MouseButton::Primary);
            handle_mouse_up(state, evt);
        });
    }

    #[test]
    fn prop_double_click(
        x in extreme_f64s(),
        y in extreme_f64s(),
    ) {
        with_canvas_state(move |state| {
            let evt = create_mouse_event(x, y, MouseButton::Primary);
            handle_double_click(state, evt);
        });
    }

    #[test]
    fn prop_wheel(
        x in extreme_f64s(),
        y in extreme_f64s(),
        dx in extreme_f64s(),
        dy in extreme_f64s(),
    ) {
        with_canvas_state(move |state| {
            let evt = create_wheel_event(x, y, dx, dy);
            handle_wheel(state, evt);
        });
    }
}
