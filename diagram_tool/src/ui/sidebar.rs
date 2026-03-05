#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use base64::{engine::general_purpose, Engine as _};
use dioxus::prelude::*;

use crate::{
    app::DraggedIconPayload,
    icons::{icon_index, IconMeta, ICONS},
    ui::{
        mobile::{close_sidebar, open_sidebar, SidebarUiState},
        sidebar_primitives::{
            Sidebar as SidebarPanel, SidebarCollapsible, SidebarGroup, SidebarHeader, SidebarInset,
            SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarMenuSub, SidebarMenuSubItem,
            SidebarOverlay, SidebarProvider, SidebarRail, SidebarSheet, SidebarSide,
            SidebarTrigger, SidebarVariant,
        },
        theme::{BG_BASE, BG_ELEVATED, BG_SURFACE, BORDER, BORDER_SUBTLE, TEXT_MAIN, TEXT_MUTED},
    },
};

const INITIAL_PROVIDER_LIMIT: usize = 72;
const LOAD_MORE_STEP: usize = 48;
const MAX_SEARCH_RESULTS: usize = 180;
const DEFAULT_EXPANDED_PROVIDER: &str = "aws";
const DEFAULT_EXPANDED_CATEGORY: &str = "aws/compute";

#[derive(Clone, PartialEq)]
struct CategoryBucket {
    name: String,
    icons: Vec<IconMeta>,
}

#[derive(Clone, PartialEq)]
struct ProviderBucket {
    provider: String,
    total_count: usize,
    visible_count: usize,
    has_more: bool,
    categories: Vec<CategoryBucket>,
}

fn matches_query(icon: &IconMeta, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let query_lower = query.to_ascii_lowercase();
    let category = icon.category_path.join(" ").to_ascii_lowercase();

    icon.icon_key.to_ascii_lowercase().contains(&query_lower)
        || icon
            .display_name
            .to_ascii_lowercase()
            .contains(&query_lower)
        || icon.provider.to_ascii_lowercase().contains(&query_lower)
        || category.contains(&query_lower)
}

fn category_label(icon: &IconMeta) -> String {
    if icon.category_path.is_empty() {
        String::from("General")
    } else {
        icon.category_path.join(" / ")
    }
}

fn category_key(provider: &str, category_label: &str) -> String {
    let normalized = category_label
        .split('/')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>()
        .join("/");

    format!("{}/{}", provider.to_ascii_lowercase(), normalized)
}

fn bucket_icons_by_category(icons: Vec<IconMeta>) -> Vec<CategoryBucket> {
    let grouped =
        icons
            .into_iter()
            .fold(BTreeMap::<String, Vec<IconMeta>>::new(), |mut acc, icon| {
                acc.entry(category_label(&icon)).or_default().push(icon);
                acc
            });

    grouped
        .into_iter()
        .map(|(name, icons)| CategoryBucket { name, icons })
        .collect()
}

fn search_matches(index: &[IconMeta], query: &str) -> (usize, Vec<IconMeta>) {
    index.iter().fold(
        (0_usize, Vec::<IconMeta>::new()),
        |(count, mut visible), icon| {
            if matches_query(icon, query) {
                if visible.len() < MAX_SEARCH_RESULTS {
                    visible.push(icon.clone());
                }
                (count + 1, visible)
            } else {
                (count, visible)
            }
        },
    )
}

fn build_provider_buckets(
    query: &str,
    provider_limits: &BTreeMap<String, usize>,
) -> (Vec<ProviderBucket>, bool) {
    let index = icon_index();

    if query.is_empty() {
        let buckets = index
            .by_provider
            .keys()
            .map(|provider| {
                let provider_icons = index.icons_by_provider(provider);
                let limit = provider_limits
                    .get(provider)
                    .copied()
                    .unwrap_or(INITIAL_PROVIDER_LIMIT);
                let visible_icons: Vec<IconMeta> = provider_icons
                    .iter()
                    .take(limit)
                    .map(|icon| (*icon).clone())
                    .collect();
                let visible_count = visible_icons.len();
                let total_count = provider_icons.len();

                ProviderBucket {
                    provider: provider.clone(),
                    total_count,
                    visible_count,
                    has_more: total_count > visible_count,
                    categories: bucket_icons_by_category(visible_icons),
                }
            })
            .collect();
        (buckets, false)
    } else {
        let (total_match_count, limited) = search_matches(&icon_index().all, query);
        let grouped =
            limited
                .into_iter()
                .fold(BTreeMap::<String, Vec<IconMeta>>::new(), |mut acc, icon| {
                    acc.entry(icon.provider.clone()).or_default().push(icon);
                    acc
                });

        let buckets = grouped
            .into_iter()
            .map(|(provider, icons)| {
                let visible_count = icons.len();
                ProviderBucket {
                    provider,
                    total_count: visible_count,
                    visible_count,
                    has_more: false,
                    categories: bucket_icons_by_category(icons),
                }
            })
            .collect();

        (buckets, total_match_count > MAX_SEARCH_RESULTS)
    }
}

fn icon_data_url(icon: &IconMeta) -> Option<String> {
    let file = ICONS.get_file(&icon.file_relpath)?;
    let mime = std::path::Path::new(&icon.file_relpath)
        .extension()
        .and_then(|ext| ext.to_str())
        .map_or("image/png", |ext| {
            if ext.eq_ignore_ascii_case("svg") {
                "image/svg+xml"
            } else {
                "image/png"
            }
        });

    Some(format!(
        "data:{mime};base64,{}",
        general_purpose::STANDARD.encode(file.contents())
    ))
}

#[component]
fn IconTile(icon: IconMeta, dragging_icon: Signal<Option<DraggedIconPayload>>) -> Element {
    let data_url = icon_data_url(&icon);
    let data_url_for_drag = data_url.clone();
    let data_url_for_drag_start = data_url.clone();
    let icon_key_for_drag = icon.icon_key.clone();
    let icon_key_for_title = icon.icon_key.clone();
    let category_for_title = if icon.category_path.is_empty() {
        String::from("General")
    } else {
        icon.category_path.join(" / ")
    };
    let icon_label_for_drag = icon.display_name.clone();

    rsx! {
        button {
            class: "icon-item",
            "data-testid": "icon-item",
            title: "{icon.display_name}\n{icon_key_for_title}\n{category_for_title}",
            draggable: "true",
            onmousedown: move |_| {
                dragging_icon.set(Some(DraggedIconPayload {
                    icon_key: icon_key_for_drag.clone(),
                    label: Some(icon_label_for_drag.clone()),
                    image_data_url: data_url_for_drag.clone(),
                }));
            },
            ondragstart: move |_| {
                dragging_icon.set(Some(DraggedIconPayload {
                    icon_key: icon.icon_key.clone(),
                    label: Some(icon.display_name.clone()),
                    image_data_url: data_url_for_drag_start.clone(),
                }));
            },
            style: "cursor: grab; border: 1px solid {BORDER}; border-radius: 6px; padding: 5px; display: flex; justify-content: center; align-items: center; background: linear-gradient(180deg, {BG_BASE} 0%, {BG_ELEVATED} 100%); aspect-ratio: 1/1; box-shadow: inset 0 0 0 1px color-mix(in oklch, {BORDER} 60%, transparent);",

            if let Some(src) = data_url {
                img {
                    src: "{src}",
                    width: "32px",
                    height: "32px",
                    style: "object-fit: contain; pointer-events: none;",
                    draggable: "false"
                }
            } else {
                div {
                    style: "width: 32px; height: 32px; border-radius: 4px; background: #1f2937;"
                }
            }
        }
    }
}

#[component]
pub fn Sidebar() -> Element {
    let mut search = use_signal(String::new);
    let mut expanded_providers: Signal<BTreeSet<String>> =
        use_signal(|| BTreeSet::from([String::from(DEFAULT_EXPANDED_PROVIDER)]));
    let mut expanded_categories: Signal<BTreeSet<String>> =
        use_signal(|| BTreeSet::from([String::from(DEFAULT_EXPANDED_CATEGORY)]));
    let mut provider_limits: Signal<BTreeMap<String, usize>> = use_signal(BTreeMap::new);
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
                sidebar_ui,
                side: SidebarSide::Left,
                variant: SidebarVariant::Sidebar,
                collapsible: SidebarCollapsible::Offcanvas,
                SidebarTrigger {
                    label: String::from("Browse icons"),
                    title: String::from("Open icon browser"),
                    style: Some(format!("position: fixed; top: 64px; left: 10px; z-index: 72; border-radius: 999px; border: 1px solid {BORDER}; background: color-mix(in oklch, {BG_SURFACE} 92%, transparent); color: {TEXT_MAIN}; padding: 7px 12px; cursor: pointer; backdrop-filter: blur(8px); box-shadow: 0 8px 16px color-mix(in oklch, black 20%, transparent);")),
                }
            }
        };
    }

    if !ui_state.is_mobile && !ui_state.open {
        return rsx! {
            SidebarProvider {
                sidebar_ui,
                side: SidebarSide::Left,
                variant: SidebarVariant::Sidebar,
                collapsible: SidebarCollapsible::Offcanvas,
                SidebarRail {
                    label: String::from(">"),
                    title: String::from("Expand sidebar"),
                    onclick: move |_| {
                        open_sidebar(sidebar_ui);
                    },
                }
            }
        };
    }

    let panel_style = if ui_state.is_mobile {
        format!(
            "position: fixed; top: 56px; bottom: 0; left: 0; width: min(19rem, 90vw); background: linear-gradient(180deg, {BG_SURFACE} 0%, {BG_BASE} 100%); border-right: 1px solid {BORDER_SUBTLE}; padding: max(10px, env(safe-area-inset-top)) 10px max(10px, env(safe-area-inset-bottom)); display: flex; flex-direction: column; gap: 10px; overflow-y: auto; z-index: 70; box-shadow: 0 14px 28px color-mix(in oklch, black 26%, transparent);"
        )
    } else {
        format!(
            "width: 280px; max-width: 40vw; background: linear-gradient(180deg, {BG_SURFACE} 0%, {BG_BASE} 100%); border-right: 1px solid {BORDER_SUBTLE}; padding: 10px; display: flex; flex-direction: column; gap: 10px; overflow-y: auto;"
        )
    };

    rsx! {
        SidebarProvider {
            sidebar_ui,
            side: SidebarSide::Left,
            variant: SidebarVariant::Sidebar,
            collapsible: SidebarCollapsible::Offcanvas,

            if ui_state.is_mobile {
                SidebarOverlay {
                    onclick: move |_| {
                        close_sidebar(sidebar_ui);
                    }
                }
            }

            SidebarPanel {
                style: Some(panel_style),

                SidebarSheet {
                    style: String::new(),
            SidebarHeader {
                title: String::from("Diagram Icons"),
                action_label: if ui_state.is_mobile { String::from("Close") } else { String::from("Hide") },
                onaction: move |_| {
                    close_sidebar(sidebar_ui);
                }
            }

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

            SidebarInset {
                style: Some(String::from("display: flex; flex: 1; min-height: 0; flex-direction: column;")),
                SidebarMenu {
                    for bucket in provider_buckets {
                        {
                            let provider = bucket.provider.clone();
                            let provider_ref = &provider;
                            let expanded = query_active || expanded_providers.read().contains(provider_ref);
                            let visible_count = bucket.visible_count;
                            let total_count = bucket.total_count;
                            let has_more = bucket.has_more;

                            rsx! {
                                SidebarMenuItem {
                                    SidebarGroup {
                                        provider: provider.clone(),
                                        expanded,
                                        query_active,
                                        visible_count,
                                        total_count,
                                        ontoggle: {
                                            let provider = provider.clone();
                                            move |_| {
                                                if query_active {
                                                    return;
                                                }
                                                if expanded_providers.read().contains(&provider) {
                                                    let _ = expanded_providers.write().remove(&provider);
                                                } else {
                                                    let _ = expanded_providers.write().insert(provider.clone());
                                                }
                                            }
                                        },
                                        children: rsx! {
                                            SidebarMenuSub {
                                                for category in bucket.categories {
                                                    {
                                                        let category_state_key = category_key(&provider, &category.name);
                                                        let category_expanded =
                                                            query_active || expanded_categories.read().contains(&category_state_key);

                                                        rsx! {
                                                            SidebarMenuSubItem {
                                                                key: "{provider}-{category.name}",
                                                                div {
                                                                    style: "display: flex; flex-direction: column; gap: 4px;",

                                                                    button {
                                                                        style: "width: 100%; margin: 0; border: none; background: transparent; color: {TEXT_MUTED}; text-transform: uppercase; letter-spacing: 0.04em; font-size: 10px; text-align: left; padding: 0; cursor: pointer;",
                                                                        onclick: {
                                                                            move |_| {
                                                                                if query_active {
                                                                                    return;
                                                                                }
                                                                                if expanded_categories.read().contains(&category_state_key) {
                                                                                    let _ = expanded_categories.write().remove(&category_state_key);
                                                                                } else {
                                                                                    let _ = expanded_categories
                                                                                        .write()
                                                                                        .insert(category_state_key.clone());
                                                                                }
                                                                            }
                                                                        },
                                                                        if query_active {
                                                                            "{category.name}"
                                                                        } else if category_expanded {
                                                                            "▼ {category.name}"
                                                                        } else {
                                                                            "▶ {category.name}"
                                                                        }
                                                                    }

                                                                    if category_expanded {
                                                                        div {
                                                                            class: "icon-grid",
                                                                            style: "display: grid; grid-template-columns: repeat(4, 1fr); gap: 5px;",
                                    for icon in category.icons.iter().cloned() {
                                        IconTile {
                                            key: "{icon.icon_key}",
                                            icon,
                                            dragging_icon,
                                        }
                                    }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            if has_more {
                                                SidebarMenuButton {
                                                    label: String::from("Load more"),
                                                    onclick: move |_| {
                                                        let current_limit = provider_limits
                                                            .read()
                                                            .get(&provider)
                                                            .copied()
                                                            .unwrap_or(INITIAL_PROVIDER_LIMIT);
                                                        provider_limits
                                                            .write()
                                                            .insert(provider.clone(), current_limit + LOAD_MORE_STEP);
                                                    },
                                                }
                                            }
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            }
                }
            }
        }
    }
}
