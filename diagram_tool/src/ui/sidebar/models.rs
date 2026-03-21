use std::collections::BTreeMap;

use crate::icons::{icon_index, IconMeta};

pub const INITIAL_PROVIDER_LIMIT: usize = 50;
pub const LOAD_MORE_STEP: usize = 50;
pub const MAX_SEARCH_RESULTS: usize = 150;
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

#[derive(Clone, PartialEq)]
pub struct ProviderBucketsResult {
    pub buckets: Vec<ProviderBucket>,
    pub is_truncated: bool,
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

pub fn search_matches(index: &[IconMeta], query: LowercasedQuery<'_>) -> (usize, Vec<IconMeta>) {
    let mut visible: Vec<IconMeta> = index
        .iter()
        .filter(|icon| matches_query(icon, query))
        .take(MAX_SEARCH_RESULTS + 1)
        .cloned()
        .collect();

    let count = visible.len();
    if count > MAX_SEARCH_RESULTS {
        visible.pop();
    }

    (count, visible)
}

fn build_empty_bucket(
    provider: &String,
    provider_limits: &BTreeMap<String, usize>,
    index: &crate::icons::IconIndex,
) -> ProviderBucket {
    let icons = index.icons_by_provider(provider);
    let limit = provider_limits
        .get(provider)
        .copied()
        .unwrap_or(INITIAL_PROVIDER_LIMIT);
    let visible: Vec<IconMeta> = icons.iter().take(limit).map(|&i| i.clone()).collect();

    ProviderBucket {
        provider: provider.clone(),
        total_count: icons.len(),
        visible_count: visible.len(),
        has_more: icons.len() > visible.len(),
        categories: bucket_icons_by_category(&visible),
    }
}

fn build_search_buckets(query: LowercasedQuery<'_>) -> ProviderBucketsResult {
    let (match_count, limited) = search_matches(&icon_index().all, query);
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
        is_truncated: match_count > MAX_SEARCH_RESULTS,
    }
}

pub fn build_provider_buckets(
    query: LowercasedQuery<'_>,
    provider_limits: &BTreeMap<String, usize>,
) -> ProviderBucketsResult {
    if query.is_empty() {
        let index = icon_index();
        let buckets = index
            .by_provider
            .keys()
            .map(|provider| build_empty_bucket(provider, provider_limits, index))
            .collect();
        ProviderBucketsResult {
            buckets,
            is_truncated: false,
        }
    } else {
        build_search_buckets(query)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn given_empty_query_when_matches_query_then_returns_true() {
        let icon = IconMeta {
            icon_key: std::sync::Arc::from("aws/analytics/athena"),
            provider: std::sync::Arc::from("aws"),
            category_path: vec![std::sync::Arc::from("Analytics")],
            file_relpath: std::sync::Arc::from("aws/Analytics/athena.svg"),
            display_name: std::sync::Arc::from("Athena"),
            search_terms: std::sync::Arc::from("aws/analytics/athena athena aws analytics"),
            base64_data: std::sync::Arc::from(""),
        };
        assert!(matches_query(&icon, LowercasedQuery::empty()));
    }

    #[test]
    fn given_matching_query_when_matches_query_then_returns_true() {
        let icon = IconMeta {
            icon_key: std::sync::Arc::from("aws/analytics/athena"),
            provider: std::sync::Arc::from("aws"),
            category_path: vec![std::sync::Arc::from("Analytics")],
            file_relpath: std::sync::Arc::from("aws/Analytics/athena.svg"),
            display_name: std::sync::Arc::from("Athena"),
            search_terms: std::sync::Arc::from("aws/analytics/athena athena aws analytics"),
            base64_data: std::sync::Arc::from(""),
        };
        // The search query is expected to be already lowercased by the UI
        assert!(matches_query(
            &icon,
            LowercasedQuery::new("athena").unwrap()
        ));
        assert!(matches_query(&icon, LowercasedQuery::new("aws").unwrap()));
        assert!(matches_query(
            &icon,
            LowercasedQuery::new("analytics").unwrap()
        ));
    }

    #[test]
    fn given_non_matching_query_when_matches_query_then_returns_false() {
        let icon = IconMeta {
            icon_key: std::sync::Arc::from("aws/analytics/athena"),
            provider: std::sync::Arc::from("aws"),
            category_path: vec![std::sync::Arc::from("Analytics")],
            file_relpath: std::sync::Arc::from("aws/Analytics/athena.svg"),
            display_name: std::sync::Arc::from("Athena"),
            search_terms: std::sync::Arc::from("aws/analytics/athena athena aws analytics"),
            base64_data: std::sync::Arc::from(""),
        };
        assert!(!matches_query(
            &icon,
            LowercasedQuery::new("database").unwrap()
        ));
        assert!(!matches_query(
            &icon,
            LowercasedQuery::new("azure").unwrap()
        ));
    }
}
