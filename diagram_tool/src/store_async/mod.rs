//! Async store module.
//!
//! This module provides async database operations for the diagram tool.
//! It is split into submodules for better organization and to maintain
//! the 300-line limit per file requirement.

#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

pub mod append;
pub mod bootstrap;
pub mod error;
pub mod fetch;
pub mod parse;
pub mod revision;
pub mod types;

#[cfg(test)]
#[allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
pub mod fetch_tests;
#[cfg(test)]
#[allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
pub mod parse_tests;

// Re-export for backward compatibility
pub use append::{
    append_batch_async, append_event_async, append_idempotent_async, classify_duplicate_async,
    lookup_existing_op_id_in_tx,
};
pub use bootstrap::{bootstrap_async_store, create_async_pool, read_store_pragmas_async};
pub use error::{AsyncStoreError, DuplicateKind, CURRENT_SCHEMA_VERSION};
pub use fetch::{
    fetch_all_events, fetch_events_since, integrity_check_async, open_recovery_mode_async,
    reset_store_async,
};
pub use parse::{
    envelope_batch_to_bounded_batch, envelope_to_valid_event, parse_bounded_batch, parse_revision,
    parse_valid_event,
};
pub use revision::{current_revision, fetch_latest_revision, next_revision};
pub use types::{
    AsyncAppendResult, AsyncBatchAppendResult, AsyncStoreBootstrap, AsyncStorePragmas,
    CliErrorCode, EventRecord,
};

pub use types::map_error_code;
