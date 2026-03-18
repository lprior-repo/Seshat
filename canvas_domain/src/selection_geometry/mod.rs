#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]
#![allow(unexpected_cfgs)]

mod core;

#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod selection_bounds_tests;

#[cfg(test)]
mod selection_interaction_tests;

#[cfg(test)]
mod locked_nodes_tests;

#[cfg(kani)]
mod kani_proofs;

pub use core::{selected_node_ids, selection_bounds};
