use super::*;
use diagram_models::document::{ArrowType, DiagramDocument, EdgeStyle};
use dioxus::html::geometry::{ClientPoint, Coordinates, ElementPoint, PagePoint, ScreenPoint};
use dioxus::html::input_data::{keyboard_types::Modifiers, MouseButton, MouseButtonSet};
use std::any::Any;
use std::rc::Rc;

struct MockMouseData {
    coords: Coordinates,
    button: MouseButton,
}

impl dioxus::prelude::InteractionLocation for MockMouseData {
    fn client_coordinates(&self) -> ClientPoint {
        self.coords.client()
    }
    fn screen_coordinates(&self) -> ScreenPoint {
        self.coords.screen()
    }
    fn page_coordinates(&self) -> PagePoint {
        self.coords.page()
    }
}

impl dioxus::prelude::InteractionElementOffset for MockMouseData {
    fn coordinates(&self) -> Coordinates {
        Coordinates::new(
            self.coords.screen(),
            self.coords.client(),
            self.coords.element(),
            self.coords.page(),
        )
    }
    fn element_coordinates(&self) -> ElementPoint {
        self.coords.element()
    }
}

impl dioxus::prelude::ModifiersInteraction for MockMouseData {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}

impl dioxus::prelude::PointerInteraction for MockMouseData {
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
    let coords = Coordinates::new(
        ScreenPoint::new(x, y),
        ClientPoint::new(x, y),
        ElementPoint::new(x, y),
        PagePoint::new(x, y),
    );
    let md = MouseData::new(MockMouseData { coords, button });
    Event::new(Rc::new(md), true)
}

#[test]
fn test_handle_mousedown_selects_node() {
    let mut vdom = VirtualDom::new(|| {
        let evt = create_mouse_event(100.0, 100.0, MouseButton::Primary);
        let id = NodeId::new("node1".to_string());

        let interaction_mode = Signal::new(InteractionMode::Select);
        let doc_signal = Signal::new(DiagramDocument::default());
        let space_pan_active = Signal::new(false);

        handle_mousedown(
            evt,
            id,
            false,
            ToolMode::Select,
            (0.0, 0.0, 1.0),
            false,
            (0.0, 0.0),
            interaction_mode,
            doc_signal,
            space_pan_active,
            false,
        );

        assert!(matches!(
            *interaction_mode.read(),
            InteractionMode::DraggingSelection { .. }
        ));

        rsx! { div {} }
    });
    let () = vdom.rebuild_in_place();
}
