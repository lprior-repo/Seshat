#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use clap::Parser;
use diagram_tool::cli::{run_cli, Cli};

fn main() {
    let cli = Cli::parse();
    run_cli(&cli);
}
