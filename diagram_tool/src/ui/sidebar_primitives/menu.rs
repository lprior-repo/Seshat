use crate::ui::sidebar_primitives::utils::merge_class;
use crate::ui::theme::{BG_ELEVATED, BORDER, TEXT_MAIN};
use dioxus::prelude::*;

#[component]
pub fn SidebarMenu(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let menu_class = merge_class("sidebar-menu", class.as_deref());
    let menu_style = style.unwrap_or_else(|| {
        String::from(
            "display: flex; min-width: 0; flex-direction: column; gap: 8px; margin: 0; padding: 0; list-style: none;",
        )
    });

    rsx! {
        ul {
            class: "{menu_class}",
            style: menu_style,
            {children}
        }
    }
}

#[component]
pub fn SidebarMenuItem(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let item_class = merge_class("sidebar-menu-item", class.as_deref());
    let item_style = style.unwrap_or_default();

    rsx! {
        li {
            class: "{item_class}",
            style: item_style,
            {children}
        }
    }
}

#[component]
pub fn SidebarMenuButton(
    label: String,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
) -> Element {
    let button_class = merge_class("sidebar-menu-button", class.as_deref());
    let button_style = style.unwrap_or_else(|| {
        format!(
            "width: 100%; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_ELEVATED}; color: {TEXT_MAIN}; font-size: 11px; padding: 6px; cursor: pointer;"
        )
    });

    rsx! {
        button {
            class: "{button_class}",
            style: button_style,
            onclick: move |evt| {
                if let Some(handler) = &onclick {
                    handler.call(evt);
                }
            },
            "{label}"
        }
    }
}

#[component]
pub fn SidebarMenuSub(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let submenu_class = merge_class("sidebar-menu-sub", class.as_deref());
    let submenu_style = style.unwrap_or_else(|| {
        String::from(
            "display: flex; min-width: 0; flex-direction: column; gap: 6px; margin: 0; padding: 0; list-style: none;",
        )
    });

    rsx! {
        ul {
            class: "{submenu_class}",
            style: submenu_style,
            {children}
        }
    }
}

#[component]
pub fn SidebarMenuSubItem(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let item_class = merge_class("sidebar-menu-sub-item", class.as_deref());
    let item_style = style.unwrap_or_default();

    rsx! {
        li {
            class: "{item_class}",
            style: item_style,
            {children}
        }
    }
}
