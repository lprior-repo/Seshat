//! Test Builder Helpers
//!
//! Consolidated test helpers for creating nodes, edges, and documents in tests.
//! This module provides a fluent builder pattern for test data construction.

#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![allow(dead_code)]
#![allow(clippy::pedantic)]

pub mod doc;
pub mod edge;
pub mod node;

pub use doc::*;
pub use edge::*;
pub use node::*;
