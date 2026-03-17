#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use anyhow::Result;
use clap::{Parser, Subcommand};

pub mod apply;
pub mod common;
pub mod db;
pub mod export;
pub mod generate_scene;
pub mod import;
pub mod layout;
pub mod op;
pub mod outbox;
pub mod patch;
pub mod render;
pub mod validate;

use common::{emit_event, error_code, exit_code, CliEvent};

#[derive(Parser, Debug, Clone)]
#[command(name = "diagram_tool")]
#[command(version = "0.1.0")]
#[command(about = "Diagram Tool CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    Render {
        #[arg(long)]
        input: String,
        #[arg(long)]
        output: String,
    },
    Layout {
        #[arg(long)]
        input: String,
        #[arg(long)]
        output: String,
    },
    Validate {
        #[arg(long)]
        input: String,
    },
    Patch {
        #[arg(long)]
        input: String,
        #[arg(long)]
        patch: String,
        #[arg(long)]
        output: String,
    },
    GenerateScene {
        #[arg(long, default_value_t = 3000)]
        nodes: u32,
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(long)]
        output: String,
    },
    Apply {
        #[arg(long)]
        input: String,
        #[arg(long)]
        subgraph: String,
        #[arg(long)]
        output: String,
    },
    Export {
        #[arg(long)]
        input: String,
        #[arg(long)]
        format: String,
        #[arg(long)]
        output: String,
    },
    Import {
        #[arg(long)]
        input: String,
        #[arg(long)]
        format: String,
        #[arg(long)]
        output: String,
    },
    // Database commands
    DbInit {
        #[arg(long)]
        path: String,
    },
    DbStatus {
        #[arg(long)]
        path: String,
    },
    DbRevision {
        #[arg(long)]
        path: String,
    },
    DbEvents {
        #[arg(long)]
        path: String,
        #[arg(long, default_value_t = 0)]
        since: i64,
    },
    DbConflictDiff {
        #[arg(long)]
        path: String,
        #[arg(long)]
        assumed_revision: i64,
    },
    // Operation commands
    OpStart {
        #[arg(long)]
        path: String,
        #[arg(long)]
        operation_id: String,
        #[arg(long)]
        total_steps: u32,
        #[arg(long)]
        author_id: String,
        #[arg(long)]
        description: String,
    },
    OpStatus {
        #[arg(long)]
        path: String,
        #[arg(long)]
        operation_id: String,
    },
    OpList {
        #[arg(long)]
        path: String,
        #[arg(long, default_value_t = String::from("in_progress"))]
        state: String,
    },
    OpComplete {
        #[arg(long)]
        path: String,
        #[arg(long)]
        operation_id: String,
    },
    OpFail {
        #[arg(long)]
        path: String,
        #[arg(long)]
        operation_id: String,
        #[arg(long)]
        error: String,
    },
    // Outbox commands
    OutboxList {
        #[arg(long)]
        path: String,
        #[arg(long, default_value_t = 10)]
        limit: u32,
    },
    OutboxAdd {
        #[arg(long)]
        path: String,
        #[arg(long)]
        id: String,
        #[arg(long)]
        side_effect_type: String,
        #[arg(long)]
        payload: String,
    },
}

pub fn run_cli(cli: &Cli) {
    if let Some(cmd) = &cli.command {
        emit_event(&CliEvent::start(command_name(cmd)));
        match execute_command(cmd) {
            Ok(()) => {
                emit_event(&CliEvent::finish(
                    command_name(cmd),
                    true,
                    String::from("ok"),
                ));
            }
            Err(err) => {
                emit_event(&CliEvent::error(
                    command_name(cmd),
                    error_code(&err),
                    err.to_string(),
                ));
                emit_event(&CliEvent::finish(
                    command_name(cmd),
                    false,
                    error_code(&err),
                ));
                std::process::exit(exit_code(&err));
            }
        }
    }
}

fn command_name(cmd: &Commands) -> String {
    match cmd {
        Commands::Render { .. } => String::from("render"),
        Commands::Layout { .. } => String::from("layout"),
        Commands::Validate { .. } => String::from("validate"),
        Commands::Patch { .. } => String::from("patch"),
        Commands::GenerateScene { .. } => String::from("generate_scene"),
        Commands::Apply { .. } => String::from("apply"),
        Commands::Export { .. } => String::from("export"),
        Commands::Import { .. } => String::from("import"),
        Commands::DbInit { .. } => String::from("db_init"),
        Commands::DbStatus { .. } => String::from("db_status"),
        Commands::DbRevision { .. } => String::from("db_revision"),
        Commands::DbEvents { .. } => String::from("db_events"),
        Commands::DbConflictDiff { .. } => String::from("db_conflict_diff"),
        Commands::OpStart { .. } => String::from("op_start"),
        Commands::OpStatus { .. } => String::from("op_status"),
        Commands::OpList { .. } => String::from("op_list"),
        Commands::OpComplete { .. } => String::from("op_complete"),
        Commands::OpFail { .. } => String::from("op_fail"),
        Commands::OutboxList { .. } => String::from("outbox_list"),
        Commands::OutboxAdd { .. } => String::from("outbox_add"),
    }
}

fn execute_command(cmd: &Commands) -> Result<()> {
    match cmd {
        Commands::Render { input, output } => render::handle(input, output),
        Commands::Layout { input, output } => layout::handle(input, output),
        Commands::Validate { input } => validate::handle(input),
        Commands::Patch {
            input,
            patch,
            output,
        } => patch::handle(input, patch, output),
        Commands::GenerateScene {
            nodes,
            seed,
            output,
        } => generate_scene::handle(*nodes, *seed, output),
        Commands::Apply {
            input,
            subgraph,
            output,
        } => apply::handle(input, subgraph, output),
        Commands::Export {
            input,
            format,
            output,
        } => export::handle(input, format, output),
        Commands::Import {
            input,
            format,
            output,
        } => import::handle(input, format, output),
        Commands::DbInit { path } => db::handle_init(path),
        Commands::DbStatus { path } => db::handle_status(path),
        Commands::DbRevision { path } => db::handle_revision(path),
        Commands::DbEvents { path, since } => db::handle_events(path, *since),
        Commands::DbConflictDiff {
            path,
            assumed_revision,
        } => db::handle_conflict_diff(path, *assumed_revision),
        Commands::OpStart {
            path,
            operation_id,
            total_steps,
            author_id,
            description,
        } => op::handle_start(path, operation_id, *total_steps, author_id, description),
        Commands::OpStatus { path, operation_id } => op::handle_status(path, operation_id),
        Commands::OpList { path, state } => op::handle_list(path, state),
        Commands::OpComplete { path, operation_id } => op::handle_complete(path, operation_id),
        Commands::OpFail {
            path,
            operation_id,
            error,
        } => op::handle_fail(path, operation_id, error),
        Commands::OutboxList { path, limit } => outbox::handle_list(path, *limit),
        Commands::OutboxAdd {
            path,
            id,
            side_effect_type,
            payload,
        } => outbox::handle_add(path, id, side_effect_type, payload),
    }
}
