#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]
#![allow(dead_code)]

#[path = "store/sqlx/mod.rs"]
pub mod sqlx_store;
pub use sqlx_store::*;
