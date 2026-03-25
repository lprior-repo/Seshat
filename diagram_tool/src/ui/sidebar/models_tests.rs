#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
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
                base64_data: std::sync::Arc::from(""),
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
