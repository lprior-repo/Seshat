#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

pub mod apply;
pub mod conflict_resolution;
pub mod history_append;
pub mod validation;
