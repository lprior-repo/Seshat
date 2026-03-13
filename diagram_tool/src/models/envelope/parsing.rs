//! Parsing functions for domain operations
//!
//! This module provides parsing from JSON to domain types.

#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

pub mod composite_ops;
pub mod dispatch;
pub mod edge_ops;
pub mod helpers;
pub mod node_ops;
pub mod zorder_ops;

pub use dispatch::parse_domain_op;
