use anyhow::Result;
use clap::{Parser, Subcommand};

use super::{
    apply, db, export, generate_scene, import, layout, op, outbox, patch, render, validate,
};

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

impl Commands {
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Render { .. } => "render",
            Self::Layout { .. } => "layout",
            Self::Validate { .. } => "validate",
            Self::Patch { .. } => "patch",
            Self::GenerateScene { .. } => "generate_scene",
            Self::Apply { .. } => "apply",
            Self::Export { .. } => "export",
            Self::Import { .. } => "import",
            Self::DbInit { .. } => "db_init",
            Self::DbStatus { .. } => "db_status",
            Self::DbRevision { .. } => "db_revision",
            Self::DbEvents { .. } => "db_events",
            Self::DbConflictDiff { .. } => "db_conflict_diff",
            Self::OpStart { .. } => "op_start",
            Self::OpStatus { .. } => "op_status",
            Self::OpList { .. } => "op_list",
            Self::OpComplete { .. } => "op_complete",
            Self::OpFail { .. } => "op_fail",
            Self::OutboxList { .. } => "outbox_list",
            Self::OutboxAdd { .. } => "outbox_add",
        }
    }

    /// Execute the command
    ///
    /// # Errors
    /// Returns an error if the underlying handler fails
    pub fn execute(&self) -> Result<()> {
        match self {
            Self::Render { input, output } => render::handle(input, output),
            Self::Layout { input, output } => layout::handle(input, output),
            Self::Validate { input } => validate::handle(input),
            Self::Patch {
                input,
                patch,
                output,
            } => patch::handle(input, patch, output),
            Self::GenerateScene {
                nodes,
                seed,
                output,
            } => generate_scene::handle(*nodes, *seed, output),
            Self::Apply {
                input,
                subgraph,
                output,
            } => apply::handle(input, subgraph, output),
            Self::Export {
                input,
                format,
                output,
            } => export::handle(input, format, output),
            Self::Import {
                input,
                format,
                output,
            } => import::handle(input, format, output),
            Self::DbInit { path } => db::handle_init(path),
            Self::DbStatus { path } => db::handle_status(path),
            Self::DbRevision { path } => db::handle_revision(path),
            Self::DbEvents { path, since } => db::handle_events(path, *since),
            Self::DbConflictDiff {
                path,
                assumed_revision,
            } => db::handle_conflict_diff(path, *assumed_revision),
            Self::OpStart {
                path,
                operation_id,
                total_steps,
                author_id,
                description,
            } => op::handle_start(path, operation_id, *total_steps, author_id, description),
            Self::OpStatus { path, operation_id } => op::handle_status(path, operation_id),
            Self::OpList { path, state } => op::handle_list(path, state),
            Self::OpComplete { path, operation_id } => op::handle_complete(path, operation_id),
            Self::OpFail {
                path,
                operation_id,
                error,
            } => op::handle_fail(path, operation_id, error),
            Self::OutboxList { path, limit } => outbox::handle_list(path, *limit),
            Self::OutboxAdd {
                path,
                id,
                side_effect_type,
                payload,
            } => outbox::handle_add(path, id, side_effect_type, payload),
        }
    }
}
