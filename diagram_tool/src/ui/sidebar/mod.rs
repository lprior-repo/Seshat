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
        mobile::{close_sidebar, open_sidebar, SidebarUiState},
        sidebar_primitives::{
            Sidebar as SidebarPanel, SidebarCollapsible, SidebarGroup, SidebarHeader, SidebarInset,
            SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarMenuSub, SidebarMenuSubItem,
            SidebarOverlay, SidebarProvider, SidebarRail, SidebarSheet, SidebarSide,
            SidebarTrigger, SidebarVariant,
        },
        theme::{BG_BASE, BG_SURFACE, BORDER, BORDER_SUBTLE, TEXT_MAIN, TEXT_MUTED},
    },
};

use self::icon_tile::IconTile;
use self::models::{
    build_provider_buckets, category_key, ProviderBucket, DEFAULT_EXPANDED_CATEGORY,
    DEFAULT_EXPANDED_PROVIDER, INITIAL_PROVIDER_LIMIT, LOAD_MORE_STEP, MAX_SEARCH_RESULTS,
};

#[component]
fn SearchBox(mut search: Signal<String>, search_is_truncated: bool) -> Element {
    rsx! {
        input {
            placeholder: "Search icons...",
            value: "{search}",
            style: "padding: 6px 8px; width: 100%; border-radius: 6px; border: 1px solid {BORDER}; background: {BG_BASE}; color: {TEXT_MAIN};",
            oninput: move |evt| search.set(evt.value())
        }
        if search_is_truncated {
            div {
                style: "font-size: 11px; color: {TEXT_MUTED};",
                "Showing first {MAX_SEARCH_RESULTS} matches. Refine search to narrow results."
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
    dragging_icon: Signal<Option<DraggedIconPayload>>,
) -> Element {
    let category_state_key = category_key(&provider, &category.name);
    let category_expanded =
        query_active || expanded_categories.read().contains(&category_state_key);
    let name = category.name.clone();

    rsx! {
        SidebarMenuSubItem {
            key: "{provider}-{name}",
            div {
                style: "display: flex; flex-direction: column; gap: 4px;",
                button {
                    style: "width: 100%; margin: 0; border: none; background: transparent; color: {TEXT_MUTED}; text-transform: uppercase; letter-spacing: 0.04em; font-size: 10px; text-align: left; padding: 0; cursor: pointer;",
                    onclick: move |_| {
                        if query_active { return; }
                        let mut state = expanded_categories.write();
                        if state.contains(&category_state_key) {
                            state.remove(&category_state_key);
                        } else {
                            state.insert(category_state_key.clone());
                        }
                    },
                    if query_active { "{name}" } else if category_expanded { "▼ {name}" } else { "▶ {name}" }
                }

                if category_expanded {
                    div {
                        class: "icon-grid",
                        style: "display: grid; grid-template-columns: repeat(4, 1fr); gap: 5px;",
                        for icon in category.icons.iter().cloned() {
                            IconTile { key: "{icon.icon_key}", icon, dragging_icon }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ProviderAccordion(
    bucket: ProviderBucket,
    query_active: bool,
    mut expanded_providers: Signal<BTreeSet<String>>,
    expanded_categories: Signal<BTreeSet<String>>,
    mut provider_limits: Signal<BTreeMap<String, usize>>,
    dragging_icon: Signal<Option<DraggedIconPayload>>,
) -> Element {
    let provider = bucket.provider.clone();
    let provider_for_toggle = provider.clone();
    let expanded = query_active || expanded_providers.read().contains(&provider);

    rsx! {
        SidebarMenuItem {
            SidebarGroup {
                provider: provider.clone(),
                expanded,
                query_active,
                total_count: bucket.total_count,
                ontoggle: move |_| {
                    if query_active { return; }
                    let mut state = expanded_providers.write();
                    if state.contains(&provider_for_toggle) {
                        state.remove(&provider_for_toggle);
                    } else {
                        state.insert(provider_for_toggle.clone());
                    }
                },
                children: rsx! {
                    SidebarMenuSub {
                        for category in bucket.categories {
                            CategoryAccordion {
                                provider: provider.clone(),
                                category,
                                query_active,
                                expanded_categories,
                                dragging_icon,
                            }
                        }
                    }
                    if bucket.has_more {
                        SidebarMenuButton {
                            label: String::from("Load more"),
                            onclick: move |_| {
                                let mut limits = provider_limits.write();
                                let current_limit = limits.get(&provider).copied().unwrap_or(INITIAL_PROVIDER_LIMIT);
                                limits.insert(provider.clone(), current_limit + LOAD_MORE_STEP);
                            },
                        }
                    }
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

#[component]
pub fn Sidebar() -> Element {
    let total_components = use_memo(move || crate::icons::icon_index().all.len());
    let search = use_signal(String::new);
    let expanded_providers: Signal<BTreeSet<String>> =
        use_signal(|| BTreeSet::from([String::from(DEFAULT_EXPANDED_PROVIDER)]));
    let expanded_categories: Signal<BTreeSet<String>> =
        use_signal(|| BTreeSet::from([String::from(DEFAULT_EXPANDED_CATEGORY)]));
    let provider_limits: Signal<BTreeMap<String, usize>> = use_signal(BTreeMap::new);
    let dragging_icon = use_context::<Signal<Option<DraggedIconPayload>>>();
    let sidebar_ui = use_context::<Signal<SidebarUiState>>();

    let trimmed_query = search.read().trim().to_ascii_lowercase();
    let query_active = !trimmed_query.is_empty();
    let (provider_buckets, search_is_truncated) =
        build_provider_buckets(&trimmed_query, &provider_limits.read());
    let ui_state = *sidebar_ui.read();

    if ui_state.is_mobile && !ui_state.open_mobile {
        return rsx! {
            SidebarProvider {
                sidebar_ui, side: SidebarSide::Left, variant: SidebarVariant::Sidebar, collapsible: SidebarCollapsible::Offcanvas,
                SidebarTrigger {
                    label: String::from("Browse icons"), title: String::from("Open icon browser"),
                    style: Some(format!("position: fixed; top: 64px; left: 10px; z-index: 72; border-radius: 999px; border: 1px solid {BORDER}; background: color-mix(in oklch, {BG_SURFACE} 92%, transparent); color: {TEXT_MAIN}; padding: 7px 12px; cursor: pointer; backdrop-filter: blur(8px); box-shadow: 0 8px 16px color-mix(in oklch, black 20%, transparent);")),
                }
            }
        };
    }

    if !ui_state.is_mobile && !ui_state.open {
        return rsx! {
            SidebarProvider {
                sidebar_ui, side: SidebarSide::Left, variant: SidebarVariant::Sidebar, collapsible: SidebarCollapsible::Offcanvas,
                SidebarRail { label: String::from(">"), title: String::from("Expand sidebar"), onclick: move |_| { open_sidebar(sidebar_ui); }, }
            }
        };
    }

    let panel_style = if ui_state.is_mobile {
        format!("position: fixed; top: 56px; bottom: 0; left: 0; width: min(19rem, 90vw); background: linear-gradient(180deg, {BG_SURFACE} 0%, {BG_BASE} 100%); border-right: 1px solid {BORDER_SUBTLE}; padding: max(10px, env(safe-area-inset-top)) 10px max(10px, env(safe-area-inset-bottom)); display: flex; flex-direction: column; gap: 10px; overflow-y: auto; z-index: 70; box-shadow: 0 14px 28px color-mix(in oklch, black 26%, transparent);")
    } else {
        format!("width: 280px; max-width: 40vw; background: linear-gradient(180deg, {BG_SURFACE} 0%, {BG_BASE} 100%); border-right: 1px solid {BORDER_SUBTLE}; padding: 10px; display: flex; flex-direction: column; gap: 10px; overflow-y: auto;")
    };

    rsx! {
        SidebarProvider {
            sidebar_ui, side: SidebarSide::Left, variant: SidebarVariant::Sidebar, collapsible: SidebarCollapsible::Offcanvas,
            if ui_state.is_mobile {
                SidebarOverlay { onclick: move |_| { close_sidebar(sidebar_ui); } }
            }
            SidebarPanel {
                style: Some(panel_style),
                SidebarSheet {
                    style: String::new(),
                    SidebarHeader {
                        title: String::from("Components"),
                        action_label: if ui_state.is_mobile { String::from("Close") } else { String::from("Hide") },
                        onaction: move |_| { close_sidebar(sidebar_ui); }
                    }
                    SearchBox { search, search_is_truncated }
                    SidebarInset {
                        style: Some(String::from("display: flex; flex: 1; min-height: 0; flex-direction: column;")),
                        SidebarMenu {
                            for bucket in provider_buckets {
                                ProviderAccordion {
                                    bucket, query_active, expanded_providers, expanded_categories, provider_limits, dragging_icon
                                }
                            }
                        }
                    }
                    SidebarFooter { total_components: total_components() }
                }
            }
        }
    }
}
