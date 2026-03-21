#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]
#![allow(clippy::imprecise_flops)]
#![allow(clippy::suboptimal_flops)]

pub mod edge;
pub mod math;
pub mod node;
#[cfg(test)]
pub mod tests;

pub use edge::*;
pub use math::*;
