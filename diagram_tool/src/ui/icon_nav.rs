#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::ignored_unit_patterns)]
#![forbid(unsafe_code)]

use base64::{engine::general_purpose, Engine as _};
use dioxus::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

use crate::icons::{icon_index, IconMeta, ICONS};

static MAX_ICONS_PER_PROVIDER: usize = 50;

#[derive(Clone, PartialEq)]
struct ProviderState {
    expanded: bool,
    visible_count: usize,
}

fn icon_data_url(icon: &IconMeta) -> Option<String> {
    let file = ICONS.get_file(&icon.file_relpath)?;

    let ext = std::path::Path::new(&icon.file_relpath)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");

    let mime = if ext == "svg" {
        "image/svg+xml"
    } else {
        "image/png"
    };
    Some(format!(
        "data:{mime};base64,{}",
        general_purpose::STANDARD.encode(file.contents())
    ))
}

#[component]
pub fn IconNav(
    selected_icon_key: Signal<Option<String>>,
    dragging_icon: Signal<Option<String>>,
) -> Element {
    let mut search = use_signal(String::new);
    let mut provider_states: Signal<BTreeMap<String, ProviderState>> = use_signal(BTreeMap::new);
    let index = icon_index();

    let filtered = index.filter(&search.read());

    let providers: BTreeSet<String> = filtered.iter().map(|icon| icon.provider.clone()).collect();

    rsx! {
        div {
            class: "icon-nav",
            style: "
                width: 280px;
                background: #1e1e2e;
                border-left: 1px solid #313244;
                display: flex;
                flex-direction: column;
                height: 100%;
                color: #cdd6f4;
                font-family: system-ui, -apple-system, sans-serif;
            ",

            div {
                style: "padding: 12px; border-bottom: 1px solid #313244;",
                h3 {
                    style: "margin: 0 0 8px 0; font-size: 12px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: #a6adc8;",
                    "Diagrams Icons"
                }
                input {
                    style: "
                        width: 100%;
                        padding: 8px 12px;
                        border: 1px solid #45475a;
                        border-radius: 6px;
                        background: #313244;
                        color: #cdd6f4;
                        font-size: 13px;
                        outline: none;
                        box-sizing: border-box;
                    ",
                    placeholder: "Search icons...",
                    value: "{search}",
                    oninput: move |evt| search.set(evt.value()),
                }
            }

            div {
                style: "flex: 1; overflow-y: auto; padding: 8px;",
                "data-testid": "icon-grid",

                if search.read().is_empty() {
                    for provider in providers {
                        {
                            let state = provider_states.read()
                                .get(&provider)
                                .cloned()
                                .unwrap_or(ProviderState { expanded: false, visible_count: MAX_ICONS_PER_PROVIDER });

                            let provider_icons: Vec<IconMeta> = filtered
                                .iter()
                                .filter(|icon| icon.provider == provider)
                                .take(state.visible_count)
                                .map(|icon| (*icon).clone())
                                .collect();

                            let total_count = index.icons_by_provider(&provider).len();
                            let has_more = total_count > state.visible_count;

                            rsx! {
                                ProviderSection {
                                    key: "{provider}",
                                    provider: provider.clone(),
                                    expanded: state.expanded,
                                    icons: provider_icons,
                                    total_count,
                                    has_more,
                                    selected_icon_key,
                                    dragging_icon,
                                    on_toggle: {
                                        let provider = provider.clone();
                                        move |_| {
                                            let current = provider_states.read()
                                                .get(&provider)
                                                .cloned()
                                                .unwrap_or(ProviderState { expanded: false, visible_count: MAX_ICONS_PER_PROVIDER });
                                            provider_states.write().insert(
                                                provider.clone(),
                                                ProviderState {
                                                    expanded: !current.expanded,
                                                    visible_count: current.visible_count
                                                }
                                            );
                                        }
                                    },
                                    on_load_more: {
                                        let provider = provider.clone();
                                        move |_| {
                                            let current = provider_states.read()
                                                .get(&provider)
                                                .cloned()
                                                .unwrap_or(ProviderState { expanded: false, visible_count: MAX_ICONS_PER_PROVIDER });
                                            provider_states.write().insert(
                                                provider.clone(),
                                                ProviderState {
                                                    expanded: current.expanded,
                                                    visible_count: current.visible_count + MAX_ICONS_PER_PROVIDER
                                                }
                                            );
                                        }
                                    },
                                }
                            }
                        }
                    }
                } else {
                    div {
                        style: "display: grid; grid-template-columns: repeat(3, 1fr); gap: 4px;",
                        for icon in filtered.iter().take(150) {
                            IconItem {
                                key: "{icon.icon_key}",
                                icon: (*icon).clone(),
                                selected_icon_key,
                                dragging_icon,
                            }
                        }
                    }
                    if filtered.len() > 150 {
                        div {
                            style: "text-align: center; padding: 12px; color: #6c7086; font-size: 11px;",
                            "Showing 150 of {filtered.len()} results"
                        }
                    }
                }
            }

            if let Some(ref key) = *selected_icon_key.read() {
                div {
                    style: "
                        padding: 12px;
                        border-top: 1px solid #313244;
                        background: #181825;
                        font-size: 11px;
                        color: #89b4fa;
                    ",
                    "data-testid": "selected-icon",
                    "Selected: {key}"
                }
            }
        }
    }
}

#[component]
fn ProviderSection(
    provider: String,
    expanded: bool,
    icons: Vec<IconMeta>,
    total_count: usize,
    has_more: bool,
    selected_icon_key: Signal<Option<String>>,
    dragging_icon: Signal<Option<String>>,
    on_toggle: EventHandler<()>,
    on_load_more: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            style: "margin-bottom: 8px;",
            "data-testid": "provider-{provider}",

            button {
                style: "
                    width: 100%;
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    padding: 6px 8px;
                    background: transparent;
                    border: none;
                    border-radius: 4px;
                    color: #cdd6f4;
                    font-size: 12px;
                    font-weight: 500;
                    cursor: pointer;
                    text-align: left;
                ",
                "data-testid": "provider-toggle-{provider}",
                onclick: move |_| on_toggle.call(()),

                span {
                    style: "display: flex; align-items: center; gap: 6px;",
                    span { style: "font-size: 10px; opacity: 0.6;", if expanded { "▼" } else { "▶" } }
                    span { "{provider}" }
                    span {
                        style: "font-size: 10px; color: #6c7086;",
                        "({total_count})"
                    }
                }
            }

            if expanded {
                div {
                    style: "
                        display: grid;
                        grid-template-columns: repeat(3, 1fr);
                        gap: 4px;
                        padding: 4px 8px;
                    ",
                    for icon in &icons {
                        IconItem {
                            key: "{icon.icon_key}",
                            icon: icon.clone(),
                            selected_icon_key,
                            dragging_icon,
                        }
                    }
                }

                if has_more {
                    button {
                        style: "
                            width: calc(100% - 16px);
                            margin: 4px 8px;
                            padding: 6px;
                            background: #313244;
                            border: 1px solid #45475a;
                            border-radius: 4px;
                            color: #a6adc8;
                            font-size: 11px;
                            cursor: pointer;
                        ",
                        onclick: move |_| on_load_more.call(()),
                        "Load more..."
                    }
                }
            }
        }
    }
}

#[component]
fn IconItem(
    icon: IconMeta,
    selected_icon_key: Signal<Option<String>>,
    dragging_icon: Signal<Option<String>>,
) -> Element {
    let data_url = icon_data_url(&icon);
    let is_selected = selected_icon_key.read().as_ref() == Some(&icon.icon_key);
    let icon_key_for_drag = icon.icon_key.clone();
    let icon_key_for_click = icon.icon_key.clone();

    let border_style = if is_selected {
        "2px solid #89b4fa"
    } else {
        "1px solid #45475a"
    };
    let bg_style = if is_selected { "#313244" } else { "#1e1e2e" };

    let img_src = data_url.clone().unwrap_or_default();

    rsx! {
        button {
            style: "
                aspect-ratio: 1;
                padding: 4px;
                background: {bg_style};
                border: {border_style};
                border-radius: 4px;
                cursor: grab;
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                transition: all 0.15s ease;
            ",
            "data-testid": "icon-item-{icon.icon_key}",
            "data-icon-key": "{icon.icon_key}",
            title: "{icon.display_name}\n{icon.icon_key}",
            draggable: "true",
            onmousedown: move |_| {
                dragging_icon.set(Some(icon_key_for_drag.clone()));
            },
            onmouseup: move |_| {
                dragging_icon.set(None);
            },
            onclick: move |_| {
                selected_icon_key.set(Some(icon_key_for_click.clone()));
            },

            if let Some(_src) = data_url {
                img {
                    src: "{img_src}",
                    style: "width: 32px; height: 32px; object-fit: contain; pointer-events: none;",
                    draggable: "false",
                }
            } else {
                div {
                    style: "width: 32px; height: 32px; background: #45475a; border-radius: 4px;",
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_icon_meta_when_creating_data_url_then_returns_base64_string() {
        let index = icon_index();
        assert!(!index.all.is_empty(), "Icon index should have icons");

        let first_icon = &index.all[0];
        let data_url = icon_data_url(first_icon);

        assert!(data_url.is_some(), "Should find icon file");
        let url = data_url.unwrap();
        assert!(url.starts_with("data:image/"), "Should be data URL");
        assert!(url.contains(";base64,"), "Should be base64 encoded");
    }

    #[test]
    fn given_icon_index_when_filtering_by_query_then_returns_matching_icons() {
        let index = icon_index();

        let all = index.filter("");
        assert!(!all.is_empty(), "Empty query should return all icons");

        let aws = index.filter("aws");
        assert!(!aws.is_empty(), "Should find AWS icons");
        assert!(
            aws.iter().all(|i| i.icon_key.contains("aws")),
            "All results should contain aws"
        );

        let compute = index.filter("compute");
        assert!(!compute.is_empty(), "Should find compute icons");
    }

    #[test]
    fn given_icon_index_when_getting_by_provider_then_returns_provider_icons() {
        let index = icon_index();

        let aws_icons = index.icons_by_provider("aws");
        assert!(!aws_icons.is_empty(), "AWS should have icons");

        let k8s_icons = index.icons_by_provider("k8s");
        assert!(!k8s_icons.is_empty(), "K8s should have icons");

        let unknown = index.icons_by_provider("nonexistent");
        assert!(unknown.is_empty(), "Unknown provider should return empty");
    }

    #[test]
    fn given_icon_index_when_checking_structure_then_has_required_fields() {
        let index = icon_index();

        for icon in &index.all {
            assert!(!icon.icon_key.is_empty(), "icon_key should not be empty");
            assert!(!icon.provider.is_empty(), "provider should not be empty");
            assert!(
                !icon.file_relpath.is_empty(),
                "file_relpath should not be empty"
            );
            assert!(
                !icon.display_name.is_empty(),
                "display_name should not be empty"
            );
        }
    }

    #[test]
    fn given_icon_index_when_checking_by_key_then_all_icons_retrievable() {
        let index = icon_index();

        for icon in &index.all {
            let retrieved = index.by_key.get(&icon.icon_key);
            assert!(
                retrieved.is_some(),
                "Icon {} should be in by_key",
                icon.icon_key
            );
            assert_eq!(retrieved.unwrap().icon_key, icon.icon_key);
        }
    }

    #[test]
    fn given_icon_index_when_counting_providers_then_has_expected_count() {
        let index = icon_index();

        assert!(
            index.by_provider.contains_key("aws"),
            "Should have AWS provider"
        );
        assert!(
            index.by_provider.contains_key("azure"),
            "Should have Azure provider"
        );
        assert!(
            index.by_provider.contains_key("gcp"),
            "Should have GCP provider"
        );
        assert!(
            index.by_provider.contains_key("k8s"),
            "Should have K8s provider"
        );

        let total_from_providers: usize = index.by_provider.values().map(Vec::len).sum();
        assert_eq!(
            total_from_providers,
            index.all.len(),
            "Provider totals should match all count"
        );
    }
}
