use crate::history::History;
use canvas_domain::interaction_reducer::commit_inline_edit;
use diagram_models::document::DiagramDocument;
use diagram_models::document::{EdgeId, NodeId};
use dioxus::html::input_data::keyboard_types::Key;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct InlineEditProps {
    pub edit_value: Signal<String>,
    pub style: String,
    pub doc_signal: Signal<DiagramDocument>,
    pub history_signal: Signal<History>,
    pub editing_node: Signal<Option<NodeId>>,
    pub editing_edge: Signal<Option<EdgeId>>,
    pub db_tx: Option<Coroutine<diagram_models::envelope::EventEnvelope>>,
}

#[component]
pub fn InlineEdit(props: InlineEditProps) -> Element {
    let style = props.style;
    let mut edit_value = props.edit_value;
    let doc_signal = props.doc_signal;
    let history_signal = props.history_signal;
    let mut editing_node = props.editing_node;
    let editing_edge = props.editing_edge;
    let db_tx = props.db_tx;

    rsx! {
        input {
            value: "{edit_value}",
            style: "{style}",
            onmousedown: move |evt| evt.stop_propagation(),
            oninput: move |evt| edit_value.set(evt.value()),
            onblur: move |_| {
                commit_inline_edit(
                    doc_signal,
                    history_signal,
                    editing_node,
                    editing_edge,
                    edit_value,
                    db_tx,
                )
                .ok();
            },
            onkeydown: move |evt| {
                if evt.key() == Key::Enter {
                    commit_inline_edit(
                        doc_signal,
                        history_signal,
                        editing_node,
                        editing_edge,
                        edit_value,
                        db_tx,
                    )
                    .ok();
                } else if evt.key() == Key::Escape {
                    editing_node.set(None);
                }
            }
        }
    }
}
