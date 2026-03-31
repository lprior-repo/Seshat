#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::cast_precision_loss)]
#![forbid(unsafe_code)]
#![allow(dead_code)]

pub mod hit_test_margin;
pub mod intersection;
pub mod operations;
pub mod polygon;
pub mod primitives;
pub mod routing;
pub mod snap;
pub mod transforms;

pub use canvas_math::{MAX_ZOOM, MIN_ZOOM};

#[cfg(kani)]
pub mod transforms_kani;

#[cfg(kani)]
pub mod operations_kani;

#[cfg(test)]
#[allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
pub mod hit_test_margin_tests;

#[cfg(test)]
#[allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
pub mod operations_tests;

pub use hit_test_margin::{hit_test_with_margin, screen_to_world_margin, HitTestError};
pub use intersection::{
    line_line_intersection, line_line_intersects, line_rect_intersections, line_rect_intersects,
    IntersectionError, LineSegment,
};
pub use operations::*;
pub use polygon::*;
pub use primitives::*;
pub use routing::*;
pub use snap::{SnapError, SnapNode, SnapState};
pub use transforms::*;

pub mod path;
pub use path::*;

#[cfg(test)]
#[allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
mod tests;
