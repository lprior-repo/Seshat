use std::collections::BTreeMap;

use crate::icons::{icon_index, IconMeta};

pub const INITIAL_PROVIDER_LIMIT: usize = 25;
pub const LOAD_MORE_STEP: usize = 25;
pub const MAX_SEARCH_RESULTS: usize = 150;
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

    fn create_test_index(categories: &[(&str, usize)]) -> crate::icons::IconIndex {
        let mut all = Vec::new();
        let mut by_provider = std::collections::BTreeMap::new();
        let mut by_key = std::collections::HashMap::new();
        let provider_name = "test_provider";
        let mut provider_keys = Vec::new();

        for (cat_name, count) in categories {
            for i in 0..*count {
                let icon_key = format!("{}/{}/{}", provider_name, cat_name, i);
                provider_keys.push(icon_key.clone());

                let icon = IconMeta {
                    icon_key: std::sync::Arc::from(icon_key.as_str()),
                    provider: std::sync::Arc::from(provider_name),
                    category_path: if cat_name.is_empty() {
                        vec![]
                    } else {
                        vec![std::sync::Arc::from(*cat_name)]
                    },
                    file_relpath: std::sync::Arc::from(""),
                    display_name: std::sync::Arc::from(""),
                    search_terms: std::sync::Arc::from(""),
                };
                all.push(icon.clone());
                by_key.insert(icon_key, icon);
            }
        }
        by_provider.insert(provider_name.to_string(), provider_keys);

        crate::icons::IconIndex {
            all,
            by_provider,
            by_key,
        }
    }

    #[test]
    fn test_rejects_request_when_provider_not_in_registry() {
        let index = create_test_index(&[]);
        let result = build_provider_bucket("nonexistent", 25, &index);
        assert_eq!(result, Err(Error::ProviderNotFound));
    }

    #[test]
    fn test_rejects_request_when_limit_is_zero() {
        let index = create_test_index(&[]);
        let result = build_provider_bucket("aws", 0, &index);
        assert_eq!(result, Err(Error::InvalidLimit(0)));
    }

    #[test]
    fn test_rejects_request_when_limit_is_not_multiple_of_25() {
        let index = create_test_index(&[]);
        let result = build_provider_bucket("aws", 33, &index);
        assert_eq!(result, Err(Error::InvalidLimit(33)));
    }

    #[test]
    fn test_slicing_across_multiple_smaller_categories() {
        let index = create_test_index(&[("A", 10), ("B", 10), ("C", 10)]);
        let bucket = build_provider_bucket("test_provider", 25, &index).unwrap();

        assert_eq!(bucket.categories.len(), 3);
        assert_eq!(bucket.categories[0].name, "A");
        assert_eq!(bucket.categories[0].icons.len(), 10);
        assert_eq!(bucket.categories[1].name, "B");
        assert_eq!(bucket.categories[1].icons.len(), 10);
        assert_eq!(bucket.categories[2].name, "C");
        assert_eq!(bucket.categories[2].icons.len(), 5);
        assert_eq!(bucket.visible_count, 25);
        assert_eq!(bucket.total_count, 30);
        assert!(bucket.has_more);
    }

    #[test]
    fn test_slicing_exactly_on_category_boundary() {
        let index = create_test_index(&[("X", 25), ("Y", 25)]);
        let bucket = build_provider_bucket("test_provider", 25, &index).unwrap();

        assert_eq!(bucket.categories.len(), 1);
        assert_eq!(bucket.categories[0].name, "X");
        assert_eq!(bucket.categories[0].icons.len(), 25);
        assert_eq!(bucket.visible_count, 25);
    }

    #[test]
    fn test_empty_categories_are_omitted_from_bucket_entirely() {
        let index = create_test_index(&[("Empty", 0), ("Data", 30)]);
        let bucket = build_provider_bucket("test_provider", 25, &index).unwrap();

        assert_eq!(bucket.categories.len(), 1);
        assert_eq!(bucket.categories[0].name, "Data");
        assert_eq!(bucket.categories[0].icons.len(), 25);
    }

    #[test]
    fn test_mid_category_slicing() {
        let index = create_test_index(&[("Mega", 100)]);
        let bucket = build_provider_bucket("test_provider", 25, &index).unwrap();

        assert_eq!(bucket.categories.len(), 1);
        assert_eq!(bucket.categories[0].name, "Mega");
        assert_eq!(bucket.categories[0].icons.len(), 25);
    }
}
