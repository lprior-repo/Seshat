//! `SQLite` storage module
//!
//! Provides SQLite-based storage. This module re-exports the async store.

#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

// Re-export async store
pub use crate::store_async::{
    append_event_async, bootstrap_async_store, integrity_check_async, open_recovery_mode_async,
    AsyncStoreBootstrap, AsyncStoreError,
};

pub const CURRENT_SCHEMA_VERSION: i32 = 1;
