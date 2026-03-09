//! `SQLite` storage module
//!
//! Provides SQLite-based storage with WAL mode and full synchronous durability.

#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

pub mod append;
pub mod batch;
pub mod cli;
pub mod config;
pub mod connection;
pub mod errors;
pub mod idempotent;
pub mod recovery;
pub mod revision;
pub mod types;

#[cfg(test)]
mod tests;

pub use append::*;
pub use batch::*;
pub use cli::*;
pub use config::*;
pub use connection::*;
pub use errors::*;
pub use idempotent::*;
pub use recovery::*;
pub use revision::*;
pub use types::*;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::store_async::{
    append_event_async, bootstrap_async_store, integrity_check_async, open_recovery_mode_async,
    AsyncStoreBootstrap, AsyncStoreError,
};

pub const CURRENT_SCHEMA_VERSION: i32 = 1;
