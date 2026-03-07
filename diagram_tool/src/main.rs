#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

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
mod cli_persistence;
mod core;
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
mod store;
#[cfg(all(not(target_arch = "wasm32"), feature = "async-db"))]
mod store_async;
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
