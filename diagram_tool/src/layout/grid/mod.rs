#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![allow(clippy::cast_possible_truncation)]
#![forbid(unsafe_code)]

mod algorithm;
mod bounds;
#[cfg(test)]
mod tests;

pub use algorithm::calculate_grid_layout;
pub use bounds::GridError;
