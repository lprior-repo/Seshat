//! Async store module.
//!
//! This module provides async database operations for the diagram tool.
//! It is split into submodules for better organization and to maintain
//! the 300-line limit per file requirement.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

pub mod append;
pub mod bootstrap;
pub mod error;
pub mod fetch;
pub mod parse;
pub mod revision;
pub mod types;

// Re-export for backward compatibility
pub use error::{AsyncStoreError, DuplicateKind, CURRENT_SCHEMA_VERSION};
pub use types::{
    AsyncAppendResult, AsyncBatchAppendResult, AsyncStoreBootstrap, AsyncStorePragmas,
    CliErrorCode, EventRecord,
};
pub use parse::{
    envelope_to_valid_event, envelope_batch_to_bounded_batch, parse_valid_event,
    parse_bounded_batch, parse_revision,
};
pub use bootstrap::{create_async_pool, bootstrap_async_store, read_store_pragmas_async};
pub use revision::{fetch_latest_revision, current_revision, next_revision};
pub use append::{
    append_event_async, append_batch_async, lookup_existing_op_async,
    classify_duplicate_async, append_idempotent_async,
};
pub use fetch::{fetch_events_since, fetch_all_events, integrity_check_async, open_recovery_mode_async};

pub use types::map_error_code;
