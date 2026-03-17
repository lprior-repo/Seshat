#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

pub mod size;
pub mod snapping;

#[cfg(test)]
mod tests;

pub use size::*;
pub use snapping::*;
