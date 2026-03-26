use std::collections::{BTreeMap, BTreeSet};

use dioxus::prelude::*;

use crate::{
    app::DraggedIconPayload,
    ui::sidebar_primitives::{
        SidebarGroup, SidebarMenuButton, SidebarMenuItem, SidebarMenuSub, SidebarMenuSubItem,
    },
};

use super::icon_tile::IconTile;
use super::models::{self, category_key, ProviderBucket, INITIAL_PROVIDER_LIMIT, LOAD_MORE_STEP};
use super::{toggle_set, SidebarState};

#[component]
pub fn SearchBox(mut search: Signal<String>, search_is_truncated: bool) -> Element {
    rsx! {
        div {
            class: "relative mb-4",
            div {
                class: "pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground",
                crate::ui::icons::Icon { kind: crate::ui::icons::IconKind::Search, size: 14, color: None }
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
                "Showing first {models::MAX_SEARCH_RESULTS} matches. Refine search to narrow results."
            }
        }
    }
}

pub fn get_category_icon(category_name: &str) -> crate::ui::icons::IconKind {
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
pub fn CategoryButton(
    name: String,
    count: usize,
    icon_kind: crate::ui::icons::IconKind,
    expanded: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let base_class = "w-full flex justify-between items-center py-1.5 px-2 rounded-md border border-transparent bg-transparent text-foreground cursor-pointer text-[13px] hover:bg-[var(--bg-elevated)] transition-colors";
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
pub fn CategoryGrid(
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
pub fn CategoryAccordion(
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
pub fn LoadMoreButton(
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
pub fn ProviderAccordion(
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
pub fn SidebarFooter(total_components: usize) -> Element {
    rsx! {
        div {
            class: "mt-auto pt-4 pb-1 border-t border-border text-center text-[11px] text-muted-foreground leading-relaxed",
            "{total_components} components available"
            br {}
            "Drag to canvas to add"
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::icons::IconMeta;
    use dioxus::prelude::*;

    #[test]
    fn test_search_box_rendering() {
        let mut vdom = VirtualDom::new(|| {
            let search = use_signal(|| String::from("database"));
            rsx! {
                SearchBox { search, search_is_truncated: true }
            }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Showing first"));
        assert!(html.contains("database"));
    }

    #[test]
    fn test_category_button_rendering() {
        let mut vdom = VirtualDom::new(|| {
            rsx! {
                CategoryButton {
                    name: String::from("Analytics"),
                    count: 42,
                    icon_kind: crate::ui::icons::IconKind::Database,
                    expanded: true,
                    onclick: |_| {}
                }
            }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Analytics"));
        assert!(html.contains("42"));
    }

    #[test]
    fn test_sidebar_footer_rendering() {
        let mut vdom = VirtualDom::new(|| {
            rsx! {
                SidebarFooter {
                    total_components: 1337
                }
            }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("1337 components available"));
    }
}
