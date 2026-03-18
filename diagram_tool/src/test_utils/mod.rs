//! Test Infrastructure for Seshat Diagram Tool
//!
//! This module provides the test harness for running the 240 test cases
//! organized into 11 categories as specified in the architecture spec.
//!
//! ## Design by Contract
//!
//! - **P1**: Test category ID is valid (compile-time via enum)
//! - **P2**: Golden scene file exists (Runtime Result)
//! - **P3**: Golden scene is valid JSON (Runtime Result)
//! - **P4**: Schema version matches expected (Runtime Result)
//! - **P5**: Test environment is isolated (no external network types)
//! - **P6**: Test database path is unique per test (Debug-only assert)
//! - **P7**: Browser is available for E2E tests (Runtime Result)

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(unused_imports, clippy::unnecessary_lazy_evaluations)]

pub mod builders;
pub mod fixtures;
pub mod generators;
pub mod harness;
pub mod types;

pub use builders::*;
pub use fixtures::*;
pub use generators::*;
pub use harness::*;
pub use types::*;

// Tests
// ============================================================================

#[cfg(test)]
mod tests_fixtures;
#[cfg(test)]
mod tests_generators;
#[cfg(test)]
mod tests_harness;
#[cfg(test)]
mod tests_types;
