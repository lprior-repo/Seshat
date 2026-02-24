#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use include_dir::{include_dir, Dir};

pub const ICONS: Dir = include_dir!("$CARGO_MANIFEST_DIR/resources");

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

pub fn icon_src(icon: &IconMeta) -> String {
    format!("/resources/{}", icon.file_relpath)
}
