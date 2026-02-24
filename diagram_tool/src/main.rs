#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use clap::Parser;
use dioxus::prelude::*;

mod models;
mod ui;
mod icons;
mod app;
mod cli;
mod layout;
mod history;
mod hooks;
mod patch;
mod export;

use crate::app::App;
use crate::cli::Cli;

fn main() {
    let cli = Cli::parse();
    
    if cli.command.is_some() {
        cli::run_cli(&cli);
    } else {
        dioxus::launch(App);
    }
}
