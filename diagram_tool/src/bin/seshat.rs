#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

use clap::Parser;
use diagram_tool::cli::{run_cli, Cli};

fn main() {
    let cli = Cli::parse();
    run_cli(&cli);
}
