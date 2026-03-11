#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::cast_precision_loss)]
#![forbid(unsafe_code)]
#![allow(dead_code)]

pub mod operations;
pub mod polygon;
pub mod primitives;
pub mod snap;
pub mod transforms;

pub use operations::*;
pub use polygon::*;
pub use primitives::*;
pub use snap::{SnapError, SnapNode, SnapState};
pub use transforms::*;

pub mod path;
pub use path::*;

#[cfg(test)]
mod tests;
