#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
#[allow(dead_code)]
mod generated {
    include!(concat!(env!("OUT_DIR"), "/icons_index.rs"));
}

pub use generated::{IconIndex, IconMeta};

static ICON_INDEX: std::sync::OnceLock<IconIndex> = std::sync::OnceLock::new();

pub fn icon_index() -> &'static IconIndex {
    ICON_INDEX.get_or_init(IconIndex::load)
}

#[must_use]
pub fn icon_src(icon: &IconMeta) -> Option<String> {
    crate::ui::canvas::document_ops::queries::icon_url_for_relpath(&icon.file_relpath)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn icon_src_returns_url_path() {
        let meta = IconMeta {
            icon_key: Arc::from("aws/Analytics/athena"),
            provider: Arc::from("aws"),
            category_path: vec![Arc::from("Analytics")],
            file_relpath: Arc::from("aws/Analytics/athena.svg"),
            display_name: Arc::from("Athena"),
            search_terms: Arc::from("athena aws analytics"),
        };
        assert_eq!(
            icon_src(&meta),
            Some("/assets/resources/aws/Analytics/athena.svg".to_string())
        );
    }
}
