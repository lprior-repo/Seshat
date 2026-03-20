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
    DEFAULT_EXPANDED_PROVIDER, INITIAL_PROVIDER_LIMIT, LOAD_MORE_STEP, MAX_SEARCH_RESULTS,
};

#[component]
fn SearchBox(mut search: Signal<String>, search_is_truncated: bool) -> Element {
    rsx! {
        div {
            class: "relative mb-4",
            crate::ui::icons::Icon {
                kind: crate::ui::icons::IconKind::Search,
                size: 16,
                color: Some("var(--text-muted)"),
            }
            // we will position the icon absolute using inline style or utility class if we had an outer container.
            // Let's use standard tailwind for absolute positioning:
            div {
                class: "absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground",
                crate::ui::icons::Icon {
                    kind: crate::ui::icons::IconKind::Search,
                    size: 14,
                    color: None,
                }
            }
            input {
                placeholder: "Search icons...",
                value: "{search}",
                class: "pl-8 pr-2 py-1.5 w-full rounded-md border border-[var(--border-subtle)] bg-[oklch(0.13_0.005_260)] text-foreground text-[13px] outline-none focus:border-[var(--accent)] transition-colors",
                oninput: move |evt| search.set(evt.value())
            }
        }
        if search_is_truncated {
            div {
                class: "text-[11px] text-muted-foreground mb-2",
                "Showing first {MAX_SEARCH_RESULTS} matches. Refine search to narrow results."
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
        crate::ui::icons::IconKind::Grid // generic
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
    let count = category.icons.len();
    let icon_kind = get_category_icon(&name);

    let base_btn_class = "w-full flex justify-between items-center py-1.5 px-2 rounded-md border border-transparent bg-transparent text-foreground cursor-pointer text-[13px] hover:bg-white/5 transition-colors";
    let active_btn_class = "w-full flex justify-between items-center py-1.5 px-2 rounded-md border border-[var(--accent)] bg-[oklch(0.12_0.02_165)] text-foreground cursor-pointer text-[13px] transition-colors";
    let btn_class = if category_expanded {
        active_btn_class
    } else {
        base_btn_class
    };

    rsx! {
        SidebarMenuSubItem {
            key: "{provider}-{name}",
            div {
                class: "flex flex-col gap-1 w-full",
                button {
                    class: "{btn_class}",
                    onclick: move |_| {
                        if query_active { return; }
                        let mut state = expanded_categories.write();
                        if state.contains(&category_state_key) {
                            state.remove(&category_state_key);
                        } else {
                            state.insert(category_state_key.clone());
                        }
                    },
                    div {
                        class: "flex items-center gap-2",
                        crate::ui::icons::Icon {
                            kind: if category_expanded { crate::ui::icons::IconKind::ChevronDown } else { crate::ui::icons::IconKind::ChevronRight },
                            size: 14,
                            color: Some("currentColor")
                        }
                        crate::ui::icons::Icon {
                            kind: icon_kind,
                            size: 14,
                            color: Some("currentColor") // or subtle color
                        }
                        span { class: "font-normal", "{name}" }
                    }
                    span { class: "text-muted-foreground text-[11px]", "{count}" }
                }

                if category_expanded {
                    div {
                        class: "grid grid-cols-3 gap-[5px] mt-1 pl-4",
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
    let app_state = use_context::<crate::app::AppState>();
    let dragging_icon = app_state.dragging_icon;
    let sidebar_ui = app_state.sidebar;

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
                    class: Some(String::from("fixed top-[64px] left-[10px] z-[72] rounded-full border border-border bg-surface/90 text-foreground py-1.5 px-3 cursor-pointer backdrop-blur-md shadow-lg")),
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

    let panel_class = if ui_state.is_mobile {
        String::from("fixed top-[56px] bottom-0 left-0 w-[min(19rem,90vw)] bg-gradient-to-b from-surface to-background border-r border-border-subtle p-[max(10px,env(safe-area-inset-top))] flex flex-col gap-2.5 overflow-y-auto z-[70] shadow-2xl")
    } else {
        String::from("w-[280px] max-w-[40vw] bg-gradient-to-b from-surface to-background border-r border-border-subtle p-2.5 flex flex-col gap-2.5 overflow-y-auto")
    };

    rsx! {
        SidebarProvider {
            sidebar_ui, side: SidebarSide::Left, variant: SidebarVariant::Sidebar, collapsible: SidebarCollapsible::Offcanvas,
            if ui_state.is_mobile {
                SidebarOverlay { onclick: move |_| { close_sidebar(sidebar_ui); } }
            }
            SidebarPanel {
                class: Some(panel_class),
                SidebarSheet {
                    style: String::new(),
                    SidebarHeader {
                        title: String::from("Components"),
                        action_label: if ui_state.is_mobile { String::from("Close") } else { String::from("Hide") },
                        onaction: move |_| { close_sidebar(sidebar_ui); }
                    }
                    SearchBox { search, search_is_truncated }
                    SidebarInset {
                        class: Some(String::from("flex flex-1 min-h-0 flex-col")),
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
