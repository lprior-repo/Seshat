#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

pub mod apply;
pub mod commands;
pub mod common;
pub mod db;
pub mod export;
pub mod import;
pub mod layout;
pub mod op;
pub mod outbox;
pub mod patch;
pub mod render;
pub mod validate;

pub use commands::{Cli, Commands};
use common::{emit_event, error_code, exit_code, CliEvent};

pub fn run_cli(cli: &Cli) {
    if let Some(cmd) = &cli.command {
        let name = cmd.name().to_string();
        emit_event(&CliEvent::start(name.clone()));

        match cmd.execute() {
            Ok(()) => {
                emit_event(&CliEvent::finish(name, true, String::from("ok")));
            }
            Err(err) => {
                let err_ref: &(dyn std::error::Error + Send + Sync) = err.as_ref();
                let code = error_code(err_ref);
                emit_event(&CliEvent::error(
                    name.clone(),
                    code.clone(),
                    err.to_string(),
                ));
                emit_event(&CliEvent::finish(name, false, code));
                std::process::exit(exit_code(err_ref));
            }
        }
    }
}
