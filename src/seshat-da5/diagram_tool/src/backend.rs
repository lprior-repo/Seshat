//! Backend module - DEPRECATED
//!
//! This module was previously used for the redb database backend.
//! It has been decommissioned in favor of `SQLite` storage.
//!
//! Any code attempting to use this module will fail at compile time
//! due to the absence of public APIs.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

// Compile-time guard: this module should not be used
const _DEPRECATED_BACKEND: &str = "Backend module deprecated - use SQLite storage instead";
