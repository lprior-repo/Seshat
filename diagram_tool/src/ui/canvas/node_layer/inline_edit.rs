use crate::history::History;
use canvas_domain::interaction_reducer::commit_inline_edit;
use diagram_models::document::DiagramDocument;
use dioxus::html::input_data::keyboard_types::Key;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct InlineEditProps {
    pub edit_value: Signal<String>,
    #[props(default)]
    pub class: String,
    pub style: String,
    pub doc_signal: Signal<DiagramDocument>,
    pub history_signal: Signal<History>,
    pub editor_state: Signal<crate::ui::canvas::state::EditorState>,
    pub db_tx: Option<Coroutine<diagram_models::envelope::EventEnvelope>>,
}

#[component]
pub fn InlineEdit(props: InlineEditProps) -> Element {
    let style = props.style;
    let mut edit_value = props.edit_value;
    let doc_signal = props.doc_signal;
    let history_signal = props.history_signal;
    let mut editor_state = props.editor_state;
    let db_tx = props.db_tx;

    rsx! {
        input {
            "data-testid": "inline-edit-input",
            class: "{props.class}",
            value: "{edit_value}",
            style: "{style}",
            autofocus: true,
            onmousedown: move |evt| evt.stop_propagation(),
            oninput: move |evt| edit_value.set(evt.value()),
            onblur: move |_| {
                let (node_target, edge_target) = match *editor_state.read() {
                    crate::ui::canvas::state::EditorState::EditingNode(ref id) => (Some(id.clone()), None),
                    crate::ui::canvas::state::EditorState::EditingEdge(ref id) => (None, Some(id.clone())),
                    _ => (None, None),
                };
                commit_inline_edit(
                    doc_signal,
                    history_signal,
                    node_target,
                    edge_target,
                    edit_value,
                    db_tx,
                )
                .ok();
                editor_state.set(crate::ui::canvas::state::EditorState::Idle);
            },
            onkeydown: move |evt| {
                if evt.key() == Key::Enter {
                    let (node_target, edge_target) = match *editor_state.read() {
                        crate::ui::canvas::state::EditorState::EditingNode(ref id) => (Some(id.clone()), None),
                        crate::ui::canvas::state::EditorState::EditingEdge(ref id) => (None, Some(id.clone())),
                        _ => (None, None),
                    };
                    commit_inline_edit(
                        doc_signal,
                        history_signal,
                        node_target,
                        edge_target,
                        edit_value,
                        db_tx,
                    )
                    .ok();
                    editor_state.set(crate::ui::canvas::state::EditorState::Idle);
                } else if evt.key() == Key::Escape {
                    editor_state.set(crate::ui::canvas::state::EditorState::Idle);
                }
            }
        }
    }
}
