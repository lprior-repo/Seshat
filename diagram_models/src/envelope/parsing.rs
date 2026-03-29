//! Parsing functions for domain operations
//!
//! This module provides parsing from JSON to domain types.

#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

pub mod composite_ops;
pub mod dispatch;
pub mod edge_ops;
pub mod helpers;
pub mod node_ops;
pub mod zorder_ops;

pub use dispatch::parse_domain_op;
