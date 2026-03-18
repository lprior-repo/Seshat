#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::history::History;
use crate::ui::theme::{BG_BASE, BORDER, BORDER_SUBTLE, TEXT_DIM, TEXT_MAIN, TEXT_MUTED};
use diagram_models::document::{DiagramDocument, LockState, Node, NodeId};
use dioxus::prelude::*;

use super::update::update_node_if_changed;

#[component]
#[allow(clippy::approx_constant, clippy::float_cmp)]
pub fn MetaPropsPanel(
    id: NodeId,
    node: Node,
    connection_rows: Vec<(String, String, String)>,
) -> Element {
    let mut doc_signal = use_context::<Signal<DiagramDocument>>();
    let mut history = use_context::<Signal<History>>();

    let id_lock = id.clone();

    rsx! {
        div {
            label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Lock" }
            button {
                style: "padding: 6px 10px; cursor: pointer; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
                onclick: move |_| {
                    update_node_if_changed(
                        &mut doc_signal,
                        &mut history,
                        &id_lock,
                        |_| true, // Always apply toggle
                        |n| {
                            n.lock_state = if n.lock_state.is_locked() {
                                LockState::Unlocked
                            } else {
                                LockState::Locked
                            };
                        },
                    );
                },
                if node.lock_state.is_locked() { "Locked" } else { "Unlocked" }
            }
        }

        if !node.tags.is_empty() {
            div {
                label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Tags" }
                div {
                    style: "margin-top: 4px; display: flex; flex-wrap: wrap; gap: 4px;",
                    for tag in node.tags.clone() {
                        span {
                            key: "{id}-{tag}",
                            style: "display: inline-flex; align-items: center; padding: 2px 6px; border-radius: 999px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN}; font-size: 10px;",
                            "{tag}"
                        }
                    }
                }
            }
        }

        div { style: "height: 1px; background: {BORDER_SUBTLE};" }

        div {
            label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "Connections" }
            div {
                style: "margin-top: 4px; display: flex; flex-direction: column; gap: 4px;",
                if connection_rows.is_empty() {
                    div {
                        style: "font-size: 11px; color: {TEXT_DIM}; font-style: italic;",
                        "No connections"
                    }
                }
                for (index, (direction, other_label, edge_label)) in connection_rows.iter().enumerate() {
                    div {
                        key: "{id}-connection-{index}",
                        style: "padding: 4px 6px; border-radius: 6px; border: 1px solid {BORDER_SUBTLE}; background: {BG_BASE}; font-size: 11px; color: {TEXT_MAIN};",
                        if edge_label.is_empty() {
                            "{direction} -> {other_label}"
                        } else {
                            "{direction} -> {other_label} ({edge_label})"
                        }
                    }
                }
            }
        }

        div {
            label { style: "display: block; font-size: 12px; color: {TEXT_MUTED};", "ID" }
            p {
                style: "margin: 3px 0 0 0; font-size: 10px; color: {TEXT_DIM}; word-break: break-all;",
                "{id}"
            }
        }
    }
}
