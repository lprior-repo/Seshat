//! Diagram locking module - provides per-diagram queue and file lock discipline.
//!
//! This module implements:
//! - Per-diagram mutation serialization
//! - File-level locking for cross-process safety
//! - Parallel work across different diagrams
//! - Integration with atomic persistence

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

pub mod error;
pub mod manager;
pub mod file_lock;

pub use error::LockError;
pub use manager::DiagramLockManager;
pub use file_lock::FileLock;
