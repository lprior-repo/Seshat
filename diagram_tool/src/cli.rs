#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::cli_persistence::{
    emit_stage_event, load_workspace_with_lkg, save_workspace_atomic, StageDetails,
};
use crate::export::png::export_png;
use crate::export::svg::generate_svg_string;
use crate::models::document::DiagramDocument;
use crate::mutation::ops::apply_layout;
use crate::mutation::pipeline::run_mutation;
use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::path::Path;

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

#[derive(Serialize)]
struct CliEvent {
    event: String,
    command: String,
    ok: bool,
    code: String,
    message: Option<String>,
}

impl CliEvent {
    fn start(command: String) -> Self {
        Self {
            event: String::from("start"),
            command,
            ok: true,
            code: String::from("start"),
            message: None,
        }
    }

    fn error(command: String, code: String, message: String) -> Self {
        Self {
            event: String::from("error"),
            command,
            ok: false,
            code,
            message: Some(message),
        }
    }

    fn finish(command: String, ok: bool, code: String) -> Self {
        Self {
            event: String::from("finish"),
            command,
            ok,
            code,
            message: None,
        }
    }
}

fn command_name(cmd: &Commands) -> String {
    match cmd {
        Commands::Render { .. } => String::from("render"),
        Commands::Layout { .. } => String::from("layout"),
        Commands::Validate { .. } => String::from("validate"),
    }
}

fn error_code(err: &anyhow::Error) -> String {
    let msg = err.to_string().to_lowercase();
    if msg.contains("schema") {
        String::from("schema_violation")
    } else if msg.contains("dag") || msg.contains("cycle") {
        String::from("dag_cycle")
    } else if msg.contains("parse")
        || msg.contains("deserialize")
        || msg.contains("unknown variant")
        || msg.contains("failed to parse")
    {
        String::from("parse_error")
    } else {
        String::from("command_error")
    }
}

fn exit_code(err: &anyhow::Error) -> i32 {
    let code = error_code(err);
    match code.as_str() {
        "parse_error" | "command_error" => 2,
        _ => 1,
    }
}

fn emit_event(event: &CliEvent) {
    match serde_json::to_string(&event) {
        Ok(line) => println!("{line}"),
        Err(_) => {
            println!("{{\"event\":\"error\",\"ok\":false,\"code\":\"jsonl_encode_error\"}}");
        }
    }
}

fn execute_command(cmd: &Commands) -> Result<()> {
    match cmd {
        Commands::Render { input, output } => {
            let doc = load_doc(input)?;
            if Path::new(output)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
            {
                export_png(&doc, output)?;
            } else if Path::new(output)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
            {
                let svg = generate_svg_string(&doc);
                let mut file = File::create(output).context("Failed to create SVG file")?;
                file.write_all(svg.as_bytes())
                    .context("Failed to write SVG content")?;
            } else {
                return Err(anyhow!(
                    "unknown output format; expected .png or .svg extension"
                ));
            }
        }
        Commands::Layout { input, output } => {
            emit_stage_event(
                "validating",
                &StageDetails::new().with_path(Path::new(input)),
            );
            let doc = load_doc(input)?;
            let laid_out_doc = run_mutation(&doc, |current| Ok(apply_layout(current, 200.0)))
                .map_err(|err| anyhow!(err.to_string()))?;
            save_workspace_atomic(&laid_out_doc, Path::new(output))
                .map_err(|e| anyhow!("Failed to save workspace: {e}"))?;
        }
        Commands::Validate { input } => {
            emit_stage_event(
                "validating",
                &StageDetails::new().with_path(Path::new(input)),
            );
            let _doc = load_doc(input)?;
        }
    }
    Ok(())
}

fn load_doc(path: &str) -> Result<DiagramDocument> {
    load_workspace_with_lkg(Path::new(path))
        .map_err(|e| anyhow!("Failed to load document from {path}: {e}"))
}
