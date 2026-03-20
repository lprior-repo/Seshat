use crate::ui::mobile::SidebarUiState;
use crate::ui::sidebar_primitives::provider::{resolve_sidebar_signal, use_sidebar_provider};
use crate::ui::sidebar_primitives::types::{SidebarCollapsible, SidebarSide, SidebarVariant};
use crate::ui::sidebar_primitives::utils::merge_class;
use dioxus::prelude::*;

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
pub fn SidebarOverlay(onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        div {
            style: "position: fixed; inset: 56px 0 0 0; background: color-mix(in oklch, black 32%, transparent); backdrop-filter: blur(1px); z-index: 65;",
            onclick: move |evt| onclick.call(evt),
        }
    }
}

#[component]
pub fn SidebarSheet(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let sheet_class = merge_class("sidebar", class.as_deref());
    let sheet_style = style.unwrap_or_default();

    rsx! {
        div {
            class: "{sheet_class}",
            style: "{sheet_style}",
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
