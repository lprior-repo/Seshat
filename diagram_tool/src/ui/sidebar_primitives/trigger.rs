use crate::ui::mobile::SidebarUiState;
use crate::ui::sidebar_primitives::provider::{resolve_sidebar_signal, toggle_sidebar};
use crate::ui::sidebar_primitives::utils::merge_class;
use crate::ui::theme::{BG_SURFACE, BORDER_SUBTLE, TEXT_MUTED};
use dioxus::prelude::*;

#[component]
pub fn SidebarTrigger(
    label: String,
    #[props(default = String::from("Toggle sidebar"))] title: String,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    #[props(default)] sidebar_ui: Option<Signal<SidebarUiState>>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
) -> Element {
    let mut signal = resolve_sidebar_signal(sidebar_ui);
    let trigger_class = merge_class("sidebar-trigger", class.as_deref());
    let trigger_style = style.unwrap_or_default();

    rsx! {
        button {
            class: "{trigger_class}",
            style: trigger_style,
            title: "{title}",
            "aria-label": "{title}",
            onclick: move |evt| {
                if let Some(handler) = &onclick {
                    handler.call(evt);
                }
                if let Some(ref mut sidebar_ui) = signal {
                    toggle_sidebar(sidebar_ui);
                }
            },
            "{label}"
        }
    }
}

#[component]
pub fn SidebarRail(
    label: String,
    title: String,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    #[props(default)] sidebar_ui: Option<Signal<SidebarUiState>>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
) -> Element {
    let mut signal = resolve_sidebar_signal(sidebar_ui);
    let rail_class = merge_class("sidebar-rail-wrap", class.as_deref());
    let rail_style = style.unwrap_or_else(|| {
        format!(
            "width: 18px; border-right: 1px solid {BORDER_SUBTLE}; background: {BG_SURFACE}; display: flex; align-items: center; justify-content: center;"
        )
    });

    rsx! {
        div {
            class: "{rail_class}",
            style: rail_style,
            button {
                style: "border: none; background: transparent; color: {TEXT_MUTED}; cursor: pointer; font-size: 12px; padding: 4px;",
                title: "{title}",
                "aria-label": "{title}",
                onclick: move |evt| {
                    if let Some(handler) = &onclick {
                        handler.call(evt);
                    } else if let Some(ref mut sidebar_ui) = signal {
                        toggle_sidebar(sidebar_ui);
                    }
                },
                "{label}"
            }
        }
    }
}
