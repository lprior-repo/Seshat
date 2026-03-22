use dioxus::prelude::*;
use dioxus::html::input_data::{MouseButton, MouseButtonSet, keyboard_types::Modifiers};
use dioxus::html::geometry::{ClientPoint, Coordinates, ElementPoint, PagePoint, ScreenPoint};

struct MockMouseData {
    coords: Coordinates,
    button: MouseButton,
}

impl dioxus::html::HasMouseData for MockMouseData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl dioxus::html::InteractionLocation for MockMouseData {
    fn client_coordinates(&self) -> ClientPoint { self.coords.client() }
    fn screen_coordinates(&self) -> ScreenPoint { self.coords.screen() }
    fn page_coordinates(&self) -> PagePoint { self.coords.page() }
}

impl dioxus::html::InteractionElementOffset for MockMouseData {
    fn element_coordinates(&self) -> ElementPoint { self.coords.element() }
    fn coordinates(&self) -> Coordinates { self.coords }
}

impl dioxus::html::ModifiersInteraction for MockMouseData {
    fn modifiers(&self) -> Modifiers { Modifiers::empty() }
}

impl dioxus::html::PointerInteraction for MockMouseData {
    fn trigger_button(&self) -> Option<MouseButton> { Some(self.button) }
    fn held_buttons(&self) -> MouseButtonSet { MouseButtonSet::empty() }
}
