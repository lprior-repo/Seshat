use dioxus::html::geometry::{ClientPoint, Coordinates, ElementPoint, PagePoint, ScreenPoint};
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use std::rc::Rc;

#[test]
fn test_mouse_data() {
    let coords = Coordinates::new(
        ScreenPoint::new(0.0, 0.0),
        ClientPoint::new(0.0, 0.0),
        ElementPoint::new(0.0, 0.0),
        PagePoint::new(0.0, 0.0),
    );
    let md = dioxus::prelude::MouseData::new(
        coords,
        Some(MouseButton::Primary),
        dioxus::html::input_data::keyboard_types::Modifiers::empty(),
    );
    let evt = Event::new(Rc::new(md), true);
}
