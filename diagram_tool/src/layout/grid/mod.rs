#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![allow(clippy::cast_possible_truncation)]
#![forbid(unsafe_code)]

mod algorithm;
mod bounds;
#[cfg(test)]
mod tests;

pub use algorithm::calculate_grid_layout;
pub use bounds::{CellSize, GridError};
