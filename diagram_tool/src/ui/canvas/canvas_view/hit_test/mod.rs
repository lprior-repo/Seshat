#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
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
