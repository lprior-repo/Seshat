#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

pub mod icon_tile;
pub mod models;

use std::collections::{BTreeMap, BTreeSet};

use dioxus::prelude::*;

use crate::{
    app::DraggedIconPayload,
    ui::{
        mobile::{close_sidebar, open_sidebar},
        sidebar_primitives::{
            Sidebar as SidebarPanel, SidebarCollapsible, SidebarGroup, SidebarHeader, SidebarInset,
            SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarMenuSub, SidebarMenuSubItem,
            SidebarOverlay, SidebarProvider, SidebarRail, SidebarSheet, SidebarSide,
            SidebarTrigger, SidebarVariant,
        },
    },
};

use self::icon_tile::IconTile;
use self::models::{
    build_provider_buckets, category_key, ProviderBucket, DEFAULT_EXPANDED_CATEGORY,
    DEFAULT_EXPANDED_PROVIDER, INITIAL_PROVIDER_LIMIT, LOAD_MORE_STEP,
};

#[derive(Clone, Copy, PartialEq)]
struct SidebarState {
    search: Signal<String>,
    debounced_search: Signal<String>,
    expanded_providers: Signal<BTreeSet<String>>,
    expanded_categories: Signal<BTreeSet<String>>,
    provider_limits: Signal<BTreeMap<String, usize>>,
}

fn use_sidebar_state() -> SidebarState {
    let search = use_signal(String::new);
    let mut debounced_search = use_signal(String::new);

    use_resource(move || async move {
        let current = search();
        if !current.is_empty() {
            gloo_timers::future::sleep(std::time::Duration::from_millis(200)).await;
        }
        debounced_search.set(current);
    });

    SidebarState {
        search,
        debounced_search,
        expanded_providers: use_signal(|| {
            BTreeSet::from([String::from(DEFAULT_EXPANDED_PROVIDER)])
        }),
        expanded_categories: use_signal(|| {
            BTreeSet::from([String::from(DEFAULT_EXPANDED_CATEGORY)])
        }),
        provider_limits: use_signal(BTreeMap::new),
    }
}

fn toggle_set(state: &mut Signal<BTreeSet<String>>, key: &str, query_active: bool) {
    if query_active {
        return;
    }
    let mut state = state.write();
    if !state.remove(key) {
        state.insert(key.to_string());
    }
}

#[component]
fn SearchBox(mut search: Signal<String>, search_is_truncated: bool) -> Element {
    rsx! {
        div {
            class: "relative mb-4",
            crate::ui::icons::Icon { kind: crate::ui::icons::IconKind::Search, size: 16, color: Some("var(--text-muted)") }
            div {
                class: "absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground",
                crate::ui::icons::Icon { kind: crate::ui::icons::IconKind::Search, size: 14, color: None }
            }
            input {
                placeholder: "Search icons...", value: "{search}",
                class: "pl-8 pr-2 py-1.5 w-full rounded-md border border-[var(--border-subtle)] bg-[oklch(0.13_0.005_260)] text-foreground text-[13px] outline-none focus:border-[var(--accent)] transition-colors",
                oninput: move |evt| search.set(evt.value())
            }
        }
        if search_is_truncated {
            div {
                class: "text-[11px] text-muted-foreground mb-2",
                "Showing first {models::MAX_SEARCH_RESULTS} matches. Refine search to narrow results."
            }
        }
    }
}

fn get_category_icon(category_name: &str) -> crate::ui::icons::IconKind {
    let lower = category_name.to_lowercase();
    if lower.contains("compute") || lower.contains("server") {
        crate::ui::icons::IconKind::Server
    } else if lower.contains("database") || lower.contains("storage") {
        crate::ui::icons::IconKind::Database
    } else if lower.contains("network") {
        crate::ui::icons::IconKind::Network
    } else if lower.contains("security") {
        crate::ui::icons::IconKind::Shield
    } else {
        crate::ui::icons::IconKind::Grid
    }
}

#[component]
fn CategoryButton(
    name: String,
    count: usize,
    icon_kind: crate::ui::icons::IconKind,
    expanded: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let base_class = "w-full flex justify-between items-center py-1.5 px-2 rounded-md border border-transparent bg-transparent text-foreground cursor-pointer text-[13px] hover:bg-white/5 transition-colors";
    let active_class = "w-full flex justify-between items-center py-1.5 px-2 rounded-md border border-[var(--accent)] bg-[oklch(0.12_0.02_165)] text-foreground cursor-pointer text-[13px] transition-colors";

    rsx! {
        button {
            class: if expanded { active_class } else { base_class },
            onclick: move |e| onclick.call(e),
            div {
                class: "flex items-center gap-2",
                crate::ui::icons::Icon { kind: if expanded { crate::ui::icons::IconKind::ChevronDown } else { crate::ui::icons::IconKind::ChevronRight }, size: 14, color: Some("currentColor") }
                crate::ui::icons::Icon { kind: icon_kind, size: 14, color: Some("currentColor") }
                span { class: "font-normal", "{name}" }
            }
            span { class: "text-muted-foreground text-[11px]", "{count}" }
        }
    }
}

#[component]
fn CategoryGrid(
    icons: Vec<crate::icons::IconMeta>,
    dragging_icon: Signal<Option<DraggedIconPayload>>,
) -> Element {
    rsx! {
        div {
            class: "grid grid-cols-3 gap-[5px] mt-1 pl-4",
            for icon in icons {
                IconTile { key: "{icon.icon_key}", icon, dragging_icon }
            }
        }
    }
}

#[component]
fn CategoryAccordion(
    provider: String,
    category: models::CategoryBucket,
    query_active: bool,
    mut expanded_categories: Signal<BTreeSet<String>>,
) -> Element {
    let category_state_key = category_key(&provider, &category.name);
    let expanded = query_active || expanded_categories.read().contains(&category_state_key);
    let app_state = use_context::<crate::app::AppState>();

    rsx! {
        SidebarMenuSubItem { key: "{provider}-{category.name}",
            div { class: "flex flex-col gap-1 w-full",
                CategoryButton {
                    name: category.name.clone(), count: category.icons.len(), icon_kind: get_category_icon(&category.name), expanded,
                    onclick: move |_| toggle_set(&mut expanded_categories, &category_state_key, query_active)
                }
                if expanded { CategoryGrid { icons: category.icons, dragging_icon: app_state.dragging_icon } }
            }
        }
    }
}

#[component]
fn LoadMoreButton(
    provider: String,
    mut provider_limits: Signal<BTreeMap<String, usize>>,
) -> Element {
    rsx! {
        SidebarMenuButton {
            label: String::from("Load more"),
            onclick: move |_| {
                let mut limits = provider_limits.write();
                let current = limits.get(&provider).copied().unwrap_or(INITIAL_PROVIDER_LIMIT);
                limits.insert(provider.clone(), current + LOAD_MORE_STEP);
            },
        }
    }
}

#[component]
fn ProviderAccordion(
    bucket: ProviderBucket,
    query_active: bool,
    mut state: SidebarState,
) -> Element {
    let provider = bucket.provider.clone();
    let expanded = query_active || state.expanded_providers.read().contains(&provider);

    rsx! {
        SidebarMenuItem {
            SidebarGroup {
                provider: provider.clone(), expanded, query_active, total_count: bucket.total_count,
                ontoggle: move |_| toggle_set(&mut state.expanded_providers, &bucket.provider, query_active),
                children: rsx! {
                    SidebarMenuSub {
                        for category in bucket.categories {
                            CategoryAccordion { provider: provider.clone(), category, query_active, expanded_categories: state.expanded_categories }
                        }
                    }
                    if bucket.has_more { LoadMoreButton { provider: provider, provider_limits: state.provider_limits } }
                }
            }
        }
    }
}

#[component]
fn SidebarFooter(total_components: usize) -> Element {
    rsx! {
        div {
            class: "mt-auto pt-4 pb-1 border-t border-border text-center text-[11px] text-muted-foreground leading-relaxed",
            "{total_components} components available"
            br {}
            "Drag to canvas to add"
        }
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
        String::from("fixed top-[56px] bottom-0 left-0 w-[min(19rem,90vw)] bg-gradient-to-b from-surface to-background border-r border-border-subtle p-[max(10px,env(safe-area-inset-top))] flex flex-col gap-2.5 overflow-hidden z-[70] shadow-2xl h-full")
    } else {
        String::from("w-[280px] max-w-[40vw] bg-gradient-to-b from-surface to-background border-r border-border-subtle p-2.5 flex flex-col gap-2.5 overflow-hidden h-full")
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
            SearchBox { search: state.search, search_is_truncated: result.is_truncated }
            SidebarInset { class: Some(String::from("flex flex-1 min-h-0 flex-col overflow-y-auto pr-2 -mr-2")),
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

    let ui_state = *sidebar_ui.read();
    if ui_state.is_mobile && !ui_state.open_mobile {
        return render_mobile_closed(sidebar_ui);
    }
    if !ui_state.is_mobile && !ui_state.open {
        return render_desktop_closed(sidebar_ui);
    }

    let result = use_memo(move || {
        let trimmed = state.debounced_search.read().trim().to_lowercase();
        let lowercased =
            models::LowercasedQuery::new(&trimmed).unwrap_or_else(models::LowercasedQuery::empty);
        build_provider_buckets(lowercased, &state.provider_limits.read())
    });

    let buckets_result = result.read().clone();
    render_open_sidebar(sidebar_ui, ui_state.is_mobile, buckets_result, state)
}
