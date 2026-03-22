use canvas_domain::interaction_reducer::{finalize_motion_release, InteractionMode};
use canvas_domain::perf::to_canvas_coords;
use canvas_domain::{CanvasCoord, ScreenCoord};
use diagram_models::document::{ArrowType, DiagramDocument, EdgeStyle, NodeId};
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

use crate::history::History;
use crate::ui::canvas::document_ops::{flush_pending_pointer_update, sync_canvas_origin};
use crate::ui::editor::ToolMode;

use super::state::{apply_event, CanvasError, CanvasEvent, CanvasState};

#[allow(clippy::too_many_arguments)]
pub fn handle_mousedown(
    evt: Event<dioxus::prelude::MouseData>,
    id: NodeId,
    multi_touch_active: bool,
    tool: ToolMode,
    doc: DiagramDocument,
    additive: bool,
    canvas_origin: (f64, f64),
    mut interaction_mode: Signal<InteractionMode>,
    mut doc_signal: Signal<DiagramDocument>,
    _space_pan_active: Signal<bool>,
    space_pressed: bool,
) {
    if multi_touch_active {
        return;
    }

    let is_middle = evt.data.trigger_button() == Some(MouseButton::Auxiliary);
    let is_right = evt.data.trigger_button() == Some(MouseButton::Secondary);

    if space_pressed || is_middle || is_right || tool == ToolMode::Pan {
        // Let panning events bubble up to the root container
        return;
    }

    evt.stop_propagation();
    let is_primary = evt.data.trigger_button() == Some(MouseButton::Primary);
    let coords = evt.data.coordinates().client();
    let origin = sync_canvas_origin().unwrap_or(canvas_origin);
    let local_x = coords.x - origin.0;
    let local_y = coords.y - origin.1;
    let pos = to_canvas_coords(
        ScreenCoord(local_x, local_y),
        CanvasCoord(doc.editor_state.camera_x.0, doc.editor_state.camera_y.0),
        doc.editor_state.zoom.0,
    );

    let event = if is_primary {
        if tool == ToolMode::Edge {
            Some(CanvasEvent::EdgeDrawingStarted {
                from_node: id,
                current_pos: CanvasCoord(pos.0, pos.1),
            })
        } else {
            Some(CanvasEvent::NodeSelected {
                id,
                additive,
                canvas_pos: CanvasCoord(pos.0, pos.1),
                client_pos: ScreenCoord(local_x, local_y),
            })
        }
    } else {
        None
    };

    if let Some(event) = event {
        let initial_state = CanvasState {
            document: doc_signal.read().clone(),
            interaction_mode: interaction_mode.read().clone(),
        };
        if let Ok(new_state) = apply_event(initial_state, event) {
            doc_signal.set(new_state.document);
            interaction_mode.set(new_state.interaction_mode);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn handle_mouseup(
    evt: Event<dioxus::prelude::MouseData>,
    id: NodeId,
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    mut interaction_mode: Signal<InteractionMode>,
    pending_pointer_sample: Signal<Option<(f64, f64)>>,
    db_tx: Option<Coroutine<diagram_models::envelope::EventEnvelope>>,
    mut tool_signal: Signal<ToolMode>,
    edge_style_default: EdgeStyle,
    arrow_type_default: ArrowType,
    canvas_origin: (f64, f64),
    toast: crate::ui::toast::ToastApi,
) {
    let mode = interaction_mode.read().clone();

    // Let panning release events bubble up to the root container
    if matches!(mode, InteractionMode::Panning { .. }) {
        return;
    }

    evt.stop_propagation();
    flush_pending_pointer_update(
        doc_signal,
        history_signal,
        interaction_mode,
        pending_pointer_sample,
        db_tx,
    );
    let mode = interaction_mode.read().clone();

    let event = match mode {
        InteractionMode::DrawingEdge { from_node, .. } => {
            let doc_now = doc_signal.read().clone();
            let coords = evt.data.coordinates().client();
            let origin = sync_canvas_origin().unwrap_or(canvas_origin);
            let local_x = coords.x - origin.0;
            let local_y = coords.y - origin.1;
            let pos = to_canvas_coords(
                ScreenCoord(local_x, local_y),
                CanvasCoord(
                    doc_now.editor_state.camera_x.0,
                    doc_now.editor_state.camera_y.0,
                ),
                doc_now.editor_state.zoom.0,
            );
            Some(CanvasEvent::EdgeDrawingFinished {
                from_node,
                to_node: id,
                current_pos: CanvasCoord(pos.0, pos.1),
                continue_drawing: *tool_signal.read() == ToolMode::Edge,
                edge_style: edge_style_default,
                arrow_type: arrow_type_default,
            })
        }
        InteractionMode::DraggingSelection { .. } | InteractionMode::ResizingSelection { .. } => {
            let mut doc_clone = doc_signal.read().clone();
            interaction_mode.with_mut(|mode_mut| {
                let did_change = finalize_motion_release(mode_mut, &mut doc_clone, &db_tx);
                if did_change {
                    doc_signal.set(doc_clone);
                }
            });
            None
        }
        _ => None,
    };

    if let Some(event) = event {
        let initial_state = CanvasState {
            document: doc_signal.read().clone(),
            interaction_mode: interaction_mode.read().clone(),
        };
        match apply_event(initial_state, event) {
            Ok(new_state) => {
                let history = history_signal.read().clone();
                *history_signal.write() = history.push(doc_signal.read().clone());
                doc_signal.set(new_state.document);
                interaction_mode.set(new_state.interaction_mode);
            }
            Err(CanvasError::CircularConnectionRejected) => {
                let _ = toast.show(
                    crate::ui::toast::ToastIntent::Warning,
                    "Cannot create circular connection",
                    None,
                );
            }
            Err(_) => {}
        }
    }

    if *tool_signal.read() != ToolMode::Edge {
        tool_signal.set(ToolMode::Select);
    }
}

#[cfg(test)]
mod tests {
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
            let doc = DiagramDocument::default();

            let interaction_mode = Signal::new(InteractionMode::Select);
            let doc_signal = Signal::new(DiagramDocument::default());
            let space_pan_active = Signal::new(false);

            handle_mousedown(
                evt,
                id.clone(),
                false,
                ToolMode::Select,
                doc,
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
        let _ = vdom.rebuild_in_place();
    }
}
