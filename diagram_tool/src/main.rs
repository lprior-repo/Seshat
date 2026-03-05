#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use clap::Parser;
use dioxus::prelude::*;

mod app;
mod backend;
mod cli;
mod cli_events_tests;
mod cli_persistence;
mod export;
mod geometry;
mod history;
mod hooks;
mod icons;
mod layout;
mod models;
mod mutation;
mod perf;
mod store;
mod test_harness;
mod ui;

use crate::app::App;
use crate::cli::Cli;

fn main() {
    let cli = Cli::parse();

    if cli.command.is_some() {
        cli::run_cli(&cli);
    } else {
        dioxus::LaunchBuilder::new()
            .with_context(server_only! { ServeConfig::builder() })
            .launch(App);
    }
}
