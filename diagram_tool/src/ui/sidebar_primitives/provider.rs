use crate::ui::mobile::SidebarUiState;
use crate::ui::sidebar_primitives::types::{
    SidebarCollapsible, SidebarProviderContext, SidebarSide, SidebarVariant,
};
use crate::ui::sidebar_primitives::utils::merge_class;
use dioxus::prelude::*;

#[allow(clippy::missing_const_for_fn)]
pub fn local_storage_set_sidebar_open(_open: bool) {
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

pub fn resolve_sidebar_signal(
    sidebar_ui: Option<Signal<SidebarUiState>>,
) -> Option<Signal<SidebarUiState>> {
    sidebar_ui.or_else(|| use_sidebar_provider().map(|ctx| ctx.sidebar_ui))
}

pub fn toggle_sidebar(sidebar_ui: &mut Signal<SidebarUiState>) {
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
