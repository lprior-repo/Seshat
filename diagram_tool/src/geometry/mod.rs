#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::cast_precision_loss)]
#![forbid(unsafe_code)]
#![allow(dead_code)]

pub mod primitives;
pub mod transforms;
pub mod operations;
pub mod polygon;
pub mod snap;

pub use primitives::*;
pub use transforms::*;
pub use operations::*;
pub use polygon::*;
pub use snap::{SnapNode, SnapState, SnapError};

#[cfg(test)]
mod geometry_tests;
