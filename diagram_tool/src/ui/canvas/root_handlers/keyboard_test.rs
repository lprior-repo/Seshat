#[cfg(test)]
mod tests {
    use crate::history::History;
    use crate::ui::canvas::root_handlers::keyboard::process_keyboard_event;
    use crate::ui::canvas::state::EditorState;
    use crate::ui::editor::ToolMode;
    use canvas_domain::interaction_reducer::InteractionMode;
    use diagram_models::document::DiagramDocument;
    use dioxus::prelude::*;

    #[test]
    fn test_process_keyboard_event_escape_cancels_editing() {
        let mut vdom = VirtualDom::new(|| {
            let mut space_pressed = Signal::new(false);
            let mut shift_pressed = Signal::new(false);
            let mut ctrl_pressed = Signal::new(false);
            let mut meta_pressed = Signal::new(false);
            let mut nudge_batch_active = Signal::new(false);
            let mut space_pan_active = Signal::new(false);

            let mut interaction_mode = Signal::new(InteractionMode::Select);
            let mut tool_signal = Signal::new(ToolMode::Select);

            let mut doc_signal = Signal::new(DiagramDocument::default());
            let mut history_signal = Signal::new(History::new());

            let mut editor_state = Signal::new(EditorState::EditingNode(
                diagram_models::document::NodeId::new("test_node".to_string()),
            ));
            let mut edit_value = Signal::new("editing text".to_string());
            let mut viewport_size = Signal::new((1000.0, 1000.0));

            let db_tx: Option<Coroutine<diagram_models::envelope::EventEnvelope>> = None;

            process_keyboard_event(
                "keydown",
                "Escape",
                false,
                false,
                false,
                &mut space_pressed,
                &mut shift_pressed,
                &mut ctrl_pressed,
                &mut meta_pressed,
                &mut nudge_batch_active,
                &mut space_pan_active,
                &mut interaction_mode,
                &mut tool_signal,
                &mut doc_signal,
                &mut history_signal,
                &mut editor_state,
                &mut edit_value,
                &mut viewport_size,
                &db_tx,
            );

            assert!(matches!(*editor_state.read(), EditorState::Idle));
            assert_eq!(*edit_value.read(), "");

            rsx! { div {} }
        });
        let () = vdom.rebuild_in_place();
    }

    #[test]
    fn test_process_keyboard_event_tool_shortcuts() {
        let mut vdom = VirtualDom::new(|| {
            let mut space_pressed = Signal::new(false);
            let mut shift_pressed = Signal::new(false);
            let mut ctrl_pressed = Signal::new(false);
            let mut meta_pressed = Signal::new(false);
            let mut nudge_batch_active = Signal::new(false);
            let mut space_pan_active = Signal::new(false);

            let mut interaction_mode = Signal::new(InteractionMode::Select);
            let mut tool_signal = Signal::new(ToolMode::Select);

            let mut doc_signal = Signal::new(DiagramDocument::default());
            let mut history_signal = Signal::new(History::new());
            let mut editor_state = Signal::new(EditorState::Idle);
            let mut edit_value = Signal::new(String::new());
            let mut viewport_size = Signal::new((1000.0, 1000.0));

            let db_tx: Option<Coroutine<diagram_models::envelope::EventEnvelope>> = None;

            // Test setting Pan tool
            process_keyboard_event(
                "keydown",
                "h",
                false,
                false,
                false,
                &mut space_pressed,
                &mut shift_pressed,
                &mut ctrl_pressed,
                &mut meta_pressed,
                &mut nudge_batch_active,
                &mut space_pan_active,
                &mut interaction_mode,
                &mut tool_signal,
                &mut doc_signal,
                &mut history_signal,
                &mut editor_state,
                &mut edit_value,
                &mut viewport_size,
                &db_tx,
            );
            assert_eq!(*tool_signal.read(), ToolMode::Pan);

            rsx! { div {} }
        });
        let () = vdom.rebuild_in_place();
    }
}
