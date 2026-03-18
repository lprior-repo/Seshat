use crate::ui::theme::{BG_BASE, BORDER, BORDER_SUBTLE, TEXT_MAIN, TEXT_MUTED};
use dioxus::prelude::*;

#[component]
pub fn SidebarHeader(
    title: String,
    action_label: String,
    onaction: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; justify-content: space-between; gap: 8px;",
            h3 {
                style: "margin: 0; font-size: 12px; letter-spacing: 0.08em; text-transform: uppercase; color: {TEXT_MUTED};",
                "{title}"
            }
            button {
                style: "border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN}; border-radius: 6px; padding: 4px 8px; cursor: pointer; font-size: 11px;",
                onclick: move |evt| onaction.call(evt),
                "{action_label}"
            }
        }
    }
}

#[component]
pub fn SidebarGroup(
    provider: String,
    expanded: bool,
    query_active: bool,
    total_count: usize,
    ontoggle: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    rsx! {
        section {
            key: "{provider}",
            style: "display: flex; flex-direction: column; gap: 6px; border: 1px solid {BORDER_SUBTLE}; border-radius: 8px; background: {BG_BASE}; padding: 6px; box-shadow: inset 0 0 0 1px color-mix(in oklch, {BORDER_SUBTLE} 40%, transparent);",

            button {
                style: "width: 100%; background: transparent; border: none; padding: 2px 4px; color: {TEXT_MAIN}; display: flex; justify-content: space-between; align-items: center; cursor: pointer; text-transform: uppercase; letter-spacing: 0.04em; font-size: 11px;",
                onclick: move |evt| ontoggle.call(evt),

                span {
                    if query_active {
                        "{provider}"
                    } else if expanded {
                        "▼ {provider}"
                    } else {
                        "▶ {provider}"
                    }
                }
                span { "{total_count}" }
            }

            if expanded {
                {children}
            }
        }
    }
}
