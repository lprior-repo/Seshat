#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

mod validation;
pub use validation::*;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod proptests;

#[cfg(test)]
mod boon_smoke_tests;

#[cfg(test)]
mod serde_schema_alignment;
