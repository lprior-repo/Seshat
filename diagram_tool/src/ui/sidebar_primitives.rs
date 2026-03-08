#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::ui::mobile::SidebarUiState;
use crate::ui::theme::{
    BG_BASE, BG_ELEVATED, BG_SURFACE, BORDER, BORDER_SUBTLE, TEXT_MAIN, TEXT_MUTED,
};
use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum SidebarSide {
    #[default]
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum SidebarVariant {
    #[default]
    Sidebar,
    Floating,
    Inset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum SidebarCollapsible {
    #[default]
    Offcanvas,
    Icon,
    None,
}

#[derive(Clone, Copy)]
pub struct SidebarProviderContext {
    pub sidebar_ui: Signal<SidebarUiState>,
    pub side: SidebarSide,
    pub variant: SidebarVariant,
    pub collapsible: SidebarCollapsible,
}

fn merge_class(base: &str, class: Option<&str>) -> String {
    class.map_or_else(|| String::from(base), |extra| format!("{base} {extra}"))
}

#[allow(clippy::missing_const_for_fn)]
fn local_storage_set_sidebar_open(_open: bool) {
    #[cfg(target_arch = "wasm32")]
    {
        let value = if _open { "true" } else { "false" };
        let _eval = document::eval(&format!(
            "try {{ localStorage.setItem('diagram_tool.sidebar_open', '{value}'); }} catch (_) {{}}"
        ));
    }
}

#[must_use]
pub fn use_sidebar_provider() -> Option<SidebarProviderContext> {
    try_use_context::<SidebarProviderContext>()
}

fn resolve_sidebar_signal(
    sidebar_ui: Option<Signal<SidebarUiState>>,
) -> Option<Signal<SidebarUiState>> {
    sidebar_ui.or_else(|| use_sidebar_provider().map(|ctx| ctx.sidebar_ui))
}

fn toggle_sidebar(sidebar_ui: &mut Signal<SidebarUiState>) {
    sidebar_ui.with_mut(|state| {
        if state.is_mobile {
            state.open_mobile = !state.open_mobile;
            return;
        }

        state.open = !state.open;
        local_storage_set_sidebar_open(state.open);
    });
}

#[component]
pub fn SidebarProvider(
    sidebar_ui: Signal<SidebarUiState>,
    #[props(default = SidebarSide::Left)] side: SidebarSide,
    #[props(default = SidebarVariant::Sidebar)] variant: SidebarVariant,
    #[props(default = SidebarCollapsible::Offcanvas)] collapsible: SidebarCollapsible,
    #[props(default = String::from("280px"))] sidebar_width: String,
    #[props(default = String::from("19rem"))] sidebar_width_mobile: String,
    #[props(default = String::from("3rem"))] sidebar_width_icon: String,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    use_context_provider(|| SidebarProviderContext {
        sidebar_ui,
        side,
        variant,
        collapsible,
    });

    let mut provider_style = format!(
        "--sidebar-width: {sidebar_width}; --sidebar-width-mobile: {sidebar_width_mobile}; --sidebar-width-icon: {sidebar_width_icon};"
    );
    if let Some(extra_style) = style {
        provider_style.push_str(&extra_style);
    }

    rsx! {
        div {
            class: "{merge_class(\"sidebar-provider\", class.as_deref())}",
            style: "{provider_style}",
            {children}
        }
    }
}

#[component]
pub fn Sidebar(
    #[props(default)] sidebar_ui: Option<Signal<SidebarUiState>>,
    #[props(default)] side: Option<SidebarSide>,
    #[props(default)] variant: Option<SidebarVariant>,
    #[props(default)] collapsible: Option<SidebarCollapsible>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    #[props(default)] mobile_style: Option<String>,
    #[props(default)] collapsed_style: Option<String>,
    children: Element,
) -> Element {
    let context = use_sidebar_provider();
    let signal = resolve_sidebar_signal(sidebar_ui);
    let final_style = style.unwrap_or_default();
    let side = side
        .or_else(|| context.map(|ctx| ctx.side))
        .unwrap_or_default();
    let variant = variant
        .or_else(|| context.map(|ctx| ctx.variant))
        .unwrap_or_default();
    let collapsible = collapsible
        .or_else(|| context.map(|ctx| ctx.collapsible))
        .unwrap_or_default();

    let side_class = match side {
        SidebarSide::Left => "sidebar-left",
        SidebarSide::Right => "sidebar-right",
    };
    let variant_class = match variant {
        SidebarVariant::Sidebar => "sidebar-variant-sidebar",
        SidebarVariant::Floating => "sidebar-variant-floating",
        SidebarVariant::Inset => "sidebar-variant-inset",
    };
    let collapsible_class = match collapsible {
        SidebarCollapsible::Offcanvas => "sidebar-collapsible-offcanvas",
        SidebarCollapsible::Icon => "sidebar-collapsible-icon",
        SidebarCollapsible::None => "sidebar-collapsible-none",
    };

    let root_class = merge_class(
        &format!("sidebar {side_class} {variant_class} {collapsible_class}"),
        class.as_deref(),
    );

    if let Some(signal) = signal {
        let state = *signal.read();

        if state.is_mobile {
            if !state.open_mobile {
                return rsx! {};
            }

            let panel_style = mobile_style.unwrap_or_else(|| {
                if final_style.is_empty() {
                    String::from("width: min(19rem, 90vw);")
                } else {
                    final_style.clone()
                }
            });
            return rsx! {
                div {
                    class: "{root_class}",
                    style: "{panel_style}",
                    {children}
                }
            };
        }

        if !state.open {
            match collapsible {
                SidebarCollapsible::None => {}
                SidebarCollapsible::Offcanvas => return rsx! {},
                SidebarCollapsible::Icon => {
                    let style = collapsed_style
                        .unwrap_or_else(|| String::from("width: var(--sidebar-width-icon, 3rem);"));
                    return rsx! {
                        div {
                            class: "{root_class}",
                            style: "{style}",
                            {children}
                        }
                    };
                }
            }
        }
    }

    rsx! {
        div {
            class: "{root_class}",
            style: final_style,
            {children}
        }
    }
}

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

#[component]
pub fn SidebarOverlay(onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        div {
            style: "position: fixed; inset: 56px 0 0 0; background: color-mix(in oklch, black 32%, transparent); backdrop-filter: blur(1px); z-index: 65;",
            onclick: move |evt| onclick.call(evt),
        }
    }
}

#[component]
pub fn SidebarSheet(style: String, children: Element) -> Element {
    rsx! {
        div {
            class: "sidebar",
            style: "{style}",
            {children}
        }
    }
}

#[component]
pub fn SidebarInset(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let inset_class = merge_class("sidebar-inset", class.as_deref());
    let inset_style = style.unwrap_or_else(|| {
        String::from("display: flex; flex: 1; min-width: 0; flex-direction: column;")
    });

    rsx! {
        main {
            class: "{inset_class}",
            style: inset_style,
            {children}
        }
    }
}

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
