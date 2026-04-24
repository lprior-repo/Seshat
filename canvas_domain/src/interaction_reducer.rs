#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![forbid(unsafe_code)]

#[path = "interaction_reducer/commit.rs"]
mod commit;
#[path = "interaction_reducer/geometry.rs"]
mod geometry;
#[path = "interaction_reducer/release.rs"]
mod release;
#[path = "interaction_reducer/resize.rs"]
mod resize;
#[path = "interaction_reducer/types.rs"]
mod types;

pub use commit::commit_inline_edit;
pub use release::finalize_motion_release;
pub use resize::start_resize_interaction;
pub use types::{InteractionMode, ResizeHandle};

#[cfg(test)]
pub use geometry::{resize_target_ids, safe_zoom, within};

#[cfg(test)]
mod tests;
