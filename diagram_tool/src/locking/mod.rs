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
#![allow(dead_code)]

pub mod error;
pub mod file_lock;
pub mod manager;
