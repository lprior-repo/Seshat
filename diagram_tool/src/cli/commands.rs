#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use super::apply::ApplyCommand;
use super::export::ExportCommand;
use super::import::ImportCommand;
use super::layout::LayoutCommand;
use super::patch::PatchCommand;
use super::render::RenderCommand;
use super::validate::ValidateCommand;

#[derive(Parser, Debug)]
#[command(name = "seshat")]
#[command(about = "Seshat diagram tool CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Validate a diagram document")]
    Validate(ValidateArgs),

    #[command(about = "Apply changes to a diagram")]
    Apply(ApplyArgs),

    #[command(about = "Apply a JSON patch to a diagram")]
    Patch(PatchArgs),

    #[command(about = "Render diagram to PNG or SVG")]
    Render(RenderArgs),

    #[command(about = "Auto-arrange nodes using DAG layout")]
    Layout(LayoutArgs),

    #[command(about = "Export diagram to JSON")]
    Export(ExportArgs),

    #[command(about = "Import diagram from JSON")]
    Import(ImportArgs),
}

#[derive(Parser, Debug)]
pub struct ValidateArgs {
    #[arg(short, long, value_name = "FILE")]
    pub input: PathBuf,
}

#[derive(Parser, Debug)]
pub struct ApplyArgs {
    #[arg(short, long, value_name = "FILE")]
    pub input: PathBuf,
}

#[derive(Parser, Debug)]
pub struct PatchArgs {
    #[arg(short, long, value_name = "FILE")]
    pub input: PathBuf,

    #[arg(short, long, value_name = "FILE")]
    pub patch: PathBuf,

    #[arg(short, long, value_name = "FILE")]
    pub output: PathBuf,
}

#[derive(Parser, Debug)]
pub struct RenderArgs {
    #[arg(short, long, value_name = "FILE")]
    pub input: PathBuf,

    #[arg(short, long, value_name = "FILE")]
    pub output: PathBuf,
}

#[derive(Parser, Debug)]
pub struct LayoutArgs {
    #[arg(short, long, value_name = "FILE")]
    pub input: PathBuf,

    #[arg(short, long, value_name = "FILE")]
    pub output: PathBuf,
}

#[derive(Parser, Debug)]
pub struct ExportArgs {
    #[arg(short, long, value_name = "FILE")]
    pub input: PathBuf,

    #[arg(long, value_name = "FORMAT", default_value = "json")]
    pub format: String,
}

#[derive(Parser, Debug)]
pub struct ImportArgs {
    #[arg(short, long, value_name = "FILE")]
    pub input: PathBuf,
}

pub trait Command {
    fn name(&self) -> &'static str;
    fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

impl Commands {
    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match self {
            Commands::Validate(args) => ValidateCommand::new(args.input.clone()).execute(),
            Commands::Apply(args) => ApplyCommand::new(args.input.clone()).execute(),
            Commands::Patch(args) => {
                PatchCommand::new(args.input.clone(), args.patch.clone(), args.output.clone())
                    .execute()
            }
            Commands::Render(args) => {
                RenderCommand::new(args.input.clone(), args.output.clone()).execute()
            }
            Commands::Layout(args) => {
                LayoutCommand::new(args.input.clone(), args.output.clone()).execute()
            }
            Commands::Export(args) => {
                ExportCommand::new(args.input.clone(), args.format.clone()).execute()
            }
            Commands::Import(args) => ImportCommand::new(args.input.clone()).execute(),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Commands::Validate(_) => "validate",
            Commands::Apply(_) => "apply",
            Commands::Patch(_) => "patch",
            Commands::Render(_) => "render",
            Commands::Layout(_) => "layout",
            Commands::Export(_) => "export",
            Commands::Import(_) => "import",
        }
    }
}
