#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

mod model;
mod transitions;

pub use model::{GhostDiffError, GhostDiffState, GhostDiffStateMode, PendingProposal};

#[cfg(test)]
mod basic_tests;

#[cfg(test)]
mod contract_accept_tests;

#[cfg(test)]
mod contract_tests;

#[cfg(test)]
mod contract_receive_tests;

#[cfg(test)]
mod contract_toggle_tests;

#[cfg(test)]
mod fuzz_simulation;

#[cfg(kani)]
mod kani_harnesses;
