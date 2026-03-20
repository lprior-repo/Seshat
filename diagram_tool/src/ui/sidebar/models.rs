use std::collections::BTreeMap;

use crate::icons::{icon_index, IconMeta};

pub const INITIAL_PROVIDER_LIMIT: usize = 3000;
pub const LOAD_MORE_STEP: usize = 3000;
pub const MAX_SEARCH_RESULTS: usize = 3000;
pub const DEFAULT_EXPANDED_PROVIDER: &str = "aws";
pub const DEFAULT_EXPANDED_CATEGORY: &str = "aws/analytics";

#[derive(Clone, PartialEq)]
pub struct CategoryBucket {
    pub name: String,
    pub icons: Vec<IconMeta>,
}

#[derive(Clone, PartialEq)]
pub struct ProviderBucket {
    pub provider: String,
    pub total_count: usize,
    pub visible_count: usize,
    pub has_more: bool,
    pub categories: Vec<CategoryBucket>,
}

pub fn matches_query(icon: &IconMeta, query: &str) -> bool {
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

pub fn category_label(icon: &IconMeta) -> String {
    if icon.category_path.is_empty() {
        String::from("General")
    } else {
        icon.category_path.join(" / ")
    }
}

pub fn category_key(provider: &str, category_label: &str) -> String {
    let normalized = category_label
        .split('/')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>()
        .join("/");

    format!("{}/{}", provider.to_ascii_lowercase(), normalized)
}

pub fn bucket_icons_by_category(icons: Vec<IconMeta>) -> Vec<CategoryBucket> {
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

pub fn search_matches(index: &[IconMeta], query: &str) -> (usize, Vec<IconMeta>) {
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

pub fn build_provider_buckets(
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
