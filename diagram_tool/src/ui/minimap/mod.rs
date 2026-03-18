#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

pub mod component;
pub mod models;
#[cfg(test)]
pub mod tests;

pub use component::Minimap;
