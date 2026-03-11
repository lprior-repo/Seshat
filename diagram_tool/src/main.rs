#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(unexpected_cfgs)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#[cfg(not(target_arch = "wasm32"))]
use clap::Parser;
use dioxus::prelude::*;

mod app;
#[cfg(not(target_arch = "wasm32"))]
mod backend;
#[cfg(not(target_arch = "wasm32"))]
mod cli;
#[cfg(not(target_arch = "wasm32"))]
mod cli_events_tests;
#[cfg(not(target_arch = "wasm32"))]
pub mod cli_persistence;
pub mod config;
pub mod core;
mod export;
mod geometry;
mod history;
mod hooks;
mod icons;
mod layout;
mod models;
mod mutation;
#[cfg(not(target_arch = "wasm32"))]
mod perf;
#[cfg(not(target_arch = "wasm32"))]
pub mod store;
#[cfg(feature = "async-db")]
pub mod store_async;
#[cfg(feature = "async-db")]
pub mod store_durable;
#[cfg(feature = "async-db")]
pub mod store_bridge;
mod test_harness;
mod ui;

use crate::app::App;
#[cfg(not(target_arch = "wasm32"))]
use crate::cli::Cli;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let cli = Cli::parse();
        if cli.command.is_some() {
            cli::run_cli(&cli);
            return;
        }
    }

    dioxus::LaunchBuilder::new()
        .with_context(server_only! { ServeConfig::builder() })
        .launch(App);
}
