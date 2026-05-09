#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

pub mod components;
pub mod icon_tile;
pub mod models;

use std::collections::{BTreeMap, BTreeSet};

use dioxus::prelude::*;

use crate::ui::{
    mobile::{close_sidebar, open_sidebar},
    sidebar_primitives::{
        Sidebar as SidebarPanel, SidebarCollapsible, SidebarHeader, SidebarInset, SidebarMenu,
        SidebarOverlay, SidebarProvider, SidebarRail, SidebarSheet, SidebarSide, SidebarTrigger,
        SidebarVariant,
    },
};

use self::components::{ProviderAccordion, SearchBox, SidebarFooter};
use self::models::{
    build_provider_buckets, category_keys_for_visible_provider_icons, DEFAULT_EXPANDED_CATEGORY,
    DEFAULT_EXPANDED_PROVIDER, INITIAL_PROVIDER_LIMIT, MAX_SEARCH_RESULTS,
};

#[derive(Clone, Copy, PartialEq)]
pub struct SidebarState {
    pub search: Signal<String>,
    pub debounced_search: Signal<String>,
    pub expanded_providers: Signal<BTreeSet<String>>,
    pub expanded_categories: Signal<BTreeSet<String>>,
    pub provider_limits: Signal<BTreeMap<String, usize>>,
    pub search_limit: Signal<usize>,
}

fn use_sidebar_state() -> SidebarState {
    let search = use_signal(String::new);
    let mut debounced_search = use_signal(String::new);
    let mut search_limit = use_signal(|| MAX_SEARCH_RESULTS);

    use_resource(move || {
        let current = search();
        async move {
            if !current.is_empty() {
                gloo_timers::future::sleep(std::time::Duration::from_millis(200)).await;
            }
            debounced_search.set(current);
            search_limit.set(MAX_SEARCH_RESULTS);
        }
    });

    SidebarState {
        search,
        debounced_search,
        expanded_providers: use_signal(|| {
            BTreeSet::from([String::from(DEFAULT_EXPANDED_PROVIDER)])
        }),
        expanded_categories: use_signal(|| {
            match category_keys_for_visible_provider_icons(
                DEFAULT_EXPANDED_PROVIDER,
                INITIAL_PROVIDER_LIMIT,
                crate::icons::icon_index(),
            ) {
                Ok(keys) => keys.into_iter().collect(),
                Err(_) => BTreeSet::from([String::from(DEFAULT_EXPANDED_CATEGORY)]),
            }
        }),
        provider_limits: use_signal(BTreeMap::new),
        search_limit,
    }
}

pub fn toggle_set(state: &mut Signal<BTreeSet<String>>, key: &str, query_active: bool) {
    if query_active {
        return;
    }
    let mut state = state.write();
    if !state.remove(key) {
        state.insert(key.to_string());
    }
}

fn render_mobile_closed(sidebar_ui: Signal<crate::ui::mobile::SidebarUiState>) -> Element {
    rsx! {
        SidebarProvider {
            sidebar_ui, side: SidebarSide::Left, variant: SidebarVariant::Sidebar, collapsible: SidebarCollapsible::Offcanvas,
            SidebarTrigger {
                label: String::from("Browse icons"), title: String::from("Open icon browser"),
                class: Some(String::from("fixed top-[64px] left-[10px] z-[72] rounded-full border border-border bg-surface/90 text-foreground py-1.5 px-3 cursor-pointer backdrop-blur-md shadow-lg")),
            }
        }
    }
}

fn render_desktop_closed(sidebar_ui: Signal<crate::ui::mobile::SidebarUiState>) -> Element {
    rsx! {
        SidebarProvider {
            sidebar_ui, side: SidebarSide::Left, variant: SidebarVariant::Sidebar, collapsible: SidebarCollapsible::Offcanvas,
            SidebarRail { label: String::from(">"), title: String::from("Expand sidebar"), onclick: move |_| { open_sidebar(sidebar_ui); }, }
        }
    }
}

fn get_panel_class(is_mobile: bool) -> String {
    if is_mobile {
        String::from("fixed top-[56px] bottom-0 left-0 w-[min(19rem,90vw)] bg-gradient-to-b from-surface to-base border-r border-border-subtle p-[max(10px,env(safe-area-inset-top))] flex flex-col gap-2.5 overflow-hidden z-[70] shadow-2xl")
    } else {
        String::from("w-[280px] max-w-[40vw] bg-gradient-to-b from-surface to-base border-r border-border-subtle p-2.5 flex flex-col gap-2.5 overflow-hidden h-full")
    }
}

fn render_sidebar_content(
    sidebar_ui: Signal<crate::ui::mobile::SidebarUiState>,
    is_mobile: bool,
    result: models::ProviderBucketsResult,
    state: SidebarState,
) -> Element {
    let query_active = !state.debounced_search.read().trim().is_empty();
    let action_label = if is_mobile { "Close" } else { "Hide" }.to_string();
    rsx! {
        SidebarSheet { class: Some(String::from("flex flex-col flex-1 min-h-0 gap-2.5")),
            SidebarHeader { title: String::from("Components"), action_label, onaction: move |_| { close_sidebar(sidebar_ui); } }
            SearchBox {
                search: state.search,
                search_is_truncated: result.is_truncated,
                search_visible_count: result.visible_count,
                search_limit: Some(state.search_limit),
            }
            SidebarInset { class: Some(String::from("flex flex-1 min-h-0 flex-col overflow-y-auto pr-1")),
                SidebarMenu {
                    for bucket in result.buckets.into_iter() { ProviderAccordion { bucket, query_active, state } }
                }
            }
            SidebarFooter { total_components: crate::icons::icon_index().all.len() }
        }
    }
}

fn render_open_sidebar(
    sidebar_ui: Signal<crate::ui::mobile::SidebarUiState>,
    is_mobile: bool,
    result: models::ProviderBucketsResult,
    state: SidebarState,
) -> Element {
    rsx! {
        SidebarProvider {
            sidebar_ui, side: SidebarSide::Left, variant: SidebarVariant::Sidebar, collapsible: SidebarCollapsible::Offcanvas,
            if is_mobile { SidebarOverlay { onclick: move |_| { close_sidebar(sidebar_ui); } } }
            SidebarPanel {
                class: Some(get_panel_class(is_mobile)),
                { render_sidebar_content(sidebar_ui, is_mobile, result, state) }
            }
        }
    }
}

#[component]
pub fn Sidebar() -> Element {
    let state = use_sidebar_state();
    let app_state = use_context::<crate::app::AppState>();
    let sidebar_ui = app_state.sidebar;

    let result = use_memo(move || {
        let trimmed = state.debounced_search.read().trim().to_lowercase();
        let lowercased =
            models::LowercasedQuery::new(&trimmed).unwrap_or_else(models::LowercasedQuery::empty);
        build_provider_buckets(
            lowercased,
            &state.provider_limits.read(),
            *state.search_limit.read(),
        )
    });

    let ui_state = *sidebar_ui.read();
    if ui_state.is_mobile && !ui_state.open_mobile {
        return render_mobile_closed(sidebar_ui);
    }
    if !ui_state.is_mobile && !ui_state.open {
        return render_desktop_closed(sidebar_ui);
    }

    let buckets_result = result.read().clone();
    render_open_sidebar(sidebar_ui, ui_state.is_mobile, buckets_result, state)
}
