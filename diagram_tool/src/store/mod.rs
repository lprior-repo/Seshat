#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

#[cfg(all(not(target_arch = "wasm32"), feature = "async-db"))]
pub mod durable;

pub mod types;
pub use types::*;

#[cfg(all(not(target_arch = "wasm32"), feature = "async-db"))]
pub use crate::store_async::{
    append_event_async, bootstrap_async_store, fetch_all_events, fetch_events_since,
    integrity_check_async, open_recovery_mode_async, AsyncStoreBootstrap, AsyncStoreError,
};
