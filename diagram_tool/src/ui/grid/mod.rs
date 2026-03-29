#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

pub mod size;
pub mod snapping;

#[cfg(test)]
mod tests;

pub use size::*;
pub use snapping::*;
