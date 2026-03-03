//! Diagram Tool Library
//!
//! This module exposes the library components for use in integration tests.

#![allow(dead_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

pub mod app;
pub mod backend;
pub mod cli;
pub mod cli_persistence;
pub mod export;
pub mod geometry;
pub mod history;
pub mod hooks;
pub mod icons;
pub mod layout;
pub mod models;
pub mod mutation;
pub mod perf;
pub mod store;
pub mod ui;
pub mod viewport;
