use std::collections::BTreeMap;

use crate::icons::{icon_index, IconMeta};

pub const INITIAL_PROVIDER_LIMIT: usize = 25;
pub const LOAD_MORE_STEP: usize = 25;
pub const MAX_SEARCH_RESULTS: usize = 96;
pub const DEFAULT_EXPANDED_PROVIDER: &str = "aws";
pub const DEFAULT_EXPANDED_CATEGORY: &str = "aws/analytics";

#[derive(Clone, PartialEq, Debug)]
pub struct CategoryBucket {
    pub name: String,
    pub icons: Vec<IconMeta>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ProviderBucket {
    pub provider: String,
    pub total_count: usize,
    pub visible_count: usize,
    pub has_more: bool,
    pub categories: Vec<CategoryBucket>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ProviderBucketsResult {
    pub buckets: Vec<ProviderBucket>,
    pub is_truncated: bool,
    pub visible_count: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LowercasedQuery<'a>(&'a str);

impl<'a> LowercasedQuery<'a> {
    pub fn new(query: &'a str) -> Option<Self> {
        if query
            .chars()
            .all(|c| c.is_lowercase() || !c.is_alphabetic())
        {
            Some(Self(query))
        } else {
            None
        }
    }

    pub fn empty() -> Self {
        Self("")
    }

    pub fn as_str(&self) -> &'a str {
        self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

pub fn matches_query(icon: &IconMeta, query: LowercasedQuery<'_>) -> bool {
    if query.is_empty() {
        return true;
    }
    icon.search_terms.contains(query.as_str())
}

pub fn category_label(icon: &IconMeta) -> String {
    if icon.category_path.is_empty() {
        String::from("General")
    } else {
        icon.category_path.join(" / ")
    }
}

pub fn category_key(provider: &str, category_label: &str) -> String {
    let mut normalized = String::with_capacity(category_label.len());
    for (i, segment) in category_label
        .split('/')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .enumerate()
    {
        if i > 0 {
            normalized.push('/');
        }
        for c in segment.chars() {
            normalized.extend(c.to_lowercase());
        }
    }
    format!("{}/{}", provider.to_ascii_lowercase(), normalized)
}

pub fn category_keys_for_visible_provider_icons(
    provider: &str,
    limit: usize,
    index: &crate::icons::IconIndex,
) -> Result<Vec<String>, Error> {
    if limit == 0 || !limit.is_multiple_of(LOAD_MORE_STEP) {
        return Err(Error::InvalidLimit(limit));
    }

    if !index.by_provider.contains_key(provider) {
        return Err(Error::ProviderNotFound);
    }

    let keys = index
        .icons_by_provider(provider)
        .into_iter()
        .take(limit)
        .map(category_label)
        .map(|label| category_key(provider, &label))
        .fold(Vec::<String>::new(), |mut acc, key| {
            if !acc.contains(&key) {
                acc.push(key);
            }
            acc
        });

    Ok(keys)
}

pub fn bucket_icons_by_category(icons: &[IconMeta]) -> Vec<CategoryBucket> {
    use itertools::Itertools;
    icons
        .iter()
        .sorted_by_key(|icon| category_label(icon))
        .chunk_by(|icon| category_label(icon))
        .into_iter()
        .map(|(name, group)| CategoryBucket {
            name,
            icons: group.cloned().collect(),
        })
        .collect()
}

pub fn search_matches(
    index: &[IconMeta],
    query: LowercasedQuery<'_>,
    limit: usize,
) -> (usize, Vec<IconMeta>) {
    let cap = limit.max(1);
    let mut visible: Vec<IconMeta> = index
        .iter()
        .filter(|icon| matches_query(icon, query))
        .take(cap.saturating_add(1))
        .cloned()
        .collect();

    let count = visible.len();
    if count > cap {
        visible.pop();
    }

    (count, visible)
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("provider not found")]
    ProviderNotFound,
    #[error("invalid limit: {0}")]
    InvalidLimit(usize),
    #[error("malformed registry")]
    MalformedRegistry,
}

pub fn build_provider_bucket(
    provider: &str,
    limit: usize,
    index: &crate::icons::IconIndex,
) -> Result<ProviderBucket, Error> {
    if limit == 0 || !limit.is_multiple_of(LOAD_MORE_STEP) {
        return Err(Error::InvalidLimit(limit));
    }

    if !index.by_provider.contains_key(provider) {
        return Err(Error::ProviderNotFound);
    }

    let icons = index.icons_by_provider(provider);
    let total_count = icons.len();
    let visible: Vec<IconMeta> = icons.into_iter().take(limit).cloned().collect();
    let visible_count = visible.len();

    let categories = bucket_icons_by_category(&visible)
        .into_iter()
        .filter(|c| !c.icons.is_empty())
        .collect();

    Ok(ProviderBucket {
        provider: provider.to_string(),
        total_count,
        visible_count,
        has_more: total_count > visible_count,
        categories,
    })
}

fn build_search_buckets(query: LowercasedQuery<'_>, search_limit: usize) -> ProviderBucketsResult {
    let (match_count, limited) = search_matches(&icon_index().all, query, search_limit);
    let visible_count = limited.len();
    let mut grouped = BTreeMap::<String, Vec<IconMeta>>::new();

    for icon in limited {
        grouped
            .entry(icon.provider.to_string())
            .or_default()
            .push(icon);
    }

    let buckets = grouped
        .into_iter()
        .map(|(provider, icons)| ProviderBucket {
            total_count: icons.len(),
            visible_count: icons.len(),
            has_more: false,
            categories: bucket_icons_by_category(&icons),
            provider,
        })
        .collect();

    ProviderBucketsResult {
        buckets,
        is_truncated: match_count > search_limit.max(1),
        visible_count,
    }
}

pub fn build_provider_buckets(
    query: LowercasedQuery<'_>,
    provider_limits: &BTreeMap<String, usize>,
    search_limit: usize,
) -> ProviderBucketsResult {
    if query.is_empty() {
        let index = icon_index();
        let buckets = index
            .by_provider
            .keys()
            .filter_map(|provider| {
                let limit = provider_limits
                    .get(provider)
                    .copied()
                    .unwrap_or(INITIAL_PROVIDER_LIMIT);
                build_provider_bucket(provider, limit, index).ok()
            })
            .collect();
        ProviderBucketsResult {
            buckets,
            is_truncated: false,
            visible_count: 0,
        }
    } else {
        build_search_buckets(query, search_limit)
    }
}

#[cfg(test)]
#[path = "models_tests.rs"]
mod models_tests;
