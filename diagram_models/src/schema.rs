#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

mod validation;
pub use validation::*;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod proptests;
