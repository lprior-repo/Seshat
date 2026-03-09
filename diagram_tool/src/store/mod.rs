#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

pub mod append;
pub mod error;
pub mod read;
pub mod recovery;
pub mod session;
pub mod types;

pub use append::*;
pub use error::*;
pub use read::*;
pub use recovery::*;
pub use session::*;
pub use types::*;

#[cfg(all(not(target_arch = "wasm32"), feature = "async-db"))]
pub use crate::store_async::{
    append_event_async, bootstrap_async_store, integrity_check_async, open_recovery_mode_async,
    AsyncStoreBootstrap, AsyncStoreError,
};
